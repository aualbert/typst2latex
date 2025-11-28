use anyhow::{Context, Result};
use std::process::Command;

/// Converts Typst content to Latex using pandoc
/// Removes trailing newline added by pandoc
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
    Ok(typst_output.trim_end().to_string())
}
