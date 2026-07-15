use nova_core::{Args, Binding, Context, FromArgs, Function, Namespace};
use nova_reflect::Value;

#[derive(Debug)]
pub struct Json;

impl Namespace for Json {
    fn member(&self, name: &str) -> Option<Binding> {
        match name {
            "encode" => Some(Binding::callable(Function::func("json.encode", json_encode))),
            "decode" => Some(Binding::callable(Function::func("json.decode", json_decode))),
            _ => None,
        }
    }

    fn members(&self) -> Vec<String> {
        vec!["encode".to_string(), "decode".to_string()]
    }
}

#[derive(Debug)]
pub struct Yaml;

impl Namespace for Yaml {
    fn member(&self, name: &str) -> Option<Binding> {
        match name {
            "encode" => Some(Binding::callable(Function::func("yaml.encode", yaml_encode))),
            "decode" => Some(Binding::callable(Function::func("yaml.decode", yaml_decode))),
            _ => None,
        }
    }

    fn members(&self) -> Vec<String> {
        vec!["encode".to_string(), "decode".to_string()]
    }
}

pub struct EncodeArgs {
    pub value: nova_reflect::Value,
}

impl FromArgs for EncodeArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
        Ok(Self { value: args.at(0) })
    }
}

pub struct DecodeArgs {
    pub source: String,
}

impl FromArgs for DecodeArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
        let source = args.str(0).ok_or(nova_core::Error::message("value must be a string"))?;

        Ok(Self { source })
    }
}

pub fn json_encode(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
    let encoded = serde_json::to_string(&EncodeArgs::from_args(args)?.value)?;
    Ok(Binding::new(Value::from(encoded)))
}

pub fn json_decode(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str::<Binding>(&DecodeArgs::from_args(args)?.source)?)
}

pub fn yaml_encode(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
    let encoded = serde_norway::to_string(&EncodeArgs::from_args(args)?.value)?;
    Ok(Binding::new(Value::from(encoded)))
}

pub fn yaml_decode(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
    Ok(serde_norway::from_str::<Binding>(&DecodeArgs::from_args(args)?.source)?)
}
