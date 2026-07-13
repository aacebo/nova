use candle_core::Device;
use tokenizers::Tokenizer;

use super::config::Config;
use crate::models::distilbert;
use crate::pipelines::common::Batch;
use crate::resources::{Error, Repo, Result};
use crate::types::{Polarity, Sentiment as Output};

pub struct Sentiment {
    classifier: distilbert::SequenceClassifier,
    tokenizer: Tokenizer,
    device: Device,
}

impl Sentiment {
    pub(super) fn new(config: Config) -> Result<Self> {
        let repo = Repo::open(config.model, config.device, config.dtype)?;
        let model: distilbert::Config = repo.config()?;
        let device = repo.device().clone();

        Ok(Self {
            classifier: distilbert::SequenceClassifier::new(repo.vars()?, &model)?,
            tokenizer: repo.tokenizer()?,
            device,
        })
    }

    pub fn predict<S: AsRef<str>>(&self, text: &[S]) -> Result<Vec<Output>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let text: Vec<&str> = text.iter().map(AsRef::as_ref).collect();
        let encodings = self.tokenizer.encode_batch(text, true).map_err(Error::tokenize)?;
        let batch = Batch::new(encodings, &self.device)?;

        let probs = self.classifier.forward(&batch.ids, &batch.padding()?)?;

        Ok(probs
            .into_iter()
            .map(|row| {
                // SST-2 label order: 0 = NEGATIVE, 1 = POSITIVE.
                let (negative, positive) = (row[0], row[1]);

                if positive >= negative {
                    Output {
                        polarity: Polarity::Positive,
                        score: positive as f64,
                    }
                } else {
                    Output {
                        polarity: Polarity::Negative,
                        score: negative as f64,
                    }
                }
            })
            .collect())
    }
}
