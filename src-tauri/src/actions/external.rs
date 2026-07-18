use crate::platform::windows::command_output;
use anyhow::{anyhow, Result};
use std::process::Output;

pub fn prune_huggingface() -> Result<(u64, u64, String)> {
    let output = run("hf", &["cache", "prune", "--yes"])?;
    Ok((
        0,
        0,
        concise_output("Hugging Face cache prune completed", &output),
    ))
}

pub fn clean_npm_cache() -> Result<(u64, u64, String)> {
    let output = run("npm", &["cache", "clean", "--force"])?;
    Ok((0, 0, concise_output("npm cache cleanup completed", &output)))
}

pub fn prune_docker() -> Result<(u64, u64, String)> {
    let output = run(
        "docker",
        &["system", "prune", "--force", "--filter", "until=168h"],
    )?;
    Ok((
        0,
        0,
        concise_output(
            "Docker conservative prune completed; volumes were not included",
            &output,
        ),
    ))
}

fn run(executable: &str, arguments: &[&str]) -> Result<Output> {
    let output = command_output(executable, arguments)
        .map_err(|error| anyhow!("Unable to start {executable}: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() {
            String::new()
        } else {
            format!(": {stderr}")
        };
        return Err(anyhow!(
            "{executable} exited with {}{detail}",
            output.status
        ));
    }

    Ok(output)
}

fn concise_output(prefix: &str, output: &Output) -> String {
    let text = String::from_utf8_lossy(&output.stdout)
        .lines()
        .rfind(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_string();

    if text.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}: {text}")
    }
}
