use crate::platform::Platform;
use anyhow::Result;
use colored::*;
use std::process::{Command, Stdio};
use which::which;

pub async fn handle_command(args: &[String]) -> Result<()> {
    let command = &args[0];
    let command_name = if Platform::current() == Platform::Windows && !command.ends_with(".exe") {
        format!("{}.exe", command)
    } else {
        command.to_string()
    };

    // Check if command exists
    if which(&command_name).is_err() {
        println!("{} not found. Attempting to install...", command.yellow());

        // Try to install via homebrew
        if let Err(e) = crate::package_manager::install_package(command).await {
            println!("Failed to install {}: {}", command.red(), e);
            return Ok(());
        }
    }

    // Execute command with remaining args
    let status = Command::new(&command_name)
        .args(&args[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("Command failed with exit code: {}", status);
    }

    Ok(())
}
