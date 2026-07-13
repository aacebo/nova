use super::stopwords;
use crate::types::Offset;

const MIN_LENGTH: usize = 3;

pub struct Candidate {
    pub text: String,
    pub offsets: Vec<Offset>,
}

/// Distinct, non-stopword words, each carrying every BYTE offset it occurs at.
/// (`routines/keywords.rs` converts these to char offsets.)
pub fn extract(text: &str) -> Vec<Candidate> {
    let mut candidates: Vec<Candidate> = Vec::new();

    for (start, word) in words(text) {
        let lowered = word.to_lowercase();

        if lowered.chars().count() < MIN_LENGTH || stopwords::contains(&lowered) {
            continue;
        }

        let offset = Offset::new(start as u32, (start + word.len()) as u32);

        match candidates.iter_mut().find(|candidate| candidate.text == lowered) {
            Some(candidate) => candidate.offsets.push(offset),
            None => candidates.push(Candidate {
                text: lowered,
                offsets: vec![offset],
            }),
        }
    }

    candidates
}

/// Alphanumeric runs paired with their starting BYTE offset.
fn words(text: &str) -> Vec<(usize, &str)> {
    let mut words = Vec::new();
    let mut start: Option<usize> = None;

    for (index, ch) in text.char_indices() {
        match (ch.is_alphanumeric(), start) {
            (true, None) => start = Some(index),
            (false, Some(begin)) => {
                words.push((begin, &text[begin..index]));
                start = None;
            }
            _ => {}
        }
    }

    if let Some(begin) = start {
        words.push((begin, &text[begin..]));
    }

    words
}
