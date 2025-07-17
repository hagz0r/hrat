use async_trait::async_trait;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};

pub struct TaskManager;
#[async_trait]
impl Actor for TaskManager {
    fn new() -> Self {
        Self
    }
    async fn handler(&mut self, _command: Command, _writerr: WsMessageSender) -> HandlerResult {
        todo!()
    }
}
