use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};
use async_trait::async_trait;

pub struct KeyLogger;

#[async_trait]
impl Actor for KeyLogger {
    fn new() -> Self {
        Self
    }
    async fn handler(&mut self, _args: Command, _socket: WsMessageSender) -> HandlerResult {
        Ok(())
    }
}
