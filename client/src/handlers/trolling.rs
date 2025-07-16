use async_trait::async_trait;

use crate::{
    handlers::func::{Function, HandlerResult},
    router::SocketWriter,
};

pub struct Trolling;

#[async_trait]
impl Function for Trolling {
    async fn handler(args: serde_json::Value, socket: &mut SocketWriter) -> HandlerResult {
        todo!()
    }
}
