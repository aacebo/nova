use std::sync::Arc;

pub trait FileSystem {
    fn fs(self) -> Self;
}

impl FileSystem for nova_core::Builder {
    fn fs(self) -> Self {
        self.var("fs", nova_core::Value::from_object(Fs))
    }
}

#[derive(Debug)]
pub struct Fs;

impl nova_core::Reflect for Fs {
    fn get_value(self: &Arc<Self>, key: &nova_core::Value) -> Option<nova_core::Value> {
        match key.as_str()? {
            "read" => Some(nova_core::Value::from_object(nova_core::Function::func("fs.read", read))),
            "write" => Some(nova_core::Value::from_object(nova_core::Function::action("fs.write", write))),
            _ => None,
        }
    }
}

pub fn read(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let path = args.at(0);
    let base = std::env::current_dir()?;
    let path = path.as_str().ok_or(nova_core::Error::message("path must be a string"))?;
    let data = std::fs::read_to_string(base.join(path))?;
    Ok(data.into())
}

pub fn write(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<(), Box<dyn std::error::Error>> {
    let path = args.at(0);
    let base = std::env::current_dir()?;
    let path = path.as_str().ok_or(nova_core::Error::message("path must be a string"))?;
    let data = args.at(1);

    if let Some(data) = data.as_bytes() {
        Ok(std::fs::write(base.join(path), data)?)
    } else {
        Err(Box::new(nova_core::Error::message("invalid data type")))
    }
}
