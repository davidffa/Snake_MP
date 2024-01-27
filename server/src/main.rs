mod game;
mod packet;
mod util;

use std::{
    collections::HashMap,
    io::{self, Read, Write},
    sync::{Arc, RwLock, RwLockWriteGuard},
    thread::{self, sleep},
    time::Duration,
};

use game::Snake;
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use packet::Packet;
use util::Point;

use crate::game::{Direction, GameContext};

const PORT: u16 = 14300;
const TICK_INTERVAL: f32 = 0.05;
const SERVER: Token = Token(0);

fn setup_gameloop(
    context: &Arc<RwLock<GameContext>>,
    clients: &Arc<RwLock<HashMap<Token, TcpStream>>>,
) {
    let context = Arc::clone(context);
    let clients = Arc::clone(clients);

    thread::spawn(move || loop {
        {
            let mut context = context.write().unwrap();
            context.update();
        }

        {
            let context = context.read().unwrap();

            let mut packet = Packet::new();
            packet.write(0x4);

            for (id, snake) in context.snakes.iter() {
                packet.write(*id);
                packet.write(snake.head.0 as u8);
                packet.write(snake.head.1 as u8);
            }

            let packet = packet.build();
            let mut clients = clients.write().unwrap();

            for client in clients.values_mut() {
                let _ = client.write_all(packet);

                println!("DEBUG: Sending packet {:?}", packet);
            }
        }

        sleep(Duration::from_secs_f32(TICK_INTERVAL));
    });
}

fn client_read(
    poll: &mut Poll,
    context: Arc<RwLock<GameContext>>,
    clients: Arc<RwLock<HashMap<Token, TcpStream>>>,
    token: Token,
) {
    let clients_map = clients.read().unwrap();
    let mut client = clients_map.get(&token).unwrap();

    let mut buffer = [0; 128];
    let snake_id = token.0 as u8;

    let buff_size = match client.read(&mut buffer) {
        Ok(0) => {
            println!("INFO: Client disconnected, token = {}", token.0);

            // Unlock clients
            drop(clients_map);

            let mut clients = clients.write().unwrap();
            let mut context = context.write().unwrap();

            let mut disconnected_stream = clients.remove(&token).unwrap();
            let _ = poll.registry().deregister(&mut disconnected_stream);

            context.kill_snake(token.0 as u8);

            let mut packet = Packet::with_capacity(2);
            packet.write(0x6);
            packet.write(snake_id);
            let packet = packet.build();

            for client in clients.values_mut() {
                let _ = client.write_all(packet);
            }
            return;
        }
        Ok(n) => n,
        Err(err) => {
            eprintln!("Read failed: {err}");
            return;
        }
    };

    println!("DEBUG: Message received: {:?}", &buffer[..buff_size]);

    let mut offset = 2;

    while offset < buff_size {
        if buffer[offset] == 0x3 {
            let direction = match buffer[offset + 1] {
                0x1 => Some(Direction::Up),
                0x2 => Some(Direction::Down),
                0x3 => Some(Direction::Left),
                0x4 => Some(Direction::Right),
                _ => None,
            };

            if let Some(direction) = direction {
                let mut context = context.write().unwrap();

                context
                    .snakes
                    .get_mut(&snake_id)
                    .unwrap()
                    .change_direction(direction);
            }
        }

        offset += 2;
    }
}

fn send_fullstate(stream: &mut TcpStream, context: &Arc<RwLock<GameContext>>) -> io::Result<()> {
    let context = Arc::clone(context);
    let context = context.read().unwrap();

    let mut packet = Packet::new();
    packet.write(0x1);

    for (id, snake) in context.snakes.iter() {
        packet.write(*id);
        packet.write_u16_le(snake.body.len() as u16 + 1);

        for Point(x, y) in snake.body.iter() {
            packet.write(*x as u8);
            packet.write(*y as u8);
        }

        packet.write(snake.head.0 as u8);
        packet.write(snake.head.1 as u8);
    }

    packet.write(0xff);
    packet.write(context.food.0 as u8);
    packet.write(context.food.1 as u8);

    drop(context);

    let packet = packet.build();

    println!("DEBUG: Sending initial packet: {:?}", packet);

    stream.write_all(packet)?;

    Ok(())
}

fn broadcast_snake(
    clients: &mut RwLockWriteGuard<HashMap<Token, TcpStream>>,
    snake_id: u8,
    snake: &Snake,
) {
    let mut packet = Packet::with_capacity(6);
    packet.write(0x5);
    packet.write(snake_id);
    packet.write(snake.body.len() as u8 + 1);

    for Point(x, y) in snake.body.iter() {
        packet.write(*x as u8);
        packet.write(*y as u8);
    }

    packet.write(snake.head.0 as u8);
    packet.write(snake.head.1 as u8);

    let packet = packet.build();

    for client in clients.values_mut() {
        let _ = client.write_all(packet);
    }
}

fn main() -> io::Result<()> {
    let context = Arc::new(RwLock::new(GameContext::new()));

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    let mut counter = 0;

    let clients: Arc<RwLock<HashMap<Token, TcpStream>>> = Arc::new(RwLock::new(HashMap::new()));

    let mut listener = TcpListener::bind(format!("0.0.0.0:{PORT}").parse().unwrap())?;

    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    println!("INFO: TCP Socket listening to port {PORT}");

    setup_gameloop(&context, &clients);

    loop {
        if let Err(err) = poll.poll(&mut events, None) {
            eprintln!("Failed to poll: {err}");
            continue;
        }

        for token in events.iter().map(|event| event.token()) {
            match token {
                Token(0) => match listener.accept() {
                    Ok((mut stream, client_addr)) => {
                        let clients = Arc::clone(&clients);
                        let clients_map = clients.read().unwrap();

                        if clients_map.len() > 8 {
                            let _ = stream.write_all(&[0x7]);
                            continue;
                        }

                        // Release the lock
                        drop(clients_map);

                        counter += 1;
                        let token = Token(counter);

                        match poll
                            .registry()
                            .register(&mut stream, token, Interest::READABLE)
                        {
                            Ok(_) => {
                                let context = Arc::clone(&context);
                                let mut clients = clients.write().unwrap();

                                {
                                    let mut context = context.write().unwrap();

                                    context.spawn_snake(counter as u8);
                                    broadcast_snake(
                                        &mut clients,
                                        counter as u8,
                                        context.snakes.get(&(counter as u8)).unwrap(),
                                    );
                                }

                                if send_fullstate(&mut stream, &context).is_ok() {
                                    clients.insert(token, stream);

                                    println!(
                                        "INFO: Client {client_addr} connected, with token {counter}!"
                                    );
                                }
                            }
                            Err(err) => eprintln!("Could not register stream {err}"),
                        }
                    }
                    Err(err) => eprintln!("Accept call failed: {err}"),
                },
                token => {
                    let context = Arc::clone(&context);
                    let clients = Arc::clone(&clients);

                    client_read(&mut poll, context, clients, token);
                }
            }
        }
    }
}
