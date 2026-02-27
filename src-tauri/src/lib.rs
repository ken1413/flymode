pub mod commands;
pub mod config;
pub mod crypto;
pub mod notes;
pub mod p2p;
pub mod scheduler;
pub mod sync;
pub mod terminal;
pub mod transfer;
pub mod wireless;

pub const fn tauri_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
