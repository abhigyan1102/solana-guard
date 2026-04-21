use crate::constants::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::system_program;

pub fn handler(ctx: Context<FundVault>, amount: u64) -> Result<()> {
    let cpi_accounts = system_program::Transfer {
        from: ctx.accounts.owner.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.system_program.to_account_info(), cpi_accounts);
    system_program::transfer(cpi_ctx, amount)?;

    msg!(
        "SolanaGuard: Vault funded for agent {} with {} lamports",
        ctx.accounts.agent_config.agent,
        amount
    );

    Ok(())
}

#[derive(Accounts)]
pub struct FundVault<'info> {
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

    pub system_program: Program<'info, System>,
}
