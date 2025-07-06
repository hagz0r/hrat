use crate::handlers::func::Function;

pub struct TaskManager;
impl Function for TaskManager {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        todo!()
    }
}
