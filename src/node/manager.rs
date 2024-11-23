use super::types::*;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::{process::Command, thread, time::Duration};

pub struct NodeManager {
    package_manager: NodePackageManager,
}

impl NodeManager {
    pub fn new(package_manager: NodePackageManager) -> Self {
        Self { package_manager }
    }

    pub async fn install_package(&self, name: &str, version: Option<&str>) -> Result<()> {
        if !self.is_node_installed() {
            println!("Node.js is required. Installing Node.js first...");
            // Use homebrew module to install node
            crate::homebrew::install_formula_version("node", None, false).await?;
        }

        let mut args = self.package_manager.install_args();
        let package_with_version = match version {
            Some(v) => format!("{}@{}", name, v),
            None => name.to_string(),
        };
        args.push(&package_with_version);

        println!(
            "Installing {} via {}...",
            package_with_version.cyan(),
            self.package_manager.command()
        );

        let progress_bar = ProgressBar::new(100);
        let mut child = Command::new(self.package_manager.command())
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        // Create a simple spinner style
        progress_bar.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}"));
        progress_bar.set_message(&format!("Installing {}", name));

        while child.try_wait()?.is_none() {
            progress_bar.tick();
            thread::sleep(Duration::from_millis(100));
        }

        // Just wait for the process to complete
        let status = child.wait()?;

        if status.success() {
            progress_bar.set_style(ProgressStyle::default_spinner().template("{msg}"));
            progress_bar.finish_with_message(&format!(
                "{} Successfully installed {}",
                "✔".green(),
                name
            ));
            return Ok(());
        } else {
            progress_bar.set_style(ProgressStyle::default_spinner().template("{msg}"));
            progress_bar.finish_with_message(&format!("{} Failed to install {}", "✘".red(), name));
            anyhow::bail!("Failed to install {}", name);
        }
    }

    pub async fn uninstall_package(&self, name: &str) -> Result<()> {
        let mut args = self.package_manager.uninstall_args();
        args.push(name);

        println!(
            "Uninstalling {} via {}...",
            name.cyan(),
            self.package_manager.command()
        );

        let status = Command::new(self.package_manager.command())
            .args(&args)
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to uninstall {}", name);
        }

        println!("{} {} successfully", "Uninstalled".green(), name);
        Ok(())
    }

    pub async fn update_packages(&self, packages: &[String]) -> Result<()> {
        let mut args = self.package_manager.update_args();
        args.extend(packages.iter().map(|s| s.as_str()));

        println!(
            "Updating packages via {}...",
            self.package_manager.command()
        );

        let status = Command::new(self.package_manager.command())
            .args(&args)
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to update packages");
        }

        println!("{}", "Packages updated successfully".green());
        Ok(())
    }

    pub async fn list_packages(&self) -> Result<()> {
        let output = Command::new(self.package_manager.command())
            .args(self.package_manager.list_args())
            .output()?;

        if output.status.success() {
            let packages = String::from_utf8_lossy(&output.stdout);
            for package in packages.lines().skip(1) {
                println!("  {}", package);
            }
        }

        Ok(())
    }

    pub fn is_node_installed(&self) -> bool {
        Command::new("node")
            .arg("--version")
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}
