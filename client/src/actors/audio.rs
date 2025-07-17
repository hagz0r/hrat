use async_trait::async_trait;

use crate::actors::{Actor, HandlerResult, WsMessageSender};

pub struct Audio;

#[async_trait]
impl Actor for Audio {
    fn new() -> Self {
        Self
    }

    async fn handler(&mut self, _: serde_json::Value, _: WsMessageSender) -> HandlerResult {
        Ok(())
    }
}
