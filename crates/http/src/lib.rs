mod response;

use std::sync::Arc;

pub use response::*;

pub struct Http;

impl nova::Import for Http {
    fn import(self, builder: nova::Builder) -> Result<nova::Builder, Box<dyn std::error::Error>> {
        Ok(builder.func("http.get", get))
    }
}

pub fn get(args: &nova::Args, _scope: &nova::Scope) -> Result<nova::Value, Box<dyn std::error::Error>> {
    let uri = args.at(0);
    let uri = uri.as_str().ok_or(nova::Error::message("uri must be a string"))?;
    let res = reqwest::blocking::get(uri)?;

    Ok(nova::Value::from_dyn_object(Arc::new(Response::try_from(res)?)))
}
