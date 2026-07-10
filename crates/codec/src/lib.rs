use std::sync::Arc;

pub struct Codec;

impl nova::Import for Codec {
    fn import(self, builder: nova::Builder) -> Result<nova::Builder, Box<dyn std::error::Error>> {
        Ok(builder
            .var("json", nova::Value::from_object(Json))
            .var("yaml", nova::Value::from_object(Yaml)))
    }
}

#[derive(Debug)]
pub struct Json;

impl nova::Reflect for Json {
    fn get_value(self: &Arc<Self>, key: &nova::Value) -> Option<nova::Value> {
        match key.as_str()? {
            "encode" => Some(nova::Value::from_object(nova::Function::func("json.encode", json_encode))),
            "decode" => Some(nova::Value::from_object(nova::Function::func("json.decode", json_decode))),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Yaml;

impl nova::Reflect for Yaml {
    fn get_value(self: &Arc<Self>, key: &nova::Value) -> Option<nova::Value> {
        match key.as_str()? {
            "encode" => Some(nova::Value::from_object(nova::Function::func("yaml.encode", yaml_encode))),
            "decode" => Some(nova::Value::from_object(nova::Function::func("yaml.decode", yaml_decode))),
            _ => None,
        }
    }
}

pub fn json_encode(args: &nova::Args, _scope: &nova::Scope) -> Result<nova::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::to_string(&args.at(0))?.into())
}

pub fn json_decode(args: &nova::Args, _scope: &nova::Scope) -> Result<nova::Value, Box<dyn std::error::Error>> {
    let value = args.at(0);
    let source = value.as_str().ok_or(nova::Error::message("value must be a string"))?;
    Ok(serde_json::from_str::<nova::Value>(source)?)
}

pub fn yaml_encode(args: &nova::Args, _scope: &nova::Scope) -> Result<nova::Value, Box<dyn std::error::Error>> {
    Ok(serde_norway::to_string(&args.at(0))?.into())
}

pub fn yaml_decode(args: &nova::Args, _scope: &nova::Scope) -> Result<nova::Value, Box<dyn std::error::Error>> {
    let value = args.at(0);
    let source = value.as_str().ok_or(nova::Error::message("value must be a string"))?;
    Ok(serde_norway::from_str::<nova::Value>(source)?)
}
