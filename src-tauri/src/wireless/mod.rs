use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WirelessStatus {
    pub wifi_enabled: bool,
    pub bluetooth_enabled: bool,
    pub airplane_mode: bool,
}

impl Default for WirelessStatus {
    fn default() -> Self {
        Self {
            wifi_enabled: true,
            bluetooth_enabled: true,
            airplane_mode: false,
        }
    }
}

pub fn get_wireless_status() -> WirelessStatus {
    #[cfg(target_os = "linux")]
    {
        get_linux_status()
    }
    #[cfg(target_os = "windows")]
    {
        get_windows_status()
    }
    #[cfg(target_os = "macos")]
    {
        get_macos_status()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        WirelessStatus::default()
    }
}

#[cfg(target_os = "linux")]
fn get_linux_status() -> WirelessStatus {
    let wifi_enabled = Command::new("nmcli")
        .args(["radio", "wifi"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("enabled"))
        .unwrap_or(true);

    let bluetooth_enabled = Command::new("rfkill")
        .args(["list", "bluetooth"])
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            !output.contains("Soft blocked: yes")
        })
        .unwrap_or(true);

    let airplane_mode = Command::new("rfkill")
        .args(["list"])
        .output()
        .map(|o| {
            let output = String::from_utf8_lossy(&o.stdout);
            output.contains("Soft blocked: yes")
        })
        .unwrap_or(false);

    WirelessStatus {
        wifi_enabled,
        bluetooth_enabled,
        airplane_mode,
    }
}

#[cfg(target_os = "windows")]
fn get_windows_status() -> WirelessStatus {
    use std::process::Command;

    let wifi_enabled = Command::new("powershell")
        .args([
            "-Command",
            "Get-NetAdapter -Name 'Wi-Fi' | Select-Object -ExpandProperty Status",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Up"))
        .unwrap_or(true);

    let bluetooth_enabled = Command::new("powershell")
        .args([
            "-Command",
            "Get-Service bthserv | Select-Object -ExpandProperty Status",
        ])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Running"))
        .unwrap_or(true);

    WirelessStatus {
        wifi_enabled,
        bluetooth_enabled,
        airplane_mode: false,
    }
}

#[cfg(target_os = "macos")]
fn get_macos_status() -> WirelessStatus {
    let wifi_enabled = Command::new("networksetup")
        .args(["-getairportpower", "en0"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("On"))
        .unwrap_or(true);

    let bluetooth_enabled = Command::new("blueutil")
        .args(["--power"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "1")
        .unwrap_or(true);

    WirelessStatus {
        wifi_enabled,
        bluetooth_enabled,
        airplane_mode: false,
    }
}
