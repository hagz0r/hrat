use async_trait::async_trait;

// Chat will always use 4042 port
use crate::handlers::func::{Function, HandlerResult};

enum Sender {
    Hacker,
    Victim,
}

struct Message {
    sender: Sender,
    text: String,
}
pub struct Chat {
    messages: Vec<Message>,
}

#[async_trait]
impl Function for Chat {
    async fn handler(
        args: serde_json::Value,
        socket: &mut crate::router::SocketWriter,
    ) -> HandlerResult {
        todo!()
    }
}

impl Chat {
    fn start(&self) {}
}

pub(crate) async fn handler(
    args: serde_json::Value,
    socket: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        tokio_tungstenite::tungstenite::Message,
    >,
) -> Result<(), anyhow::Error> {
    todo!()
}
