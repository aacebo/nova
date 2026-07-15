use std::collections::HashMap;

use nova_reflect::{DynamicRef, Object, ToType, ToValue, Type, ValueRef};

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

impl Object for Response {
    fn field(&self, name: &str) -> ValueRef<'_> {
        match name {
            "status" => ValueRef::from(self.status),
            "headers" => self.headers.to_value_ref(),
            "data" => self.data.to_value_ref(),
            "text" => match std::str::from_utf8(&self.data) {
                Ok(text) => ValueRef::Str(text),
                Err(_) => ValueRef::Undefined,
            },
            _ => ValueRef::Undefined,
        }
    }
}

impl ToValue for Response {
    fn to_value_ref(&self) -> ValueRef<'_> {
        ValueRef::Dynamic(DynamicRef::from_object(self))
    }
}
