mod game;
mod packet;
mod renderer;
mod util;

use std::{collections::VecDeque, error::Error, io::Read};

use game::{Direction, GameContext, Snake};
use mio::{net::TcpStream, Events, Interest, Poll, Token};
use packet::Packet;
use renderer::{Renderer, WINDOW_HEIGHT, WINDOW_WIDTH};
use sdl2::{event::Event, keyboard::Keycode};

use crate::{game::State, util::Point};

const ADDR: &str = "127.0.0.1:14300";
const CLIENT: Token = Token(250);

fn read_snake(packet: &mut Packet) -> Snake {
    let snake_sz = packet.read_u16_le() as usize;

    let mut body = VecDeque::with_capacity(snake_sz - 1);

    for _ in 0..snake_sz - 1 {
        body.push_back(Point(packet.read() as i32, packet.read() as i32));
    }

    let head = Point(packet.read() as i32, packet.read() as i32);

    Snake::new(body, head)
}

fn read_packet(stream: &mut TcpStream, context: &mut GameContext) -> bool {
    let mut buffer = [0; 128];

    let bytes: Vec<u8> = match stream.read(&mut buffer) {
        Ok(0) => {
            println!("INFO: Disconnected!");
            return false;
        }
        Ok(n) => buffer[..n].to_vec(),
        Err(err) => {
            eprintln!("Error reading the tcp stream {err}");
            return false;
        }
    };

    println!("DEBUG: Packet received: {:?}", bytes);

    let mut packet = Packet::from(bytes);
    let ptype = packet.read();

    match ptype {
        0x1 => {
            let mut obj_type = packet.read();

            while obj_type != 0xff {
                // Read snakes
                let snake = read_snake(&mut packet);

                context.snakes.insert(obj_type, snake);

                println!("Inserting snake {obj_type}");

                obj_type = packet.read();
            }

            context.food = Point(packet.read() as i32, packet.read() as i32);

            context.state = State::Playing;
        }
        0x2 => {
            context.food = Point(packet.read() as i32, packet.read() as i32);
        }
        0x4 => {
            while packet.remaining() > 0 {
                let snake_id = packet.read();

                println!("Snake ID = {snake_id}");

                let head = Point(packet.read() as i32, packet.read() as i32);

                let snake = context.snakes.get_mut(&snake_id).unwrap();

                snake.body.push_back(snake.head);
                snake.head = head;
                snake.body.pop_front();
            }
        }
        _ => {
            eprintln!("WARN: Received unknown packet!");
        }
    }

    true
}

fn main() -> Result<(), Box<dyn Error>> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Snake Multiplayer", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = Renderer::new(window)?;
    let mut context = GameContext::new();

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut stream = TcpStream::connect(ADDR.parse().unwrap())?;
    poll.registry()
        .register(&mut stream, CLIENT, Interest::READABLE)?;

    println!("INFO: TCP Socket connected to {ADDR}");

    let mut next_direction = Direction::Right;

    'running: loop {
        if let Err(err) = poll.poll(&mut events, None) {
            eprintln!("Failed to poll: {err}");
            continue;
        }

        for _ in events.iter() {
            let res = read_packet(&mut stream, &mut context);

            if !res {
                break 'running;
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    next_direction = match keycode {
                        Keycode::W | Keycode::Up => Direction::Up,
                        Keycode::A | Keycode::Left => Direction::Left,
                        Keycode::S | Keycode::Down => Direction::Down,
                        Keycode::D | Keycode::Right => Direction::Right,
                        _ => next_direction,
                    }
                }
                _ => {}
            }
        }

        // TODO: Send direction update if needed
        renderer.render(&context)?;
    }

    Ok(())
}
