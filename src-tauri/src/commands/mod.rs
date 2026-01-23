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
pub mod logging;
pub mod pipeline;
pub mod pool;
pub mod security;
pub mod skill;
pub mod workflow;

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
pub use logging::*;
pub use pipeline::*;
pub use pool::*;
pub use security::*;
pub use skill::*;
pub use workflow::*;
