use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::SolanaGuardError;
use crate::constants::*;

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
) -> Result<()> {
    let agent_config = &ctx.accounts.agent_config;
    let policy = &mut ctx.accounts.policy;
    let tx_log = &mut ctx.accounts.tx_log;
    let agent_nonce = &mut ctx.accounts.agent_nonce;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;

    // ---- Check 1: Agent must be active ----
    if !agent_config.is_active {
        emit_rejection(
            agent_config,
            policy,
            amount,
            target_protocol,
            policy.daily_spent,
            now,
            REJECTION_AGENT_NOT_ACTIVE,
        );
        return err!(SolanaGuardError::AgentNotActive);
    }

    // ---- Check 2: Policy must be active ----
    if !policy.is_active {
        emit_rejection(
            agent_config,
            policy,
            amount,
            target_protocol,
            policy.daily_spent,
            now,
            REJECTION_POLICY_NOT_ACTIVE,
        );
        return err!(SolanaGuardError::PolicyNotActive);
    }

    // ---- Check 3: Per-transaction limit ----
    if amount > policy.max_spend_per_tx {
        emit_rejection(
            agent_config,
            policy,
            amount,
            target_protocol,
            policy.daily_spent,
            now,
            REJECTION_EXCEEDS_PER_TX_LIMIT,
        );
        return err!(SolanaGuardError::ExceedsPerTxLimit);
    }

    // ---- Check 4: Daily limit (with 24h reset) ----
    let should_reset_day = now - policy.day_start >= SECONDS_PER_DAY;
    let current_daily_spent = if should_reset_day {
        0
    } else {
        policy.daily_spent
    };

    if current_daily_spent.checked_add(amount).unwrap_or(u64::MAX) > policy.daily_limit {
        emit_rejection(
            agent_config,
            policy,
            amount,
            target_protocol,
            current_daily_spent,
            now,
            REJECTION_EXCEEDS_DAILY_LIMIT,
        );
        return err!(SolanaGuardError::ExceedsDailyLimit);
    }

    // ---- Check 5: Protocol allowlist ----
    if !policy.allowed_protocols.contains(&target_protocol) {
        emit_rejection(
            agent_config,
            policy,
            amount,
            target_protocol,
            current_daily_spent,
            now,
            REJECTION_PROTOCOL_NOT_ALLOWED,
        );
        return err!(SolanaGuardError::ProtocolNotAllowed);
    }

    // ---- All checks passed — update state ----
    if should_reset_day {
        // Reset the daily counter — new day
        policy.daily_spent = 0;
        policy.day_start = now;
    }
    policy.daily_spent = policy.daily_spent.checked_add(amount).unwrap();

    // Write transaction log
    tx_log.agent = agent_config.agent;
    tx_log.owner = agent_config.owner;
    tx_log.amount = amount;
    tx_log.target_protocol = target_protocol;
    tx_log.executed_at = now;
    tx_log.was_approved = true;
    tx_log.nonce = agent_nonce.nonce;
    tx_log.bump = ctx.bumps.tx_log;

    // Increment nonce for next transaction
    agent_nonce.nonce = agent_nonce.nonce.checked_add(1).unwrap();

    msg!(
        "SolanaGuard: ✅ APPROVED — Agent {} spent {} lamports on protocol {}. Daily total: {}/{}",
        agent_config.agent,
        amount,
        target_protocol,
        policy.daily_spent,
        policy.daily_limit
    );

    Ok(())
}

fn emit_rejection(
    agent_config: &AgentConfig,
    policy: &Policy,
    amount: u64,
    target_protocol: Pubkey,
    daily_spent: u64,
    rejected_at: i64,
    reason_code: u8,
) {
    emit!(TransactionRejected {
        agent: agent_config.agent,
        owner: agent_config.owner,
        amount,
        target_protocol,
        daily_spent,
        daily_limit: policy.daily_limit,
        rejected_at,
        reason_code,
    });
}

#[derive(Accounts)]
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
