use crate::Socket;

pub struct Context<'a> {
    pub socket: &'a mut Socket,
    pub conn: &'a Connection,
}

pub trait Function {
    fn handler(payload: &[u8], ctx: &mut Context) -> anyhow::Result<()>;
}
