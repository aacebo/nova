use crate::{Action, Args, call};

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
    fn invoke(&self, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
        call!(&self.entrypoint, args.clone());
        Ok(())
    }
}
