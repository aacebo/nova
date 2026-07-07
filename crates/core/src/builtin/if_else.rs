use crate::{Action, Context};

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
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
        let args = ctx.args().clone();

        if ctx.call(&self.condition, args.clone())?.map(|v| v.is_true()).unwrap_or(false) {
            ctx.call(&self.then_branch, args)?;
        } else if let Some(branch) = &self.else_branch {
            ctx.call(branch, args)?;
        }

        Ok(())
    }
}
