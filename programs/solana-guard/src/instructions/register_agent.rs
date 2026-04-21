use crate::constants::*;
use crate::state::*;
use anchor_lang::prelude::*;

/// Registers a new AI agent under the caller's ownership.
/// Creates an AgentConfig PDA and an AgentNonce tracker.
pub fn handler(ctx: Context<RegisterAgent>) -> Result<()> {
    let agent_config = &mut ctx.accounts.agent_config;
    let agent_nonce = &mut ctx.accounts.agent_nonce;
    let vault = &mut ctx.accounts.vault;
    let clock = Clock::get()?;

    agent_config.owner = ctx.accounts.owner.key();
    agent_config.agent = ctx.accounts.agent.key();
    agent_config.is_active = true;
    agent_config.registered_at = clock.unix_timestamp;
    agent_config.bump = ctx.bumps.agent_config;

    agent_nonce.owner = ctx.accounts.owner.key();
    agent_nonce.agent = ctx.accounts.agent.key();
    agent_nonce.nonce = 0;
    agent_nonce.bump = ctx.bumps.agent_nonce;

    vault.owner = ctx.accounts.owner.key();
    vault.agent = ctx.accounts.agent.key();
    vault.bump = ctx.bumps.vault;

    msg!(
        "SolanaGuard: Agent {} registered by owner {} with guarded vault {}",
        ctx.accounts.agent.key(),
        ctx.accounts.owner.key(),
        ctx.accounts.vault.key()
    );

    Ok(())
}

#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    /// The user registering the agent (pays for account creation)
    #[account(mut)]
    pub owner: Signer<'info>,

    /// The agent's public key (does not need to sign registration)
    /// CHECK: This is the agent's pubkey, validated by PDA derivation
    pub agent: UncheckedAccount<'info>,

    /// PDA storing the agent configuration
    #[account(
        init,
        payer = owner,
        space = 8 + AgentConfig::INIT_SPACE,
        seeds = [AGENT_SEED, owner.key().as_ref(), agent.key().as_ref()],
        bump,
    )]
    pub agent_config: Account<'info, AgentConfig>,

    /// PDA tracking the agent's transaction nonce
    #[account(
        init,
        payer = owner,
        space = 8 + AgentNonce::INIT_SPACE,
        seeds = [NONCE_SEED, owner.key().as_ref(), agent.key().as_ref()],
        bump,
    )]
    pub agent_nonce: Account<'info, AgentNonce>,

    /// PDA storing funds that can only be spent through guardrail checks
    #[account(
        init,
        payer = owner,
        space = 8 + Vault::INIT_SPACE,
        seeds = [VAULT_SEED, owner.key().as_ref(), agent.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,

    pub system_program: Program<'info, System>,
}
