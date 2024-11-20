use crate::node::NodePackageManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub node_package_manager: String, // "npm", "yarn", or "pnpm"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_settings: Option<serde_json::Value>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_package_manager: "npm".to_string(),
            backup_dir: None,
            custom_settings: None,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        // Ensure the config directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn set_node_package_manager(&mut self, manager: NodePackageManager) -> Result<()> {
        self.node_package_manager = manager.command().to_string();
        self.save()
    }

    pub fn get_node_package_manager(&self) -> Result<NodePackageManager> {
        match self.node_package_manager.as_str() {
            "npm" => Ok(NodePackageManager::Npm),
            "yarn" => Ok(NodePackageManager::Yarn),
            "pnpm" => Ok(NodePackageManager::Pnpm),
            _ => Ok(NodePackageManager::Npm), // Default to npm if invalid
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    Ok(home_dir.join(".bert").join("config.json"))
}
