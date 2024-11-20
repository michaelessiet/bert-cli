use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;

// Import our local modules
mod backup_manager;
mod command_handler;
mod config;
mod homebrew;
mod node;
mod package_manager;
mod platform;
mod self_update;

#[derive(Parser)]
#[command(
    name = "bert",
    author = "Michael Essiet <emsaa2002@gmail.com>",
    version = env!("CARGO_PKG_VERSION"),
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

    /// Install a node package globally
    #[arg(long, global = true)]
    node: bool,

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
    SetManager {
        /// Package manager to use (npm, yarn, or pnpm)
        manager: String,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    // Enable colored output on Windows
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).ok();

    // Load config at startup
    let mut config = config::Config::load()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::SetManager { manager }) => {
            let npm_manager = node::NodePackageManager::from_str(&manager)?;
            config.set_node_package_manager(npm_manager)?;
            println!("Package manager set to: {}", manager.green());
        }
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
            package_manager::uninstall_package(&package, cli.cask, cli.node).await?;
        }
        Some(Commands::Install { package }) => {
            // Parse package name and version
            let (name, version) = parse_package_spec(&package);
            println!("Installing package: {} 🐕", name.cyan());
            if let Some(ver) = version {
                println!("Version: {}", ver.cyan());
            }

            package_manager::install_package_version(name, version, cli.cask, cli.node)
                .await
                .with_context(|| format!("Failed to install package: {}", package))?;
        }
        Some(Commands::Search { query }) => {
            println!("Searching for packages matching: {} 🐕", query.cyan());
            package_manager::search_package(&query, cli.cask, cli.node).await?;
        }
        Some(Commands::Update { packages }) => {
            crate::package_manager::update_packages(&packages, cli.node).await?;
        }
        Some(Commands::List) => {
            println!("{}", "Installed packages:".cyan());
            package_manager::list_packages(cli.node).await?;
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
