use regex::Regex;
use std::collections::HashSet;

pub fn parse_bib(content: &str) -> HashSet<String> {
    let mut citations = HashSet::new();

    // Regex to match @entry_type{citation_name,
    let re = Regex::new(r#"@\w+\{([^,]+),\s*$"#).unwrap();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('%') {
            continue;
        }

        // Try regex match first (more robust)
        if let Some(caps) = re.captures(line) {
            if let Some(citation) = caps.get(1) {
                citations.insert(citation.as_str().trim().to_string());
                continue;
            }
        }

        // Fallback: simple string matching for @ entries
        if line.starts_with('@') && !line.starts_with("@comment") && !line.starts_with("@preamble")
        {
            if let Some(start) = line.find('{') {
                if let Some(end) = line.find(',') {
                    let citation = &line[start + 1..end].trim();
                    if !citation.is_empty() {
                        citations.insert(citation.to_string());
                    }
                } else {
                    // No comma found, take everything until the end (malformed but try to recover)
                    let citation = &line[start + 1..].trim();
                    if !citation.is_empty() && !citation.ends_with('}') {
                        citations.insert(citation.to_string());
                    }
                }
            }
        }
    }

    citations
}
