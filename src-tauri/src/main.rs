mod commands;
mod config;
mod crypto;
mod notes;
mod p2p;
mod scheduler;
mod sync;
mod terminal;
mod transfer;
mod wireless;

use commands::{ConfigState, NotesState, P2PState, PairState, SyncStateType, TerminalState, TransferState, *};
use config::AppConfig;
use notes::NotesStore;
use p2p::pair::PairServer;
use p2p::P2PManager;
use scheduler::Scheduler;
use sync::SyncEngine;
use terminal::TerminalManager;
use transfer::TransferManager;
use std::sync::Arc;
use tauri::Manager;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::image::Image;
use tokio::sync::RwLock;
use tracing::{error, info};

fn main() {
    tracing_subscriber::fmt::init();

    let config = AppConfig::load().unwrap_or_else(|e| {
        error!("Failed to load config: {}, using default", e);
        AppConfig::default()
    });

    let p2p_config = p2p::P2PConfig::load().unwrap_or_else(|e| {
        error!("Failed to load P2P config: {}, using default", e);
        p2p::P2PConfig::default()
    });

    let device_id = p2p_config.device_id.clone();

    let config_state: ConfigState = Arc::new(RwLock::new(config.clone()));
    let scheduler = Scheduler::new(config_state.clone());

    let notes_store = NotesStore::new(device_id.clone())
        .expect("Failed to initialize notes store");
    let notes_state: NotesState = Arc::new(notes_store);

    let p2p_manager = P2PManager::new().expect("Failed to initialize P2P manager");
    let p2p_state: P2PState = Arc::new(p2p_manager);

    let sync_engine = SyncEngine::new(notes_state.clone(), p2p_state.clone())
        .expect("Failed to initialize sync engine");
    let sync_state: SyncStateType = Arc::new(sync_engine);

    let transfer_manager = TransferManager::new();
    let transfer_state: TransferState = Arc::new(transfer_manager);

    let terminal_manager = TerminalManager::new();
    let terminal_state: TerminalState = Arc::new(terminal_manager);

    // PairServer shares the same P2PConfig Arc as P2PManager
    let p2p_config_arc = p2p_state.config.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(config_state)
        .manage(notes_state)
        .manage(p2p_state)
        .manage(sync_state)
        .manage(transfer_state)
        .manage(terminal_state)
        .manage(Arc::new(RwLock::new(scheduler)))
        .setup(move |app| {
            // Create and manage PairServer
            let pair_server = PairServer::new(p2p_config_arc, app.handle().clone());
            let pair_state: PairState = Arc::new(pair_server);
            app.manage(pair_state.clone());

            // Spawn TCP pair listener
            let pair_listener = pair_state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = pair_listener.start_listener().await {
                    error!("Pair listener failed: {}", e);
                }
            });

            let scheduler_state: Arc<RwLock<Scheduler>> = app.state::<Arc<RwLock<Scheduler>>>().inner().clone();

            tauri::async_runtime::spawn(async move {
                let sched = scheduler_state.read().await;
                sched.start().await;
            });

            let sync_state_clone: SyncStateType = app.state::<SyncStateType>().inner().clone();

            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                info!("Auto-sync would start if enabled...");
                sync_state_clone.start_auto_sync().await;
            });

            // System tray icon
            let show_item = MenuItemBuilder::with_id("show", "Show FlyMode").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let tray_icon = Image::from_path("icons/32x32.png")
                .unwrap_or_else(|_| Image::from_bytes(include_bytes!("../icons/32x32.png")).expect("Failed to load embedded icon"));

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("FlyMode")
                .menu(&tray_menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Handle window close → minimize to tray
            let app_handle = app.handle().clone();
            let config_state_for_close: ConfigState = app.state::<ConfigState>().inner().clone();
            if let Some(window) = app.get_webview_window("main") {
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let minimize_to_tray = {
                            let config = config_state_for_close.blocking_read();
                            config.minimize_to_tray
                        };
                        if minimize_to_tray {
                            api.prevent_close();
                            if let Some(win) = app_handle.get_webview_window("main") {
                                let _ = win.hide();
                            }
                        }
                    }
                });
            }

            info!("Application starting...");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            add_rule,
            update_rule,
            delete_rule,
            toggle_rule,
            execute_rule_now,
            get_status,
            toggle_wifi,
            toggle_bluetooth,
            toggle_airplane_mode,
            run_custom_command,
            create_note,
            update_note,
            delete_note,
            get_note,
            list_notes,
            search_notes,
            get_note_colors,
            get_note_categories,
            get_p2p_config,
            save_p2p_config,
            add_peer,
            remove_peer,
            update_peer,
            check_peer_status,
            check_all_peers,
            discover_tailscale,
            get_sync_state,
            sync_with_peer,
            sync_all_peers,
            export_notes,
            import_notes,
            get_sync_folder,
            get_transfer_queue,
            upload_file,
            download_file,
            cancel_transfer,
            clear_completed_transfers,
            get_transfer_progress,
            browse_remote_files,
            pair_with_peer,
            get_pending_pair_requests,
            accept_pair_request,
            reject_pair_request,
            get_build_info,
            verify_system_password,
            get_device_id,
            get_device_name,
            check_openclaw_status,
            open_terminal,
            send_terminal_input,
            resize_terminal,
            close_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
