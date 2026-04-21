#![cfg(feature = "sbf-tests")]

use {
    anchor_lang::{
        solana_program::instruction::Instruction, solana_program::pubkey::Pubkey,
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    std::{fs, io::ErrorKind, path::PathBuf},
};

struct TestContext {
    svm: LiteSVM,
    owner: Keypair,
    agent: Keypair,
    recipient: Keypair,
    allowed_protocol: Pubkey,
    agent_config_pda: Pubkey,
    agent_nonce_pda: Pubkey,
    policy_pda: Pubkey,
    vault_pda: Pubkey,
}

#[test]
fn test_register_agent() {
    let Some(mut ctx) = setup() else {
        return;
    };

    register_agent(&mut ctx);

    let agent_config_account = ctx
        .svm
        .get_account(&ctx.agent_config_pda)
        .expect("agent config should exist");
    let agent_config =
        solana_guard::AgentConfig::try_deserialize(&mut agent_config_account.data.as_slice())
            .expect("agent config should deserialize");

    assert_eq!(agent_config.owner, ctx.owner.pubkey());
    assert_eq!(agent_config.agent, ctx.agent.pubkey());
    assert!(agent_config.is_active);
}

#[test]
fn test_validate_and_execute_enforces_daily_tx_limit_and_slippage() {
    let Some(mut ctx) = setup() else {
        return;
    };

    register_agent(&mut ctx);
    set_policy(&mut ctx, 1_000_000, 2_000_000, 1, 50);
    fund_vault(&mut ctx, 1_000_000);
    let allowed_protocol = ctx.allowed_protocol;
    let recipient_before = balance(&ctx, &ctx.recipient.pubkey());
    let vault_before = balance(&ctx, &ctx.vault_pda);

    let first_attempt = validate_and_execute(&mut ctx, 500_000, allowed_protocol, 25);
    assert!(first_attempt.is_ok(), "first guarded tx should pass");
    let approved_log = fetch_tx_log(&ctx, 0);

    let policy_after_success = fetch_policy(&ctx);
    assert_eq!(policy_after_success.daily_spent, 500_000);
    assert_eq!(policy_after_success.tx_count_today, 1);
    assert_eq!(policy_after_success.max_tx_per_day, 1);
    assert_eq!(policy_after_success.slippage_bps, 50);
    assert_eq!(
        balance(&ctx, &ctx.recipient.pubkey()),
        recipient_before + 500_000
    );
    assert_eq!(balance(&ctx, &ctx.vault_pda), vault_before - 500_000);
    assert!(approved_log.was_approved);
    assert_eq!(approved_log.reason_code, solana_guard::REJECTION_NONE);

    let second_attempt = validate_and_execute(&mut ctx, 100_000, allowed_protocol, 25);
    assert!(second_attempt.is_ok(), "rejected tx should still be logged");
    let rejected_tx_limit_log = fetch_tx_log(&ctx, 1);

    let policy_after_failures = fetch_policy(&ctx);
    assert_eq!(policy_after_failures.daily_spent, 500_000);
    assert_eq!(policy_after_failures.tx_count_today, 1);
    assert_eq!(
        balance(&ctx, &ctx.recipient.pubkey()),
        recipient_before + 500_000
    );
    assert_eq!(balance(&ctx, &ctx.vault_pda), vault_before - 500_000);
    assert!(!rejected_tx_limit_log.was_approved);
    assert_eq!(
        rejected_tx_limit_log.reason_code,
        solana_guard::REJECTION_EXCEEDS_TX_LIMIT
    );
}

#[test]
fn test_validate_and_execute_logs_rejected_slippage_attempts() {
    let Some(mut ctx) = setup() else {
        return;
    };

    register_agent(&mut ctx);
    set_policy(&mut ctx, 1_000_000, 2_000_000, 3, 50);
    fund_vault(&mut ctx, 1_000_000);
    let allowed_protocol = ctx.allowed_protocol;
    let recipient_before = balance(&ctx, &ctx.recipient.pubkey());
    let vault_before = balance(&ctx, &ctx.vault_pda);

    let rejected_attempt = validate_and_execute(&mut ctx, 250_000, allowed_protocol, 75);
    assert!(
        rejected_attempt.is_ok(),
        "slippage rejection should be logged"
    );

    let rejected_log = fetch_tx_log(&ctx, 0);
    let policy_after = fetch_policy(&ctx);

    assert!(!rejected_log.was_approved);
    assert_eq!(
        rejected_log.reason_code,
        solana_guard::REJECTION_EXCEEDS_SLIPPAGE_LIMIT
    );
    assert_eq!(policy_after.daily_spent, 0);
    assert_eq!(policy_after.tx_count_today, 0);
    assert_eq!(balance(&ctx, &ctx.recipient.pubkey()), recipient_before);
    assert_eq!(balance(&ctx, &ctx.vault_pda), vault_before);
}

fn setup() -> Option<TestContext> {
    let program_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/solana_guard.so");
    let bytes = match fs::read(&program_path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            eprintln!(
                "Skipping LiteSVM tests because {} does not exist. Run `anchor build` to generate it.",
                program_path.display()
            );
            return None;
        }
        Err(err) => panic!("failed to read {}: {err}", program_path.display()),
    };

    let program_id = solana_guard::id();
    let owner = Keypair::new();
    let agent = Keypair::new();
    let recipient = Keypair::new();
    let allowed_protocol = recipient.pubkey();
    let mut svm = LiteSVM::new();
    svm.add_program(program_id, &bytes).unwrap();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&agent.pubkey(), 10_000_000_000).unwrap();
    svm.airdrop(&recipient.pubkey(), 1_000_000).unwrap();

    let (agent_config_pda, _) = Pubkey::find_program_address(
        &[b"agent", owner.pubkey().as_ref(), agent.pubkey().as_ref()],
        &program_id,
    );
    let (agent_nonce_pda, _) = Pubkey::find_program_address(
        &[b"nonce", owner.pubkey().as_ref(), agent.pubkey().as_ref()],
        &program_id,
    );
    let (policy_pda, _) = Pubkey::find_program_address(
        &[b"policy", owner.pubkey().as_ref(), agent.pubkey().as_ref()],
        &program_id,
    );
    let (vault_pda, _) = Pubkey::find_program_address(
        &[b"vault", owner.pubkey().as_ref(), agent.pubkey().as_ref()],
        &program_id,
    );

    Some(TestContext {
        svm,
        owner,
        agent,
        recipient,
        allowed_protocol,
        agent_config_pda,
        agent_nonce_pda,
        policy_pda,
        vault_pda,
    })
}

fn register_agent(ctx: &mut TestContext) {
    let instruction = Instruction::new_with_bytes(
        solana_guard::id(),
        &solana_guard::instruction::RegisterAgent {}.data(),
        solana_guard::accounts::RegisterAgent {
            owner: ctx.owner.pubkey(),
            agent: ctx.agent.pubkey(),
            agent_config: ctx.agent_config_pda,
            agent_nonce: ctx.agent_nonce_pda,
            vault: ctx.vault_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    );

    let result = send_tx(
        &mut ctx.svm,
        &[instruction],
        &ctx.owner.pubkey(),
        &[&ctx.owner],
    );
    assert!(result.is_ok(), "register_agent failed: {:?}", result.err());
}

fn fund_vault(ctx: &mut TestContext, amount: u64) {
    let instruction = Instruction::new_with_bytes(
        solana_guard::id(),
        &solana_guard::instruction::FundVault { amount }.data(),
        solana_guard::accounts::FundVault {
            owner: ctx.owner.pubkey(),
            agent_config: ctx.agent_config_pda,
            vault: ctx.vault_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    );

    let result = send_tx(
        &mut ctx.svm,
        &[instruction],
        &ctx.owner.pubkey(),
        &[&ctx.owner],
    );
    assert!(result.is_ok(), "fund_vault failed: {:?}", result.err());
}

fn set_policy(
    ctx: &mut TestContext,
    max_spend_per_tx: u64,
    daily_limit: u64,
    max_tx_per_day: u64,
    slippage_bps: u16,
) {
    let instruction = Instruction::new_with_bytes(
        solana_guard::id(),
        &solana_guard::instruction::SetPolicy {
            max_spend_per_tx,
            daily_limit,
            max_tx_per_day,
            allowed_protocols: vec![ctx.allowed_protocol],
            slippage_bps,
        }
        .data(),
        solana_guard::accounts::SetPolicy {
            owner: ctx.owner.pubkey(),
            agent_config: ctx.agent_config_pda,
            policy: ctx.policy_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    );

    let result = send_tx(
        &mut ctx.svm,
        &[instruction],
        &ctx.owner.pubkey(),
        &[&ctx.owner],
    );
    assert!(result.is_ok(), "set_policy failed: {:?}", result.err());
}

fn validate_and_execute(
    ctx: &mut TestContext,
    amount: u64,
    target_protocol: Pubkey,
    observed_slippage_bps: u16,
) -> Result<(), String> {
    let nonce = fetch_nonce(ctx);
    let (tx_log_pda, _) = Pubkey::find_program_address(
        &[b"tx_log", ctx.agent.pubkey().as_ref(), &nonce.to_le_bytes()],
        &solana_guard::id(),
    );

    let instruction = Instruction::new_with_bytes(
        solana_guard::id(),
        &solana_guard::instruction::ValidateAndExecute {
            amount,
            target_protocol,
            observed_slippage_bps,
        }
        .data(),
        solana_guard::accounts::ValidateAndExecute {
            agent: ctx.agent.pubkey(),
            owner: ctx.owner.pubkey(),
            agent_config: ctx.agent_config_pda,
            policy: ctx.policy_pda,
            vault: ctx.vault_pda,
            recipient: ctx.recipient.pubkey(),
            tx_log: tx_log_pda,
            agent_nonce: ctx.agent_nonce_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    );

    send_tx(
        &mut ctx.svm,
        &[instruction],
        &ctx.agent.pubkey(),
        &[&ctx.agent],
    )
    .map(|_| ())
}

fn balance(ctx: &TestContext, address: &Pubkey) -> u64 {
    ctx.svm
        .get_account(address)
        .map(|account| account.lamports)
        .unwrap_or(0)
}

fn fetch_policy(ctx: &TestContext) -> solana_guard::Policy {
    let policy_account = ctx
        .svm
        .get_account(&ctx.policy_pda)
        .expect("policy account should exist");
    solana_guard::Policy::try_deserialize(&mut policy_account.data.as_slice())
        .expect("policy should deserialize")
}

fn fetch_nonce(ctx: &TestContext) -> u64 {
    let nonce_account = ctx
        .svm
        .get_account(&ctx.agent_nonce_pda)
        .expect("agent nonce should exist");
    let nonce = solana_guard::AgentNonce::try_deserialize(&mut nonce_account.data.as_slice())
        .expect("nonce should deserialize");
    nonce.nonce
}

fn fetch_tx_log(ctx: &TestContext, nonce: u64) -> solana_guard::TransactionLog {
    let (tx_log_pda, _) = Pubkey::find_program_address(
        &[b"tx_log", ctx.agent.pubkey().as_ref(), &nonce.to_le_bytes()],
        &solana_guard::id(),
    );
    let tx_log_account = ctx
        .svm
        .get_account(&tx_log_pda)
        .expect("tx log should exist");
    solana_guard::TransactionLog::try_deserialize(&mut tx_log_account.data.as_slice())
        .expect("tx log should deserialize")
}

fn send_tx(
    svm: &mut LiteSVM,
    instructions: &[Instruction],
    payer: &Pubkey,
    signers: &[&Keypair],
) -> Result<(), String> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(instructions, Some(payer), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers)
        .map_err(|err| format!("failed to sign tx: {err}"))?;

    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|err| format!("{:?}", err.err))
}
