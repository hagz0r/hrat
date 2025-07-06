use tungstenite::connect;

// Chat will always use 4042 port
use crate::handlers::func::Function;

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

impl Function for Chat {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        let url = format!("ws://{}:{}", ctx.conn.ip, 4042);
        let (socket, _) = connect(url).expect("Failed to connect");
        Ok(())
    }
}

impl Chat {
    fn start(&self) {}
}
