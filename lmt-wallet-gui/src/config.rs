use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub cli_path: String,
    pub network: String,
    pub last_wallet: String,
    pub session_timeout_min: u32,
    pub contacts: Vec<Contact>,
    pub seed_backups_confirmed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub address: String,
    pub note: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 2,
            cli_path: String::new(),
            network: "mainnet".into(),
            last_wallet: String::new(),
            session_timeout_min: 15,
            contacts: Vec::new(),
            seed_backups_confirmed: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default());
        base.join("lapis-monetae")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("wallet-gui-config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, data);
        }
    }

    pub fn resolve_cli(&self) -> Option<PathBuf> {
        if !self.cli_path.is_empty() {
            let p = PathBuf::from(&self.cli_path);
            if p.exists() {
                return Some(p);
            }
        }
        // Try sibling directory
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let sibling = dir.join("lmt-cli");
                if sibling.exists() {
                    return Some(sibling);
                }
            }
        }
        // Try PATH
        which_cli()
    }
}

fn which_cli() -> Option<PathBuf> {
    let cmd = if cfg!(windows) { "where" } else { "which" };
    std::process::Command::new(cmd)
        .arg("lmt-cli")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                let p = PathBuf::from(s.lines().next()?);
                if p.exists() { Some(p) } else { None }
            } else {
                None
            }
        })
}
