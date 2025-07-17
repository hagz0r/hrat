use async_trait::async_trait;

use crate::actors::{Actor, Command, HandlerResult, WsMessageSender};

pub struct RemoteCodeExecution;

#[async_trait]
impl Actor for RemoteCodeExecution {
    fn new() -> Self {
        Self
    }
    async fn handler(&mut self, _command: Command, _writerr: WsMessageSender) -> HandlerResult {
        todo!()
    }
}
