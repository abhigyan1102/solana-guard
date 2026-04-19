use {
    anchor_lang::{
        solana_program::instruction::Instruction,
        solana_program::pubkey::Pubkey,
        InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_register_agent() {
    let program_id = solana_guard::id();
    let owner = Keypair::new();
    let agent = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/solana_guard.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&owner.pubkey(), 10_000_000_000).unwrap();

    // Derive PDAs
    let (agent_config_pda, _) = Pubkey::find_program_address(
        &[b"agent", owner.pubkey().as_ref(), agent.pubkey().as_ref()],
        &program_id,
    );
    let (agent_nonce_pda, _) = Pubkey::find_program_address(
        &[b"nonce", agent.pubkey().as_ref()],
        &program_id,
    );

    let instruction = Instruction::new_with_bytes(
        program_id,
        &solana_guard::instruction::RegisterAgent {}.data(),
        solana_guard::accounts::RegisterAgent {
            owner: owner.pubkey(),
            agent: agent.pubkey(),
            agent_config: agent_config_pda,
            agent_nonce: agent_nonce_pda,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&owner.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[owner]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "register_agent failed: {:?}", res.err());
}
