#[derive(Debug)]
pub enum Error {
    Load(String),
    Tokenize(String),
    Inference(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load(message) => write!(f, "failed to load model: {message}"),
            Self::Tokenize(message) => write!(f, "failed to tokenize input: {message}"),
            Self::Inference(message) => write!(f, "inference failed: {message}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn load(err: impl std::fmt::Display) -> Self {
        Self::Load(err.to_string())
    }

    pub fn tokenize(err: impl std::fmt::Display) -> Self {
        Self::Tokenize(err.to_string())
    }

    pub fn inference(err: impl std::fmt::Display) -> Self {
        Self::Inference(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
