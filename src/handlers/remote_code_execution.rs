use crate::handlers::func::Function;

pub struct RemoteCodeExecution;
impl Function for RemoteCodeExecution {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        todo!()
    }
}
