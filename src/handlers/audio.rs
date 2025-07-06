use crate::handlers::func::Function;

pub struct Audio;
impl Function for Audio {
    fn handler(payload: &[u8], ctx: &mut super::func::Context) -> anyhow::Result<()> {
        todo!()
    }
}
