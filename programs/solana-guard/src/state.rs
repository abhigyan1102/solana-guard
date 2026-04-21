use anchor_lang::prelude::*;

/// Maximum number of allowed protocols per policy
pub const MAX_ALLOWED_PROTOCOLS: usize = 10;

/// Represents a registered AI agent bound to a user
#[account]
#[derive(InitSpace)]
pub struct AgentConfig {
    /// The user (owner) who registered this agent
    pub owner: Pubkey,
    /// The agent's pubkey (the key the AI agent signs with)
    pub agent: Pubkey,
    /// Whether this agent is currently active
    pub is_active: bool,
    /// Timestamp when the agent was registered
    pub registered_at: i64,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

/// Risk policy enforced on every agent transaction
#[account]
#[derive(InitSpace)]
pub struct Policy {
    /// The owner who set this policy
    pub owner: Pubkey,
    /// The agent this policy applies to
    pub agent: Pubkey,
    /// Maximum SOL (in lamports) the agent can spend per transaction
    pub max_spend_per_tx: u64,
    /// Maximum SOL (in lamports) the agent can spend per day
    pub daily_limit: u64,
    /// Running total of SOL spent in the current day
    pub daily_spent: u64,
    /// Unix timestamp of the current spending day (resets every 24h)
    pub day_start: i64,
    /// Whether the policy is active (owner can pause)
    pub is_active: bool,
    /// List of program IDs the agent is allowed to interact with
    #[max_len(MAX_ALLOWED_PROTOCOLS)]
    pub allowed_protocols: Vec<Pubkey>,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

/// Transaction log entry for audit trail
#[account]
#[derive(InitSpace)]
pub struct TransactionLog {
    /// The agent that executed this transaction
    pub agent: Pubkey,
    /// The owner of the agent
    pub owner: Pubkey,
    /// Amount in lamports
    pub amount: u64,
    /// The target program the agent interacted with
    pub target_protocol: Pubkey,
    /// Timestamp of execution
    pub executed_at: i64,
    /// Whether the transaction was approved or rejected
    pub was_approved: bool,
    /// Sequential nonce for uniqueness
    pub nonce: u64,
    /// Bump seed
    pub bump: u8,
}

/// Event emitted when a transaction attempt is rejected.
/// Failed instructions cannot persist TransactionLog accounts, so rejected
/// attempts are exposed through transaction logs instead.
#[event]
pub struct TransactionRejected {
    /// The agent that attempted this transaction
    pub agent: Pubkey,
    /// The owner of the agent
    pub owner: Pubkey,
    /// Requested amount in lamports
    pub amount: u64,
    /// The target program the agent attempted to interact with
    pub target_protocol: Pubkey,
    /// Running daily spend used for the validation
    pub daily_spent: u64,
    /// Configured daily limit
    pub daily_limit: u64,
    /// Timestamp of rejection
    pub rejected_at: i64,
    /// Machine-readable reason code. See constants.rs.
    pub reason_code: u8,
}

/// Nonce tracker per owner/agent pair for transaction log indexing
#[account]
#[derive(InitSpace)]
pub struct AgentNonce {
    pub owner: Pubkey,
    pub agent: Pubkey,
    pub nonce: u64,
    pub bump: u8,
}
