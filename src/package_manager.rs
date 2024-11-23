use anyhow::{Ok, Result};
use colored::*;
use std::process::Command;

use crate::{homebrew, node::NodeManager};

pub async fn search_package(name: &str, is_cask: bool, is_node: bool) -> Result<()> {
    if is_node {
        if let Some(npm_info) = crate::node::get_package_info(name).await? {
            crate::node::display_package_info(&npm_info);
            return Ok(());
        }
    }

    if let Some(formula) = homebrew::search_formula(
        &name,
        if is_cask {
            Some(homebrew::HomebrewPackageType::Cask)
        } else {
            Some(homebrew::HomebrewPackageType::Formula)
        },
    )
    .await?
    {
        homebrew::display_package_info(&formula, is_cask);
        Ok(())
    } else {
        anyhow::bail!("No packages found matching: {}", name.red());
    }
}

pub async fn uninstall_package(name: &str, is_cask: bool, is_node: bool) -> Result<()> {
    if is_node {
        let config = crate::config::Config::load()?;
        let node_manager = NodeManager::new(config.get_node_package_manager()?);
        return node_manager.uninstall_package(name).await;
    }

    return crate::homebrew::uninstall_formula(name, is_cask).await;
}

pub async fn install_package(package: &str, is_cask: bool, is_node: bool) -> Result<()> {
    println!("Searching for package {} 🐕", package.cyan());

    if is_node {
        let config = crate::config::Config::load()?;
        let node_manager = NodeManager::new(config.get_node_package_manager()?);
        return node_manager.install_package(package, None).await;
    }

    if let Some(formula) = crate::homebrew::search_formula(
        package,
        if is_cask {
            Some(crate::homebrew::HomebrewPackageType::Cask)
        } else {
            Some(crate::homebrew::HomebrewPackageType::Formula)
        },
    )
    .await?
    {
        println!("Found package: {}", formula.name.green());
        if let Some(desc) = formula.desc {
            println!("Description: {}", desc);
        }
        println!("Version: {}", formula.versions.stable);

        crate::homebrew::install_formula(&formula.full_name, is_cask).await?;
        // println!("Successfully installed {}", package.green());
    } else {
        println!("Package {} not found in Homebrew", package.red());
    }

    Ok(())
}

pub async fn install_package_version(
    name: &str,
    version: Option<&str>,
    is_cask: bool,
    is_node: bool,
) -> Result<()> {
    println!("Searching for package {} 🐕", name.cyan());

    if is_node {
        let config = crate::config::Config::load()?;
        let node_manager = NodeManager::new(config.get_node_package_manager()?);
        return node_manager.install_package(name, version).await;
    }

    if let Some(formula) = crate::homebrew::search_formula(
        name,
        if is_cask {
            Some(crate::homebrew::HomebrewPackageType::Cask)
        } else {
            Some(crate::homebrew::HomebrewPackageType::Formula)
        },
    )
    .await?
    {
        crate::homebrew::display_package_info(&formula, is_cask);

        if let Some(_v) = version {
            if formula.versioned_formulae.is_empty() {
                println!(
                    "\n{}",
                    "Note: This package doesn't have version-specific formulae available.".yellow()
                );
                println!(
                    "Installing latest version ({}) instead 🐕",
                    formula.versions.stable
                );
            }
        }

        crate::homebrew::install_formula_version(name, version, is_cask).await?;
        // println!("Successfully installed {}", name.green());
    } else {
        println!("Package {} not found in Homebrew", name.red());
    }

    Ok(())
}

pub async fn update_packages(packages: &Vec<String>, is_node: bool) -> Result<()> {
    if is_node {
        let config = crate::config::Config::load()?;
        let node_manager = NodeManager::new(config.get_node_package_manager()?);
        return node_manager.update_packages(packages).await;
    }

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
        packages.clone()
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

    Ok(())
}

pub async fn list_packages(is_node: bool) -> Result<()> {
    if is_node {
        let config = crate::config::Config::load()?;
        let node_manager = NodeManager::new(config.get_node_package_manager()?);
        return node_manager.list_packages().await;
    }

    return crate::homebrew::list_packages();
}
