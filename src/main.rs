use anyhow::{Context, Result};
use clap::{Arg, Command};
use pest::Parser;
use pest_derive::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
};

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
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let typst_path = Path::new(matches.get_one::<String>("input").unwrap());
    let bib_path = matches.get_one::<String>("bib").map(Path::new);
    let latex_path = match matches.get_one::<&str>("output") {
        Some(filename) => PathBuf::from(filename),
        None => typ2tex(typst_path),
    };
    let verbose = matches.get_flag("verbose");

    // Read the file
    let content = fs::read_to_string(typst_path)
        .with_context(|| format!("Failed to read file: {:?}", typst_path))?;

    let pairs = TypstParser::parse(Rule::program, &content)
        .with_context(|| "Failed to parse input according to grammar")?;

    // Temp printing
    for pair in pairs {
        // A pair is a combination of the rule which matched and a span of input
        println!("Rule:    {:?}", pair.as_rule());
        println!("Span:    {:?}", pair.as_span());
        println!("Text:    {}", pair.as_str());
    }

    Ok(())
}
