use anyhow::Result;
use colored::*;
use std::path::PathBuf;

pub async fn install_package(package: &str) -> Result<()> {
    println!("Searching for package {}...", package.cyan());

    if let Some(formula) = crate::homebrew::search_formula(package).await? {
        println!("Found package: {}", formula.name.green());
        if let Some(desc) = formula.desc {
            println!("Description: {}", desc);
        }
        println!("Version: {}", formula.versions.stable);

        crate::homebrew::install_formula(&formula.full_name).await?;
        println!("Successfully installed {}", package.green());
    } else {
        println!("Package {} not found in Homebrew", package.red());
    }

    Ok(())
}

pub fn get_bin_path() -> PathBuf {
    PathBuf::from("/usr/local/bin")
}

pub async fn install_package_version(name: &str, version: Option<&str>) -> Result<()> {
    println!("Searching for package {}...", name.cyan());

    if let Some(formula) = crate::homebrew::search_formula(name).await? {
        crate::homebrew::display_package_info(&formula);

        if let Some(v) = version {
            if formula.versioned_formulae.is_empty() {
                println!(
                    "\n{}",
                    "Note: This package doesn't have version-specific formulae available.".yellow()
                );
                println!(
                    "Installing latest version ({}) instead...",
                    formula.versions.stable
                );
            }
        }

        crate::homebrew::install_formula_version(name, version).await?;
        println!("Successfully installed {}", name.green());
    } else {
        println!("Package {} not found in Homebrew", name.red());
    }

    Ok(())
}