use async_trait::async_trait;
use serde_json::Value;

use crate::{
    handlers::func::{Function, HandlerResult},
    router::SocketWriter,
};

pub struct RemoteCodeExecution;

#[async_trait]
impl Function for RemoteCodeExecution {
    async fn handler<'a>(args: Value, socket: &'a mut SocketWriter) -> HandlerResult {
        todo!()
    }
}
