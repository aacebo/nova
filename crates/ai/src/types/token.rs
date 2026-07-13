use super::Offset;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub text: String,
    pub label: String,
    pub score: f64,
    pub offset: Option<Offset>,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.text, self.label)
    }
}
