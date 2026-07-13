use std::collections::HashMap;

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

impl nova_core::Reflect for Response {
    fn get_value(self: &std::sync::Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        let key = key.as_str()?;

        match key {
            "status" => Some(self.status.into()),
            "headers" => Some(self.headers.clone().into()),
            "data" => Some(nova_core::Value::from_bytes(self.data.clone())),
            "text" => Some(String::from_utf8_lossy(&self.data).into_owned().into()),
            _ => None,
        }
    }
}
