mod game;
mod renderer;
mod util;

use std::{thread::sleep, time::Duration};

use game::GameContext;
use renderer::Renderer;
use sdl2::{event::Event, keyboard::Keycode};

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

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
                } => match keycode {
                    Keycode::W => context.snake.move_up(),
                    Keycode::A => context.snake.move_left(),
                    Keycode::S => context.snake.move_down(),
                    Keycode::D => context.snake.move_right(),
                    _ => {}
                },
                _ => {}
            }
        }

        context.update();

        renderer
            .render(&context)
            .map_err(|err| eprintln!("ERROR: Error when rendering frame: {err}"))?;

        sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }

    Ok(())
}
