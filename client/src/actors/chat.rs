use async_trait::async_trait;

use crate::actors::{Actor, HandlerResult, WsMessageSender};

// Chat will always use 4042 port

enum Sender {
    Hacker,
    Victim,
}

struct Message {
    sender: Sender,
    text: String,
}
pub struct Chat {
    messages: Vec<Message>,
}

#[async_trait]
impl Actor for Chat {
    fn new() -> Self {
        Self { messages: vec![] }
    }

    async fn handler(&mut self, _: serde_json::Value, _: WsMessageSender) -> HandlerResult {
        Ok(())
    }
}

impl Chat {
    fn start(&self) {}
}
