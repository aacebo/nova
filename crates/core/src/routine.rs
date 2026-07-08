use crate::{Action, KArgs, Value, scope};

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
    fn invoke(&self, args: &[Value], kargs: &KArgs) -> Result<(), Box<dyn std::error::Error>> {
        scope().call(&self.entrypoint, args.to_vec(), kargs.clone())?;
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
    fn invoke(&self, args: &[Value], kargs: &KArgs) -> Result<(), Box<dyn std::error::Error>> {
        for step in &self.steps {
            scope().call(step, args.to_vec(), kargs.clone())?;
        }

        Ok(())
    }
}
