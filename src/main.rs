mod bib_parser;
mod document;
mod pandoc;
mod text;

use anyhow::{Context, Result};
use bib_parser::parse_bib;
use clap::{Arg, Command};
use document::Document;
use pest::{
    Parser,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use text::{Text, to_latex};

const DEFAULT_TEMPLATE: &str = include_str!("template.tex");

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TypstParser;

fn typ2tex(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    match path.parent() {
        Some(parent) => parent.join(format!("{}.tex", stem)),
        None => format!("{}.tex", stem).into(),
    }
}

fn process_text(pair: Pair<Rule>) -> Vec<Text> {
    fn process_inner(pair: Pair<Rule>, current: &mut String, result: &mut Vec<Text>) {
        match pair.as_rule() {
            Rule::newline => {
                current.push('\n');
            }
            Rule::raw_text | Rule::math | Rule::grid => {
                current.push_str(pair.as_str());
            }
            Rule::citation => {
                if !current.is_empty() {
                    result.push(Text::Raw(std::mem::take(current)));
                }
                result.push(Text::Citation(pair.as_str().into()));
            }
            Rule::paren_text | Rule::paren_line => {
                current.push('(');
                for inner_pair in pair.into_inner() {
                    process_inner(inner_pair, current, result);
                }
                current.push(')');
            }
            Rule::brack_text | Rule::brack_line => {
                current.push('[');
                for inner_pair in pair.into_inner() {
                    process_inner(inner_pair, current, result);
                }
                current.push(']');
            }
            Rule::quote_text | Rule::quote_line => {
                current.push('\"');
                for inner_pair in pair.into_inner() {
                    process_inner(inner_pair, current, result);
                }
                current.push('\"');
            }
            _ => {
                // For other rules, recursively process their inner pairs
                for inner_pair in pair.into_inner() {
                    process_inner(inner_pair, current, result);
                }
            }
        }
    }

    let mut result = Vec::new();
    let mut current = String::new();
    process_inner(pair, &mut current, &mut result);
    if !current.is_empty() {
        result.push(Text::Raw(current))
    }
    result
}

fn explore(pairs: Pairs<Rule>, citations: HashSet<String>) -> Result<Document> {
    let mut content = String::new();
    let mut document = Document::default();

    fn get_str(pair: Pair<Rule>, citations: &HashSet<String>) -> Result<String> {
        to_latex(process_text(pair), citations)
    }

    fn get_inner_str(pair: Pair<Rule>, citations: &HashSet<String>) -> Result<String> {
        let vec = pair
            .into_inner()
            .next()
            .map(process_text)
            .unwrap_or_default();
        to_latex(vec, citations)
    }

    macro_rules! gs {
        ($pair:expr) => {
            get_str($pair, &citations)?
        };
    }

    macro_rules! gis {
        ($pair:expr) => {
            get_inner_str($pair, &citations)?
        };
    }

    for pair in pairs {
        match pair.as_rule() {
            Rule::newline => content += "\n",
            Rule::section => content += &format!("\\section{{{}}}\n", gis!(pair)),
            Rule::subsection => content += &format!("\\subsection{{{}}}\n", gis!(pair)),
            Rule::subsubsection => content += &format!("\\subsubsection{{{}}}\n", gis!(pair)),
            Rule::line => content += &gs!(pair),
            Rule::proof => content += &format!("\\begin{{proof}}{}\\end{{proof}}", gis!(pair)),
            Rule::figure => {
                let mut fcontent = String::new();
                let mut caption = String::new();
                let mut label = String::new();
                for p in pair.into_inner() {
                    match p.as_rule() {
                        Rule::fig_content => {
                            fcontent = gis!(p);
                        }
                        Rule::caption => {
                            caption = gis!(p);
                        }
                        Rule::label => {
                            label = format!("\\label{{{}}}", p.as_str());
                        }
                        _ => {}
                    }
                }
                content += &format!(
                    "\\begin{{figure}}\n{fcontent}\n\\caption{{{caption}}}\n{label}\\end{{figure}}"
                )
            }
            Rule::theorem => {
                let mut ttype = String::new();
                let mut title = String::new();
                let mut tcontent = String::new();
                let mut label = String::new();
                for p in pair.into_inner() {
                    match p.as_rule() {
                        Rule::th_type => {
                            ttype = p.as_str().to_string();
                        }
                        Rule::th_title => {
                            title = format!("{{{}}}", gis!(p));
                        }
                        Rule::th_content => {
                            tcontent = gis!(p);
                        }
                        Rule::label => {
                            label = format!("\\label{{{}}}", p.as_str());
                        }
                        _ => {}
                    }
                }
                content +=
                    &format!("\\begin{{{ttype}}}{title}{label}\n{tcontent}\n\\end{{{ttype}}}\n");
            }
            Rule::header => {
                for p in pair.into_inner() {
                    match p.as_rule() {
                        Rule::my_title => document.title = Some(gis!(p)),
                        Rule::my_abstract => document.abstractt = Some(gis!(p)),
                        Rule::my_name => document.authors = Some(p.as_str().into()),
                        Rule::my_bib => document.bibliography = Some(gis!(p)),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    document.content = content;
    Ok(document)
}

fn main() -> Result<()> {
    let matches = Command::new("Typst Parser")
        .version("1.0")
        .author("Your Name")
        .about("Parses Typst files using a custom Pest grammar")
        .arg(
            Arg::new("input")
                .help("The input typst file to parse")
                .required(true),
        )
        .arg(
            Arg::new("bib")
                .short('b')
                .long("bib")
                .help("A bib file for distinguishing citations and references"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("The output latex file to generate"),
        )
        .arg(
            Arg::new("template")
                .short('t')
                .long("template")
                .help("The latex template to use"),
        )
        .arg(
            Arg::new("backend")
                .long("backend")
                .help("The backend for converting typst to latex")
                .value_parser(["pandoc"])
                .default_value("pandoc"),
        )
        .get_matches();

    let typst_path = Path::new(matches.get_one::<String>("input").unwrap());
    let template_path = matches.get_one::<String>("template").map(Path::new);
    let bib_path = matches.get_one::<String>("bib").map(Path::new);
    let latex_path = matches
        .get_one::<&str>("output")
        .map_or(typ2tex(typst_path), PathBuf::from);

    // Read the typst file
    let content = fs::read_to_string(typst_path)
        .with_context(|| format!("Failed to read file: {:?}", typst_path))?;

    let pairs = TypstParser::parse(Rule::program, &content)
        .with_context(|| "Failed to parse input according to grammar")?;

    // Read the latex template
    let template = match template_path {
        Some(path) => fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", typst_path))?,
        None => DEFAULT_TEMPLATE.into(),
    };

    // Read the bib file
    let citations = match bib_path {
        Some(path) => parse_bib(
            &fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {:?}", typst_path))?,
        ),
        None => HashSet::<String>::new(),
    };

    let document = explore(pairs, citations)?;

    // Write the latex file
    fs::write(&latex_path, document.to_latex(template))
        .with_context(|| format!("Failed to write file: {:?}", latex_path))?;

    Ok(())
}
