use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::process::Command;

// Import our local modules
mod command_handler;
mod homebrew;
mod package_manager;
mod platform;

/// Goon: A cross-platform package manager built on top of Homebrew
#[derive(Parser)]
#[command(
    name = "bert",
    author = "Michael Essiet <emsaa2002@gmail.com>",
    version = "0.1.0",
    about = "A cross-platform package manager built on top of Homebrew",
    long_about = "Bert is a package manager that leverages Homebrew's package repository to provide \
                  cross-platform package management. It automatically handles installation of missing \
                  commands and manages Homebrew installation. Heavily inspired by Bert Solana's #1 dog."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Command to execute if no subcommand is provided
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a package
    Install {
        /// Name of the package to install
        package: String,
    },
    /// Search for a package
    Search {
        /// Name of the package to search for
        query: String,
    },
    /// Update installed packages
    Update {
        /// Optional package names to update
        #[arg(trailing_var_arg = true)]
        packages: Vec<String>,
    },
    /// List installed packages
    List,
}
#[tokio::main]
async fn main() -> Result<()> {
    // Enable colored output on Windows
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).ok();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install { package }) => {
            // Parse package name and version
            let (name, version) = parse_package_spec(&package);
            println!("Installing package: {}", name.cyan());
            if let Some(ver) = version {
                println!("Version: {}", ver.cyan());
            }

            package_manager::install_package_version(name, version)
                .await
                .with_context(|| format!("Failed to install package: {}", package))?;
        }
        Some(Commands::Search { query }) => {
            println!("Searching for packages matching: {}", query.cyan());
            if let Some(formula) = homebrew::search_formula(&query).await? {
                homebrew::display_package_info(&formula);
            } else {
                println!("No packages found matching: {}", query.red());
            }
        }
        Some(Commands::Update { packages }) => {
            if packages.is_empty() {
                println!("{}", "Updating Homebrew...".cyan());
                let status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                    .arg("update")
                    .status()?;

                if !status.success() {
                    println!("{}", "Failed to update Homebrew".red());
                    return Ok(());
                }
                println!("{}", "Homebrew updated successfully".green());
            }

            let packages_to_update = if packages.is_empty() {
                // Get list of all installed packages
                let output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                    .args(["list", "--formula"])
                    .output()?;

                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(String::from)
                    .collect::<Vec<_>>()
            } else {
                packages
            };

            for package in packages_to_update {
                println!("Updating {}", package.cyan());
                let status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                    .args(["upgrade", &package])
                    .status()?;

                if status.success() {
                    println!("{} updated successfully", package.green());
                } else {
                    println!("Failed to update {}", package.red());
                }
            }
        }
        Some(Commands::List) => {
            println!("{}", "Installed packages:".cyan());
            let output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                .args(["list", "--versions"])
                .output()?;

            if output.status.success() {
                let packages = String::from_utf8_lossy(&output.stdout);
                for package in packages.lines() {
                    println!("  {}", package);
                }
            } else {
                println!("{}", "Failed to list packages".red());
            }
        }
        None => {
            if !cli.args.is_empty() {
                command_handler::handle_command(&cli.args)
                    .await
                    .with_context(|| format!("Failed to execute command: {}", cli.args[0]))?;
            } else {
                println!("No command specified. Use --help for usage information.");
            }
        }
    }

    Ok(())
}

// Error handling utilities
fn print_error(err: &anyhow::Error) {
    eprintln!("{} {}", "Error:".red().bold(), err);

    // Print error chain
    let mut source = err.source();
    while let Some(err) = source {
        eprintln!("Caused by: {}", err);
        source = err.source();
    }
}

fn parse_package_spec(spec: &str) -> (&str, Option<&str>) {
    if let Some(idx) = spec.find('@') {
        let (name, version) = spec.split_at(idx);
        (name, Some(&version[1..]))
    } else {
        (spec, None)
    }
}