use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::SolanaGuardError;
use crate::constants::*;

/// Sets or updates the risk policy for a registered agent.
/// Only the owner can set policies for their agents.
pub fn handler(
    ctx: Context<SetPolicy>,
    max_spend_per_tx: u64,
    daily_limit: u64,
    allowed_protocols: Vec<Pubkey>,
) -> Result<()> {
    // Validate inputs
    require!(max_spend_per_tx > 0, SolanaGuardError::InvalidSpendingLimit);
    require!(daily_limit >= max_spend_per_tx, SolanaGuardError::InvalidDailyLimit);
    require!(
        allowed_protocols.len() <= MAX_ALLOWED_PROTOCOLS,
        SolanaGuardError::TooManyProtocols
    );

    let policy = &mut ctx.accounts.policy;
    let clock = Clock::get()?;

    policy.owner = ctx.accounts.owner.key();
    policy.agent = ctx.accounts.agent_config.agent;
    policy.max_spend_per_tx = max_spend_per_tx;
    policy.daily_limit = daily_limit;
    policy.daily_spent = 0;
    policy.day_start = clock.unix_timestamp;
    policy.is_active = true;
    policy.allowed_protocols = allowed_protocols;
    policy.bump = ctx.bumps.policy;

    msg!(
        "SolanaGuard: Policy set for agent {} — max/tx: {} lamports, daily: {} lamports",
        policy.agent,
        max_spend_per_tx,
        daily_limit
    );

    Ok(())
}

#[derive(Accounts)]
pub struct SetPolicy<'info> {
    /// The owner setting the policy
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The agent config (must belong to the owner)
    #[account(
        seeds = [AGENT_SEED, owner.key().as_ref(), agent_config.agent.as_ref()],
        bump = agent_config.bump,
        has_one = owner @ SolanaGuardError::UnauthorizedOwner,
    )]
    pub agent_config: Account<'info, AgentConfig>,

    /// The policy PDA (created or updated)
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + Policy::INIT_SPACE,
        seeds = [POLICY_SEED, owner.key().as_ref(), agent_config.agent.as_ref()],
        bump,
    )]
    pub policy: Account<'info, Policy>,

    pub system_program: Program<'info, System>,
}
