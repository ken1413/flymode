use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    Enable,
    Disable,
    Toggle,
    RunCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetType {
    Wifi,
    Bluetooth,
    AirplaneMode,
    CustomCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub action: ActionType,
    pub target: TargetType,
    pub start_time: String,
    pub end_time: Option<String>,
    pub days: Vec<u8>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub rules: Vec<ScheduleRule>,
    pub check_interval_seconds: u64,
    pub show_notifications: bool,
    pub minimize_to_tray: bool,
    pub auto_start: bool,
    #[serde(default)]
    pub require_password: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            check_interval_seconds: 60,
            show_notifications: true,
            minimize_to_tray: true,
            auto_start: false,
            require_password: false,
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("flymode")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = Self::config_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(Self::config_path(), content)?;
        Ok(())
    }
}

impl ScheduleRule {
    pub fn new(name: String, action: ActionType, target: TargetType) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            enabled: true,
            action,
            target,
            start_time: "09:00".to_string(),
            end_time: None,
            days: vec![0, 1, 2, 3, 4, 5, 6],
            command: None,
        }
    }
}
