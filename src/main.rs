use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::process::Command;

// Import our local modules
mod backup_manager;
mod command_handler;
mod homebrew;
mod package_manager;
mod platform;
mod self_update;

#[derive(Parser)]
#[command(
    name = "bert",
    author = "Michael Essiet <emsaa2002@gmail.com>",
    version = "0.1.6",
    about = "A friendly cross-platform package assistant built on top of Homebrew",
    long_about = "Bert 🐕 is a friendly package assistant that leverages Homebrew's package repository to provide \
                  cross-platform package management. He automatically handles installation of missing \
                  commands and manages Homebrew installations. Heavily inspired by Bert Solana's #1 dog Bertram the Pomeranian!"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Install as a cask application
    #[arg(long, global = true)]
    cask: bool,

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
    /// Uninstall a package
    Uninstall {
        /// Name of the package to uninstall
        package: String,
    },
    /// List installed packages
    List,
    /// Update bert to the latest version
    SelfUpdate,
    /// Create a backup of installed formulas and casks
    Backup {
        /// Optional custom path for the backup file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Restore packages from a backup file
    Restore {
        /// Optional path to the backup file (uses latest backup if not specified)
        #[arg(short, long)]
        input: Option<String>,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    // Enable colored output on Windows
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).ok();

    let cli = Cli::parse();

    match cli.command {
        // ... other command matches ...
        Some(Commands::Backup { output }) => {
            backup_manager::create_backup(output.as_deref()).await?;
        }
        Some(Commands::Restore { input }) => {
            backup_manager::restore_backup(input.as_deref()).await?;
        }
        Some(Commands::SelfUpdate) => {
            self_update::self_update().await?;
        }
        Some(Commands::Uninstall { package }) => {
            package_manager::uninstall_package(&package).await?;
        }
        Some(Commands::Install { package }) => {
            // Parse package name and version
            let (name, version) = parse_package_spec(&package);
            println!("Installing package: {} 🐕", name.cyan());
            if let Some(ver) = version {
                println!("Version: {}", ver.cyan());
            }

            package_manager::install_package_version(name, version, cli.cask)
                .await
                .with_context(|| format!("Failed to install package: {}", package))?;
        }
        Some(Commands::Search { query }) => {
            println!("Searching for packages matching: {} 🐕", query.cyan());
            if let Some(formula) = homebrew::search_formula(
                &query,
                if cli.cask {
                    Some(homebrew::HomebrewPackageType::Cask)
                } else {
                    Some(homebrew::HomebrewPackageType::Formula)
                },
            )
            .await?
            {
                homebrew::display_package_info(&formula, cli.cask);
            } else {
                println!("No packages found matching: {}", query.red());
            }
        }
        Some(Commands::Update { packages }) => {
            if packages.is_empty() {
                println!("{}", "Updating Homebrew 🐕".cyan());
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
                println!("Updating {} 🐕", package.cyan());
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
            let formula_output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                .args(["list", "--versions", "--formula"])
                .output()?;

            if formula_output.status.success() {
                let packages = String::from_utf8_lossy(&formula_output.stdout);
                println!("{}", "Formulae:".cyan());
                for package in packages.lines() {
                    println!("  {}", package);
                }
            } else {
                println!("{}", "Failed to list packages".red());
            }

            let cask_output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                .args(["list", "--versions", "--cask"])
                .output()?;

            if cask_output.status.success() {
                let packages = String::from_utf8_lossy(&cask_output.stdout);
                println!("{}", "Casks:".cyan());
                for package in packages.lines() {
                    println!("  {}", package);
                }
            } else {
                println!("{}", "Failed to list casks".red());
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

fn parse_package_spec(spec: &str) -> (&str, Option<&str>) {
    if let Some(idx) = spec.find('@') {
        let (name, version) = spec.split_at(idx);
        (name, Some(&version[1..]))
    } else {
        (spec, None)
    }
}
