use std::{
    io,
    net::{Ipv4Addr, TcpListener},
    sync::{Arc, RwLock},
    thread::{self, sleep},
    time::Duration,
};

use crate::game::GameContext;

mod game;
mod util;

const PORT: u16 = 14300;
const TICK_INTERVAL: f32 = 0.05;

fn setup_gameloop(context: &Arc<RwLock<GameContext>>) {
    let context = Arc::clone(context);

    thread::spawn(move || loop {
        {
            let mut context = context.write().unwrap();
            context.update();
        }

        sleep(Duration::from_secs_f32(TICK_INTERVAL));
    });
}

fn main() -> io::Result<()> {
    let context = Arc::new(RwLock::new(GameContext::new()));
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, PORT))?;

    println!("TCP Socket listening to port {PORT}");

    setup_gameloop(&context);

    // TODO: Check how to use mio (https://github.com/tokio-rs/mio) and implement non blocking I/O tcp connections

    Ok(())
}
