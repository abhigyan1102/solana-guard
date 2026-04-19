use anchor_lang::prelude::*;

/// PDA seeds
pub const AGENT_SEED: &[u8] = b"agent";
pub const POLICY_SEED: &[u8] = b"policy";
pub const TX_LOG_SEED: &[u8] = b"tx_log";
pub const NONCE_SEED: &[u8] = b"nonce";

/// Time constants
pub const SECONDS_PER_DAY: i64 = 86_400;
