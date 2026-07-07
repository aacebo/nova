use std::fmt;
use std::sync::Arc;

use crate::{Args, Diagnostic, Environment, Object, Scope, Traced, Value};

#[derive(Clone)]
pub struct Context {
    env: Arc<Environment<'static>>,
    scope: Scope,
}

impl Context {
    pub const KEY: &'static str = "__$ctx__";

    pub fn new(env: Arc<Environment<'static>>, scope: Scope) -> Self {
        Self { env, scope }
    }

    pub fn trace_id(&self) -> &ulid::Ulid {
        self.scope.trace_id()
    }

    pub fn args(&self) -> &Args {
        self.scope.args()
    }

    pub fn env(&self) -> &Environment<'static> {
        &self.env
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn take_diagnostics(&self) -> Vec<Diagnostic> {
        self.scope.take_diagnostics()
    }

    pub fn emit(&mut self, diagnostic: Diagnostic) -> &mut Self {
        self.scope.emit(diagnostic);
        self
    }

    pub fn child(&self, args: impl Into<Args>) -> Self {
        Self {
            env: self.env.clone(),
            scope: self.scope.fork(args),
        }
    }

    pub fn call(&self, name: impl AsRef<str>, args: impl Into<Args>) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let name = name.as_ref();
        let func = self.scope.get_func(name)?;
        let mut ctx = self.child(args);
        let result = {
            let _guard = crate::enter_trace(*ctx.trace_id());
            func.invoke(&mut ctx)
        };
        self.scope.merge(name, ctx.scope());
        result
    }

    pub fn eval(&self, src: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let expr = self.env().compile_expression(src)?;
        Ok(expr.eval(Value::from_object(self.clone()))?)
    }

    pub fn render(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let tmpl = self.env().get_template(name)?;
        Ok(tmpl.render(Value::from_object(self.clone()))?)
    }

    pub fn render_str(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.env().render_str(source, Value::from_object(self.clone()))?)
    }
}

impl Traced for Context {
    fn trace_id(&self) -> ulid::Ulid {
        *self.scope.trace_id()
    }
}

impl minijinja::value::Object for Context {
    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        let name = key.as_str()?;

        if name == Self::KEY {
            return Some(Value::from_object(Self::clone(self)));
        }

        if let Some(value) = self.scope.args().get(name) {
            return Some(value.clone());
        }

        let slot = self.scope.get(name)?;

        match &*slot {
            Object::Var(var) => Some(var.value.clone()),
            Object::Func(func) => Some(Value::from_object(func.clone())),
            _ => None,
        }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("trace_id", self.trace_id())
            .finish_non_exhaustive()
    }
}

impl std::ops::Deref for Context {
    type Target = Scope;

    fn deref(&self) -> &Self::Target {
        &self.scope
    }
}

impl std::ops::DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.scope
    }
}
