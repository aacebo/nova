use std::sync::Arc;

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

pub fn json_encode(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string(&args.at(0))?.into())
}

pub fn json_decode(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let value = args.at(0);
    let source = value.as_str().ok_or(nova_core::Error::message("value must be a string"))?;
    Ok(serde_json::from_str::<nova_core::Value>(source)?)
}

pub fn yaml_encode(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    Ok(serde_norway::to_string(&args.at(0))?.into())
}

pub fn yaml_decode(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let value = args.at(0);
    let source = value.as_str().ok_or(nova_core::Error::message("value must be a string"))?;
    Ok(serde_norway::from_str::<nova_core::Value>(source)?)
}
