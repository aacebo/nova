#![cfg(feature = "minijinja")]

use std::borrow::Cow;

use minijinja::Environment;
use nova_reflect::{Callable, Dynamic, Int, Map, MapType, Number, Object, Str, ToType, ToValue, Type, Value};
use nova_reflect_macros::*;

fn any_map() -> Map<'static> {
    let ty = MapType::new(Type::Any, Type::Any, Type::Any);
    Map::new(&ty)
}

fn user_value() -> Value<'static> {
    let mut map = any_map();
    map.insert(Value::Str(Str(Cow::Borrowed("name"))), Value::Str(Str(Cow::Borrowed("alex"))));
    map.insert(
        Value::Str(Str(Cow::Borrowed("age"))),
        Value::Number(Number::Int(Int::U64(30))),
    );
    Value::Map(map)
}

fn nested_value() -> Value<'static> {
    let mut inner = any_map();
    inner.insert(Value::Str(Str(Cow::Borrowed("city"))), Value::Str(Str(Cow::Borrowed("nyc"))));

    let mut outer = any_map();
    outer.insert(Value::Str(Str(Cow::Borrowed("addr"))), Value::Map(inner));
    Value::Map(outer)
}

#[test]
fn renders_field_access() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(user_value());
    let out = env
        .render_str("{{ v.name }} is {{ v.age }}", minijinja::context! { v => value })
        .unwrap();

    assert_eq!(out, "alex is 30");
}

#[test]
fn renders_nested_field_access() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(nested_value());
    let out = env
        .render_str("{{ v.addr.city }}", minijinja::context! { v => value })
        .unwrap();

    assert_eq!(out, "nyc");
}

#[test]
fn iterates_map_keys() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(user_value());
    let out = env
        .render_str("{% for k in v %}{{ k }};{% endfor %}", minijinja::context! { v => value })
        .unwrap();

    assert!(out.contains("name;"));
    assert!(out.contains("age;"));
}

#[test]
fn reports_length() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(user_value());
    let out = env
        .render_str("{{ v | length }}", minijinja::context! { v => value })
        .unwrap();

    assert_eq!(out, "2");
}

#[test]
fn missing_field_is_undefined() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(user_value());
    let out = env
        .render_str("{{ v.nope | default('missing') }}", minijinja::context! { v => value })
        .unwrap();

    assert_eq!(out, "missing");
}

#[test]
fn renders_via_display() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(Value::Map(any_map()));
    let out = env.render_str("{{ v }}", minijinja::context! { v => value }).unwrap();
    assert!(out.contains('{'));
}

#[derive(Debug, Clone, Reflect)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

static POINT: std::sync::LazyLock<Point> = std::sync::LazyLock::new(|| Point { x: 3, y: 7 });
static SEQ: std::sync::LazyLock<Vec<i32>> = std::sync::LazyLock::new(|| vec![10_i32, 20, 30]);

fn point_value() -> Value<'static> {
    POINT.to_value()
}

fn seq_value() -> Value<'static> {
    SEQ.to_value()
}

#[test]
fn renders_reflected_struct_object() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(point_value());
    let out = env
        .render_str("{{ v.x }},{{ v.y }}", minijinja::context! { v => value })
        .unwrap();

    assert_eq!(out, "3,7");
}

#[test]
fn enumerates_reflected_struct_fields() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(point_value());
    let out = env
        .render_str("{% for k in v %}{{ k }};{% endfor %}", minijinja::context! { v => value })
        .unwrap();

    assert!(out.contains("x;"));
    assert!(out.contains("y;"));
}

#[test]
fn renders_sequence_index_and_iteration() {
    let env = Environment::new();
    let indexed = env
        .render_str(
            "{{ v[1] }}",
            minijinja::context! { v => minijinja::Value::from_object(seq_value()) },
        )
        .unwrap();
    assert_eq!(indexed, "20");

    let iterated = env
        .render_str(
            "{% for x in v %}{{ x }};{% endfor %}",
            minijinja::context! { v => minijinja::Value::from_object(seq_value()) },
        )
        .unwrap();
    assert_eq!(iterated, "10;20;30;");
}

#[derive(Debug, Clone)]
pub struct Doubler;

impl nova_reflect::TypeOf for Doubler {
    fn type_of() -> Type {
        Type::Any
    }
}

impl ToType for Doubler {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl Callable for Doubler {
    fn call(&self, args: &[Value]) -> Result<Value<'_>, String> {
        let n = args
            .first()
            .and_then(|a| a.to_i64())
            .ok_or_else(|| "expected an integer argument".to_string())?;
        Ok(Value::Number(Number::Int(Int::I64(n * 2))))
    }
}

static DOUBLER: Doubler = Doubler;

fn doubler_value() -> Value<'static> {
    Value::Dynamic(Dynamic::from_callable(&DOUBLER))
}

#[test]
fn calls_callable_value() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(doubler_value());
    let out = env.render_str("{{ f(21) }}", minijinja::context! { f => value }).unwrap();

    assert_eq!(out, "42");
}

#[derive(Debug, Clone, Reflect)]
pub struct Calculator {
    pub name: String,
}

#[nova_reflect::reflect]
impl Calculator {
    pub fn double(&self, n: i64) -> i64 {
        n * 2
    }
}

static CALCULATOR: std::sync::LazyLock<Calculator> = std::sync::LazyLock::new(|| Calculator {
    name: String::from("alex"),
});

#[test]
fn calls_method_member_from_template() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(CALCULATOR.to_value());
    let out = env
        .render_str("{{ v.name }}: {{ v.double(21) }}", minijinja::context! { v => value })
        .unwrap();

    assert_eq!(out, "alex: 42");
}

#[test]
fn object_call_invokes_method_and_field_returns_fields_only() {
    assert_eq!(CALCULATOR.double(21), 42);

    let value = CALCULATOR.to_value();
    let obj = value.as_dynamic().unwrap();
    let result = Object::call(obj, "double", &[Value::Number(Number::Int(Int::I64(21)))]).unwrap();

    assert_eq!(result.to_i64(), Some(42));
    assert!(obj.field("name").is_str());
    assert!(obj.field("double").is_undefined());
    assert!(obj.field("missing").is_undefined());
    assert!(Object::call(obj, "missing", &[]).is_err());
}

#[test]
fn calling_non_callable_errors() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(user_value());
    let result = env.render_str("{{ v(1) }}", minijinja::context! { v => value });

    assert!(result.is_err());
}

#[test]
fn missing_index_is_undefined() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(seq_value());
    let out = env
        .render_str("{{ v[9] | default('none') }}", minijinja::context! { v => value })
        .unwrap();

    assert_eq!(out, "none");
}

#[test]
fn to_value_from_minijinja_scalars() {
    assert!(minijinja::Value::from(()).to_value().is_null());
    assert_eq!(minijinja::Value::from(true).to_value(), Value::Bool(true));
    assert_eq!(minijinja::Value::from(7_i64).to_value().to_i64(), Some(7));
    assert_eq!(minijinja::Value::from("hi").to_value(), Value::Str(Str(Cow::Borrowed("hi"))));
    assert!(minijinja::Value::UNDEFINED.to_value().is_null());
}

static INNER: std::sync::LazyLock<Value<'static>> = std::sync::LazyLock::new(user_value);

#[test]
fn resolves_fields_through_ref() {
    let env = Environment::new();
    let value = minijinja::Value::from_object(Value::Ref(nova_reflect::Ref {
        ty: nova_reflect::RefType(std::sync::Arc::new(Type::Any)),
        value: &INNER,
    }));

    let out = env.render_str("{{ v.name }}", minijinja::context! { v => value }).unwrap();
    assert_eq!(out, "alex");
}
