use anyhow::Result;
use chrono::Local;
use colored::*;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct BackupFile {
    created_at: String,
    formulas: Vec<FormulaBackup>,
    casks: Vec<CaskBackup>,
    taps: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct FormulaBackup {
    name: String,
    version: String,
    options: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct CaskBackup {
    name: String,
    version: String,
}

pub async fn create_backup(path: Option<&str>) -> Result<()> {
    println!("Creating backup of Homebrew packages 🐕");

    // Get all taps
    let taps_output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
        .args(["tap"])
        .output()?;
    let taps = String::from_utf8_lossy(&taps_output.stdout)
        .lines()
        .map(String::from)
        .collect::<Vec<_>>();

    // Get installed formulas
    let formulas_output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
        .args(["list", "--formula", "--versions"])
        .output()?;

    let formulas = String::from_utf8_lossy(&formulas_output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let name = parts[0];
            let version = parts.get(1).unwrap_or(&"").to_string();

            // Get install options if any
            let options_output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
                .args(["info", "--json=v2", name])
                .output()
                .unwrap();

            let options = if options_output.status.success() {
                String::from_utf8_lossy(&options_output.stdout)
                    .lines()
                    .filter(|line| line.contains("--"))
                    .map(String::from)
                    .collect()
            } else {
                Vec::new()
            };

            FormulaBackup {
                name: name.to_string(),
                version,
                options,
            }
        })
        .collect::<Vec<_>>();

    // Get installed casks
    let casks_output = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
        .args(["list", "--cask", "--versions"])
        .output()?;

    let casks = String::from_utf8_lossy(&casks_output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            CaskBackup {
                name: parts[0].to_string(),
                version: parts.get(1).unwrap_or(&"").to_string(),
            }
        })
        .collect::<Vec<_>>();

    let backup = BackupFile {
        created_at: Local::now().to_rfc3339(),
        formulas,
        casks,
        taps,
    };

    // Determine backup path
    let backup_path = get_backup_path(path)?;
    let backup_json = serde_json::to_string_pretty(&backup)?;
    fs::write(&backup_path, backup_json)?;

    println!("{}", "Backup created successfully!".green());
    println!("Backup location: {}", backup_path.display());
    println!("Summary:");
    println!("  Taps: {}", backup.taps.len());
    println!("  Formulas: {}", backup.formulas.len());
    println!("  Casks: {}", backup.casks.len());

    Ok(())
}

pub async fn restore_backup(path: Option<&str>) -> Result<()> {
    let backup_path = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        get_latest_backup()?
    };

    println!("Restoring Homebrew packages from backup 🐕");
    println!("Reading backup from: {}", backup_path.display());

    let backup_content = fs::read_to_string(&backup_path)?;
    let backup: BackupFile = serde_json::from_str(&backup_content)?;

    println!("Backup created at: {}", backup.created_at);
    println!(
        "\nRestoring {} taps, {} formulas, and {} casks 🐕",
        backup.taps.len(),
        backup.formulas.len(),
        backup.casks.len()
    );

    // First restore taps
    println!("\n{}:", "Restoring taps".cyan());
    for tap in &backup.taps {
        print!("  {:<40}", tap);
        let status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
            .args(["tap", tap])
            .status()?;

        if status.success() {
            println!("{}", "✓".green());
        } else {
            println!("{}", "✗".red());
        }
    }

    // Then restore formulas
    println!("\n{}:", "Restoring formulas".cyan());
    for formula in &backup.formulas {
        print!("  {:<40}", formula.name);

        let mut args = vec!["install"];
        args.push(&formula.name);
        args.extend(formula.options.iter().map(|s| s.as_str()));

        let status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
            .args(&args)
            .status()?;

        if status.success() {
            println!("{}", "✓".green());
        } else {
            println!("{}", "✗".red());
        }
    }

    // Finally restore casks
    println!("\n{}:", "Restoring casks".cyan());
    for cask in &backup.casks {
        print!("  {:<40}", cask.name);
        let status = Command::new(if cfg!(windows) { "brew.exe" } else { "brew" })
            .args(["install", "--cask", &cask.name])
            .status()?;

        if status.success() {
            println!("{}", "✓".green());
        } else {
            println!("{}", "✗".red());
        }
    }

    println!("\n{}", "Restore completed!".green());
    Ok(())
}

fn get_backup_path(custom_path: Option<&str>) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        Ok(PathBuf::from(path))
    } else {
        let backup_dir = get_backup_dir()?;
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        Ok(backup_dir.join(format!("bert_backup_{}.json", timestamp)))
    }
}

fn get_backup_dir() -> Result<PathBuf> {
    let backup_dir = home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
        .join(".bert")
        .join("backups");

    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)?;
    }

    Ok(backup_dir)
}

fn get_latest_backup() -> Result<PathBuf> {
    let backup_dir = get_backup_dir()?;
    let mut backups: Vec<_> = fs::read_dir(&backup_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    backups.sort_by_key(|entry| entry.metadata().unwrap().modified().unwrap());

    backups
        .last()
        .map(|entry| entry.path())
        .ok_or_else(|| anyhow::anyhow!("No backup files found"))
}
