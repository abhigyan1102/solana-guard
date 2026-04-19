pub mod register_agent;
pub mod set_policy;
pub mod validate_and_execute;
pub mod toggle_agent;
pub mod update_policy;

// Re-export everything — Anchor's #[program] macro needs full crate-level access
pub use register_agent::*;
pub use set_policy::*;
pub use validate_and_execute::*;
pub use toggle_agent::*;
pub use update_policy::*;
