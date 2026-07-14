mod bridge;

use std::sync::Arc;

use bridge::{ContextObject, from_minijinja};

use crate::{Context, Engine, Error, Pointer};

pub struct Minijinja {
    env: minijinja::Environment<'static>,
}

impl Default for Minijinja {
    fn default() -> Self {
        Self::new()
    }
}

impl Minijinja {
    pub fn new() -> Self {
        Self {
            env: minijinja::Environment::new(),
        }
    }

    fn root(ctx: &Arc<dyn Context>) -> minijinja::Value {
        minijinja::Value::from_object(ContextObject::new(ctx.clone()))
    }
}

impl std::fmt::Debug for Minijinja {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Minijinja").finish_non_exhaustive()
    }
}

impl From<minijinja::Error> for Error {
    fn from(value: minijinja::Error) -> Self {
        let error = Error::message(value.to_string());

        match value.line() {
            Some(line) => error.at_line(line),
            None => error,
        }
    }
}

impl Engine for Minijinja {
    fn add_template(&mut self, name: &str, source: &str) -> Result<(), Error> {
        self.env.add_template_owned(name.to_string(), source.to_string())?;
        Ok(())
    }

    fn render(&self, name: &str, ctx: &Arc<dyn Context>) -> Result<String, Error> {
        let template = self.env.get_template(name)?;
        Ok(template.render(Self::root(ctx))?)
    }

    fn render_str(&self, source: &str, ctx: &Arc<dyn Context>) -> Result<String, Error> {
        Ok(self.env.render_str(source, Self::root(ctx))?)
    }

    fn eval(&self, expr: &str, ctx: &Arc<dyn Context>) -> Result<Pointer, Error> {
        let expr = self.env.compile_expression(expr)?;
        let value = expr.eval(Self::root(ctx))?;
        Ok(Pointer::new(from_minijinja(&value)))
    }
}
