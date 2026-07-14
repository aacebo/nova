use nova_core::{FromArgs, Function, Namespace, Pointer, ToType, ToValue, Type, Value};

pub trait FileSystem {
    fn fs(self) -> Self;
}

impl FileSystem for nova_core::Builder {
    fn fs(self) -> Self {
        self.var("fs", Pointer::namespace(Fs))
    }
}

#[derive(Debug)]
pub struct Fs;

impl ToType for Fs {
    fn to_type(&self) -> Type {
        Type::Any
    }
}

impl ToValue for Fs {
    fn to_value(&self) -> Value<'_> {
        Value::Undefined
    }
}

impl Namespace for Fs {
    fn member(&self, name: &str) -> Option<Pointer> {
        match name {
            "read" => Some(Pointer::callable(Function::func("fs.read", read))),
            "write" => Some(Pointer::callable(Function::action("fs.write", write))),
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

    fn from_args(args: &nova_core::Args) -> Result<Self, Self::Error> {
        let path = args
            .at(0)
            .value()
            .as_str()
            .map(|s| s.to_string())
            .ok_or(nova_core::Error::message("path must be a string"))?;

        Ok(Self { path })
    }
}

pub struct WriteArgs {
    pub path: String,
    pub data: Vec<u8>,
}

impl FromArgs for WriteArgs {
    type Error = Box<dyn std::error::Error>;

    fn from_args(args: &nova_core::Args) -> Result<Self, Self::Error> {
        let path = args
            .at(0)
            .value()
            .as_str()
            .map(|s| s.to_string())
            .ok_or(nova_core::Error::message("path must be a string"))?;

        let data = args
            .at(1)
            .value()
            .as_str()
            .map(|s| s.to_string())
            .ok_or(nova_core::Error::message("invalid data type"))?;

        Ok(Self {
            path,
            data: data.into_bytes(),
        })
    }
}

pub fn read(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<Pointer, Box<dyn std::error::Error>> {
    let args = ReadArgs::from_args(args)?;
    let base = std::env::current_dir()?;
    let data = std::fs::read_to_string(base.join(args.path))?;
    Ok(Pointer::new(Value::from(data)))
}

pub fn write(args: &nova_core::Args, _scope: &nova_core::Scope) -> Result<(), Box<dyn std::error::Error>> {
    let args = WriteArgs::from_args(args)?;
    let base = std::env::current_dir()?;
    Ok(std::fs::write(base.join(args.path), args.data)?)
}
