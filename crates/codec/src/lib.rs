use std::sync::Arc;

use nova_core::FromArgs;

pub trait Codec {
    fn json(self) -> Self;
    fn yaml(self) -> Self;
}

impl Codec for nova_core::Builder {
    fn json(self) -> Self {
        self.var("json", nova_core::Value::from_object(Json))
    }

    fn yaml(self) -> Self {
        self.var("yaml", nova_core::Value::from_object(Yaml))
    }
}

#[derive(Debug)]
pub struct Json;

impl nova_core::Reflect for Json {
    fn get_value(self: &Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        match key.as_str()? {
            "encode" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "json.encode",
                json_encode,
            ))),
            "decode" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "json.decode",
                json_decode,
            ))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Yaml;

impl nova_core::Reflect for Yaml {
    fn get_value(self: &Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        match key.as_str()? {
            "encode" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "yaml.encode",
                yaml_encode,
            ))),
            "decode" => Some(nova_core::Value::from_object(nova_core::Function::func(
                "yaml.decode",
                yaml_decode,
            ))),
            _ => None,
        }
    }
}

pub struct EncodeArgs {
    pub value: nova_core::Value,
}

impl FromArgs for EncodeArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &nova_core::Args<'_>) -> Result<Self, Self::Error> {
        Ok(Self { value: args.at(0) })
    }
}

pub struct DecodeArgs {
    pub source: String,
}

impl FromArgs for DecodeArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &nova_core::Args<'_>) -> Result<Self, Self::Error> {
        let value = args.at(0);
        let source = value.as_str().ok_or(nova_core::Error::message("value must be a string"))?;

        Ok(Self {
            source: source.to_string(),
        })
    }
}

pub fn json_encode(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string(&EncodeArgs::from_args(args)?.value)?.into())
}

pub fn json_decode(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str::<nova_core::Value>(
        &DecodeArgs::from_args(args)?.source,
    )?)
}

pub fn yaml_encode(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    Ok(serde_norway::to_string(&EncodeArgs::from_args(args)?.value)?.into())
}

pub fn yaml_decode(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    Ok(serde_norway::from_str::<nova_core::Value>(
        &DecodeArgs::from_args(args)?.source,
    )?)
}
