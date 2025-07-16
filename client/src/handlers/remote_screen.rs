use async_trait::async_trait;
use scrap::{Capturer, Display};
use serde_json::Value;

use crate::{
    handlers::func::{Function, HandlerResult},
    router::SocketWriter,
};

pub struct RemoteScreen;

#[async_trait]
impl Function for RemoteScreen {
    async fn handler<'a>(args: Value, socket: &'a mut SocketWriter) -> HandlerResult {
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
