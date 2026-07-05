use crate::{Action, Context};

pub struct Routine {
    entrypoint: String,
}

impl Routine {
    pub fn new(entrypoint: impl Into<String>) -> Self {
        Self {
            entrypoint: entrypoint.into(),
        }
    }
}

impl Action for Routine {
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
        let args = ctx.args().clone();
        ctx.call(&self.entrypoint, args)
    }
}
