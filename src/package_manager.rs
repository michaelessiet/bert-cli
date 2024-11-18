use anyhow::Result;
use colored::*;
use std::process::Command;

pub async fn uninstall_package(name: &str) -> Result<()> {
    if !crate::homebrew::is_homebrew_installed().await {
        anyhow::bail!("Homebrew is not installed");
    }

    // First check if the package is installed
    let installed = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
        .args(["list", "--versions", name])
        .output()?;

    if !installed.status.success() || installed.stdout.is_empty() {
        println!("{} is not installed", name.yellow());
        return Ok(());
    }

    // Show current version before uninstalling
    let version = String::from_utf8_lossy(&installed.stdout);
    println!("Found installed package: {}", version.trim());

    println!("Uninstalling {}...", name.cyan());

    let status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
        .args(["uninstall", name])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to uninstall {}", name);
    }

    // Run cleanup
    Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
        .args(["cleanup", name])
        .status()?;

    println!("{} {} successfully", "Uninstalled".green(), name);
    Ok(())
}

pub async fn install_package(package: &str, is_cask: bool) -> Result<()> {
    println!("Searching for package {} 🐕", package.cyan());

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
        println!("Successfully installed {}", package.green());
    } else {
        println!("Package {} not found in Homebrew", package.red());
    }

    Ok(())
}

pub async fn install_package_version(
    name: &str,
    version: Option<&str>,
    is_cask: bool,
) -> Result<()> {
    println!("Searching for package {} 🐕", name.cyan());

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
        println!("Successfully installed {}", name.green());
    } else {
        println!("Package {} not found in Homebrew", name.red());
    }

    Ok(())
}
