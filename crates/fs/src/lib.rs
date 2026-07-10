use std::sync::Arc;

pub struct FileSystem;

impl nova::Import for FileSystem {
    fn import(self, builder: nova::Builder) -> Result<nova::Builder, Box<dyn std::error::Error>> {
        Ok(builder.var("fs", nova::Value::from_object(Fs)))
    }
}

#[derive(Debug)]
pub struct Fs;

impl nova::Reflect for Fs {
    fn get_value(self: &Arc<Self>, key: &nova::Value) -> Option<nova::Value> {
        match key.as_str()? {
            "read" => Some(nova::Value::from_object(nova::Function::func("fs.read", read))),
            "write" => Some(nova::Value::from_object(nova::Function::action("fs.write", write))),
            _ => None,
        }
    }
}

pub fn read(args: &nova::Args, _scope: &nova::Scope) -> Result<nova::Value, Box<dyn std::error::Error>> {
    let path = args.at(0);
    let base = std::env::current_dir()?;
    let path = path.as_str().ok_or(nova::Error::message("path must be a string"))?;
    let data = std::fs::read_to_string(base.join(path))?;
    Ok(data.into())
}

pub fn write(args: &nova::Args, _scope: &nova::Scope) -> Result<(), Box<dyn std::error::Error>> {
    let path = args.at(0);
    let base = std::env::current_dir()?;
    let path = path.as_str().ok_or(nova::Error::message("path must be a string"))?;
    let data = args.at(1);

    if let Some(data) = data.as_bytes() {
        Ok(std::fs::write(base.join(path), data)?)
    } else {
        Err(Box::new(nova::Error::message("invalid data type")))
    }
}
