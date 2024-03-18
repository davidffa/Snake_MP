mod game;
mod util;

use std::{
    collections::HashMap,
    io::{self, ErrorKind, Read, Write},
    net::Shutdown,
    sync::{Arc, RwLock, RwLockWriteGuard},
    thread::{self, sleep},
    time::Duration,
};

use common::packet::{PacketBuilder, PacketType, ReadablePacket};
use game::Snake;
use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use util::Point;

use crate::game::{Direction, GameContext};

const PORT: u16 = 14300;
const TICK_INTERVAL: f32 = 0.05;
const SERVER: Token = Token(0);
const MAX_PLAYERS: usize = 8;

fn setup_gameloop(
    context: &Arc<RwLock<GameContext>>,
    clients: &Arc<RwLock<HashMap<Token, TcpStream>>>,
) {
    let context = Arc::clone(context);
    let clients = Arc::clone(clients);

    thread::spawn(move || loop {
        {
            let mut context = context.write().unwrap();

            let (snake_id, killed_snakes) = context.update();

            if let Some(killed_snakes) = killed_snakes {
                let mut clients = clients.write().unwrap();

                for snake in killed_snakes.iter() {
                    // The snake kill itself will be handled in client_read

                    let _ = clients
                        .get_mut(&Token(*snake as usize))
                        .unwrap()
                        .shutdown(Shutdown::Both);
                }
            }

            if let Some(snake_id) = snake_id {
                let mut clients = clients.write().unwrap();

                let mut packet = PacketBuilder::with_capacity(PacketType::FoodUpdate, 3);
                packet.write(snake_id);
                packet.write(context.food.0 as u8);
                packet.write(context.food.1 as u8);
                let packet = packet.build();

                for client in clients.values_mut() {
                    let _ = client.write_all(&packet);
                }
            }
        }

        {
            let context = context.read().unwrap();

            let mut packet = PacketBuilder::new(PacketType::HeadUpdate);

            for (id, snake) in context.snakes.iter() {
                packet.write(*id);
                packet.write(snake.head.0 as u8);
                packet.write(snake.head.1 as u8);
            }

            let packet = packet.build();
            let mut clients = clients.write().unwrap();

            for client in clients.values_mut() {
                let _ = client.write_all(&packet);

                // println!("DEBUG: Sending packet {:?}", packet);
            }
        }

        sleep(Duration::from_secs_f32(TICK_INTERVAL));
    });
}

fn disconnect_client(
    snake_id: u8,
    poll: &mut Poll,
    context: &Arc<RwLock<GameContext>>,
    clients: &Arc<RwLock<HashMap<Token, TcpStream>>>,
    token: Token,
) {
    println!("INFO: Client disconnected, token = {}", token.0);

    let mut clients = clients.write().unwrap();
    let mut context = context.write().unwrap();

    let mut disconnected_stream = clients.remove(&token).unwrap();
    let _ = poll.registry().deregister(&mut disconnected_stream);

    context.kill_snake(token.0 as u8);

    let mut packet = PacketBuilder::with_capacity(PacketType::SnakeDisconnect, 1);
    packet.write(snake_id);
    let packet = packet.build();

    for client in clients.values_mut() {
        let _ = client.write_all(&packet);
    }
}

fn client_read(
    poll: &mut Poll,
    context: Arc<RwLock<GameContext>>,
    clients: Arc<RwLock<HashMap<Token, TcpStream>>>,
    token: Token,
) {
    let clients_map = clients.read().unwrap();
    let mut client = clients_map.get(&token).unwrap();

    let mut size_bytes = [0u8; 2];
    let snake_id = token.0 as u8;

    if let Err(err) = client.read_exact(&mut size_bytes) {
        if err.kind() != ErrorKind::UnexpectedEof {
            eprintln!("ERROR: Failed to read from client {snake_id}: {err}");
        }

        // Unlock clients
        drop(clients_map);

        disconnect_client(snake_id, poll, &context, &clients, token);
        return;
    }

    // println!("DEBUG: Message received: {:?}", &buffer[..buff_size]);

    let buffer_size = u16::from_le_bytes(size_bytes) as usize;

    if buffer_size == 0 {
        eprintln!("WARN: Empty packet received from {snake_id}");
        return;
    }

    let mut buffer = vec![0; buffer_size];

    if let Err(err) = client.read_exact(&mut buffer) {
        eprintln!("ERROR: Failed to read from TCP Stream (buffer) {snake_id}: {err}");
        return;
    }

    let mut packet = ReadablePacket::from_bytes(&buffer);

    match packet.r#type {
        PacketType::DirectionUpdate => {
            let direction = match packet.read() {
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
        },
        _ => {
            eprintln!("WARN: Invalid packet type received from {snake_id}");
        }
    };
}

fn send_fullstate(
    snake_id: u8,
    stream: &mut TcpStream,
    context: &Arc<RwLock<GameContext>>,
) -> io::Result<()> {
    let context = Arc::clone(context);
    let context = context.read().unwrap();

    let mut packet = PacketBuilder::new(PacketType::Info);
    packet.write(snake_id);

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

    // println!("DEBUG: Sending initial packet: {:?}", packet);

    stream.write_all(&packet)?;

    Ok(())
}

fn broadcast_snake(
    clients: &mut RwLockWriteGuard<HashMap<Token, TcpStream>>,
    snake_id: u8,
    snake: &Snake,
) {
    let mut packet = PacketBuilder::with_capacity(PacketType::SnakeConnect, 5);
    packet.write(snake_id);
    packet.write_u16_le(snake.body.len() as u16 + 1);

    for Point(x, y) in snake.body.iter() {
        packet.write(*x as u8);
        packet.write(*y as u8);
    }

    packet.write(snake.head.0 as u8);
    packet.write(snake.head.1 as u8);

    let packet = packet.build();

    for (id, client) in clients.iter_mut() {
        if id.0 as u8 == snake_id {
            continue;
        }

        // println!("DEBUG: Sending packet {:?}", packet);
        let _ = client.write_all(&packet);
    }
}

fn main() -> io::Result<()> {
    let context = Arc::new(RwLock::new(GameContext::new()));

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    let mut snake_id = 1;

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

                        if clients_map.len() > MAX_PLAYERS {
                            let _ = stream.write_all(&[1, 0, 0x7]);
                            continue;
                        }

                        while clients_map.contains_key(&Token(snake_id)) {
                            snake_id = (snake_id + 1) % 254;
                        }

                        // Release the lock
                        drop(clients_map);

                        let token = Token(snake_id);

                        match poll
                            .registry()
                            .register(&mut stream, token, Interest::READABLE)
                        {
                            Ok(_) => {
                                let context = Arc::clone(&context);
                                let mut clients = clients.write().unwrap();

                                {
                                    let mut context = context.write().unwrap();

                                    context.spawn_snake(snake_id as u8);
                                    broadcast_snake(
                                        &mut clients,
                                        snake_id as u8,
                                        context.snakes.get(&(snake_id as u8)).unwrap(),
                                    );
                                }

                                if send_fullstate(snake_id as u8, &mut stream, &context).is_ok() {
                                    clients.insert(token, stream);

                                    println!(
                                        "INFO: Client {client_addr} connected, with token {snake_id}!"
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
