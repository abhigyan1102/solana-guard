#![allow(ambiguous_glob_reexports)]

pub mod fund_vault;
pub mod register_agent;
pub mod set_policy;
pub mod toggle_agent;
pub mod update_policy;
pub mod validate_and_execute;
pub mod withdraw_vault;

// Re-export everything — Anchor's #[program] macro needs full crate-level access
pub use fund_vault::*;
pub use register_agent::*;
pub use set_policy::*;
pub use toggle_agent::*;
pub use update_policy::*;
pub use validate_and_execute::*;
pub use withdraw_vault::*;
