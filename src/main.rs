use clap::{Arg, Command};
use pest::Parser;
use std::fs;
use anyhow::{Result, Context};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct TypstParser;

fn main() -> Result<()> {
    let matches = Command::new("Typst Parser")
        .version("1.0")
        .author("Your Name")
        .about("Parses Typst files using a custom Pest grammar")
        .arg(
            Arg::new("file")
                .help("The input file to parse")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output with parse tree")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("stats")
                .short('s')
                .long("stats")
                .help("Show parsing statistics")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let filename = matches.get_one::<String>("file").unwrap();
    let verbose = matches.get_flag("verbose");
    let stats = matches.get_flag("stats");

    // Read the file
    let input = fs::read_to_string(filename)
        .with_context(|| format!("Failed to read file: {}", filename))?;

    println!("Parsing file: {}", filename);
    
    if verbose {
        println!("File content (first 500 chars):");
        println!("{}", if input.len() > 500 { &input[..500] } else { &input });
        println!("{}", "=".repeat(50));
    }

    // Parse the input
    let start_time = std::time::Instant::now();
    match parse_input(&input, verbose) {
        Ok(pairs) => {
            let duration = start_time.elapsed();
            println!("✅ Parsing successful!");
            
            if stats {
                println!("📊 Statistics:");
                println!("  - Parse time: {:?}", duration);
                println!("  - File size: {} bytes", input.len());
                println!("  - Characters: {}", input.chars().count());
                println!("  - Lines: {}", input.lines().count());
                
                // Count different statement types
                let mut stmt_counts = std::collections::HashMap::new();
                count_statements(&pairs, &mut stmt_counts);
                
                println!("  - Statements found:");
                for (stmt_type, count) in stmt_counts {
                    println!("    - {}: {}", stmt_type, count);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Parsing failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn parse_input(input: &str, verbose: bool) -> Result<Vec<pest::iterators::Pair<Rule>>> {
    let pairs = TypstParser::parse(Rule::program, input)
        .with_context(|| "Failed to parse input according to grammar")?;

    let pairs_vec: Vec<_> = pairs.collect();

    if verbose {
        println!("Parse tree:");
        print_parse_tree(&pairs_vec, 0);
    }

    Ok(pairs_vec)
}

fn print_parse_tree(pairs: &[pest::iterators::Pair<Rule>], depth: usize) {
    for pair in pairs {
        let indent = "  ".repeat(depth);
        let rule = pair.as_rule();
        let span = pair.as_span();
        let text = pair.as_str();
        
        println!("{}{:?} [{:?}]", indent, rule, span);
        
        if text.len() < 100 {
            println!("{}  Text: '{}'", indent, text.replace('\n', "\\n"));
        } else {
            println!("{}  Text: '{}...'", indent, &text[..50].replace('\n', "\\n"));
        }
        
        let inner_pairs: Vec<_> = pair.into_inner().collect();
        if !inner_pairs.is_empty() {
            print_parse_tree(&inner_pairs, depth + 1);
        }
        
        println!("{}---", indent);
    }
}

fn count_statements(pairs: &[pest::iterators::Pair<Rule>], counts: &mut std::collections::HashMap<String, usize>) {
    for pair in pairs {
        let rule = pair.as_rule();
        
        match rule {
            Rule::section | Rule::subsection | Rule::subsubsection => {
                *counts.entry("Sections".to_string()).or_insert(0) += 1;
            }
            Rule::theorem | Rule::proof => {
                *counts.entry("Theorems/Proofs".to_string()).or_insert(0) += 1;
            }
            Rule::figure => {
                *counts.entry("Figures".to_string()).or_insert(0) += 1;
            }
            Rule::header => {
                *counts.entry("Headers".to_string()).or_insert(0) += 1;
            }
            Rule::command => {
                *counts.entry("Commands".to_string()).or_insert(0) += 1;
            }
            Rule::comment => {
                *counts.entry("Comments".to_string()).or_insert(0) += 1;
            }
            Rule::citation => {
                *counts.entry("Citations".to_string()).or_insert(0) += 1;
            }
            Rule::math => {
                *counts.entry("Math blocks".to_string()).or_insert(0) += 1;
            }
            _ => {}
        }
        
        // Recursively count inner pairs
        let inner_pairs: Vec<_> = pair.into_inner().collect();
        count_statements(&inner_pairs, counts);
    }
}
