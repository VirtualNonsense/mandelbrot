use anyhow::{Context, Result, anyhow};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run(mut cmd: Command) -> Result<()> {
    eprintln!("> {:?}", cmd);
    let status = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to spawn process")?;

    if !status.success() {
        return Err(anyhow!("command failed with status: {status}"));
    }
    Ok(())
}

pub fn cmd_in_dir(exe: &str, dir: &Path) -> Command {
    let mut c = Command::new(exe);
    c.current_dir(dir);
    c
}
