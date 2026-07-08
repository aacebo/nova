use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trigger {
    Run { priority: Option<i64> },
    Call,
}

impl Trigger {
    pub fn is_run(&self) -> bool {
        matches!(self, Self::Run { .. })
    }

    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call)
    }

    pub fn priority(&self) -> Option<i64> {
        match self {
            Self::Run { priority } => *priority,
            Self::Call => None,
        }
    }
}

impl FromStr for Trigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s == "call" {
            return Ok(Self::Call);
        }

        let rest = s.strip_prefix("run").ok_or_else(|| format!("unknown trigger `{s}`"))?.trim();

        if rest.is_empty() {
            return Ok(Self::Run { priority: None });
        }

        let inner = rest
            .strip_prefix('(')
            .and_then(|r| r.strip_suffix(')'))
            .ok_or_else(|| format!("invalid trigger `{s}`, expected `run` or `run(<priority>)`"))?
            .trim();

        let priority = inner
            .parse::<i64>()
            .map_err(|_| format!("invalid priority `{inner}` in trigger `{s}`"))?;

        Ok(Self::Run {
            priority: Some(priority),
        })
    }
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Run { priority: Some(p) } => write!(f, "run({p})"),
            Self::Run { priority: None } => write!(f, "run"),
            Self::Call => write!(f, "call"),
        }
    }
}

impl serde::Serialize for Trigger {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Trigger {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}
