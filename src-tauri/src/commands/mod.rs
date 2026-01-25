// Command module re-exports
// Splits lib.rs command handlers into domain-specific modules

pub mod agent;
pub mod auto_pipeline;
pub mod chat;
pub mod config;
pub mod cost;
pub mod database;
pub mod events;
pub mod instruction;
pub mod instruction_analysis;
pub mod instruction_wizard;
pub mod logging;
pub mod security;
pub mod skill;

// Re-export all command functions for registration in lib.rs
pub use agent::*;
pub use auto_pipeline::*;
pub use chat::*;
pub use config::*;
pub use cost::*;
pub use database::*;
pub use events::*;
pub use instruction::*;
pub use instruction_analysis::*;
pub use instruction_wizard::*;
pub use logging::*;
pub use security::*;
pub use skill::*;
