use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_TARGET_EXE: &str =
    r"C:\Program Files (x86)\TimeWaverPro\TwPro.exe";
const DEFAULT_CHECK_INTERVAL_SECS: u64 = 60;
const DEFAULT_MAX_RESTART_ATTEMPTS: u32 = 5;
const DEFAULT_RESTART_COOLDOWN_SECS: u64 = 300;

const EXE_NAME: &str = "TwPro.exe";

const SEARCH_DIRS: &[&str] = &[
    r"C:\Program Files (x86)\TimeWaverPro",
    r"C:\Program Files\TimeWaverPro",
    r"C:\Program Files (x86)\TimeWaver",
    r"C:\Program Files\TimeWaver",
    r"C:\TimeWaverPro",
    r"C:\TimeWaver",
    r"D:\Program Files (x86)\TimeWaverPro",
    r"D:\Program Files\TimeWaverPro",
    r"D:\TimeWaverPro",
];

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

    /// Check if target_exe exists. If not, auto-search common paths,
    /// then fall back to a file dialog. Returns true if resolved and saved.
    pub fn resolve_target_exe(&mut self) -> bool {
        if Path::new(&self.target_exe).exists() {
            return true;
        }

        eprintln!(
            "Target exe not found: {}\nSearching common locations...",
            self.target_exe
        );

        // Auto-search common directories
        for dir in SEARCH_DIRS {
            let candidate = Path::new(dir).join(EXE_NAME);
            if candidate.exists() {
                eprintln!("Found: {}", candidate.display());
                self.target_exe = candidate.to_string_lossy().to_string();
                let _ = self.save();
                return true;
            }
        }

        eprintln!("Not found in common locations. Opening file picker...");

        // Show message box explaining why the file picker is appearing
        Self::show_message_box(
            "TimeWaver Monitor",
            &format!(
                "找不到 TimeWaver Pro 執行檔：\n{}\n\n\
                 自動搜尋常見安裝路徑也未找到。\n\
                 請手動選擇 TwPro.exe 的位置。",
                self.target_exe
            ),
        );

        // Fall back to Windows file dialog via PowerShell
        if let Some(path) = Self::open_file_dialog() {
            eprintln!("User selected: {path}");
            self.target_exe = path;
            let _ = self.save();
            return true;
        }

        eprintln!("No file selected. Please set target_exe in config manually:");
        eprintln!("  {}", Self::config_path().display());
        false
    }

    fn show_message_box(title: &str, message: &str) {
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             [System.Windows.Forms.MessageBox]::Show('{}', '{}', 'OK', 'Information')",
            message.replace('\'', "''").replace('\n', "`n"),
            title.replace('\'', "''"),
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output();
    }

    fn open_file_dialog() -> Option<String> {
        let script = r#"
Add-Type -AssemblyName System.Windows.Forms
$dlg = New-Object System.Windows.Forms.OpenFileDialog
$dlg.Title = 'Select TimeWaver Pro executable'
$dlg.Filter = 'TwPro.exe|TwPro.exe|Executable files (*.exe)|*.exe|All files (*.*)|*.*'
$dlg.InitialDirectory = 'C:\Program Files (x86)'
if ($dlg.ShowDialog() -eq 'OK') { $dlg.FileName } else { '' }
"#;
        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
            .ok()?;

        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if selected.is_empty() {
            None
        } else {
            Some(selected)
        }
    }
}
