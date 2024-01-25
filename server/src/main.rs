use std::{
    io,
    net::{Ipv4Addr, UdpSocket},
    sync::{Arc, Mutex},
    thread,
};

use crate::game::GameContext;

mod game;
mod util;

const PORT: u16 = 14300;

fn receive(socket: Arc<UdpSocket>, context: Arc<Mutex<GameContext>>) {
    let mut buf = [0; 16384];

    'reading: loop {
        // TODO: Better error handling
        let (amt, src) = socket.recv_from(&mut buf).unwrap();

        let buf = &buf[..amt];

        println!("Received bytes: {:?} from {src}", buf);

        match buf[0] {
            0x1 => {
                println!("Requested a new stuff");
            }
            0x2 => {
                println!("x2 printed");
            }
            0x3 => {
                println!("Shutting down!");
                break 'reading;
            }
            _ => {}
        }
    }
}

fn main() -> io::Result<()> {
    let context = Arc::new(Mutex::new(GameContext::new()));
    let socket = Arc::new(UdpSocket::bind((Ipv4Addr::UNSPECIFIED, PORT))?);

    println!("UDP Socket bound to port {PORT}");

    let reader = thread::spawn({
        let context = Arc::clone(&context);
        let socket = Arc::clone(&socket);

        move || {
            receive(socket, context);
        }
    });

    reader.join().unwrap();

    Ok(())
}
