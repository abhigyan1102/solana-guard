use crate::constants::*;
use crate::error::SolanaGuardError;
use crate::state::*;
use anchor_lang::prelude::*;

/// Validates a transaction against the agent's policy, then executes it.
/// This is the core guardrail enforcement — every agent transaction must
/// pass through this instruction.
///
/// Checks enforced:
/// 1. Agent is active
/// 2. Policy is active  
/// 3. Amount ≤ max_spend_per_tx
/// 4. daily_spent + amount ≤ daily_limit (resets every 24h)
/// 5. Target protocol is in the allowed list
pub fn handler(
    ctx: Context<ValidateAndExecute>,
    amount: u64,
    target_protocol: Pubkey,
    observed_slippage_bps: u16,
) -> Result<()> {
    let agent_config = &ctx.accounts.agent_config;
    let policy = &mut ctx.accounts.policy;
    let tx_log = &mut ctx.accounts.tx_log;
    let agent_nonce = &mut ctx.accounts.agent_nonce;
    let vault = &ctx.accounts.vault;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;

    // ---- Check 1: Agent must be active ----
    if !agent_config.is_active {
        record_rejection(
            tx_log,
            agent_nonce,
            agent_config,
            policy,
            amount,
            target_protocol,
            observed_slippage_bps,
            policy.daily_spent,
            policy.tx_count_today,
            now,
            REJECTION_AGENT_NOT_ACTIVE,
            ctx.bumps.tx_log,
        );
        return Ok(());
    }

    // ---- Check 2: Policy must be active ----
    if !policy.is_active {
        record_rejection(
            tx_log,
            agent_nonce,
            agent_config,
            policy,
            amount,
            target_protocol,
            observed_slippage_bps,
            policy.daily_spent,
            policy.tx_count_today,
            now,
            REJECTION_POLICY_NOT_ACTIVE,
            ctx.bumps.tx_log,
        );
        return Ok(());
    }

    // ---- Check 3: Per-transaction limit ----
    if amount > policy.max_spend_per_tx {
        record_rejection(
            tx_log,
            agent_nonce,
            agent_config,
            policy,
            amount,
            target_protocol,
            observed_slippage_bps,
            policy.daily_spent,
            policy.tx_count_today,
            now,
            REJECTION_EXCEEDS_PER_TX_LIMIT,
            ctx.bumps.tx_log,
        );
        return Ok(());
    }

    // ---- Check 4: Daily limit (with 24h reset) ----
    let should_reset_day = now - policy.day_start >= SECONDS_PER_DAY;
    let current_daily_spent = if should_reset_day {
        0
    } else {
        policy.daily_spent
    };
    let current_tx_count = if should_reset_day {
        0
    } else {
        policy.tx_count_today
    };

    if current_daily_spent.checked_add(amount).unwrap_or(u64::MAX) > policy.daily_limit {
        record_rejection(
            tx_log,
            agent_nonce,
            agent_config,
            policy,
            amount,
            target_protocol,
            observed_slippage_bps,
            current_daily_spent,
            current_tx_count,
            now,
            REJECTION_EXCEEDS_DAILY_LIMIT,
            ctx.bumps.tx_log,
        );
        return Ok(());
    }

    // ---- Check 5: Daily transaction count ----
    if current_tx_count.checked_add(1).unwrap_or(u64::MAX) > policy.max_tx_per_day {
        record_rejection(
            tx_log,
            agent_nonce,
            agent_config,
            policy,
            amount,
            target_protocol,
            observed_slippage_bps,
            current_daily_spent,
            current_tx_count,
            now,
            REJECTION_EXCEEDS_TX_LIMIT,
            ctx.bumps.tx_log,
        );
        return Ok(());
    }

    // ---- Check 6: Protocol allowlist ----
    if !policy.allowed_protocols.contains(&target_protocol) {
        record_rejection(
            tx_log,
            agent_nonce,
            agent_config,
            policy,
            amount,
            target_protocol,
            observed_slippage_bps,
            current_daily_spent,
            current_tx_count,
            now,
            REJECTION_PROTOCOL_NOT_ALLOWED,
            ctx.bumps.tx_log,
        );
        return Ok(());
    }

    // ---- Check 7: Slippage guard ----
    if observed_slippage_bps > policy.slippage_bps {
        record_rejection(
            tx_log,
            agent_nonce,
            agent_config,
            policy,
            amount,
            target_protocol,
            observed_slippage_bps,
            current_daily_spent,
            current_tx_count,
            now,
            REJECTION_EXCEEDS_SLIPPAGE_LIMIT,
            ctx.bumps.tx_log,
        );
        return Ok(());
    }

    // ---- Check 8: Vault must hold the guarded funds ----
    if vault.to_account_info().lamports() < amount {
        record_rejection(
            tx_log,
            agent_nonce,
            agent_config,
            policy,
            amount,
            target_protocol,
            observed_slippage_bps,
            current_daily_spent,
            current_tx_count,
            now,
            REJECTION_INSUFFICIENT_VAULT_BALANCE,
            ctx.bumps.tx_log,
        );
        return Ok(());
    }

    // ---- All checks passed — execute the guarded transfer ----
    let vault_info = vault.to_account_info();
    let recipient_info = ctx.accounts.recipient.to_account_info();
    **vault_info.try_borrow_mut_lamports()? -= amount;
    **recipient_info.try_borrow_mut_lamports()? += amount;

    // ---- Update state after the transfer succeeds ----
    if should_reset_day {
        // Reset the daily counter — new day
        policy.daily_spent = 0;
        policy.tx_count_today = 0;
        policy.day_start = now;
    }
    policy.daily_spent = policy.daily_spent.checked_add(amount).unwrap();
    policy.tx_count_today = policy.tx_count_today.checked_add(1).unwrap();

    // Write transaction log
    tx_log.agent = agent_config.agent;
    tx_log.owner = agent_config.owner;
    tx_log.amount = amount;
    tx_log.slippage_bps = observed_slippage_bps;
    tx_log.target_protocol = target_protocol;
    tx_log.executed_at = now;
    tx_log.was_approved = true;
    tx_log.reason_code = REJECTION_NONE;
    tx_log.nonce = agent_nonce.nonce;
    tx_log.bump = ctx.bumps.tx_log;

    // Increment nonce for next transaction
    agent_nonce.nonce = agent_nonce.nonce.checked_add(1).unwrap();

    msg!(
        "SolanaGuard: ✅ APPROVED — Agent {} spent {} lamports from vault {} to {}. Daily total: {}/{}, tx count: {}/{}, slippage: {}/{} bps",
        agent_config.agent,
        amount,
        vault.key(),
        ctx.accounts.recipient.key(),
        policy.daily_spent,
        policy.daily_limit,
        policy.tx_count_today,
        policy.max_tx_per_day,
        observed_slippage_bps,
        policy.slippage_bps
    );

    Ok(())
}

fn record_rejection(
    tx_log: &mut Account<TransactionLog>,
    agent_nonce: &mut Account<AgentNonce>,
    agent_config: &AgentConfig,
    policy: &Policy,
    amount: u64,
    target_protocol: Pubkey,
    slippage_bps: u16,
    daily_spent: u64,
    tx_count_today: u64,
    rejected_at: i64,
    reason_code: u8,
    tx_log_bump: u8,
) {
    tx_log.agent = agent_config.agent;
    tx_log.owner = agent_config.owner;
    tx_log.amount = amount;
    tx_log.slippage_bps = slippage_bps;
    tx_log.target_protocol = target_protocol;
    tx_log.executed_at = rejected_at;
    tx_log.was_approved = false;
    tx_log.reason_code = reason_code;
    tx_log.nonce = agent_nonce.nonce;
    tx_log.bump = tx_log_bump;

    agent_nonce.nonce = agent_nonce.nonce.checked_add(1).unwrap();

    emit!(TransactionRejected {
        agent: agent_config.agent,
        owner: agent_config.owner,
        amount,
        slippage_bps,
        target_protocol,
        daily_spent,
        tx_count_today,
        daily_limit: policy.daily_limit,
        rejected_at,
        reason_code,
    });

    msg!(
        "SolanaGuard: REJECTED — Agent {} attempted {} lamports to {} with reason code {}",
        agent_config.agent,
        amount,
        target_protocol,
        reason_code
    );
}

#[derive(Accounts)]
#[instruction(amount: u64, target_protocol: Pubkey, observed_slippage_bps: u16)]
pub struct ValidateAndExecute<'info> {
    /// The agent executing the transaction (must be the registered agent)
    #[account(mut)]
    pub agent: Signer<'info>,

    /// The owner's account (for PDA derivation, does not need to sign)
    /// CHECK: Validated via PDA seeds on agent_config
    pub owner: UncheckedAccount<'info>,

    /// Agent configuration (validates agent identity)
    #[account(
        seeds = [AGENT_SEED, owner.key().as_ref(), agent.key().as_ref()],
        bump = agent_config.bump,
        constraint = agent_config.agent == agent.key() @ SolanaGuardError::UnauthorizedAgent,
    )]
    pub agent_config: Account<'info, AgentConfig>,

    /// The policy to enforce
    #[account(
        mut,
        seeds = [POLICY_SEED, owner.key().as_ref(), agent.key().as_ref()],
        bump = policy.bump,
    )]
    pub policy: Account<'info, Policy>,

    /// Program-controlled vault holding the funds subject to guardrails
    #[account(
        mut,
        seeds = [VAULT_SEED, owner.key().as_ref(), agent.key().as_ref()],
        bump = vault.bump,
        has_one = owner @ SolanaGuardError::UnauthorizedOwner,
        has_one = agent @ SolanaGuardError::UnauthorizedAgent,
    )]
    pub vault: Account<'info, Vault>,

    /// CHECK: Recipient of the guarded transfer. The key must match the allowlisted target.
    #[account(mut, address = target_protocol)]
    pub recipient: UncheckedAccount<'info>,

    /// Transaction log entry (created per transaction)
    #[account(
        init,
        payer = agent,
        space = 8 + TransactionLog::INIT_SPACE,
        seeds = [
            TX_LOG_SEED,
            owner.key().as_ref(),
            agent.key().as_ref(),
            &agent_nonce.nonce.to_le_bytes(),
        ],
        bump,
    )]
    pub tx_log: Account<'info, TransactionLog>,

    /// Nonce tracker for the agent
    #[account(
        mut,
        seeds = [NONCE_SEED, owner.key().as_ref(), agent.key().as_ref()],
        bump = agent_nonce.bump,
        constraint = agent_nonce.owner == owner.key() @ SolanaGuardError::UnauthorizedOwner,
        constraint = agent_nonce.agent == agent.key() @ SolanaGuardError::UnauthorizedAgent,
    )]
    pub agent_nonce: Account<'info, AgentNonce>,

    pub system_program: Program<'info, System>,
}
