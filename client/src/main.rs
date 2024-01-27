mod game;
mod packet;
mod renderer;
mod util;

use std::{
    collections::VecDeque,
    error::Error,
    io::{Read, Write},
    net::TcpStream,
};

use game::{Direction, GameContext, Snake};
use packet::Packet;
use renderer::{Renderer, WINDOW_HEIGHT, WINDOW_WIDTH};
use sdl2::{event::Event, keyboard::Keycode};

use crate::{game::State, util::Point};

const ADDR: &str = "127.0.0.1:14300";

fn read_snake(packet: &mut Packet) -> Snake {
    let snake_sz = packet.read_u16_le() as usize;

    let mut body = VecDeque::with_capacity(snake_sz - 1);

    for _ in 0..snake_sz - 1 {
        body.push_back(Point(packet.read() as i32, packet.read() as i32));
    }

    let head = Point(packet.read() as i32, packet.read() as i32);

    Snake::new(body, head)
}

fn process_packet(bytes: Vec<u8>, context: &mut GameContext) {
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
            let snake_id = packet.read();
            let snake = context.snakes.get_mut(&snake_id).unwrap();

            snake.body.push_front(snake.old_tail);

            context.food = Point(packet.read() as i32, packet.read() as i32);
        }
        0x4 => {
            while packet.remaining() > 0 {
                let snake_id = packet.read();

                let head = Point(packet.read() as i32, packet.read() as i32);

                let snake = context.snakes.get_mut(&snake_id).unwrap();

                snake.body.push_back(snake.head);
                snake.head = head;
                snake.old_tail = snake.body.pop_front().unwrap();
            }
        }
        0x5 => {
            let snake_id = packet.read();
            let snake = read_snake(&mut packet);

            context.snakes.insert(snake_id, snake);
        }
        0x6 => {
            let snake_id = packet.read();

            context.snakes.remove(&snake_id);
        }
        _ => {
            eprintln!("WARN: Received unknown packet!");
        }
    }
}

fn read_packets(stream: &mut TcpStream, context: &mut GameContext) -> bool {
    let mut buffer = [0; 512];

    let buff_size = match stream.read(&mut buffer) {
        Ok(0) => {
            println!("INFO: Disconnected!");
            return false;
        }
        Ok(n) => n,
        Err(err) => {
            eprintln!("Error reading the tcp stream {err}");
            return false;
        }
    };

    // Server full
    if buffer[2] == 0x7 {
        return false;
    }

    let mut offset = 0;

    while offset < buff_size {
        let b0 = buffer[0] as usize;
        let b1 = buffer[1] as usize;

        let len = (b1 << 8) | b0;

        offset += 2;

        let bytes = buffer[offset..len + offset].to_vec();
        println!("DEBUG: Packet received: {:?}", bytes);

        process_packet(bytes, context);

        offset += len;
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

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut stream = TcpStream::connect(ADDR)?;

    println!("INFO: TCP Socket connected to {ADDR}");

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

            let mut packet = Packet::with_capacity(2);

            packet.write(0x3);
            packet.write(dir);

            stream.write_all(packet.build())?;
        }

        renderer.render(&context)?;
    }

    Ok(())
}
