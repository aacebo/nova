use nova_core::{Args, Binding, Context, FromArgs, Function, Namespace};
use nova_reflect::{Value, ValueRef};

#[derive(Debug)]
pub struct Fs;

impl Namespace for Fs {
    fn member(&self, name: &str) -> Option<Binding> {
        match name {
            "read" => Some(Binding::callable(Function::func("fs.read", read))),
            "write" => Some(Binding::callable(Function::action("fs.write", write))),
            _ => None,
        }
    }

    fn members(&self) -> Vec<String> {
        vec!["read".to_string(), "write".to_string()]
    }
}

pub struct ReadArgs {
    pub path: String,
}

impl FromArgs for ReadArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
        let path = args.str(0).ok_or(nova_core::Error::message("path must be a string"))?;
        Ok(Self { path })
    }
}

pub struct WriteArgs {
    pub path: String,
    pub data: Vec<u8>,
}

impl FromArgs for WriteArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &Args) -> Result<Self, Self::Error> {
        let path = args.str(0).ok_or(nova_core::Error::message("path must be a string"))?;
        let value = args.at(1);
        let data = to_bytes(&value.as_ref()).ok_or(nova_core::Error::message("invalid data type"))?;
        Ok(Self { path, data })
    }
}

fn to_bytes(value: &ValueRef<'_>) -> Option<Vec<u8>> {
    if let Some(text) = value.as_str() {
        return Some(text.as_bytes().to_vec());
    }

    let seq = value.as_dynamic()?.as_sequence()?;
    let mut bytes = Vec::with_capacity(seq.len());

    for i in 0..seq.len() {
        bytes.push(u8::try_from(seq.index(i)).ok()?);
    }

    Some(bytes)
}

pub fn read(args: &Args, _ctx: &dyn Context) -> Result<Binding, Box<dyn std::error::Error>> {
    let args = ReadArgs::from_args(args)?;
    let base = std::env::current_dir()?;
    let data = std::fs::read_to_string(base.join(args.path))?;
    Ok(Binding::new(Value::from(data)))
}

pub fn write(args: &Args, _ctx: &dyn Context) -> Result<(), Box<dyn std::error::Error>> {
    let args = WriteArgs::from_args(args)?;
    let base = std::env::current_dir()?;
    Ok(std::fs::write(base.join(args.path), args.data)?)
}
