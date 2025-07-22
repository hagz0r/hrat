use async_trait::async_trait;

use crate::actors::{Actor, HandlerResult, WsMessageSender};

// Chat will always use 4042 port

pub struct Chat;
#[async_trait]
impl Actor for Chat {
    fn new() -> Self {
        Self
    }

    async fn handler(&mut self, _: serde_json::Value, _: WsMessageSender) -> HandlerResult {
        Ok(())
    }
}
