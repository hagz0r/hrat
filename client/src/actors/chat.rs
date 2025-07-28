use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

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
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            messages: Arc::new(Mutex::new(Vec::new())),
            renderer: None,
        }
    }
}

#[async_trait]
impl Actor for Chat {
    fn new() -> Self {
        Self::default()
    }

    async fn handler(&mut self, args: Command, _socket: WsMessageSender) -> HandlerResult {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!(
                "Chat is not active. Send 'start' action first."
            ));
        }

        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("'action' is required"))?;

        match action {
            "send" => {
                let message_text = args
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("'message' is required for 'send' action"))?
                    .to_string();

                let message = Message::new(message_text, Author::Host);
                crate::dev_print!("New message from host: {:?}", message);
                self.messages.lock().unwrap().push(message);
            }
            "stop" => {
                self.stop().await?;
            }
            _ => return Err(anyhow::anyhow!("Unknown chat action: {}", action)),
        }

        Ok(())
    }
}

impl Chat {
    pub async fn process_gui_message(
        &self,
        gui_message: String,
        socket: &WsMessageSender,
    ) -> HandlerResult {
        if !self.is_active.load(Ordering::Relaxed) {
            return Ok(());
        }

        let message = Message::new(gui_message, Author::Client);
        self.messages.lock().unwrap().push(message.clone());

        let response = json!({
            "type": "chat_message",
            "author": "client",
            "text": message.text
        });

        socket.send(WsMessage::Text(response.to_string())).await?;
        crate::dev_print!("Sent message from GUI to host: {}", response.to_string());

        Ok(())
    }

    pub async fn start(&mut self) -> anyhow::Result<mpsc::Receiver<String>> {
        if self.is_active.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Chat is already active"));
        }
        self.is_active.store(true, Ordering::Relaxed);
        self.messages.lock().unwrap().clear();

        let (gui_sender, gui_receiver) = mpsc::channel();

        let mut renderer = CrossPlatformRenderer::new();
        renderer.start(Arc::clone(&self.messages), gui_sender)?;
        self.renderer = Some(renderer);

        crate::dev_print!("Chat started and GUI renderer is running.");
        Ok(gui_receiver)
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if !self.is_active.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Chat is not active"));
        }
        self.is_active.store(false, Ordering::Relaxed);

        if let Some(mut renderer) = self.renderer.take() {
            renderer.stop()?;
        }
        self.messages.lock().unwrap().clear();

        crate::dev_print!("Chat stopped.");
        Ok(())
    }
}
