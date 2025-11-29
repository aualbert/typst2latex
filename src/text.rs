use crate::pandoc::typst2latex;
use anyhow::{Context, Result};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum Text {
    Raw(String),
    Citation(String),
    Latex(String),
}

fn unique_id(count: usize) -> String {
    format!("identifier{}identifier", count)
}

fn key_to_str(key: &str, citations: &HashSet<String>) -> String {
    let clean_key = key.trim_start_matches('@');

    // Check for trailing space
    let has_trailing_space = key.ends_with(' ');

    let citation = if citations.contains(clean_key) {
        format!("\\cite{{{}}}", clean_key.trim())
    } else {
        format!("\\ref{{{}}}", clean_key.trim())
    };

    if has_trailing_space {
        format!("{} ", citation)
    } else {
        citation
    }
}

pub fn to_latex(vec: Vec<Text>, citations: &HashSet<String>) -> Result<String> {
    let id_string = build_id_string(&vec);
    let mut latex_string = typst2latex(&id_string)
        .with_context(|| format!("failed to convert to latex: {:?}", id_string))?;

    let mut count = 0;
    for text in vec {
        match text {
            Text::Citation(key) => {
                count += 1;
                latex_string =
                    latex_string.replace(&unique_id(count), &key_to_str(&key, citations));
            }
            Text::Latex(content) => {
                count += 1;
                latex_string = latex_string.replace(&unique_id(count), &content);
            }

            _ => (),
        }
    }
    Ok(latex_string)
}

fn build_id_string(vec: &Vec<Text>) -> String {
    let mut result = String::new();
    let mut count = 0;

    for text in vec {
        match text {
            Text::Raw(content) => {
                result.push_str(&content);
            }
            Text::Citation(_) | Text::Latex(_) => {
                count += 1;
                let unique_id = unique_id(count);
                result.push_str(&unique_id);
            }
        }
    }
    result
}
