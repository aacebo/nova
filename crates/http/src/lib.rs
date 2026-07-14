mod response;

use std::sync::Arc;

use nova_core::FromArgs;
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

pub enum Body {
    Text(String),
    Bytes(Vec<u8>),
}

pub struct RequestArgs {
    pub uri: String,
    pub body: Option<Body>,
    pub headers: Vec<(String, String)>,
}

impl FromArgs for RequestArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &nova_core::Args<'_>) -> Result<Self, Self::Error> {
        let uri = args.at(0);
        let uri = uri.as_str().ok_or(nova_core::Error::message("uri must be a string"))?;
        let value = args.at(1);
        let body = if let Some(text) = value.as_str() {
            Some(Body::Text(text.to_string()))
        } else {
            value.as_bytes().map(|bytes| Body::Bytes(bytes.to_vec()))
        };

        let mut headers = Vec::new();
        let value = args.key("headers");

        if !value.is_undefined() && !value.is_none() {
            for key in value.try_iter()? {
                let header = value.get_item(&key)?;
                let key = key
                    .as_str()
                    .ok_or(nova_core::Error::message("header name must be a string"))?;
                let header = header
                    .as_str()
                    .ok_or(nova_core::Error::message("header value must be a string"))?;

                headers.push((key.to_string(), header.to_string()));
            }
        }

        Ok(Self {
            uri: uri.to_string(),
            body,
            headers,
        })
    }
}

fn send(method: reqwest::Method, args: &nova_core::Args) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let args = RequestArgs::from_args(args)?;
    let mut req = reqwest::blocking::Client::new().request(method, args.uri);

    for (key, value) in args.headers {
        req = req.header(key, value);
    }

    req = match args.body {
        Some(Body::Text(text)) => req.body(text),
        Some(Body::Bytes(bytes)) => req.body(bytes),
        None => req,
    };

    let res = req.send()?;
    Ok(nova_core::Value::from_dyn_object(Arc::new(Response::try_from(res)?)))
}
