mod response;

use std::sync::Arc;

pub use response::*;

pub trait Http {
    fn http(self) -> Self;
}

impl Http for nova_core::Builder {
    fn http(self) -> Self {
        self.var("http", nova_core::Value::from_object(Client))
    }
}

#[derive(Debug)]
pub struct Client;

impl nova_core::Reflect for Client {
    fn get_value(self: &Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        match key.as_str()? {
            "get" => Some(nova_core::Value::from_object(nova_core::Function::func("http.get", get))),
            "post" => Some(nova_core::Value::from_object(nova_core::Function::func("http.post", post))),
            "put" => Some(nova_core::Value::from_object(nova_core::Function::func("http.put", put))),
            "patch" => Some(nova_core::Value::from_object(nova_core::Function::func("http.patch", patch))),
            _ => None,
        }
    }
}

pub fn get(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    send(reqwest::Method::GET, args)
}

pub fn post(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    send(reqwest::Method::POST, args)
}

pub fn put(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    send(reqwest::Method::PUT, args)
}

pub fn patch(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    send(reqwest::Method::PATCH, args)
}

fn send(method: reqwest::Method, args: &nova_core::Args) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let uri = args.at(0);
    let uri = uri.as_str().ok_or(nova_core::Error::message("uri must be a string"))?;
    let body = args.at(1);
    let mut req = reqwest::blocking::Client::new().request(method, uri);

    let headers = args.key("headers");
    if !headers.is_undefined() && !headers.is_none() {
        for key in headers.try_iter()? {
            let value = headers.get_item(&key)?;
            let key = key
                .as_str()
                .ok_or(nova_core::Error::message("header name must be a string"))?;
            let value = value
                .as_str()
                .ok_or(nova_core::Error::message("header value must be a string"))?;
            req = req.header(key, value);
        }
    }

    req = if let Some(text) = body.as_str() {
        req.body(text.to_string())
    } else if let Some(bytes) = body.as_bytes() {
        req.body(bytes.to_vec())
    } else {
        req
    };

    let res = req.send()?;
    Ok(nova_core::Value::from_dyn_object(Arc::new(Response::try_from(res)?)))
}
