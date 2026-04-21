use crate::constants::*;
use crate::error::SolanaGuardError;
use crate::state::*;
use anchor_lang::prelude::*;

/// Toggles an agent's active status (pause/unpause).
/// Only the owner can call this.
pub fn handler(ctx: Context<ToggleAgent>, is_active: bool) -> Result<()> {
    let agent_config = &mut ctx.accounts.agent_config;

    agent_config.is_active = is_active;

    msg!(
        "SolanaGuard: Agent {} is now {}",
        agent_config.agent,
        if is_active { "ACTIVE" } else { "PAUSED" }
    );

    Ok(())
}

#[derive(Accounts)]
pub struct ToggleAgent<'info> {
    /// The owner toggling the agent
    pub owner: Signer<'info>,

    /// The agent config to toggle
    #[account(
        mut,
        seeds = [AGENT_SEED, owner.key().as_ref(), agent_config.agent.as_ref()],
        bump = agent_config.bump,
        has_one = owner @ SolanaGuardError::UnauthorizedOwner,
    )]
    pub agent_config: Account<'info, AgentConfig>,
}
