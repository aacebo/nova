use nova::{Args, Value};

pub struct Input {
    pub text: Vec<String>,
    pub min_score: f32,
}

impl Input {
    pub fn from_args(args: &Args) -> Result<Self, Box<dyn std::error::Error>> {
        let text = Self::text(&args.at(0))?;
        let min_score = match args.key("min_score") {
            v if v.is_undefined() || v.is_none() => 0.0,
            v => f64::try_from(v).map_err(|_| nova::Error::message("min_score must be a number"))? as f32,
        };

        Ok(Self { text, min_score })
    }
}

impl Input {
    fn text(value: &Value) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if let Some(text) = value.as_str() {
            return Ok(vec![text.to_string()]);
        }

        let mut out = Vec::new();

        for item in value.try_iter()? {
            let item = item
                .as_str()
                .ok_or(nova::Error::message("text must be a string or list of strings"))?;
            out.push(item.to_string());
        }

        Ok(out)
    }
}
