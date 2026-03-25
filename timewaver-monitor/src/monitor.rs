use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::System;
use tracing::{error, info, warn};

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct MonitorStatus {
    pub target_running: bool,
    pub last_check: Option<chrono::DateTime<chrono::Local>>,
    pub last_restart: Option<chrono::DateTime<chrono::Local>>,
    pub restart_count: u32,
    pub consecutive_failures: u32,
    pub paused: bool,
    pub cooldown_until: Option<Instant>,
    pub last_error: Option<String>,
}

impl Default for MonitorStatus {
    fn default() -> Self {
        Self {
            target_running: false,
            last_check: None,
            last_restart: None,
            restart_count: 0,
            consecutive_failures: 0,
            paused: false,
            cooldown_until: None,
            last_error: None,
        }
    }
}

pub fn is_process_running(target_exe: &str) -> bool {
    let target_name = Path::new(target_exe)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut sys = System::new();
    sys.refresh_processes();

    for (_pid, process) in sys.processes() {
        let proc_name = process.name().to_lowercase();
        if proc_name == target_name {
            if let Some(exe_path) = process.exe() {
                let exe_str = exe_path.to_string_lossy().to_lowercase();
                let target_lower = target_exe.to_lowercase();
                if exe_str == target_lower {
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    false
}

pub fn restart_process(target_exe: &str) -> Result<u32, String> {
    let path = Path::new(target_exe);
    if !path.exists() {
        return Err(format!("Executable not found: {target_exe}"));
    }

    let working_dir = path.parent().unwrap_or(Path::new("."));

    match Command::new(target_exe)
        .current_dir(working_dir)
        .spawn()
    {
        Ok(child) => {
            let pid = child.id();
            info!("Successfully started process (PID: {pid}): {target_exe}");
            Ok(pid)
        }
        Err(e) => {
            let msg = format!("Failed to start process: {e}");
            error!("{msg}");
            Err(msg)
        }
    }
}

pub fn spawn_monitor_thread(
    config: Arc<Mutex<Config>>,
    status: Arc<Mutex<MonitorStatus>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        info!("Monitor thread started");
        loop {
            let (target_exe, interval, max_attempts, cooldown_secs, enabled) = {
                let cfg = config.lock().unwrap();
                (
                    cfg.target_exe.clone(),
                    cfg.check_interval_secs,
                    cfg.max_restart_attempts,
                    cfg.restart_cooldown_secs,
                    cfg.monitoring_enabled,
                )
            };

            let paused = {
                let st = status.lock().unwrap();
                st.paused
            };

            if enabled && !paused {
                let in_cooldown = {
                    let st = status.lock().unwrap();
                    if let Some(until) = st.cooldown_until {
                        Instant::now() < until
                    } else {
                        false
                    }
                };

                if in_cooldown {
                    info!("In cooldown period, skipping check");
                } else {
                    let running = is_process_running(&target_exe);
                    let now = chrono::Local::now();

                    let mut st = status.lock().unwrap();
                    st.target_running = running;
                    st.last_check = Some(now);

                    if running {
                        st.consecutive_failures = 0;
                        st.last_error = None;
                        info!("Process is running: {target_exe}");
                    } else {
                        warn!("Process not running: {target_exe}");

                        if st.consecutive_failures >= max_attempts {
                            warn!(
                                "Max restart attempts ({max_attempts}) reached, entering cooldown for {cooldown_secs}s"
                            );
                            st.cooldown_until =
                                Some(Instant::now() + Duration::from_secs(cooldown_secs));
                            st.consecutive_failures = 0;
                            st.last_error = Some(format!(
                                "Max attempts reached, cooldown until {}",
                                chrono::Local::now()
                                    + chrono::Duration::seconds(cooldown_secs as i64)
                            ));
                        } else {
                            drop(st);
                            match restart_process(&target_exe) {
                                Ok(pid) => {
                                    let mut st = status.lock().unwrap();
                                    st.last_restart = Some(chrono::Local::now());
                                    st.restart_count += 1;
                                    st.consecutive_failures = 0;
                                    st.last_error = None;
                                    info!("Restart successful (PID: {pid}), total restarts: {}", st.restart_count);
                                }
                                Err(e) => {
                                    let mut st = status.lock().unwrap();
                                    st.consecutive_failures += 1;
                                    st.last_error = Some(e);
                                }
                            }
                        }
                    }
                }
            }

            let sleep_secs = {
                let st = status.lock().unwrap();
                if !st.target_running && st.last_check.is_some() {
                    15 // check more frequently when process is down
                } else {
                    interval
                }
            };
            thread::sleep(Duration::from_secs(sleep_secs));
        }
    })
}
