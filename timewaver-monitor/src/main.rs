#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod monitor;
mod startup;
mod tray;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use muda::MenuEvent;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use config::Config;
use monitor::MonitorStatus;
use tray::TrayApp;

struct App {
    tray: Option<TrayApp>,
    config: Arc<Mutex<Config>>,
    status: Arc<Mutex<MonitorStatus>>,
    last_ui_update: std::time::Instant,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        if self.tray.is_none() {
            let cfg = self.config.lock().unwrap().clone();
            self.tray = Some(TrayApp::new(&cfg));
            info!("Tray icon created");
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if let Some(ref tray) = self.tray {
                let should_exit =
                    tray.handle_menu_event(&event, &self.config, &self.status);
                if should_exit {
                    event_loop.exit();
                    return;
                }
            }
        }

        let now = std::time::Instant::now();
        if now.duration_since(self.last_ui_update) >= Duration::from_secs(2) {
            self.last_ui_update = now;
            if let Some(ref tray) = self.tray {
                let st = self.status.lock().unwrap().clone();
                tray.update_status(&st);
            }
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + Duration::from_millis(500),
        ));
    }
}

fn init_logging(config: &Config) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let registry = tracing_subscriber::registry().with(env_filter);

    if config.log_to_file {
        let log_dir = Config::log_dir();
        std::fs::create_dir_all(&log_dir).ok();

        let file_appender =
            tracing_appender::rolling::daily(&log_dir, "monitor.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        std::mem::forget(_guard);

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false);

        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout);

        registry.with(file_layer).with(stdout_layer).init();
    } else {
        let stdout_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout);

        registry.with(stdout_layer).init();
    }
}

fn ensure_single_instance() {
    #[cfg(windows)]
    {
        use std::ptr;
        const MUTEX_NAME: &str = "Global\\TimeWaverMonitor_SingleInstance\0";
        let name: Vec<u8> = MUTEX_NAME.bytes().collect();
        unsafe {
            let handle = windows_sys::Win32::System::Threading::CreateMutexA(
                ptr::null(),
                1, // bInitialOwner = TRUE
                name.as_ptr(),
            );
            let err = windows_sys::Win32::Foundation::GetLastError();
            // ERROR_ALREADY_EXISTS = 183
            if handle.is_null() || err == 183 {
                eprintln!("TimeWaver Monitor is already running.");
                std::process::exit(0);
            }
            // Keep the handle alive for the entire process lifetime
            let _ = handle;
        }
    }
}

fn main() {
    ensure_single_instance();

    let mut config = Config::load();

    if !config.resolve_target_exe() {
        eprintln!("Cannot start without a valid target executable. Exiting.");
        std::process::exit(1);
    }

    println!(
        "TimeWaver Monitor v0.1.0\n  Target: {}\n  Check interval: {}s\n  Config: {}\n  Logs: {}",
        config.target_exe,
        config.check_interval_secs,
        Config::config_path().display(),
        Config::log_dir().display(),
    );

    init_logging(&config);
    info!("TimeWaver Monitor starting");
    info!("Monitoring target: {}", config.target_exe);

    let config = Arc::new(Mutex::new(config));
    let status = Arc::new(Mutex::new(MonitorStatus::default()));

    let _monitor_handle =
        monitor::spawn_monitor_thread(Arc::clone(&config), Arc::clone(&status));
    info!("Monitor thread spawned");

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let mut app = App {
        tray: None,
        config,
        status,
        last_ui_update: std::time::Instant::now(),
    };

    event_loop.run_app(&mut app).expect("Event loop failed");
    info!("TimeWaver Monitor shutting down");
}
