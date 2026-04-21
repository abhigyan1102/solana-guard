use crate::constants::*;
use crate::error::SolanaGuardError;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn handler(ctx: Context<WithdrawVault>, amount: u64) -> Result<()> {
    let vault_info = ctx.accounts.vault.to_account_info();
    let owner_info = ctx.accounts.owner.to_account_info();

    require!(
        vault_info.lamports() >= amount,
        SolanaGuardError::InsufficientVaultBalance
    );

    **vault_info.try_borrow_mut_lamports()? -= amount;
    **owner_info.try_borrow_mut_lamports()? += amount;

    msg!(
        "SolanaGuard: Owner {} withdrew {} lamports from vault for agent {}",
        ctx.accounts.owner.key(),
        amount,
        ctx.accounts.agent_config.agent
    );

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawVault<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [AGENT_SEED, owner.key().as_ref(), agent_config.agent.as_ref()],
        bump = agent_config.bump,
        has_one = owner,
    )]
    pub agent_config: Account<'info, AgentConfig>,

    #[account(
        mut,
        seeds = [VAULT_SEED, owner.key().as_ref(), agent_config.agent.as_ref()],
        bump = vault.bump,
        has_one = owner,
    )]
    pub vault: Account<'info, Vault>,
}
