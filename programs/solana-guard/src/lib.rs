pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("FRuK1VzhqjybBMhp8UGVipJ9jkyuT9Dy7YJHAREwSApw");

#[program]
pub mod solana_guard {
    use super::*;

    /// Register a new AI agent under the caller's ownership.
    /// Creates an AgentConfig PDA bound to the owner+agent pair and a vault PDA.
    pub fn register_agent(ctx: Context<RegisterAgent>) -> Result<()> {
        instructions::register_agent::handler(ctx)
    }

    /// Fund the program-controlled vault for a registered agent.
    pub fn fund_vault(ctx: Context<FundVault>, amount: u64) -> Result<()> {
        instructions::fund_vault::handler(ctx, amount)
    }

    /// Set the risk policy for a registered agent.
    /// Defines per-tx limit, daily limit, and allowed protocols.
    pub fn set_policy(
        ctx: Context<SetPolicy>,
        max_spend_per_tx: u64,
        daily_limit: u64,
        max_tx_per_day: u64,
        allowed_protocols: Vec<Pubkey>,
        slippage_bps: u16,
    ) -> Result<()> {
        instructions::set_policy::handler(
            ctx,
            max_spend_per_tx,
            daily_limit,
            max_tx_per_day,
            allowed_protocols,
            slippage_bps,
        )
    }

    /// Validate a transaction against the agent's policy and log it.
    /// This is the core guardrail — every agent action must go through here.
    pub fn validate_and_execute(
        ctx: Context<ValidateAndExecute>,
        amount: u64,
        target_protocol: Pubkey,
        observed_slippage_bps: u16,
    ) -> Result<()> {
        instructions::validate_and_execute::handler(
            ctx,
            amount,
            target_protocol,
            observed_slippage_bps,
        )
    }

    /// Toggle agent active status (emergency kill switch).
    /// Owner can pause/unpause their agent at any time.
    pub fn toggle_agent(ctx: Context<ToggleAgent>, is_active: bool) -> Result<()> {
        instructions::toggle_agent::handler(ctx, is_active)
    }

    /// Withdraw SOL from the vault back to the owner.
    pub fn withdraw_vault(ctx: Context<WithdrawVault>, amount: u64) -> Result<()> {
        instructions::withdraw_vault::handler(ctx, amount)
    }

    /// Update an existing policy's parameters.
    /// Supports partial updates — only provided fields are changed.
    pub fn update_policy(
        ctx: Context<UpdatePolicy>,
        max_spend_per_tx: Option<u64>,
        daily_limit: Option<u64>,
        max_tx_per_day: Option<u64>,
        allowed_protocols: Option<Vec<Pubkey>>,
        slippage_bps: Option<u16>,
        is_active: Option<bool>,
    ) -> Result<()> {
        instructions::update_policy::handler(
            ctx,
            max_spend_per_tx,
            daily_limit,
            max_tx_per_day,
            allowed_protocols,
            slippage_bps,
            is_active,
        )
    }
}
