use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub enum NodePackageManager {
    Npm,
    Yarn,
    Pnpm,
    Bun,
}

impl NodePackageManager {
    pub fn command(&self) -> &str {
        match self {
            NodePackageManager::Npm => "npm",
            NodePackageManager::Yarn => "yarn",
            NodePackageManager::Pnpm => "pnpm",
            NodePackageManager::Bun => "bun",
        }
    }

    pub fn install_args(&self) -> Vec<&str> {
        match self {
            NodePackageManager::Npm => vec!["install", "-g"],
            NodePackageManager::Yarn => vec!["global", "add"],
            NodePackageManager::Pnpm => vec!["add", "-g"],
            NodePackageManager::Bun => vec!["install", "-g"],
        }
    }

    pub fn uninstall_args(&self) -> Vec<&str> {
        match self {
            NodePackageManager::Npm => vec!["uninstall", "-g"],
            NodePackageManager::Yarn => vec!["global", "remove"],
            NodePackageManager::Pnpm => vec!["remove", "-g"],
            NodePackageManager::Bun => vec!["remove", "-g"],
        }
    }

    pub fn list_args(&self) -> Vec<&str> {
        match self {
            NodePackageManager::Npm => vec!["list", "-g", "--depth=0"],
            NodePackageManager::Yarn => vec!["global", "list"],
            NodePackageManager::Pnpm => vec!["list", "-g"],
            NodePackageManager::Bun => vec!["list", "-g"],
        }
    }

    pub fn update_args(&self) -> Vec<&str> {
        match self {
            NodePackageManager::Npm => vec!["update", "-g"],
            NodePackageManager::Yarn => vec!["global", "upgrade"],
            NodePackageManager::Pnpm => vec!["update", "-g"],
            NodePackageManager::Bun => vec!["update", "-g"],
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "npm" => Ok(NodePackageManager::Npm),
            "yarn" => Ok(NodePackageManager::Yarn),
            "pnpm" => Ok(NodePackageManager::Pnpm),
            "bun" => Ok(NodePackageManager::Bun),
            _ => anyhow::bail!(
                "Invalid package manager: {}. Valid options are: npm, yarn, pnpm",
                s
            ),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NpmPackageInfo {
    pub name: String,
    pub description: Option<String>,
    // pub version: String,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub author: Option<NpmAuthor>,
    // pub repository: Option<NpmRepository>,
    pub keywords: Option<Vec<String>>,
    // pub dependencies: Option<serde_json::Value>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct NpmAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NpmRepository {
    // #[serde(rename = "type")]
    // pub repo_type: Option<String>,
    // pub url: Option<String>,
}
