use tungstenite::connect;

use crate::Connection;

// Chat will always use 4042 port
pub fn handle_chat(payload: &[u8], connection: &Connection) {
	let url = format!("ws://{}:{}", connection.ip, 4042);
	let (socket, _) = connect(url).expect("Failed to connect");
}

enum Sender {
	Hacker,
	Victim,
}

struct Message {
	sender: Sender,
	text: String,
}

struct Chat {
	messages: Vec<Message>,
}

impl Chat {
	fn start(&self) {
	}
}