use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use nova_reflect::{Dynamic, Str, ToType, ToValue, Type, Value};
use nova_template::{Args, Call, Context, Engine, Error, Minijinja, Pointer};

#[derive(Debug, Clone)]
struct Scope {
    name: String,
    vars: Vec<(String, String)>,
}

impl ToType for Scope {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl nova_reflect::Object for Scope {
    fn field(&self, name: &str) -> Value<'_> {
        match name {
            "name" => Value::Str(Str(std::borrow::Cow::Borrowed(&self.name))),
            _ => Value::Undefined,
        }
    }
}

impl ToValue for Scope {
    fn to_value(&self) -> Value<'_> {
        Value::Dynamic(Dynamic::from_object(self))
    }
}

impl Context for Scope {
    fn resolve(&self, name: &str) -> Option<Pointer> {
        if name == "scope" {
            return Some(Pointer::new(self.clone()));
        }

        if name == "echo" {
            return Some(Pointer::callable(Echo::default()));
        }

        self.vars
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| Pointer::new(Value::Str(Str(std::borrow::Cow::Owned(v.clone())))))
    }

    fn names(&self) -> Vec<String> {
        self.vars.iter().map(|(k, _)| k.clone()).collect()
    }

    fn as_caller(&self) -> Pointer {
        Pointer::new(self.clone())
    }
}

/// A host function that records exactly what the engine handed it.
#[derive(Debug, Default)]
struct Echo {
    calls: AtomicUsize,
}

impl ToType for Echo {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl ToValue for Echo {
    fn to_value(&self) -> Value<'_> {
        Value::Undefined
    }
}

impl Call for Echo {
    fn call(&self, args: &Args) -> Result<Pointer, Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);

        // caller must be recoverable as the concrete Scope
        let caller = args
            .caller()
            .and_then(|c| c.downcast::<Scope>())
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "<no caller>".to_string());

        let positional: Vec<String> = args.args().iter().map(|a| a.value().to_string()).collect();

        let mut kw: Vec<String> = args.kargs().iter().map(|(k, v)| format!("{k}={}", v.value())).collect();
        kw.sort();

        Ok(Pointer::new(Value::Str(Str(std::borrow::Cow::Owned(format!(
            "caller={caller} pos=[{}] kw=[{}]",
            positional.join(","),
            kw.join(",")
        ))))))
    }
}

fn scope() -> Arc<dyn Context> {
    Arc::new(Scope {
        name: "root".into(),
        vars: vec![("store".into(), "acme".into())],
    })
}

#[test]
fn resolves_a_context_name() {
    let engine = Minijinja::new();
    let out = engine.render_str("{{ store }}", &scope()).unwrap();
    assert_eq!(out, "acme");
}

#[test]
fn reads_a_field_off_a_stored_object() {
    let engine = Minijinja::new();
    let out = engine.render_str("{{ scope.name }}", &scope()).unwrap();
    assert_eq!(out, "root");
}

/// THE contract test: a host call must receive positional args, keyword args,
/// and a caller that downcasts back to the concrete host type.
#[test]
fn host_call_marshals_args_and_recovers_caller() {
    let engine = Minijinja::new();
    let out = engine.render_str("{{ echo(1, 'a', k=2) }}", &scope()).unwrap();

    assert_eq!(out, "caller=root pos=[1,a] kw=[k=2]");
}

#[test]
fn host_call_with_no_args() {
    let engine = Minijinja::new();
    let out = engine.render_str("{{ echo() }}", &scope()).unwrap();

    assert_eq!(out, "caller=root pos=[] kw=[]");
}

#[test]
fn named_template_renders() {
    let mut engine = Minijinja::new();
    engine.add_template("greet", "hello {{ store }}").unwrap();
    assert_eq!(engine.render("greet", &scope()).unwrap(), "hello acme");
}

#[test]
fn eval_returns_a_pointer_value() {
    let engine = Minijinja::new();
    let got = engine.eval("store", &scope()).unwrap();
    assert_eq!(got.value().to_string(), "acme");
}

#[test]
fn unknown_template_is_an_error() {
    let engine = Minijinja::new();
    assert!(engine.render("nope", &scope()).is_err());
}

#[test]
fn u64_max_survives_the_round_trip() {
    // guards the number-coercion regression called out in the plan
    let engine = Minijinja::new();
    let got = engine.eval(&format!("{}", u64::MAX), &scope()).unwrap();
    assert_eq!(got.value().to_u64(), Some(u64::MAX));
}

#[derive(Debug, Clone)]
struct Inner {
    city: String,
}

impl ToType for Inner {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl nova_reflect::Object for Inner {
    fn field(&self, name: &str) -> Value<'_> {
        match name {
            "city" => Value::Str(Str(std::borrow::Cow::Borrowed(&self.city))),
            _ => Value::Undefined,
        }
    }
}

impl ToValue for Inner {
    fn to_value(&self) -> Value<'_> {
        Value::Dynamic(Dynamic::from_object(self))
    }
}

#[derive(Debug, Clone)]
struct Outer {
    inner: Inner,
}

impl ToType for Outer {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl nova_reflect::Object for Outer {
    fn field(&self, name: &str) -> Value<'_> {
        match name {
            // a field that is ITSELF a dynamic object
            "inner" => self.inner.to_value(),
            _ => Value::Undefined,
        }
    }
}

impl ToValue for Outer {
    fn to_value(&self) -> Value<'_> {
        Value::Dynamic(Dynamic::from_object(self))
    }
}

#[derive(Debug)]
struct Nested;

impl Context for Nested {
    fn resolve(&self, name: &str) -> Option<Pointer> {
        if name == "outer" {
            return Some(Pointer::new(Outer {
                inner: Inner { city: "nyc".into() },
            }));
        }
        None
    }
    fn names(&self) -> Vec<String> {
        vec!["outer".into()]
    }
    fn as_caller(&self) -> Pointer {
        Pointer::new(Value::Null)
    }
}

#[test]
fn nested_dynamic_field_resolves() {
    let engine = Minijinja::new();
    let ctx: Arc<dyn Context> = Arc::new(Nested);
    let out = engine.render_str("{{ outer.inner.city }}", &ctx).unwrap();
    assert_eq!(out, "nyc");
}

#[derive(Debug, Clone)]
struct WithVec {
    vector: Vec<f32>,
}

impl ToType for WithVec {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl nova_reflect::Object for WithVec {
    fn field(&self, name: &str) -> Value<'_> {
        match name {
            "vector" => self.vector.to_value(),
            _ => Value::Undefined,
        }
    }
}

impl ToValue for WithVec {
    fn to_value(&self) -> Value<'_> {
        Value::Dynamic(Dynamic::from_object(self))
    }
}

#[derive(Debug)]
struct VecCtx;

impl Context for VecCtx {
    fn resolve(&self, name: &str) -> Option<Pointer> {
        if name == "obj" {
            return Some(Pointer::new(WithVec {
                vector: vec![1.0, 2.0, 3.0],
            }));
        }
        None
    }
    fn names(&self) -> Vec<String> {
        vec!["obj".into()]
    }
    fn as_caller(&self) -> Pointer {
        Pointer::new(Value::Null)
    }
}

#[test]
fn length_filter_on_a_nested_sequence_field() {
    let engine = Minijinja::new();
    let ctx: Arc<dyn Context> = Arc::new(VecCtx);
    let out = engine.render_str("{{ obj.vector | length }}", &ctx).unwrap();
    assert_eq!(out, "3");
}
