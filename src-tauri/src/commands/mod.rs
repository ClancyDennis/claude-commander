// Command module re-exports
// Splits lib.rs command handlers into domain-specific modules

pub mod agent;
pub mod chat;
pub mod pool;
pub mod workflow;
pub mod pipeline;
pub mod cost;
pub mod logging;
pub mod instruction;
pub mod instruction_analysis;
pub mod skill;
pub mod database;
pub mod auto_pipeline;
pub mod security;
pub mod events;

// Re-export all command functions for registration in lib.rs
pub use agent::*;
pub use chat::*;
pub use pool::*;
pub use workflow::*;
pub use pipeline::*;
pub use cost::*;
pub use logging::*;
pub use instruction::*;
pub use instruction_analysis::*;
pub use skill::*;
pub use database::*;
pub use auto_pipeline::*;
pub use security::*;
pub use events::*;
