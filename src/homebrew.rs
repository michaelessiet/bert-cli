use crate::platform::Platform;
use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use serde::Deserialize;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

const HOMEBREW_API_URL: &str = "https://formulae.brew.sh/api/formula";

#[cfg(target_os = "windows")]
const HOMEBREW_INSTALL_URL: &str =
    "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.ps1";
#[cfg(not(target_os = "windows"))]
const HOMEBREW_INSTALL_URL: &str =
    "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh";

#[derive(Debug, Deserialize, Clone)]
pub struct Formula {
    pub name: String,
    pub full_name: String,
    pub desc: Option<String>,
    pub homepage: Option<String>,
    pub versions: Versions,
    #[serde(default)]
    pub versioned_formulae: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub tap: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Versions {
    #[serde(default)]
    pub stable: String,
    #[serde(default)]
    pub head: Option<String>, // Changed to Option
    #[serde(default)]
    pub bottle: bool,
}

impl Formula {
    pub fn get_install_name(&self, version: Option<&str>) -> String {
        if let Some(v) = version {
            let versioned_name = format!("{}@{}", self.name, v);
            if !self.versioned_formulae.is_empty()
                && self.versioned_formulae.contains(&versioned_name)
            {
                versioned_name
            } else {
                println!("{}", format!("Warning: Version {} not found.", v).yellow());

                if !self.versioned_formulae.is_empty() {
                    println!("Available versions:");
                    println!("  Latest: {}", self.versions.stable);
                    println!(
                        "  Other versions: {}",
                        self.versioned_formulae
                            .iter()
                            .map(|v| v.split('@').nth(1).unwrap_or(""))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                } else if !self.versions.stable.is_empty() {
                    println!(
                        "Only latest version ({}) is available.",
                        self.versions.stable
                    );
                } else {
                    println!("No version information available.");
                }

                println!("{}", "Installing latest version instead.".yellow());
                self.name.clone()
            }
        } else {
            self.name.clone()
        }
    }
}

pub fn display_package_info(formula: &Formula) {
    println!("\nPackage Information:");
    println!("  Name: {}", formula.name.green());
    if let Some(desc) = &formula.desc {
        println!("  Description: {}", desc);
    }
    if let Some(license) = &formula.license {
        println!("  License: {}", license);
    }
    if let Some(homepage) = &formula.homepage {
        println!("  Homepage: {}", homepage);
    }
    if let Some(tap) = &formula.tap {
        println!("  Tap: {}", tap);
    }

    println!("\nVersions:");
    if !formula.versions.stable.is_empty() {
        println!("  Current: {} (latest)", formula.versions.stable.green());
    }

    if !formula.versioned_formulae.is_empty() {
        println!("  Other available versions:");
        for version in &formula.versioned_formulae {
            println!("    {}", version.cyan());
        }
    }

    if !formula.aliases.is_empty() {
        println!("\nAliases:");
        for alias in &formula.aliases {
            println!("    {}", alias);
        }
    }
}

pub async fn is_homebrew_installed() -> bool {
    match Platform::current() {
        Platform::Windows => which::which("brew.exe").is_ok(),
        _ => which::which("brew").is_ok(),
    }
}

pub async fn get_homebrew_prefix() -> Result<PathBuf> {
    match Platform::current() {
        Platform::Windows => Ok(PathBuf::from("C:\\Program Files\\Homebrew")),
        Platform::MacOS => Ok(PathBuf::from("/usr/local")),
        Platform::Linux => Ok(PathBuf::from("/home/linuxbrew/.linuxbrew")),
    }
}

pub async fn install_homebrew() -> Result<()> {
    println!("{}", "Homebrew is required but not installed.".yellow());

    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Would you like to install Homebrew?")
        .default(true)
        .interact()?
    {
        anyhow::bail!("Homebrew is required to continue.");
    }

    println!("Installing Homebrew...");

    match Platform::current() {
        Platform::Windows => {
            // Download and execute PowerShell install script
            let install_script = reqwest::get(HOMEBREW_INSTALL_URL).await?.text().await?;

            let status = Command::new("powershell")
                .arg("-Command")
                .arg(&install_script)
                .status()?;

            if !status.success() {
                anyhow::bail!("Failed to install Homebrew");
            }

            // Add Homebrew to PATH
            let home =
                home::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
            let homebrew_path = home.join(".homebrew/bin");

            Command::new("powershell")
                .args(&[
                    "-Command",
                    &format!("$env:Path += ';{}'", homebrew_path.display()),
                    "&",
                    "setx",
                    "PATH",
                    "$env:Path",
                ])
                .status()?;
        }
        Platform::Linux | Platform::MacOS => {
            // Download and execute bash install script
            let install_script = reqwest::get(HOMEBREW_INSTALL_URL).await?.text().await?;

            let status = Command::new("bash")
                .arg("-c")
                .arg(&install_script)
                .status()?;

            if !status.success() {
                anyhow::bail!("Failed to install Homebrew");
            }

            // Source Homebrew in shell configuration
            let shell_config = match std::env::var("SHELL") {
                Ok(shell) if shell.contains("zsh") => home::home_dir().unwrap().join(".zshrc"),
                _ => home::home_dir().unwrap().join(".bashrc"),
            };

            if shell_config.exists() {
                let homebrew_env = match Platform::current() {
                    Platform::Linux => "\neval $(/home/linuxbrew/.linuxbrew/bin/brew shellenv)",
                    _ => "\neval \"$(/usr/local/bin/brew shellenv)\"",
                };

                let mut config_file = std::fs::OpenOptions::new()
                    .append(true)
                    .open(&shell_config)
                    .expect("Failed to open shell configuration file");

                config_file
                    .write(homebrew_env.as_bytes())
                    .expect("Failed to write to shell configuration file");
            }
        }
    }

    println!("{}", "Homebrew installed successfully!".green());
    println!(
        "{}",
        "Please restart your terminal for the changes to take effect.".yellow()
    );
    Ok(())
}

pub async fn install_formula_version(name: &str, version: Option<&str>) -> Result<()> {
    if !is_homebrew_installed().await {
        install_homebrew().await?;
    }

    // For custom taps, we can install directly
    if name.matches('/').count() == 2 {
        println!("Installing {} via Homebrew...", name.cyan());

        let status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
            .args(["install", name])
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to install {}", name);
        }

        println!("{} {} successfully", "Installed".green(), name);
        return Ok(());
    }

    // Regular formula installation
    if let Some(formula) = search_formula(name).await? {
        let install_name = formula.get_install_name(version);
        println!("Installing {} via Homebrew...", install_name.cyan());

        let status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
            .args(["install", &install_name])
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to install {}", install_name);
        }

        println!("{} {} successfully", "Installed".green(), install_name);
    } else {
        anyhow::bail!("Package {} not found", name);
    }

    Ok(())
}

pub async fn search_formula(name: &str) -> Result<Option<Formula>> {
    // Check if the name includes a tap
    let parts: Vec<&str> = name.split('/').collect();
    match parts.len() {
        3 => {
            // Format: tap_user/tap_name/formula (e.g., oven-sh/bun/bun)
            let tap = format!("{}/{}", parts[0], parts[1]);
            let formula_name = parts[2];

            // First ensure the tap is added
            let tap_status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                .args(["tap", &tap])
                .status()?;

            if !tap_status.success() {
                anyhow::bail!("Failed to add tap {}", tap);
            }

            // Try to get formula info
            let output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                .args(["info", "--json=v2", name])
                .output()?;

            if output.status.success() {
                #[derive(Deserialize)]
                struct BrewResponse {
                    formulae: Vec<Formula>,
                }

                let response: BrewResponse = serde_json::from_slice(&output.stdout)?;
                Ok(response.formulae.into_iter().next())
            } else {
                Ok(None)
            }
        }
        1 => {
            // Regular formula from main homebrew/core tap
            let client = reqwest::Client::new();
            let url = format!("https://formulae.brew.sh/api/formula/{}.json", name);
            let response = client.get(&url).send().await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let formula: Formula = resp.json().await?;
                        Ok(Some(formula))
                    } else {
                        Ok(None)
                    }
                }
                Err(_) => Ok(None),
            }
        }
        _ => {
            println!(
                "{}",
                "Invalid package format. Use either 'package' or 'user/tap/package'".yellow()
            );
            Ok(None)
        }
    }
}

pub async fn install_formula(name: &str) -> Result<()> {
    install_formula_version(name, None).await?;

    Ok(())
}
