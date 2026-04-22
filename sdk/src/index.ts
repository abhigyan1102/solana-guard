import { Program, AnchorProvider, BN, web3 } from "@coral-xyz/anchor";
import { PublicKey, Connection, Keypair, SystemProgram } from "@solana/web3.js";
import idl from "../../target/idl/solana_guard.json";

// Program ID — deployed on devnet
export const PROGRAM_ID = new PublicKey(
  "FRuK1VzhqjybBMhp8UGVipJ9jkyuT9Dy7YJHAREwSApw"
);

// PDA seeds
const AGENT_SEED = Buffer.from("agent");
const POLICY_SEED = Buffer.from("policy");
const TX_LOG_SEED = Buffer.from("tx_log");
const NONCE_SEED = Buffer.from("nonce");

// ============================================================
// PDA Derivation Helpers
// ============================================================

export function getAgentConfigPda(
  owner: PublicKey,
  agent: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [AGENT_SEED, owner.toBuffer(), agent.toBuffer()],
    PROGRAM_ID
  );
}

export function getPolicyPda(
  owner: PublicKey,
  agent: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [POLICY_SEED, owner.toBuffer(), agent.toBuffer()],
    PROGRAM_ID
  );
}

export function getAgentNoncePda(agent: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [NONCE_SEED, agent.toBuffer()],
    PROGRAM_ID
  );
}

export function getTxLogPda(
  agent: PublicKey,
  nonce: number | BN
): [PublicKey, number] {
  const nonceBuffer = new BN(nonce).toArrayLike(Buffer, "le", 8);
  return PublicKey.findProgramAddressSync(
    [TX_LOG_SEED, agent.toBuffer(), nonceBuffer],
    PROGRAM_ID
  );
}

// ============================================================
// Account Types
// ============================================================

export interface AgentConfig {
  owner: PublicKey;
  agent: PublicKey;
  isActive: boolean;
  registeredAt: BN;
  bump: number;
}

export interface Policy {
  owner: PublicKey;
  agent: PublicKey;
  maxSpendPerTx: BN;
  dailyLimit: BN;
  dailySpent: BN;
  dayStart: BN;
  isActive: boolean;
  allowedProtocols: PublicKey[];
  bump: number;
}

export interface TransactionLog {
  agent: PublicKey;
  owner: PublicKey;
  amount: BN;
  targetProtocol: PublicKey;
  executedAt: BN;
  wasApproved: boolean;
  nonce: BN;
  bump: number;
}

export interface AgentNonce {
  agent: PublicKey;
  nonce: BN;
  bump: number;
}

// ============================================================
// SolanaGuard Client
// ============================================================

export class SolanaGuardClient {
  public program: Program;
  public connection: Connection;
  public provider: AnchorProvider;

  constructor(provider: AnchorProvider) {
    this.provider = provider;
    this.connection = provider.connection;
    this.program = new Program(idl as any, provider);
  }

  // ----------------------------------------------------------
  // Static factory
  // ----------------------------------------------------------
  static fromConnection(
    connection: Connection,
    wallet: any
  ): SolanaGuardClient {
    const provider = new AnchorProvider(connection, wallet, {
      commitment: "confirmed",
    });
    return new SolanaGuardClient(provider);
  }

  // ----------------------------------------------------------
  // Instructions
  // ----------------------------------------------------------

  /**
   * Register a new AI agent under the caller's ownership.
   * @param agentPubkey - The agent's public key
   * @returns Transaction signature
   */
  async registerAgent(agentPubkey: PublicKey): Promise<string> {
    const owner = this.provider.wallet.publicKey;
    const [agentConfigPda] = getAgentConfigPda(owner, agentPubkey);
    const [agentNoncePda] = getAgentNoncePda(agentPubkey);

    const tx = await this.program.methods
      .registerAgent()
      .accounts({
        owner: owner,
        agent: agentPubkey,
        agentConfig: agentConfigPda,
        agentNonce: agentNoncePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return tx;
  }

  /**
   * Set the risk policy for a registered agent.
   * @param agentPubkey - The agent's public key
   * @param maxSpendPerTx - Maximum lamports per transaction
   * @param dailyLimit - Maximum lamports per day
   * @param allowedProtocols - List of allowed program IDs
   * @returns Transaction signature
   */
  async setPolicy(
    agentPubkey: PublicKey,
    maxSpendPerTx: number | BN,
    dailyLimit: number | BN,
    allowedProtocols: PublicKey[]
  ): Promise<string> {
    const owner = this.provider.wallet.publicKey;
    const [agentConfigPda] = getAgentConfigPda(owner, agentPubkey);
    const [policyPda] = getPolicyPda(owner, agentPubkey);

    const tx = await this.program.methods
      .setPolicy(new BN(maxSpendPerTx), new BN(dailyLimit), allowedProtocols)
      .accounts({
        owner: owner,
        agentConfig: agentConfigPda,
        policy: policyPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    return tx;
  }

  /**
   * Validate and execute a transaction through the guardrail.
   * Called by the AGENT (agent must be the signer).
   * @param agentKeypair - The agent's keypair (signer)
   * @param ownerPubkey - The owner's public key
   * @param amount - Amount in lamports
   * @param targetProtocol - The program the agent wants to interact with
   * @returns Transaction signature
   */
  async validateAndExecute(
    agentKeypair: Keypair,
    ownerPubkey: PublicKey,
    amount: number | BN,
    targetProtocol: PublicKey
  ): Promise<string> {
    const agentPubkey = agentKeypair.publicKey;
    const [agentConfigPda] = getAgentConfigPda(ownerPubkey, agentPubkey);
    const [policyPda] = getPolicyPda(ownerPubkey, agentPubkey);
    const [agentNoncePda] = getAgentNoncePda(agentPubkey);

    // Fetch current nonce to derive tx_log PDA
    const nonceAccount = await this.fetchAgentNonce(agentPubkey);
    const currentNonce = nonceAccount ? nonceAccount.nonce : new BN(0);
    const [txLogPda] = getTxLogPda(agentPubkey, currentNonce);

    const tx = await this.program.methods
      .validateAndExecute(new BN(amount), targetProtocol)
      .accounts({
        agent: agentPubkey,
        owner: ownerPubkey,
        agentConfig: agentConfigPda,
        policy: policyPda,
        txLog: txLogPda,
        agentNonce: agentNoncePda,
        systemProgram: SystemProgram.programId,
      })
      .signers([agentKeypair])
      .rpc();

    return tx;
  }

  /**
   * Toggle agent active status (emergency kill switch).
   * @param agentPubkey - The agent's public key
   * @param isActive - true = active, false = paused
   * @returns Transaction signature
   */
  async toggleAgent(
    agentPubkey: PublicKey,
    isActive: boolean
  ): Promise<string> {
    const owner = this.provider.wallet.publicKey;
    const [agentConfigPda] = getAgentConfigPda(owner, agentPubkey);

    const tx = await this.program.methods
      .toggleAgent(isActive)
      .accounts({
        owner: owner,
        agentConfig: agentConfigPda,
      })
      .rpc();

    return tx;
  }

  /**
   * Update an existing policy's parameters (partial update).
   * @param agentPubkey - The agent's public key
   * @param opts - Fields to update (null/undefined = no change)
   * @returns Transaction signature
   */
  async updatePolicy(
    agentPubkey: PublicKey,
    opts: {
      maxSpendPerTx?: number | BN | null;
      dailyLimit?: number | BN | null;
      allowedProtocols?: PublicKey[] | null;
      isActive?: boolean | null;
    }
  ): Promise<string> {
    const owner = this.provider.wallet.publicKey;
    const [agentConfigPda] = getAgentConfigPda(owner, agentPubkey);
    const [policyPda] = getPolicyPda(owner, agentPubkey);

    const tx = await this.program.methods
      .updatePolicy(
        opts.maxSpendPerTx ? new BN(opts.maxSpendPerTx) : null,
        opts.dailyLimit ? new BN(opts.dailyLimit) : null,
        opts.allowedProtocols ?? null,
        opts.isActive ?? null
      )
      .accounts({
        owner: owner,
        agentConfig: agentConfigPda,
        policy: policyPda,
      })
      .rpc();

    return tx;
  }

  // ----------------------------------------------------------
  // Fetch Account Data
  // ----------------------------------------------------------

  async fetchAgentConfig(
    owner: PublicKey,
    agent: PublicKey
  ): Promise<AgentConfig | null> {
    const [pda] = getAgentConfigPda(owner, agent);
    try {
      const account = await this.program.account.agentConfig.fetch(pda);
      return account as unknown as AgentConfig;
    } catch {
      return null;
    }
  }

  async fetchPolicy(
    owner: PublicKey,
    agent: PublicKey
  ): Promise<Policy | null> {
    const [pda] = getPolicyPda(owner, agent);
    try {
      const account = await this.program.account.policy.fetch(pda);
      return account as unknown as Policy;
    } catch {
      return null;
    }
  }

  async fetchAgentNonce(agent: PublicKey): Promise<AgentNonce | null> {
    const [pda] = getAgentNoncePda(agent);
    try {
      const account = await this.program.account.agentNonce.fetch(pda);
      return account as unknown as AgentNonce;
    } catch {
      return null;
    }
  }

  async fetchTransactionLog(
    agent: PublicKey,
    nonce: number | BN
  ): Promise<TransactionLog | null> {
    const [pda] = getTxLogPda(agent, nonce);
    try {
      const account = await this.program.account.transactionLog.fetch(pda);
      return account as unknown as TransactionLog;
    } catch {
      return null;
    }
  }

  /**
   * Fetch all transaction logs for a specific agent.
   * Iterates from nonce 0 to current nonce.
   */
  async fetchAllTransactionLogs(
    agent: PublicKey
  ): Promise<TransactionLog[]> {
    const nonceAccount = await this.fetchAgentNonce(agent);
    if (!nonceAccount) return [];

    const logs: TransactionLog[] = [];
    const currentNonce = nonceAccount.nonce.toNumber();

    for (let i = 0; i < currentNonce; i++) {
      const log = await this.fetchTransactionLog(agent, i);
      if (log) logs.push(log);
    }

    return logs;
  }

  /**
   * Fetch all agents registered by a specific owner.
   * Uses getProgramAccounts with memcmp filter on the owner field.
   */
  async fetchAllAgentsByOwner(owner: PublicKey): Promise<AgentConfig[]> {
    const accounts = await this.program.account.agentConfig.all([
      {
        memcmp: {
          offset: 8, // discriminator
          bytes: owner.toBase58(),
        },
      },
    ]);
    return accounts.map((a) => a.account as unknown as AgentConfig);
  }
}

// Re-export everything
export { BN } from "@coral-xyz/anchor";
export { PublicKey, Keypair, Connection } from "@solana/web3.js";
