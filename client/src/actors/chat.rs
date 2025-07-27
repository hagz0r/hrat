use std::sync::Arc;

use async_trait::async_trait;
use futures_util::lock::Mutex;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::actors::{Actor, HandlerResult, WsMessageSender};

type ActivityState = Arc<Mutex<bool>>;

#[derive(Clone, Copy)]
enum Author {
    Host,
    Client,
}

#[derive(Clone)]
struct Message {
    author: Author,
    text: String,
}

impl Message {
    pub fn from(text: String, author: Author) -> Self {
        Self { text, author }
    }
    pub fn display(messages: &Vec<Self>) {
        todo!();
    }
    pub fn to_json(&self) -> String {
        let author = match self.author {
            Author::Host => "host",
            Author::Client => "client",
        }
        .to_string();

        json!({
            "author" : author,
            "text": self.text})
        .to_string()
    }
}

pub struct Chat {
    is_active: ActivityState,
    messages: Vec<Message>,
}

#[async_trait]
impl Actor for Chat {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    async fn handler(&mut self, args: serde_json::Value, socket: WsMessageSender) -> HandlerResult {
        let action = args.get("action");
        if action.is_none() {
            return Err(anyhow::anyhow!("No action provided"));
        }

        let action = action.unwrap().as_str().unwrap_or("");

        let _res = match action {
            "start" => self.start().await,
            "stop" => self.stop().await,
            "post" => self.send(socket).await,
            "get" => self.get(args).await,
            _ => {
                return Err(anyhow::anyhow!("No known method"));
            }
        };

        Ok(())
    }
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            is_active: Arc::new(Mutex::new(false)),
            messages: Vec::new(),
        }
    }
}

impl Chat {
    async fn start(&mut self) -> anyhow::Result<()> {
        if *self.is_active.lock().await {
            return Err(anyhow::anyhow!("Chat is already active"));
        }
        *self.is_active.lock().await = true;
        crate::dev_print!("New chat started");

        SystemRenderer::start()
    }
    async fn stop(&mut self) -> anyhow::Result<()> {
        if !*self.is_active.lock().await {
            return Err(anyhow::anyhow!("No existing chat to be stopped"));
        }
        *self.is_active.lock().await = false;
        crate::dev_print!("Chat stopped");

        SystemRenderer::stop()
    }

    async fn send(&mut self, socket: WsMessageSender) -> anyhow::Result<()> {
        if !*self.is_active.lock().await {
            return Err(anyhow::anyhow!("No existing chat to send the message"));
        }

        let message_text = "Penis".to_string(); // decide how to give an interface for writing the message from the client
        let message = Message::from(message_text.clone(), Author::Client);

        self.messages.push(message.clone());

        // send to new message to the host
        socket.blocking_send(WsMessage::Text(message.to_json()))?;
        crate::dev_print!("New message {} sent", message.to_json());

        SystemRenderer::update()
    }

    async fn get(&mut self, args: serde_json::Value) -> anyhow::Result<()> {
        if !*self.is_active.lock().await {
            return Err(anyhow::anyhow!("No existing chat to get the message"));
        }

        let message_text = args.get("message").unwrap().to_string();
        let message = Message::from(message_text, Author::Host);

        crate::dev_print!("New message {} got", message.to_json());

        self.messages.push(message);

        SystemRenderer::update()
    }
}

#[cfg(target_os = "windows")]
type SystemRenderer = WindowsRenderer;

#[cfg(not(target_os = "windows"))]
type SystemRenderer = LinuxRenderer;

#[allow(dead_code)]
trait ChatRenderer {
    fn start() -> anyhow::Result<()>;
    fn update() -> anyhow::Result<()>;
    fn stop() -> anyhow::Result<()>;
}

#[allow(dead_code)]
struct WindowsRenderer;
#[allow(unused_variables)]
impl ChatRenderer for WindowsRenderer {
    fn start() -> anyhow::Result<()> {
        todo!()
    }

    fn update() -> anyhow::Result<()> {
        todo!()
    }

    fn stop() -> anyhow::Result<()> {
        todo!()
    }
}
#[allow(dead_code)]
struct LinuxRenderer;
#[allow(unused_variables)]
impl ChatRenderer for LinuxRenderer {
    fn start() -> anyhow::Result<()> {
        todo!()
    }

    fn update() -> anyhow::Result<()> {
        todo!()
    }

    fn stop() -> anyhow::Result<()> {
        todo!()
    }
}
