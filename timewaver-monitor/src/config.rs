use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_TARGET_EXE: &str =
    r"C:\Program Files (x86)\TimeWaverPro\TwPro.exe";
const DEFAULT_CHECK_INTERVAL_SECS: u64 = 60;
const DEFAULT_MAX_RESTART_ATTEMPTS: u32 = 5;
const DEFAULT_RESTART_COOLDOWN_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub target_exe: String,
    pub check_interval_secs: u64,
    pub auto_start: bool,
    pub log_to_file: bool,
    pub max_restart_attempts: u32,
    pub restart_cooldown_secs: u64,
    pub monitoring_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_exe: DEFAULT_TARGET_EXE.to_string(),
            check_interval_secs: DEFAULT_CHECK_INTERVAL_SECS,
            auto_start: false,
            log_to_file: true,
            max_restart_attempts: DEFAULT_MAX_RESTART_ATTEMPTS,
            restart_cooldown_secs: DEFAULT_RESTART_COOLDOWN_SECS,
            monitoring_enabled: true,
        }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("timewaver-monitor")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn log_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("timewaver-monitor")
            .join("logs")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Failed to parse config, using defaults: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read config file, using defaults: {e}");
                }
            }
        }
        let config = Self::default();
        let _ = config.save();
        config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }
}
