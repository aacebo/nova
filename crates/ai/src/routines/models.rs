use std::cell::OnceCell;

use rust_bert::pipelines::keywords_extraction::{KeywordExtractionConfig, KeywordExtractionModel};
use rust_bert::pipelines::ner::NERModel;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use rust_bert::pipelines::sentiment::SentimentModel;
use rust_bert::pipelines::summarization::SummarizationModel;
use rust_bert::pipelines::token_classification::{TokenClassificationConfig, TokenClassificationModel};

type BoxError = Box<dyn std::error::Error>;

thread_local! {
    static SENTIMENT: OnceCell<SentimentModel> = const { OnceCell::new() };
    static NER: OnceCell<NERModel> = const { OnceCell::new() };
    static KEYWORDS: OnceCell<KeywordExtractionModel<'static>> = const { OnceCell::new() };
    static PII: OnceCell<TokenClassificationModel> = const { OnceCell::new() };
    static EMBEDDINGS: OnceCell<SentenceEmbeddingsModel> = const { OnceCell::new() };
    static SUMMARIZATION: OnceCell<SummarizationModel> = const { OnceCell::new() };
}

fn with_model<M, R>(
    cell: &'static std::thread::LocalKey<OnceCell<M>>,
    init: impl FnOnce() -> Result<M, BoxError>,
    f: impl FnOnce(&M) -> R,
) -> Result<R, BoxError> {
    cell.with(|cell| {
        if cell.get().is_none() {
            let _ = cell.set(init()?);
        }

        Ok(f(cell.get().expect("model initialized above")))
    })
}

pub fn with_sentiment<R>(f: impl FnOnce(&SentimentModel) -> R) -> Result<R, BoxError> {
    with_model(&SENTIMENT, || Ok(SentimentModel::new(Default::default())?), f)
}

pub fn with_ner<R>(f: impl FnOnce(&NERModel) -> R) -> Result<R, BoxError> {
    with_model(&NER, || Ok(NERModel::new(Default::default())?), f)
}

pub fn with_keywords<R>(f: impl FnOnce(&KeywordExtractionModel<'static>) -> R) -> Result<R, BoxError> {
    with_model(
        &KEYWORDS,
        || Ok(KeywordExtractionModel::new(KeywordExtractionConfig::default())?),
        f,
    )
}

pub fn with_pii<R>(f: impl FnOnce(&TokenClassificationModel) -> R) -> Result<R, BoxError> {
    with_model(
        &PII,
        || Ok(TokenClassificationModel::new(TokenClassificationConfig::default())?),
        f,
    )
}

pub fn with_embeddings<R>(f: impl FnOnce(&SentenceEmbeddingsModel) -> R) -> Result<R, BoxError> {
    with_model(
        &EMBEDDINGS,
        || Ok(SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2).create_model()?),
        f,
    )
}

pub fn with_summarization<R>(f: impl FnOnce(&SummarizationModel) -> R) -> Result<R, BoxError> {
    with_model(&SUMMARIZATION, || Ok(SummarizationModel::new(Default::default())?), f)
}
