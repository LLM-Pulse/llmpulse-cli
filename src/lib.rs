pub mod api_manifest;
pub mod cli;
pub mod client;
pub mod commands;
pub mod config;
pub mod error;
pub mod output;
pub mod query;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
