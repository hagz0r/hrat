use crate::handlers::func::Function;

pub struct Trolling;

impl Function for Trolling {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        todo!()
    }
}
