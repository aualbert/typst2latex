mod bib_parser;
mod document;

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

fn print_pairs(pairs: Pairs<Rule>) -> () {
    fn print_depth(pairs: Pairs<Rule>, depth: usize) -> () {
        for pair in pairs {
            let indent = "   ".repeat(depth);
            println!("{indent}{:?}", pair.as_rule());
            print_depth(pair.into_inner(), depth + 1);
        }
    }
    print_depth(pairs, 0)
}

fn explore(pairs: Pairs<Rule>) -> Document {
    Document::default()
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
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let verbose = matches.get_flag("verbose");
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

    explore(pairs);

    // TODO change for content
    // Write the latex file
    fs::write(&latex_path, &content)
        .with_context(|| format!("Failed to write file: {:?}", latex_path))?;

    Ok(())
}
