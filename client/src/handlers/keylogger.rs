use crate::handlers::func::Function;

pub struct KeyLogger;
impl Function for KeyLogger {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        todo!()
    }
}
