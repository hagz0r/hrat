use async_trait::async_trait;

use crate::{
    handlers::func::{Function, HandlerResult},
    router::SocketWriter,
};

pub struct TaskManager;
#[async_trait]
impl Function for TaskManager {
    async fn handler(args: serde_json::Value, socket: &mut SocketWriter) -> HandlerResult {
        todo!()
    }
}
