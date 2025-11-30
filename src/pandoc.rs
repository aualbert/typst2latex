use anyhow::{Context, Result};
use std::process::Command;

/// Converts Typst content to Latex using pandoc
pub fn typst2latex(content: &str) -> Result<String> {
    // Create pandoc process
    let mut pandoc = Command::new("pandoc");

    pandoc
        .args(["-f", "typst", "-t", "latex"]) // From Typst to Latex
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    // Spawn the process
    let mut child = pandoc
        .spawn()
        .context("Failed to spawn pandoc process. Is pandoc installed?")?;

    // Write LaTeX content to stdin
    {
        let stdin = child.stdin.as_mut().context("Failed to get pandoc stdin")?;
        std::io::Write::write_all(stdin, content.as_bytes())
            .context("Failed to write LaTeX content to pandoc")?;
    }

    // Wait for completion and get output
    let output = child
        .wait_with_output()
        .context("Failed to get pandoc output")?;

    // Check if pandoc succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Pandoc conversion failed: {}", stderr);
    }

    // Get the converted content
    let typst_output =
        String::from_utf8(output.stdout).context("Pandoc produced invalid UTF-8 output")?;

    // Remove trailing newline that pandoc adds
    Ok(apply_unicode2tex(typst_output.trim_end()))
}

// Postprocessing to fix pandoc output. Pandoc WILL output unicode character rather than math commands for the usual symbols, e.g. 𝛼 instead of \alpha.

fn apply_unicode2tex(text: &str) -> String {
    let mut result = String::new();
    for c in text.chars() {
        if let Some(tex_cmd) = unicode2tex(c) {
            result.push_str(tex_cmd);
        } else {
            result.push(c);
        }
    }

    result
}

fn unicode2tex(c: char) -> Option<&'static str> {
    match c {
        // Lowercase
        'α' => Some("\\alpha"),
        'β' => Some("\\beta"),
        'γ' => Some("\\gamma"),
        'δ' => Some("\\delta"),
        'ε' => Some("\\varepsilon"),
        'ζ' => Some("\\zeta"),
        'η' => Some("\\eta"),
        'θ' => Some("\\theta"),
        'ι' => Some("\\iota"),
        'κ' => Some("\\kappa"),
        'λ' => Some("\\lambda"),
        'μ' => Some("\\mu"),
        'ν' => Some("\\nu"),
        'ξ' => Some("\\xi"),
        'ο' => Some("\\omicron"),
        'π' => Some("\\pi"),
        'ρ' => Some("\\rho"),
        'σ' => Some("\\sigma"),
        'τ' => Some("\\tau"),
        'υ' => Some("\\upsilon"),
        'φ' => Some("\\varphi"),
        'χ' => Some("\\chi"),
        'ψ' => Some("\\psi"),
        'ω' => Some("\\omega"),

        // Uppercase
        'Α' => Some("\\Alpha"),
        'Β' => Some("\\Beta"),
        'Γ' => Some("\\Gamma"),
        'Δ' => Some("\\Delta"),
        'Ε' => Some("\\Epsilon"),
        'Ζ' => Some("\\Zeta"),
        'Η' => Some("\\Eta"),
        'Θ' => Some("\\Theta"),
        'Ι' => Some("\\Iota"),
        'Κ' => Some("\\Kappa"),
        'Λ' => Some("\\Lambda"),
        'Μ' => Some("\\Mu"),
        'Ν' => Some("\\Nu"),
        'Ξ' => Some("\\Xi"),
        'Ο' => Some("0"),
        'Π' => Some("\\Pi"),
        'Ρ' => Some("\\Rho"),
        'Σ' => Some("\\Sigma"),
        'Τ' => Some("\\Tau"),
        'Υ' => Some("\\Upsilon"),
        'Φ' => Some("\\Phi"),
        'Χ' => Some("\\Chi"),
        'Ψ' => Some("\\Psi"),
        'Ω' => Some("\\Omega"),

        // Variants
        'ϵ' => Some("\\epsilon"),
        'ϑ' => Some("\\vartheta"),
        'ϖ' => Some("\\varpi"),
        'ϱ' => Some("\\varrho"),
        'ς' => Some("\\varsigma"),
        'ϕ' => Some("\\phi"),

        _ => None,
    }
}
