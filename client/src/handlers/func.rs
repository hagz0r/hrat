use crate::{Connection, Socket};
pub struct Context<'a> {
    pub socket: &'a mut Socket,
    pub conn: &'a Connection,
}

impl<'a> Context<'a> {
    pub fn from(socket: &'a mut Socket, conn: &'a Connection) -> Self {
        Self { socket, conn }
    }
}

pub type HandlerFn = fn(payload: &[u8], ctx: &mut Context) -> anyhow::Result<()>;

pub trait Function {
    fn handler(payload: &[u8], ctx: &mut Context) -> anyhow::Result<()>;
}
