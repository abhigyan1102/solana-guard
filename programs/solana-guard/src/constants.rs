use anchor_lang::prelude::*;

/// PDA seeds
pub const AGENT_SEED: &[u8] = b"agent";
pub const POLICY_SEED: &[u8] = b"policy";
pub const TX_LOG_SEED: &[u8] = b"tx_log";
pub const NONCE_SEED: &[u8] = b"nonce";

/// Time constants
pub const SECONDS_PER_DAY: i64 = 86_400;

/// Rejection reason codes emitted in TransactionRejected events
pub const REJECTION_AGENT_NOT_ACTIVE: u8 = 1;
pub const REJECTION_POLICY_NOT_ACTIVE: u8 = 2;
pub const REJECTION_EXCEEDS_PER_TX_LIMIT: u8 = 3;
pub const REJECTION_EXCEEDS_DAILY_LIMIT: u8 = 4;
pub const REJECTION_PROTOCOL_NOT_ALLOWED: u8 = 5;
