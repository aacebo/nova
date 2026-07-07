#[derive(Debug)]
pub enum Error {
    Clap(clap::Error),
    Glob(glob::GlobError),
    Pattern(glob::PatternError),
    Figment(figment::Error),
    NotFound(Vec<String>),
}

impl From<clap::Error> for Error {
    fn from(value: clap::Error) -> Self {
        Self::Clap(value)
    }
}

impl From<figment::Error> for Error {
    fn from(value: figment::Error) -> Self {
        Self::Figment(value)
    }
}

impl From<glob::GlobError> for Error {
    fn from(value: glob::GlobError) -> Self {
        Self::Glob(value)
    }
}

impl From<glob::PatternError> for Error {
    fn from(value: glob::PatternError) -> Self {
        Self::Pattern(value)
    }
}
