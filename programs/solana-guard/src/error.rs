use anchor_lang::prelude::*;

#[error_code]
pub enum SolanaGuardError {
    #[msg("Agent is not active")]
    AgentNotActive,

    #[msg("Policy is not active")]
    PolicyNotActive,

    #[msg("Transaction amount exceeds per-transaction spending limit")]
    ExceedsPerTxLimit,

    #[msg("Transaction would exceed daily spending limit")]
    ExceedsDailyLimit,

    #[msg("Target protocol is not in the allowed list")]
    ProtocolNotAllowed,

    #[msg("Only the owner can perform this action")]
    UnauthorizedOwner,

    #[msg("Only the registered agent can execute transactions")]
    UnauthorizedAgent,

    #[msg("Allowed protocols list exceeds maximum capacity")]
    TooManyProtocols,

    #[msg("Agent is already registered")]
    AgentAlreadyRegistered,

    #[msg("Invalid spending limit: must be greater than zero")]
    InvalidSpendingLimit,

    #[msg("Daily limit must be greater than or equal to per-transaction limit")]
    InvalidDailyLimit,
}
