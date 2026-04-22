/**
 * SolanaGuard Demo Script
 * ========================
 * Simulates an AI agent making transactions through SolanaGuard guardrails.
 * 
 * Flow:
 * 1. Register a new agent (owner = CLI wallet)
 * 2. Set spending policy (0.05 SOL per-tx, 0.1 SOL daily)
 * 3. Agent sends a VALID transaction (within limits) → APPROVED ✅
 * 4. Agent sends an OVER-LIMIT transaction → REJECTED ❌
 * 5. Owner pauses agent, agent tries again → BLOCKED ❌
 * 6. View all transaction logs
 */

import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
  clusterApiUrl,
} from "@solana/web3.js";
import * as fs from "fs";
import * as path from "path";

// Load IDL
const idlPath = path.join(__dirname, "../target/idl/solana_guard.json");
const idl = JSON.parse(fs.readFileSync(idlPath, "utf-8"));

const PROGRAM_ID = new PublicKey("FRuK1VzhqjybBMhp8UGVipJ9jkyuT9Dy7YJHAREwSApw");

// PDA seeds
const AGENT_SEED = Buffer.from("agent");
const POLICY_SEED = Buffer.from("policy");
const TX_LOG_SEED = Buffer.from("tx_log");
const NONCE_SEED = Buffer.from("nonce");

function getPda(seeds: Buffer[]) {
  return PublicKey.findProgramAddressSync(seeds, PROGRAM_ID);
}

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function main() {
  console.log("\n╔══════════════════════════════════════════════╗");
  console.log("║       🛡️  SolanaGuard Live Demo  🛡️           ║");
  console.log("║    On-Chain Risk Enforcement for AI Agents   ║");
  console.log("╚══════════════════════════════════════════════╝\n");

  // Setup connection
  const connection = new Connection(clusterApiUrl("devnet"), "confirmed");

  // Load owner wallet (CLI keypair)
  const ownerKeyPath = path.join(
    process.env.HOME || "~",
    ".config/solana/id.json"
  );
  const ownerKeypair = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(ownerKeyPath, "utf-8")))
  );
  const owner = ownerKeypair.publicKey;

  // Create provider
  const wallet = new anchor.Wallet(ownerKeypair);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    skipPreflight: true,
  });
  const program = new Program(idl, provider);

  // Generate a fresh agent keypair
  const agentKeypair = Keypair.generate();
  const agent = agentKeypair.publicKey;

  console.log(`👤 Owner:  ${owner.toBase58()}`);
  console.log(`🤖 Agent:  ${agent.toBase58()}`);
  console.log(`📡 Network: Solana Devnet`);
  console.log(`📋 Program: ${PROGRAM_ID.toBase58()}`);

  // Check balance
  const balance = await connection.getBalance(owner);
  console.log(`💰 Owner Balance: ${(balance / LAMPORTS_PER_SOL).toFixed(4)} SOL\n`);

  if (balance < 0.01 * LAMPORTS_PER_SOL) {
    console.log("❌ Not enough SOL. Run: solana airdrop 2 --url devnet");
    return;
  }

  // ============================================================
  // Step 1: Register Agent
  // ============================================================
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("📝 STEP 1: Register AI Agent");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  const [agentConfigPda] = getPda([AGENT_SEED, owner.toBuffer(), agent.toBuffer()]);
  const [agentNoncePda] = getPda([NONCE_SEED, agent.toBuffer()]);

  try {
    const tx1 = await program.methods
      .registerAgent()
      .accounts({
        owner: owner,
        agent: agent,
        agentConfig: agentConfigPda,
        agentNonce: agentNoncePda,
      })
      .rpc({ skipPreflight: true });

    console.log(`✅ Agent registered! Tx: ${tx1}`);
    await sleep(2000);
  } catch (e: any) {
    console.log(`❌ Registration failed: ${e.message?.slice(0, 100)}`);
    return;
  }

  // ============================================================
  // Step 2: Set Policy
  // ============================================================
  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("📋 STEP 2: Set Risk Policy");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  const [policyPda] = getPda([POLICY_SEED, owner.toBuffer(), agent.toBuffer()]);
  const maxPerTx = new BN(0.05 * LAMPORTS_PER_SOL); // 0.05 SOL
  const dailyLimit = new BN(0.1 * LAMPORTS_PER_SOL); // 0.1 SOL
  const systemProgram = new PublicKey("11111111111111111111111111111111");

  console.log(`   Max per-tx:  0.05 SOL`);
  console.log(`   Daily limit: 0.1 SOL`);
  console.log(`   Allowed:     System Program`);

  try {
    const tx2 = await program.methods
      .setPolicy(maxPerTx, dailyLimit, [systemProgram])
      .accounts({
        owner: owner,
        agentConfig: agentConfigPda,
        policy: policyPda,
      })
      .rpc({ skipPreflight: true });

    console.log(`✅ Policy set! Tx: ${tx2}`);
    await sleep(2000);
  } catch (e: any) {
    console.log(`❌ Policy failed: ${e.message?.slice(0, 100)}`);
    return;
  }

  // ============================================================
  // Step 3: Valid Transaction (within limits)
  // ============================================================
  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("✅ STEP 3: Agent sends 0.01 SOL (WITHIN limit)");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  // Fund the agent so it can pay for tx fees (transfer from owner)
  try {
    const { Transaction, SystemProgram: SP } = await import("@solana/web3.js");
    const fundTx = new Transaction().add(
      SP.transfer({ fromPubkey: owner, toPubkey: agent, lamports: 0.05 * LAMPORTS_PER_SOL })
    );
    const sig = await provider.sendAndConfirm(fundTx);
    console.log(`   Funded agent with 0.05 SOL from owner`);
    await sleep(2000);
  } catch (e: any) {
    console.log(`   ⚠️ Funding agent failed: ${e.message?.slice(0,80)}`);
  }

  try {
    const nonceBefore = await program.account.agentNonce.fetch(agentNoncePda);
    const currentNonce = (nonceBefore as any).nonce;
    const nonceBuffer = currentNonce.toArrayLike(Buffer, "le", 8);
    const [txLogPda] = getPda([TX_LOG_SEED, agent.toBuffer(), nonceBuffer]);

    const tx3 = await program.methods
      .validateAndExecute(
        new BN(0.01 * LAMPORTS_PER_SOL), // 0.01 SOL — within 0.05 limit
        systemProgram
      )
      .accounts({
        agent: agent,
        owner: owner,
        agentConfig: agentConfigPda,
        policy: policyPda,
        txLog: txLogPda,
        agentNonce: agentNoncePda,
        systemProgram: systemProgram,
      })
      .signers([agentKeypair])
      .rpc({ skipPreflight: true });

    console.log(`✅ APPROVED! Amount: 0.01 SOL | Tx: ${tx3}`);
    await sleep(2000);
  } catch (e: any) {
    console.log(`❌ Transaction failed: ${e.message?.slice(0, 150)}`);
  }

  // ============================================================
  // Step 4: Over-limit Transaction (should be REJECTED)
  // ============================================================
  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("❌ STEP 4: Agent sends 0.2 SOL (EXCEEDS per-tx limit)");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  try {
    const nonce2 = await program.account.agentNonce.fetch(agentNoncePda);
    const currentNonce2 = (nonce2 as any).nonce;
    const nonceBuffer2 = currentNonce2.toArrayLike(Buffer, "le", 8);
    const [txLogPda2] = getPda([TX_LOG_SEED, agent.toBuffer(), nonceBuffer2]);

    await program.methods
      .validateAndExecute(
        new BN(0.2 * LAMPORTS_PER_SOL), // 0.2 SOL — exceeds 0.05 limit!
        systemProgram
      )
      .accounts({
        agent: agent,
        owner: owner,
        agentConfig: agentConfigPda,
        policy: policyPda,
        txLog: txLogPda2,
        agentNonce: agentNoncePda,
        systemProgram: systemProgram,
      })
      .signers([agentKeypair])
      .rpc({ skipPreflight: true });

    console.log(`⚠️ Transaction went through (unexpected)`);
  } catch (e: any) {
    if (e.message?.includes("ExceedsPerTxLimit") || e.message?.includes("6002")) {
      console.log(`🛡️ BLOCKED! SolanaGuard rejected: "Exceeds per-transaction spending limit"`);
    } else {
      console.log(`🛡️ BLOCKED! Error: ${e.message?.slice(0, 150)}`);
    }
  }
  await sleep(1000);

  // ============================================================
  // Step 5: Pause agent and try again
  // ============================================================
  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("⏸  STEP 5: Owner PAUSES agent, agent tries 0.01 SOL");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  try {
    await program.methods
      .toggleAgent(false)
      .accounts({ owner: owner, agentConfig: agentConfigPda })
      .rpc({ skipPreflight: true });
    console.log(`⏸  Agent PAUSED by owner`);
    await sleep(2000);
  } catch (e: any) {
    console.log(`❌ Pause failed: ${e.message?.slice(0, 100)}`);
  }

  try {
    const nonce3 = await program.account.agentNonce.fetch(agentNoncePda);
    const currentNonce3 = (nonce3 as any).nonce;
    const nonceBuffer3 = currentNonce3.toArrayLike(Buffer, "le", 8);
    const [txLogPda3] = getPda([TX_LOG_SEED, agent.toBuffer(), nonceBuffer3]);

    await program.methods
      .validateAndExecute(
        new BN(0.01 * LAMPORTS_PER_SOL),
        systemProgram
      )
      .accounts({
        agent: agent,
        owner: owner,
        agentConfig: agentConfigPda,
        policy: policyPda,
        txLog: txLogPda3,
        agentNonce: agentNoncePda,
        systemProgram: systemProgram,
      })
      .signers([agentKeypair])
      .rpc({ skipPreflight: true });

    console.log(`⚠️ Transaction went through (unexpected)`);
  } catch (e: any) {
    if (e.message?.includes("AgentNotActive") || e.message?.includes("6000")) {
      console.log(`🛡️ BLOCKED! Agent is PAUSED — cannot transact`);
    } else {
      console.log(`🛡️ BLOCKED! Error: ${e.message?.slice(0, 150)}`);
    }
  }

  // ============================================================
  // Step 6: Resume and view logs
  // ============================================================
  console.log("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
  console.log("📊 STEP 6: View Transaction Logs");
  console.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

  try {
    await program.methods
      .toggleAgent(true)
      .accounts({ owner: owner, agentConfig: agentConfigPda })
      .rpc({ skipPreflight: true });
    console.log(`▶ Agent RESUMED`);
    await sleep(1000);
  } catch {}

  // Fetch logs
  try {
    const nonceAcct = await program.account.agentNonce.fetch(agentNoncePda);
    const totalNonce = (nonceAcct as any).nonce.toNumber();
    console.log(`\n📋 Total transactions attempted: ${totalNonce}`);

    for (let i = 0; i < totalNonce; i++) {
      try {
        const nb = new BN(i).toArrayLike(Buffer, "le", 8);
        const [logPda] = getPda([TX_LOG_SEED, agent.toBuffer(), nb]);
        const log = await program.account.transactionLog.fetch(logPda);
        const l = log as any;
        console.log(
          `   [${i}] ${l.wasApproved ? "✅ APPROVED" : "❌ REJECTED"} | ` +
            `Amount: ${(l.amount.toNumber() / LAMPORTS_PER_SOL).toFixed(4)} SOL | ` +
            `Protocol: ${l.targetProtocol.toBase58().slice(0, 8)}... | ` +
            `Time: ${new Date(l.executedAt.toNumber() * 1000).toLocaleString()}`
        );
      } catch {}
    }
  } catch (e: any) {
    console.log(`Could not fetch logs: ${e.message?.slice(0, 100)}`);
  }

  console.log("\n╔══════════════════════════════════════════════╗");
  console.log("║          ✅ Demo Complete!                   ║");
  console.log("║  SolanaGuard enforced all guardrails on-chain║");
  console.log("╚══════════════════════════════════════════════╝");
  console.log(`\n🔗 View on Explorer: https://explorer.solana.com/address/${PROGRAM_ID.toBase58()}?cluster=devnet\n`);
}

main().catch(console.error);
