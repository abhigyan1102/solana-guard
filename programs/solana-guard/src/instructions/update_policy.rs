use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::SolanaGuardError;
use crate::constants::*;

/// Updates an existing policy's limits and allowed protocols.
/// Only the owner can update their agent's policy.
pub fn handler(
    ctx: Context<UpdatePolicy>,
    max_spend_per_tx: Option<u64>,
    daily_limit: Option<u64>,
    allowed_protocols: Option<Vec<Pubkey>>,
    is_active: Option<bool>,
) -> Result<()> {
    let policy = &mut ctx.accounts.policy;

    if let Some(max_per_tx) = max_spend_per_tx {
        require!(max_per_tx > 0, SolanaGuardError::InvalidSpendingLimit);
        policy.max_spend_per_tx = max_per_tx;
    }

    if let Some(daily) = daily_limit {
        require!(
            daily >= policy.max_spend_per_tx,
            SolanaGuardError::InvalidDailyLimit
        );
        policy.daily_limit = daily;
    }

    if let Some(protocols) = allowed_protocols {
        require!(
            protocols.len() <= MAX_ALLOWED_PROTOCOLS,
            SolanaGuardError::TooManyProtocols
        );
        policy.allowed_protocols = protocols;
    }

    if let Some(active) = is_active {
        policy.is_active = active;
    }

    msg!(
        "SolanaGuard: Policy updated for agent {} — max/tx: {}, daily: {}, active: {}",
        policy.agent,
        policy.max_spend_per_tx,
        policy.daily_limit,
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
