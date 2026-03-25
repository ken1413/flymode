use std::sync::{Arc, Mutex};

use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu, CheckMenuItem};
use tray_icon::TrayIconBuilder;
use tray_icon::Icon;
use tracing::{error, info};

use crate::config::Config;
use crate::monitor::{self, MonitorStatus};
use crate::startup;

fn create_icon(running: bool) -> Icon {
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let (r, g, b) = if running {
        (0x2eu8, 0xc7u8, 0x6eu8)
    } else {
        (0xe7u8, 0x4cu8, 0x3cu8)
    };

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - (size as f32 / 2.0) + 0.5;
            let dy = y as f32 - (size as f32 / 2.0) + 0.5;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < (size as f32 / 2.0) - 0.5 {
                rgba.extend_from_slice(&[r, g, b, 255]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Icon::from_rgba(rgba, size, size).expect("Failed to create icon")
}

pub struct TrayApp {
    pub menu_status: MenuItem,
    pub menu_last_check: MenuItem,
    pub menu_restart_count: MenuItem,
    pub menu_pause: CheckMenuItem,
    pub menu_restart_now: MenuItem,
    pub menu_auto_start: CheckMenuItem,
    pub menu_open_log: MenuItem,
    pub menu_open_config: MenuItem,
    pub menu_quit: MenuItem,
    pub _tray_icon: tray_icon::TrayIcon,
}

impl TrayApp {
    pub fn new(config: &Config) -> Self {
        let menu = Menu::new();

        let menu_title = MenuItem::new("TimeWaver Monitor v0.1.0", false, None);
        let menu_status = MenuItem::new("Status: checking...", false, None);
        let menu_last_check = MenuItem::new("Last check: --", false, None);
        let menu_restart_count = MenuItem::new("Restarts: 0", false, None);
        let menu_pause = CheckMenuItem::new("Pause Monitoring", true, false, None);
        let menu_restart_now = MenuItem::new("Restart Now", true, None);
        let menu_auto_start = CheckMenuItem::new(
            "Start with Windows",
            true,
            config.auto_start || startup::is_auto_start_enabled(),
            None,
        );
        let menu_open_log = MenuItem::new("Open Log Folder", true, None);
        let menu_open_config = MenuItem::new("Open Config", true, None);
        let menu_quit = MenuItem::new("Quit", true, None);

        menu.append(&menu_title).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        menu.append(&menu_status).unwrap();
        menu.append(&menu_last_check).unwrap();
        menu.append(&menu_restart_count).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        menu.append(&menu_pause).unwrap();
        menu.append(&menu_restart_now).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        menu.append(&menu_auto_start).unwrap();

        let advanced = Submenu::new("Advanced", true);
        advanced.append(&menu_open_log).unwrap();
        advanced.append(&menu_open_config).unwrap();
        menu.append(&advanced).unwrap();

        menu.append(&PredefinedMenuItem::separator()).unwrap();
        menu.append(&menu_quit).unwrap();

        let icon = create_icon(false);
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("TimeWaver Monitor - Starting...")
            .with_icon(icon)
            .build()
            .expect("Failed to create tray icon");

        Self {
            menu_status,
            menu_last_check,
            menu_restart_count,
            menu_pause,
            menu_restart_now,
            menu_auto_start,
            menu_open_log,
            menu_open_config,
            menu_quit,
            _tray_icon: tray_icon,
        }
    }

    pub fn update_status(&self, status: &MonitorStatus) {
        let status_text = if status.paused {
            "Status: PAUSED".to_string()
        } else if status.cooldown_until.is_some() {
            "Status: COOLDOWN (too many failures)".to_string()
        } else if status.target_running {
            "Status: Running".to_string()
        } else if status.last_check.is_some() {
            "Status: Not Running".to_string()
        } else {
            "Status: checking...".to_string()
        };
        self.menu_status.set_text(&status_text);

        let last_check_text = match status.last_check {
            Some(dt) => format!("Last check: {}", dt.format("%H:%M:%S")),
            None => "Last check: --".to_string(),
        };
        self.menu_last_check.set_text(&last_check_text);

        self.menu_restart_count
            .set_text(&format!("Restarts: {}", status.restart_count));

        let tooltip = if status.paused {
            "TimeWaver Monitor - Paused".to_string()
        } else if status.target_running {
            "TimeWaver Monitor - Running OK".to_string()
        } else {
            "TimeWaver Monitor - Process Stopped!".to_string()
        };
        let _ = self._tray_icon.set_tooltip(Some(&tooltip));

        let icon = if status.paused {
            create_icon(false)
        } else {
            create_icon(status.target_running)
        };
        let _ = self._tray_icon.set_icon(Some(icon));
    }

    pub fn handle_menu_event(
        &self,
        event: &MenuEvent,
        config: &Arc<Mutex<Config>>,
        status: &Arc<Mutex<MonitorStatus>>,
    ) -> bool {
        let id = event.id();

        if id == self.menu_quit.id() {
            info!("Quit requested from tray menu");
            return true;
        }

        if id == self.menu_pause.id() {
            let checked = self.menu_pause.is_checked();
            let mut st = status.lock().unwrap();
            st.paused = checked;
            info!("Monitoring {}", if checked { "paused" } else { "resumed" });
        }

        if id == self.menu_restart_now.id() {
            let target_exe = {
                let cfg = config.lock().unwrap();
                cfg.target_exe.clone()
            };
            info!("Manual restart requested");
            match monitor::restart_process(&target_exe) {
                Ok(pid) => {
                    let mut st = status.lock().unwrap();
                    st.last_restart = Some(chrono::Local::now());
                    st.restart_count += 1;
                    st.target_running = true;
                    st.last_error = None;
                    info!("Manual restart successful (PID: {pid})");
                }
                Err(e) => {
                    error!("Manual restart failed: {e}");
                    let mut st = status.lock().unwrap();
                    st.last_error = Some(e);
                }
            }
        }

        if id == self.menu_auto_start.id() {
            let checked = self.menu_auto_start.is_checked();
            if let Err(e) = startup::set_auto_start(checked) {
                error!("Failed to set auto-start: {e}");
                self.menu_auto_start.set_checked(!checked);
            } else {
                let mut cfg = config.lock().unwrap();
                cfg.auto_start = checked;
                let _ = cfg.save();
            }
        }

        if id == self.menu_open_log.id() {
            let log_dir = Config::log_dir();
            let _ = std::fs::create_dir_all(&log_dir);
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("explorer")
                    .arg(&log_dir)
                    .spawn();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&log_dir)
                    .spawn();
            }
        }

        if id == self.menu_open_config.id() {
            let config_path = Config::config_path();
            #[cfg(windows)]
            {
                let _ = std::process::Command::new("notepad")
                    .arg(&config_path)
                    .spawn();
            }
            #[cfg(not(windows))]
            {
                let _ = std::process::Command::new("xdg-open")
                    .arg(&config_path)
                    .spawn();
            }
        }

        false
    }
}
