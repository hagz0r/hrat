use tungstenite::connect;

use crate::Connection;

// Chat will always use 4044 port
pub fn handle_chat(payload: &[u8], connection: &Connection) {
	let url = format!("ws://{}:{}", connection.ip, 4044);
	let (socket, _) = connect(url).expect("Failed to connect");
}
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::protocol::Message;
use tungstenite::connect;
use winapi::um::winuser::*;
use tokio::sync::mpsc;
use tokio::task;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

enum Sender {
    Master,
    Slave,
}

struct ChatMessage {
    sender: Sender,
    text: String,
}

struct Chat {
    messages: Vec<ChatMessage>,
    tx: mpsc::Sender<String>, 
}

impl Chat {
    fn new(tx: mpsc::Sender<String>) -> Self {
        Chat {
            messages: Vec::new(),
            tx,
        }
    }

    // Add message to the chat history
    fn add_message(&mut self, sender: Sender, text: String) {
        self.messages.push(ChatMessage { sender, text });
    }

    // Start the chat window (with WinAPI)
    fn start(&self) {
        let chat = Arc::new(Mutex::new(self.clone()));
        loop {
            // Spawn a thread to create and handle the chat window
            let chat = Arc::clone(&chat);
            let window_created = thread::spawn(move || {
                unsafe {
                    let class_name = to_wstring("Chatix");
                    let window_name = to_wstring("Chatix");

                    let hinstance = GetModuleHandleW(std::ptr::null());
                    let wnd_class = WNDCLASSW {
                        style: CS_HREDRAW | CS_VREDRAW,
                        lpfnWndProc: Some(window_proc),
                        hInstance: hinstance,
                        lpszClassName: class_name.as_ptr(),
                        ..Default::default()
                    };

                    RegisterClassW(&wnd_class);

                    let hwnd = CreateWindowExW(
                        0,
                        class_name.as_ptr(),
                        window_name.as_ptr(),
                        WS_OVERLAPPEDWINDOW,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        400,
                        300,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        hinstance,
                        std::ptr::null_mut(),
                    );

                    ShowWindow(hwnd, SW_SHOW);
                    UpdateWindow(hwnd);

                    // Event loop for handling messages and events
                    let mut msg = MSG {
                        ..Default::default()
                    };
                    while GetMessageW(&mut msg, hwnd, 0, 0) != 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }).join();
            
            // If the window was destroyed, recreate it
            if window_created.is_err() {
                continue;
            }
        }
    }
}

// Convert string to wide string (for WinAPI usage)
fn to_wstring(string: &str) -> Vec<u16> {
    let os_string = OsString::from(string);
    os_string.encode_wide().collect()
}

// WinAPI Window Procedure
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {

	#[warn(non_snake_case)]
    match msg {
        WM_DESTROY => {
            // Respond to window close by restarting the window
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// Handle WebSocket communication (in Tokio async)
async fn handle_chat(payload: &[u8], connection: &Connection, tx: mpsc::Sender<String>) {
    let url = format!("ws://{}:{}", connection.ip, 4044);
    let (mut socket, _) = connect(url).expect("Failed to connect");

    // Send initial message to the server (using WebSocket)
    socket.write_message(Message::Text(String::from_utf8_lossy(payload).to_string())).unwrap();

    // Listen for incoming messages from WebSocket
    loop {
        let message = socket.read_message().unwrap();
        match message {
            Message::Text(msg) => {
                // Handle incoming chat message and display it in the chat window
                tx.send(msg).await.unwrap();
            }
            _ => {}
        }
    }
}

// #[tokio::main]
// async fn main() {
//     // Create a channel for communication between WebSocket and the GUI
//     let (tx, mut rx) = mpsc::channel(32);

//     // Simulate the connection structure (you can replace this with actual data)
//     let connection = Connection { ip: String::from("127.0.0.1") };

//     // Spawn the WebSocket connection handler
//     tokio::spawn(handle_chat(b"Hello, world!", &connection, tx.clone()));

//     // Create and start the chat application window
//     let chat = Chat::new(tx);
//     chat.start();

//     // Handle receiving messages and updating the GUI (could update a text box or list)
//     while let Some(message) = rx.recv().await {
//         // Update the chat history with the incoming message
//         println!("Received message: {}", message); // For now, just print to the console
//     }
// }
