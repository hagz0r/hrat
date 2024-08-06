use std::error::Error;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use lazy_static::lazy_static;
use scrap::{Capturer, Display};
use tungstenite::Message;
use tungstenite::protocol::WebSocket;
use tungstenite::stream::MaybeTlsStream;
use crate::Connection;

lazy_static! {
    static ref STREAM_STATE: Arc<Mutex<StreamState>> = Arc::new(Mutex::new(StreamState::new()));
}
pub struct StreamState {
	is_streaming: bool,
}

impl StreamState {
	fn new() -> Self {
		Self { is_streaming: false }
	}
}
pub fn handle_remote_screen(
	payload: &[u8],
	socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
	connection: &Connection
) -> Result<(), Box<dyn Error>> {
	update_streaming_state(payload)?;

	if STREAM_STATE.lock().unwrap().is_streaming {
		let display = Display::primary()?;
		let mut capturer = Capturer::new(display)?;

		loop {
			{
				let state = STREAM_STATE.lock().unwrap();
				if !state.is_streaming {
					break;
				}
			}
			capture_frame(&mut capturer, socket)?;
		}
	}

	Ok(())
}


fn update_streaming_state(payload: &[u8]) -> Result<(), Box<dyn Error>> {
	if payload.is_empty() {
		return Err("Payload is empty".into());
	}

	let mut state = STREAM_STATE.lock().unwrap();
	state.is_streaming = match payload[0] as char {
		'1'  => true,   
		'0'  => false,  
		_ => return Err("Invalid byte".into()),  
	};

	Ok(())
}

fn capture_frame(capturer: &mut Capturer, socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> Result<(), Box<dyn Error>> {
	let frame = match capturer.frame() {
		Ok(frame) => frame,
		Err(error) => {
			return if error.kind() == std::io::ErrorKind::WouldBlock {
				thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // 60 FPS
				Ok(())  
			} else {
				Err(Box::new(error))
			}
		}
	};

	socket.write(Message::Binary(frame.to_vec()))?;
	Ok(())
}

