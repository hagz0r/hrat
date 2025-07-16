use async_trait::async_trait;
use serde_json::Value;

use crate::{
    handlers::func::{Function, HandlerResult},
    router::SocketWriter,
};

pub struct KeyLogger;

#[async_trait]
impl Function for KeyLogger {
    async fn handler<'a>(args: Value, socket: &'a mut SocketWriter) -> HandlerResult {
        todo!()
    }
}
