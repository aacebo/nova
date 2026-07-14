use std::collections::HashMap;

use nova_core::{Dynamic, Reflect, ToType, ToValue, Type, Value};

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub data: Vec<u8>,
}

impl TryFrom<reqwest::blocking::Response> for Response {
    type Error = nova_core::Error;

    fn try_from(value: reqwest::blocking::Response) -> Result<Self, Self::Error> {
        Ok(Self {
            status: value.status().as_u16(),
            headers: value
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect(),
            data: value.bytes().map_err(|e| nova_core::Error::message(e.to_string()))?.to_vec(),
        })
    }
}

impl ToType for Response {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl Reflect for Response {
    fn field(&self, name: &str) -> Value<'_> {
        match name {
            "status" => Value::from(self.status),
            "headers" => self.headers.to_value(),
            "text" | "data" => Value::from(String::from_utf8_lossy(&self.data).into_owned()),
            _ => Value::Undefined,
        }
    }
}

impl ToValue for Response {
    fn to_value(&self) -> Value<'_> {
        Value::Dynamic(Dynamic::from_object(self))
    }
}
