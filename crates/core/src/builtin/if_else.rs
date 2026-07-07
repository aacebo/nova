use crate::{Action, Args, call};

pub struct If {
    condition: String,
    then_branch: String,
    else_branch: Option<String>,
}

impl If {
    pub fn new(condition: impl Into<String>, then: impl Into<String>) -> Self {
        Self {
            condition: condition.into(),
            then_branch: then.into(),
            else_branch: None,
        }
    }

    pub fn or_else(mut self, name: impl Into<String>) -> Self {
        self.else_branch = Some(name.into());
        self
    }
}

impl Action for If {
    fn invoke(&self, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
        if call!(&self.condition, args.clone()).map(|v| v.is_true()).unwrap_or(false) {
            call!(&self.then_branch, args.clone());
        } else if let Some(branch) = &self.else_branch {
            call!(branch, args.clone());
        }

        Ok(())
    }
}
