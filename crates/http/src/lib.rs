mod response;

use nova_core::Function;
use nova_template::{FromArgs, Namespace, Pointer};
pub use response::*;

pub trait Http {
    fn http(self) -> Self;
}

impl Http for nova_core::Builder {
    fn http(self) -> Self {
        self.var("http", Pointer::namespace(Client))
    }
}

#[derive(Debug)]
pub struct Client;

impl Namespace for Client {
    fn member(&self, name: &str) -> Option<Pointer> {
        match name {
            "get" => Some(Pointer::callable(Function::func("http.get", get))),
            "post" => Some(Pointer::callable(Function::func("http.post", post))),
            "put" => Some(Pointer::callable(Function::func("http.put", put))),
            "patch" => Some(Pointer::callable(Function::func("http.patch", patch))),
            _ => None,
        }
    }

    fn members(&self) -> Vec<String> {
        ["get", "post", "put", "patch"].iter().map(|s| s.to_string()).collect()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub fn get(args: &nova_template::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
    send(reqwest::Method::GET, args)
}

pub fn post(args: &nova_template::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
    send(reqwest::Method::POST, args)
}

pub fn put(args: &nova_template::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
    send(reqwest::Method::PUT, args)
}

pub fn patch(args: &nova_template::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
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

    fn from_args(args: &nova_template::Args) -> Result<Self, Self::Error> {
        let uri = args.str(0).ok_or(nova_core::Error::message("uri must be a string"))?;

        let body = args.at(1).value().as_str().map(|s| Body::Text(s.to_string()));

        let mut headers = Vec::new();

        let headers_value = args.key("headers");

        if let Some(map) = headers_value.value().as_map() {
            for (key, header) in map.iter() {
                let key = key
                    .as_str()
                    .ok_or(nova_core::Error::message("header name must be a string"))?;
                let header = header
                    .as_str()
                    .ok_or(nova_core::Error::message("header value must be a string"))?;

                headers.push((key.to_string(), header.to_string()));
            }
        }

        Ok(Self { uri, body, headers })
    }
}

fn send(method: reqwest::Method, args: &nova_template::Args) -> Result<Pointer, Box<dyn std::error::Error>> {
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
    Ok(Pointer::new(Response::try_from(res)?))
}
