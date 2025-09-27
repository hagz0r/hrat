use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::{
    actors::{Actor, Command, HandlerResult, WsMessageSender},
    gui::CrossPlatformRenderer,
};

#[derive(Clone, Copy, Debug)]
pub enum Author {
    Host,
    Client,
}

#[derive(Clone, Debug)]
pub struct Message {
    pub author: Author,
    pub text: String,
}
impl Message {
    pub fn new(text: String, author: Author) -> Self {
        Self { text, author }
    }
}

pub type SharedMessages = Arc<Mutex<Vec<Message>>>;

pub struct Chat {
    is_active: Arc<AtomicBool>,
    messages: SharedMessages,
    renderer: Option<CrossPlatformRenderer>,

    // новое
    ws: Option<WsMessageSender>,
    stop_flag: Arc<AtomicBool>,
    bg_handle: Option<JoinHandle<()>>,
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            messages: Arc::new(Mutex::new(Vec::new())),
            renderer: None,

            ws: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
            bg_handle: None,
        }
    }
}

#[async_trait]
impl Actor for Chat {
    fn new() -> Self {
        Self::default()
    }

    async fn handler(&mut self, args: Command, writer: WsMessageSender) -> HandlerResult {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("'action' is required"))?;

        match action {
            "start" => {
                if self.is_active.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Chat is already active"));
                }
                self.start(writer).await?;
            }
            "send" => {
                if !self.is_active.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Chat is not active. Send 'start' first."));
                }
                let message_text = args
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("'message' is required for 'send'"))?
                    .to_string();

                let message = Message::new(message_text, Author::Host);
                self.messages.lock().unwrap().push(message);
            }
            "stop" => {
                if !self.is_active.load(Ordering::Relaxed) {
                    return Err(anyhow::anyhow!("Chat is not active"));
                }
                self.stop().await?;
            }
            _ => return Err(anyhow::anyhow!("Unknown chat action: {}", action)),
        }
        Ok(())
    }
}

impl Chat {
    pub async fn start(&mut self, ws: WsMessageSender) -> anyhow::Result<()> {
        self.is_active.store(true, Ordering::Relaxed);
        self.stop_flag.store(false, Ordering::Relaxed);
        self.ws = Some(ws);

        self.messages.lock().unwrap().clear();

        let (gui_sender, gui_receiver) = mpsc::channel();
        let mut renderer = CrossPlatformRenderer::new();
        renderer.start(Arc::clone(&self.messages), gui_sender)?;
        self.renderer = Some(renderer);

        let stop = self.stop_flag.clone();
        let ws_sender = self.ws.as_ref().unwrap().clone();
        let messages = Arc::clone(&self.messages);

        let handle = thread::spawn(move || {
            use std::sync::mpsc::TryRecvError;
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }

                match gui_receiver.try_recv() {
                    Ok(text) => {
                        messages
                            .lock()
                            .unwrap()
                            .push(Message::new(text.clone(), Author::Client));

                        let payload = json!({
                            "type": "chat_message",
                            "author": "client",
                            "text": text
                        })
                        .to_string();

                        let _ = ws_sender.blocking_send(WsMessage::Text(payload));
                    }
                    Err(TryRecvError::Empty) => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(TryRecvError::Disconnected) => break,
                }
            }
        });
        self.bg_handle = Some(handle);

        crate::dev_print!("Chat started and GUI renderer is running.");
        Ok(())
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        self.stop_flag.store(true, Ordering::Relaxed);

        if let Some(mut renderer) = self.renderer.take() {
            renderer.stop()?;
        }
        if let Some(h) = self.bg_handle.take() {
            let _ = h.join();
        }

        self.is_active.store(false, Ordering::Relaxed);
        self.ws = None;
        self.messages.lock().unwrap().clear();

        crate::dev_print!("Chat stopped.");
        Ok(())
    }
}
