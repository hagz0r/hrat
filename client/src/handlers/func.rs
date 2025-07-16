use crate::router::SocketWriter;
use async_trait::async_trait;

pub type HandlerResult = anyhow::Result<()>;

#[async_trait]
pub trait Function {
    async fn handler<'a>(args: serde_json::Value, socket: &'a mut SocketWriter) -> HandlerResult;
}
