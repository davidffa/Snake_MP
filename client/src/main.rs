mod game;
mod renderer;
mod util;

use std::{thread::sleep, time::Duration};

use game::{Direction, GameContext};
use renderer::{Renderer, WINDOW_HEIGHT, WINDOW_WIDTH};
use sdl2::{event::Event, keyboard::Keycode};

const FPS: u32 = 20;

fn main() -> Result<(), ()> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Snake Multiplayer", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = Renderer::new(window)
        .map_err(|err| eprintln!("ERROR: Could not create the renderer: {err}"))?;

    let mut context = GameContext::new();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut next_direction = Direction::Right;

    'running: loop {
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

        context.update(next_direction);

        renderer
            .render(&context)
            .map_err(|err| eprintln!("ERROR: Error when rendering frame: {err}"))?;

        sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }

    Ok(())
}
