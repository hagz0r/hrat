use async_trait::async_trait;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};

pub struct RemoteScreen;

#[async_trait]

impl Actor for RemoteScreen {
    fn new() -> Self {
        Self
    }
    async fn handler(&mut self, _command: Command, _writerr: WsMessageSender) -> HandlerResult {
        todo!()
        // let url = format!("ws://{}:4042", ctx.conn.ip);
        // let (mut socket, _response) = connect(&url).unwrap();
        // println!("Socket connected");

        // Ok(())
    }
}

// fn start_streaming_new(socket: &mut Socket) {
//     let display = Display::primary().unwrap();
//     let mut capturer = Capturer::new(display).unwrap();
//     loop {
//         match capturer.frame() {
//             Ok(frame) => {
//                 socket.send(Message::Binary(frame.to_vec())).unwrap();
//             }
//             Err(error) => {
//                 if error.kind() == std::io::ErrorKind::WouldBlock {
//                     std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
//                     continue;
//                 }
//             }
//         }
//     }
// }
