use anyhow::Result;
use colored::*;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const REPO_OWNER: &str = "michaelessiet"; // Change this to your GitHub username
const REPO_NAME: &str = "bert-cli";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    // name: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
    html_url: String,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn self_update() -> Result<()> {
    println!("Checking for updates 🐕");

    // Get current version
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    // Get latest release from GitHub
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let response = client
        .get(&url)
        .header("User-Agent", "bert-updater")
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch latest release information");
    }

    let release: GithubRelease = response.json().await?;
    let latest_version = release.tag_name.trim_start_matches('v');

    println!("Latest version: {}", latest_version);

    if latest_version == current_version {
        println!("{}", "bert is already up to date!".green());
        return Ok(());
    }

    println!(
        "New version available: {} -> {}",
        current_version, latest_version
    );
    if let Some(body) = release.body {
        println!("\nRelease notes:\n{}", body);
    }

    // Find the appropriate asset for the current platform
    let asset_name = get_platform_asset_name();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .ok_or_else(|| anyhow::anyhow!("No compatible binary found for your platform"))?;

    println!("Downloading update...");

    // Download the new binary
    let response = client
        .get(&asset.browser_download_url)
        .header("User-Agent", "bert-updater")
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download update");
    }

    // Get current executable path
    let current_exe = env::current_exe()?;
    let temp_path = get_temp_path(&current_exe);

    // Save the new binary to a temporary location
    let bytes = response.bytes().await?;
    fs::write(&temp_path, bytes)?;

    // Make the new binary executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o755))?;
    }

    // Replace the old binary
    println!("Installing update...");

    #[cfg(windows)]
    {
        // On Windows, we need to move the current executable to a temp path first
        let old_exe = current_exe.with_extension("old.exe");
        fs::rename(&current_exe, &old_exe)?;
        fs::rename(&temp_path, &current_exe)?;
        // Try to remove the old executable, but don't fail if we can't
        fs::remove_file(old_exe).ok();
    }
    #[cfg(not(windows))]
    {
        fs::rename(&temp_path, &current_exe)?;
    }

    println!("{}", "Update completed successfully!".green());
    println!("New version: {}", latest_version);
    println!("Release page: {}", release.html_url);

    Ok(())
}

fn get_platform_asset_name() -> String {
    #[cfg(target_os = "linux")]
    {
        "bert-linux-amd64".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "aarch64") {
            "bert-darwin-arm64".to_string()
        } else {
            "bert-darwin-amd64".to_string()
        }
    }
    #[cfg(target_os = "windows")]
    {
        "bert-windows-amd64.exe".to_string()
    }
}

fn get_temp_path(current_exe: &PathBuf) -> PathBuf {
    let file_name = current_exe.file_name().unwrap();
    let temp_dir = env::temp_dir();

    #[cfg(windows)]
    {
        temp_dir.join(format!("{}.new", file_name.to_string_lossy()))
    }
    #[cfg(not(windows))]
    {
        temp_dir.join(format!("{}.new", file_name.to_string_lossy()))
    }
}
