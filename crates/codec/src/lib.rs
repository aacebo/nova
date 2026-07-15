use nova_core::Function;
use nova_reflect::Value;
use nova_template::{FromArgs, Namespace, Pointer};

pub trait Codec {
    fn json(self) -> Self;
    fn yaml(self) -> Self;
}

impl Codec for nova_core::Builder {
    fn json(self) -> Self {
        self.var("json", Pointer::namespace(Json))
    }

    fn yaml(self) -> Self {
        self.var("yaml", Pointer::namespace(Yaml))
    }
}

#[derive(Debug)]
pub struct Json;

impl Namespace for Json {
    fn member(&self, name: &str) -> Option<Pointer> {
        match name {
            "encode" => Some(Pointer::callable(Function::func("json.encode", json_encode))),
            "decode" => Some(Pointer::callable(Function::func("json.decode", json_decode))),
            _ => None,
        }
    }

    fn members(&self) -> Vec<String> {
        vec!["encode".to_string(), "decode".to_string()]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct Yaml;

impl Namespace for Yaml {
    fn member(&self, name: &str) -> Option<Pointer> {
        match name {
            "encode" => Some(Pointer::callable(Function::func("yaml.encode", yaml_encode))),
            "decode" => Some(Pointer::callable(Function::func("yaml.decode", yaml_decode))),
            _ => None,
        }
    }

    fn members(&self) -> Vec<String> {
        vec!["encode".to_string(), "decode".to_string()]
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct EncodeArgs {
    pub value: nova_reflect::Value,
}

impl FromArgs for EncodeArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &nova_template::Args) -> Result<Self, Self::Error> {
        Ok(Self { value: args.at(0) })
    }
}

pub struct DecodeArgs {
    pub source: String,
}

impl FromArgs for DecodeArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &nova_template::Args) -> Result<Self, Self::Error> {
        let source = args.str(0).ok_or(nova_core::Error::message("value must be a string"))?;

        Ok(Self { source })
    }
}

pub fn json_encode(args: &nova_template::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
    let encoded = serde_json::to_string(&EncodeArgs::from_args(args)?.value)?;
    Ok(Pointer::new(Value::from(encoded)))
}

pub fn json_decode(args: &nova_template::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str::<Pointer>(&DecodeArgs::from_args(args)?.source)?)
}

pub fn yaml_encode(args: &nova_template::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
    let encoded = serde_norway::to_string(&EncodeArgs::from_args(args)?.value)?;
    Ok(Pointer::new(Value::from(encoded)))
}

pub fn yaml_decode(args: &nova_template::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
    Ok(serde_norway::from_str::<Pointer>(&DecodeArgs::from_args(args)?.source)?)
}
