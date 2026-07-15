mod response;

use nova_core::{Args, Binding, Context, FromArgs, Function, Namespace};
pub use response::*;

#[derive(Debug)]
pub struct Client;

impl Namespace for Client {
    fn member(&self, name: &str) -> Option<Binding> {
        match name {
            "get" => Some(Binding::callable(Function::func("http.get", get))),
            "post" => Some(Binding::callable(Function::func("http.post", post))),
            "put" => Some(Binding::callable(Function::func("http.put", put))),
            "patch" => Some(Binding::callable(Function::func("http.patch", patch))),
            _ => None,
        }
    }

    fn members(&self) -> Vec<String> {
        ["get", "post", "put", "patch"].iter().map(|s| s.to_string()).collect()
    }
}

pub fn get(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
    send(reqwest::Method::GET, args)
}

pub fn post(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
    send(reqwest::Method::POST, args)
}

pub fn put(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
    send(reqwest::Method::PUT, args)
}

pub fn patch(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
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

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
        let uri = args.str(0).ok_or(nova_core::Error::message("uri must be a string"))?;

        let body = args.str(1).map(Body::Text);

        let mut headers = Vec::new();

        let headers_value = args.key("headers");

        if let Some(map) = headers_value.as_map() {
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

fn send(method: reqwest::Method, args: &Args) -> Result<Binding, Box<dyn std::error::Error>> {
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
    Ok(Binding::new(Response::try_from(res)?))
}
