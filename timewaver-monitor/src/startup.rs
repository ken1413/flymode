#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

const APP_NAME: &str = "TimeWaverMonitor";

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

#[cfg(windows)]
pub fn set_auto_start(enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu.create_subkey(RUN_KEY)?;

    if enable {
        let exe_path = std::env::current_exe()?;
        run.set_value(APP_NAME, &exe_path.to_string_lossy().as_ref())?;
        tracing::info!("Auto-start enabled in registry");
    } else {
        let _ = run.delete_value(APP_NAME);
        tracing::info!("Auto-start disabled in registry");
    }
    Ok(())
}

#[cfg(windows)]
pub fn is_auto_start_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run) = hkcu.open_subkey(RUN_KEY) {
        run.get_value::<String, _>(APP_NAME).is_ok()
    } else {
        false
    }
}

#[cfg(not(windows))]
pub fn set_auto_start(_enable: bool) -> Result<(), Box<dyn std::error::Error>> {
    Err("Auto-start is only supported on Windows".into())
}

#[cfg(not(windows))]
pub fn is_auto_start_enabled() -> bool {
    false
}
