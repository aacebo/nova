use crate::pipelines::keywords;
use crate::routines::Input;
use crate::{Annotation, Offset};

pub fn keyword_extraction(
    args: &nova_core::Args,
    _scope: &nova_core::Scope,
) -> Result<nova_core::Value, Box<dyn std::error::Error>> {
    let input = Input::from_args(args)?;
    let out = keywords::get()?.predict(&input.text)?;
    let mut annotations: Vec<Annotation> = Vec::new();

    for (index, keywords) in out.into_iter().enumerate() {
        let text = input.text.get(index).map(String::as_str).unwrap_or_default();

        for keyword in keywords.into_iter().filter(|v| v.score >= input.min_score) {
            annotations.push(Annotation {
                name: String::from("keyword"),
                label: keyword.text.clone(),
                text: keyword.text,
                score: keyword.score as f64,
                spans: keyword
                    .offsets
                    .iter()
                    .map(|offset| span_from_byte_offsets(text, offset.begin, offset.end))
                    .collect(),
            });
        }
    }

    Ok(nova_core::Value::from(
        annotations.into_iter().map(nova_core::Value::from_object).collect::<Vec<_>>(),
    ))
}

fn span_from_byte_offsets(text: &str, start: u32, end: u32) -> Offset {
    Offset::new(byte_offset_to_char_offset(text, start), byte_offset_to_char_offset(text, end))
}

fn byte_offset_to_char_offset(text: &str, byte_offset: u32) -> u32 {
    let byte_offset = (byte_offset as usize).min(text.len());

    text.char_indices().take_while(|(index, _)| *index < byte_offset).count() as u32
}
