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

pub struct Sequence {
    steps: Vec<String>,
}

impl Sequence {
    pub fn new(steps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            steps: steps.into_iter().map(|s| s.into()).collect(),
        }
    }
}

impl Action for Sequence {
    fn invoke(&self, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
        for step in &self.steps {
            call!(step, args.clone());
        }

        Ok(())
    }
}
