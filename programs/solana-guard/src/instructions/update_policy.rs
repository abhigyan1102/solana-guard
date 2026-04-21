use crate::constants::*;
use crate::error::SolanaGuardError;
use crate::state::*;
use anchor_lang::prelude::*;

/// Updates an existing policy's limits and allowed protocols.
/// Only the owner can update their agent's policy.
pub fn handler(
    ctx: Context<UpdatePolicy>,
    max_spend_per_tx: Option<u64>,
    daily_limit: Option<u64>,
    max_tx_per_day: Option<u64>,
    allowed_protocols: Option<Vec<Pubkey>>,
    slippage_bps: Option<u16>,
    is_active: Option<bool>,
) -> Result<()> {
    let policy = &mut ctx.accounts.policy;
    let new_max_spend_per_tx = max_spend_per_tx.unwrap_or(policy.max_spend_per_tx);
    let new_daily_limit = daily_limit.unwrap_or(policy.daily_limit);
    let new_max_tx_per_day = max_tx_per_day.unwrap_or(policy.max_tx_per_day);

    require!(
        new_max_spend_per_tx > 0,
        SolanaGuardError::InvalidSpendingLimit
    );
    require!(
        new_daily_limit >= new_max_spend_per_tx,
        SolanaGuardError::InvalidDailyLimit
    );
    require!(new_max_tx_per_day > 0, SolanaGuardError::InvalidTxLimit);

    policy.max_spend_per_tx = new_max_spend_per_tx;
    policy.daily_limit = new_daily_limit;
    policy.max_tx_per_day = new_max_tx_per_day;

    if let Some(protocols) = allowed_protocols {
        require!(
            protocols.len() <= MAX_ALLOWED_PROTOCOLS,
            SolanaGuardError::TooManyProtocols
        );
        policy.allowed_protocols = protocols;
    }

    if let Some(slippage_bps) = slippage_bps {
        policy.slippage_bps = slippage_bps;
    }

    if let Some(active) = is_active {
        policy.is_active = active;
    }

    msg!(
        "SolanaGuard: Policy updated for agent {} — max/tx: {}, daily: {}, tx/day: {}, slippage: {} bps, active: {}",
        policy.agent,
        policy.max_spend_per_tx,
        policy.daily_limit,
        policy.max_tx_per_day,
        policy.slippage_bps,
        policy.is_active
    );

    Ok(())
}

#[derive(Accounts)]
pub struct UpdatePolicy<'info> {
    /// The owner updating the policy
    pub owner: Signer<'info>,

    /// The agent config (validates ownership)
    #[account(
        seeds = [AGENT_SEED, owner.key().as_ref(), agent_config.agent.as_ref()],
        bump = agent_config.bump,
        has_one = owner @ SolanaGuardError::UnauthorizedOwner,
    )]
    pub agent_config: Account<'info, AgentConfig>,

    /// The policy to update
    #[account(
        mut,
        seeds = [POLICY_SEED, owner.key().as_ref(), agent_config.agent.as_ref()],
        bump = policy.bump,
        has_one = owner @ SolanaGuardError::UnauthorizedOwner,
    )]
    pub policy: Account<'info, Policy>,
}
