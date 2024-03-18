mod game;
mod renderer;
mod util;

use std::{
    collections::VecDeque,
    env,
    io::{ErrorKind, Read, Write},
    net::TcpStream,
};

use common::packet::{PacketBuilder, PacketType, ReadablePacket};
use game::{Direction, GameContext, Snake};
use renderer::{Renderer, WINDOW_HEIGHT, WINDOW_WIDTH};
use sdl2::{event::Event, keyboard::Keycode};

use crate::{game::State, util::Point};

const ADDR: &str = "127.0.0.1:14300";

fn read_snake(packet: &mut ReadablePacket) -> Snake {
    let snake_sz = packet.read_u16_le() as usize;

    let mut body = VecDeque::with_capacity(snake_sz - 1);

    for _ in 0..snake_sz - 1 {
        body.push_back(Point(packet.read() as i32, packet.read() as i32));
    }

    let head = Point(packet.read() as i32, packet.read() as i32);

    Snake::new(body, head)
}

fn process_packet(packet: &mut ReadablePacket, context: &mut GameContext) {
    match packet.r#type {
        PacketType::Info => {
            context.snake_id = packet.read();
            let mut obj_type = packet.read();

            while obj_type != 0xff {
                // Read snakes
                let snake = read_snake(packet);

                context.snakes.insert(obj_type, snake);

                println!("INFO: Spawned snake {obj_type}");

                obj_type = packet.read();
            }

            context.food = Point(packet.read() as i32, packet.read() as i32);
            context.state = State::Playing;
        }
        PacketType::FoodUpdate => {
            let snake_id = packet.read();
            let snake = context.snakes.get_mut(&snake_id).unwrap();

            snake.body.push_front(snake.old_tail);

            context.food = Point(packet.read() as i32, packet.read() as i32);
        }
        PacketType::HeadUpdate => {
            while packet.remaining() > 0 {
                let snake_id = packet.read();

                let head = Point(packet.read() as i32, packet.read() as i32);

                let snake = context.snakes.get_mut(&snake_id).unwrap();

                snake.body.push_back(snake.head);
                snake.head = head;
                snake.old_tail = snake.body.pop_front().unwrap();
            }
        }
        PacketType::SnakeConnect => {
            let snake_id = packet.read();
            let snake = read_snake(packet);

            context.snakes.insert(snake_id, snake);
            println!("INFO: Spawned a new snake {snake_id}");
        }
        PacketType::SnakeDisconnect => {
            let snake_id = packet.read();

            context.snakes.remove(&snake_id);
        }
        _ => {
            eprintln!("WARN: Received unknown packet!");
        }
    }
}

fn read_packets(stream: &mut TcpStream, context: &mut GameContext) -> bool {
    let mut size_bytes = [0u8; 2];

    if let Err(err) = stream.read_exact(&mut size_bytes) {
        if err.kind() != ErrorKind::UnexpectedEof {
            eprintln!("Error reading the TCP stream {err}");
        }

        println!("INFO: Disconnected");
        return false;
    }

    let packet_size = u16::from_le_bytes(size_bytes) as usize;

    if packet_size == 0 {
        eprintln!("WARN: Received a empty packet");
        return false;
    }

    let mut buffer = vec![0; packet_size];

    if let Err(err) = stream.read_exact(&mut buffer) {
        eprintln!("Error reading the TCP stream (buffer) {err}");
        return false;
    }

    let mut packet = ReadablePacket::from_bytes(&buffer);

    if packet.r#type == PacketType::ConnRejected {
        println!("INFO: Server full!");
        return false;
    }

    process_packet(&mut packet, context);

    true
}

fn main() -> Result<(), ()> {
    let args: Vec<_> = env::args().collect();
    let server_addr = if args.len() >= 2 {
        &args[1]
    } else {
        println!("INFO: Server IP not provided, falling back to {ADDR}");
        ADDR
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Snake Multiplayer", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = Renderer::new(window).map_err(|err| {
        eprintln!("ERROR: Could not create the renderer: {err}");
    })?;
    let mut context = GameContext::new();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut stream = TcpStream::connect(server_addr).map_err(|err| {
        eprintln!("ERROR: Could not connect to the snake server: {err}");
    })?;

    println!("INFO: TCP Socket connected to {server_addr}");

    let mut old_dir = Direction::Right;
    let mut next_direction = Direction::Right;

    'running: loop {
        let res = read_packets(&mut stream, &mut context);

        if !res {
            break 'running;
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

        if old_dir != next_direction
            && ((old_dir == Direction::Up && next_direction != Direction::Down)
                || (old_dir == Direction::Down && next_direction != Direction::Up)
                || (old_dir == Direction::Left && next_direction != Direction::Right)
                || (old_dir == Direction::Right && next_direction != Direction::Left))
        {
            old_dir = next_direction;

            let dir: u8 = match next_direction {
                Direction::Up => 1,
                Direction::Down => 2,
                Direction::Left => 3,
                Direction::Right => 4,
            };

            let mut packet = PacketBuilder::with_capacity(PacketType::DirectionUpdate, 1);
            packet.write(dir);

            stream.write_all(&packet.build()).map_err(|err| {
                eprintln!("ERROR: Could not send TCP packet: {err}");
            })?;
        }

        renderer.render(&context).map_err(|err| {
            eprintln!("ERROR: Could not render a frame: {err}");
        })?;
    }

    Ok(())
}
