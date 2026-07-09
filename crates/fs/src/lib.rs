pub struct FileSystem;

impl nova::Import for FileSystem {
    fn import(self, builder: nova::Builder) -> Result<nova::Builder, Box<dyn std::error::Error>> {
        Ok(builder.func("fs.read", read).action("fs.write", write))
    }
}

pub fn read(
    args: &[nova::Value],
    _kargs: &nova::KArgs,
    _scope: &nova::Scope,
) -> Result<Option<nova::Value>, Box<dyn std::error::Error>> {
    let path = args[0].as_str().ok_or(nova::Error::message("path must be a string"))?;
    let data = std::fs::read_to_string(path)?;
    Ok(Some(data.into()))
}

pub fn write(args: &[nova::Value], _kargs: &nova::KArgs, _scope: &nova::Scope) -> Result<(), Box<dyn std::error::Error>> {
    let path = args[0].as_str().ok_or(nova::Error::message("path must be a string"))?;

    if let Some(data) = args[1].as_bytes() {
        Ok(std::fs::write(path, data)?)
    } else {
        Err(Box::new(nova::Error::message("invalid data type")))
    }
}
