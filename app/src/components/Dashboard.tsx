import React, { useState, useEffect, useCallback } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { Program, AnchorProvider, BN } from '@coral-xyz/anchor';
import idl from '../../../target/idl/solana_guard.json';

const PROGRAM_ID = new PublicKey('FRuK1VzhqjybBMhp8UGVipJ9jkyuT9Dy7YJHAREwSApw');
const AGENT_SEED = Buffer.from('agent');
const POLICY_SEED = Buffer.from('policy');
const NONCE_SEED = Buffer.from('nonce');
const TX_LOG_SEED = Buffer.from('tx_log');

function getPda(seeds: Buffer[], programId: PublicKey) {
  return PublicKey.findProgramAddressSync(seeds, programId);
}

interface Toast { message: string; type: 'success' | 'error' | 'info'; }

const Dashboard: React.FC = () => {
  const { connection } = useConnection();
  const wallet = useWallet();
  const [tab, setTab] = useState<'agents' | 'register' | 'policy' | 'logs' | 'demo'>('agents');
  const [toast, setToast] = useState<Toast | null>(null);
  const [loading, setLoading] = useState(false);
  const [agents, setAgents] = useState<any[]>([]);
  const [logs, setLogs] = useState<any[]>([]);
  const [selectedAgent, setSelectedAgent] = useState<string>('');

  // Form states
  const [agentKey, setAgentKey] = useState('');
  const [maxPerTx, setMaxPerTx] = useState('0.1');
  const [dailyLimit, setDailyLimit] = useState('1');
  const [protocols, setProtocols] = useState('');

  const showToast = (message: string, type: Toast['type']) => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 4000);
  };

  const getProgram = useCallback(() => {
    if (!wallet.publicKey) return null;
    const provider = new AnchorProvider(connection, wallet as any, {
      commitment: 'confirmed',
      preflightCommitment: 'confirmed',
      skipPreflight: true,
    });
    return new Program(idl as any, provider);
  }, [connection, wallet]);

  const fetchAgents = useCallback(async () => {
    if (!wallet.publicKey) return;
    const program = getProgram();
    if (!program) return;
    try {
      const accts = await program.account.agentConfig.all([
        { memcmp: { offset: 8, bytes: wallet.publicKey.toBase58() } }
      ]);
      setAgents(accts.map(a => ({ ...a.account, pda: a.publicKey })));
    } catch (e) { console.error(e); }
  }, [wallet.publicKey, getProgram]);

  useEffect(() => { if (wallet.publicKey) fetchAgents(); }, [wallet.publicKey, fetchAgents]);

  const registerAgent = async () => {
    if (!wallet.publicKey) return;
    const program = getProgram();
    if (!program) return;
    setLoading(true);
    try {
      let agentPk: PublicKey;
      if (agentKey.trim()) {
        agentPk = new PublicKey(agentKey.trim());
      } else {
        const kp = Keypair.generate();
        agentPk = kp.publicKey;
        setAgentKey(agentPk.toBase58());
        showToast(`Generated agent key: ${agentPk.toBase58().slice(0,8)}...`, 'info');
      }
      const [agentConfigPda] = getPda([AGENT_SEED, wallet.publicKey.toBuffer(), agentPk.toBuffer()], PROGRAM_ID);
      const [agentNoncePda] = getPda([NONCE_SEED, agentPk.toBuffer()], PROGRAM_ID);

      await program.methods.registerAgent().accounts({
        owner: wallet.publicKey, agent: agentPk,
        agentConfig: agentConfigPda, agentNonce: agentNoncePda,
      }).rpc({ skipPreflight: true, commitment: 'confirmed' });

      showToast('Agent registered successfully!', 'success');
      await fetchAgents();
      setTab('agents');
    } catch (e: any) {
      showToast(e.message?.slice(0, 100) || 'Registration failed', 'error');
    }
    setLoading(false);
  };

  const setPolicy = async () => {
    if (!wallet.publicKey || !selectedAgent) return;
    const program = getProgram();
    if (!program) return;
    setLoading(true);
    try {
      const agentPk = new PublicKey(selectedAgent);
      const [agentConfigPda] = getPda([AGENT_SEED, wallet.publicKey.toBuffer(), agentPk.toBuffer()], PROGRAM_ID);
      const [policyPda] = getPda([POLICY_SEED, wallet.publicKey.toBuffer(), agentPk.toBuffer()], PROGRAM_ID);
      const maxLamports = new BN(parseFloat(maxPerTx) * LAMPORTS_PER_SOL);
      const dailyLamports = new BN(parseFloat(dailyLimit) * LAMPORTS_PER_SOL);
      const protoList = protocols.split(',').map(s => s.trim()).filter(Boolean).map(s => new PublicKey(s));

      await program.methods.setPolicy(maxLamports, dailyLamports, protoList).accounts({
        owner: wallet.publicKey, agentConfig: agentConfigPda, policy: policyPda,
      }).rpc({ skipPreflight: true, commitment: 'confirmed' });

      showToast('Policy set successfully!', 'success');
    } catch (e: any) {
      showToast(e.message?.slice(0, 100) || 'Set policy failed', 'error');
    }
    setLoading(false);
  };

  const toggleAgent = async (agentPk: PublicKey, active: boolean) => {
    if (!wallet.publicKey) return;
    const program = getProgram();
    if (!program) return;
    setLoading(true);
    try {
      const [agentConfigPda] = getPda([AGENT_SEED, wallet.publicKey.toBuffer(), agentPk.toBuffer()], PROGRAM_ID);
      await program.methods.toggleAgent(!active).accounts({
        owner: wallet.publicKey, agentConfig: agentConfigPda,
      }).rpc({ skipPreflight: true, commitment: 'confirmed' });
      showToast(active ? 'Agent PAUSED' : 'Agent RESUMED', active ? 'error' : 'success');
      await fetchAgents();
    } catch (e: any) {
      showToast(e.message?.slice(0, 100) || 'Toggle failed', 'error');
    }
    setLoading(false);
  };

  const fetchLogs = async (agentPk: PublicKey) => {
    const program = getProgram();
    if (!program) return;
    try {
      const [noncePda] = getPda([NONCE_SEED, agentPk.toBuffer()], PROGRAM_ID);
      const nonceAcct = await program.account.agentNonce.fetch(noncePda);
      const nonce = (nonceAcct as any).nonce.toNumber();
      const fetched: any[] = [];
      for (let i = 0; i < nonce && i < 20; i++) {
        try {
          const nb = new BN(i).toArrayLike(Buffer, 'le', 8);
          const [logPda] = getPda([TX_LOG_SEED, agentPk.toBuffer(), nb], PROGRAM_ID);
          const log = await program.account.transactionLog.fetch(logPda);
          fetched.push(log);
        } catch {}
      }
      setLogs(fetched);
    } catch { setLogs([]); }
  };

  // ======================== RENDER ========================

  if (!wallet.publicKey) {
    return (
      <div className="app-container">
        <div className="connect-screen">
          <h1>Solana<span>Guard</span></h1>
          <p>On-chain risk enforcement layer for AI agents. Set spending limits, enforce protocol allowlists, and maintain a kill switch — all on Solana.</p>
          <div className="connect-features">
            <div className="connect-feature">
              <div className="icon">🛡️</div>
              <h3>Spending Limits</h3>
              <p>Per-tx and daily caps enforced on-chain</p>
            </div>
            <div className="connect-feature">
              <div className="icon">🔒</div>
              <h3>Protocol Allowlist</h3>
              <p>Restrict which programs agents can call</p>
            </div>
            <div className="connect-feature">
              <div className="icon">🚨</div>
              <h3>Kill Switch</h3>
              <p>Instantly pause any agent with one click</p>
            </div>
          </div>
          <WalletMultiButton />
        </div>
      </div>
    );
  }

  return (
    <div className="app-container">
      {/* Navbar */}
      <nav className="navbar">
        <div className="navbar-brand">
          <div className="navbar-logo">S</div>
          <div>
            <div className="navbar-title">SolanaGuard</div>
            <div className="navbar-subtitle">AI Agent Guardrails</div>
          </div>
        </div>
        <div className="navbar-status">Devnet</div>
        <div className="wallet-section">
          <WalletMultiButton />
        </div>
      </nav>

      {/* Tabs */}
      <div className="tabs">
        {(['agents', 'register', 'policy', 'logs', 'demo'] as const).map(t => (
          <button key={t} className={`tab ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>
            {t === 'agents' ? '🤖 My Agents' : t === 'register' ? '➕ Register' : t === 'policy' ? '📋 Set Policy' : t === 'logs' ? '📊 Tx Logs' : '🎯 Demo'}
          </button>
        ))}
      </div>

      {/* Content */}
      {tab === 'agents' && (
        <div className="dashboard-grid">
          {agents.length === 0 ? (
            <div className="card full-width">
              <div className="empty-state">
                <div className="icon">🤖</div>
                <h3>No Agents Registered</h3>
                <p>Register your first AI agent to get started with on-chain guardrails.</p>
                <button className="btn btn-primary" style={{marginTop: 20}} onClick={() => setTab('register')}>Register Agent</button>
              </div>
            </div>
          ) : agents.map((a, i) => (
            <div key={i} className="card">
              <div className="card-header">
                <div className="card-title"><span className="icon">🤖</span> Agent #{i + 1}</div>
                <span className={`card-badge ${a.isActive ? 'badge-active' : 'badge-paused'}`}>
                  {a.isActive ? 'Active' : 'Paused'}
                </span>
              </div>
              <div className="form-group">
                <div className="form-label">Agent Public Key</div>
                <div className="form-input" style={{fontSize: 11, wordBreak: 'break-all', cursor: 'default'}}>{a.agent.toBase58()}</div>
              </div>
              <div className="form-group">
                <div className="form-label">Registered</div>
                <div style={{fontSize: 13, color: 'var(--sg-text-secondary)'}}>
                  {new Date(a.registeredAt.toNumber() * 1000).toLocaleString()}
                </div>
              </div>
              <div style={{display: 'flex', gap: 12, marginTop: 16}}>
                <button className="btn btn-outline btn-sm" onClick={() => { setSelectedAgent(a.agent.toBase58()); setTab('policy'); }}>
                  Set Policy
                </button>
                <button className="btn btn-outline btn-sm" onClick={() => { fetchLogs(a.agent); setTab('logs'); }}>
                  View Logs
                </button>
                <button
                  className={`btn btn-sm ${a.isActive ? 'btn-danger' : 'btn-success'}`}
                  onClick={() => toggleAgent(a.agent, a.isActive)}
                  disabled={loading}
                >
                  {a.isActive ? '⏸ Pause' : '▶ Resume'}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {tab === 'register' && (
        <div className="dashboard-grid">
          <div className="card">
            <div className="card-header">
              <div className="card-title"><span className="icon">➕</span> Register New Agent</div>
            </div>
            <div className="form-group">
              <label className="form-label">Agent Public Key (optional)</label>
              <input className="form-input" placeholder="Leave empty to auto-generate a keypair" value={agentKey} onChange={e => setAgentKey(e.target.value)} />
              <div className="form-hint">Paste an existing public key or leave blank to generate one</div>
            </div>
            <button className="btn btn-primary btn-full" onClick={registerAgent} disabled={loading}>
              {loading ? 'Registering...' : '🛡️ Register Agent'}
            </button>
          </div>
          <div className="card">
            <div className="card-header">
              <div className="card-title"><span className="icon">ℹ️</span> How It Works</div>
            </div>
            <div style={{fontSize: 14, color: 'var(--sg-text-secondary)', lineHeight: 1.8}}>
              <p><strong>1.</strong> Register an AI agent under your wallet</p>
              <p><strong>2.</strong> Set a spending policy (per-tx limit, daily limit)</p>
              <p><strong>3.</strong> The agent must call <code>validate_and_execute</code> for every transaction</p>
              <p><strong>4.</strong> SolanaGuard enforces limits on-chain — no bypass possible</p>
              <p><strong>5.</strong> Use the kill switch to instantly pause any agent</p>
            </div>
          </div>
        </div>
      )}

      {tab === 'policy' && (
        <div className="dashboard-grid">
          <div className="card">
            <div className="card-header">
              <div className="card-title"><span className="icon">📋</span> Set Risk Policy</div>
            </div>
            <div className="form-group">
              <label className="form-label">Agent</label>
              <select className="form-input" value={selectedAgent} onChange={e => setSelectedAgent(e.target.value)}>
                <option value="">Select an agent...</option>
                {agents.map((a, i) => (
                  <option key={i} value={a.agent.toBase58()}>{`Agent #${i+1} — ${a.agent.toBase58().slice(0,12)}...`}</option>
                ))}
              </select>
            </div>
            <div className="form-group">
              <label className="form-label">Max Spend Per Transaction (SOL)</label>
              <input className="form-input" type="number" step="0.01" value={maxPerTx} onChange={e => setMaxPerTx(e.target.value)} />
            </div>
            <div className="form-group">
              <label className="form-label">Daily Spending Limit (SOL)</label>
              <input className="form-input" type="number" step="0.1" value={dailyLimit} onChange={e => setDailyLimit(e.target.value)} />
            </div>
            <div className="form-group">
              <label className="form-label">Allowed Protocols (comma-separated pubkeys)</label>
              <input className="form-input" placeholder="Program IDs the agent can interact with" value={protocols} onChange={e => setProtocols(e.target.value)} />
              <div className="form-hint">e.g., JUP6...abc, 11111...111 (max 10)</div>
            </div>
            <button className="btn btn-primary btn-full" onClick={setPolicy} disabled={loading || !selectedAgent}>
              {loading ? 'Setting Policy...' : '🔒 Enforce Policy'}
            </button>
          </div>
          {/* Kill Switch */}
          <div className="card">
            <div className="card-header">
              <div className="card-title"><span className="icon">🚨</span> Emergency Kill Switch</div>
            </div>
            {selectedAgent ? (() => {
              const agent = agents.find(a => a.agent.toBase58() === selectedAgent);
              if (!agent) return <p style={{color: 'var(--sg-text-muted)'}}>Select an agent</p>;
              return (
                <div className="kill-switch">
                  <button
                    className={`kill-switch-btn ${agent.isActive ? '' : 'safe'}`}
                    onClick={() => toggleAgent(agent.agent, agent.isActive)}
                    disabled={loading}
                  >
                    <span style={{fontSize: 28}}>{agent.isActive ? '⏸' : '▶'}</span>
                    {agent.isActive ? 'PAUSE' : 'RESUME'}
                  </button>
                  <div className="kill-switch-label">
                    Agent is currently <strong>{agent.isActive ? 'ACTIVE' : 'PAUSED'}</strong>
                  </div>
                </div>
              );
            })() : <div className="empty-state"><p>Select an agent to use the kill switch</p></div>}
          </div>
        </div>
      )}

      {tab === 'logs' && (
        <div className="card full-width">
          <div className="card-header">
            <div className="card-title"><span className="icon">📊</span> Transaction Logs</div>
            <div style={{display: 'flex', gap: 8}}>
              {agents.map((a, i) => (
                <button key={i} className="btn btn-outline btn-sm" onClick={() => fetchLogs(a.agent)}>Agent #{i+1}</button>
              ))}
            </div>
          </div>
          {logs.length === 0 ? (
            <div className="empty-state">
              <div className="icon">📊</div>
              <h3>No Transaction Logs</h3>
              <p>Logs will appear here when agents attempt transactions through SolanaGuard.</p>
            </div>
          ) : (
            <div className="table-container">
              <table className="table">
                <thead>
                  <tr>
                    <th>Status</th>
                    <th>Amount (SOL)</th>
                    <th>Protocol</th>
                    <th>Time</th>
                    <th>Nonce</th>
                  </tr>
                </thead>
                <tbody>
                  {logs.map((l, i) => (
                    <tr key={i}>
                      <td>
                        <span className={`status-dot ${l.wasApproved ? 'approved' : 'rejected'}`} />
                        {l.wasApproved ? 'Approved' : 'Rejected'}
                      </td>
                      <td className="mono">{(l.amount.toNumber() / LAMPORTS_PER_SOL).toFixed(4)}</td>
                      <td className="mono">{l.targetProtocol.toBase58().slice(0,8)}...</td>
                      <td>{new Date(l.executedAt.toNumber() * 1000).toLocaleString()}</td>
                      <td className="mono">{l.nonce.toNumber()}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {tab === 'demo' && (
        <div className="dashboard-grid">
          <div className="card full-width">
            <div className="card-header">
              <div className="card-title"><span className="icon">🎯</span> Live Demo — Test Guardrails</div>
            </div>
            <div style={{textAlign: 'center', padding: 40, color: 'var(--sg-text-secondary)'}}>
              <p style={{fontSize: 16, marginBottom: 16}}>The demo lets you simulate an AI agent attempting transactions.</p>
              <p style={{fontSize: 14}}>1. Register an agent → 2. Set a policy → 3. Try exceeding limits</p>
              <p style={{fontSize: 14, marginTop: 8}}>SolanaGuard will block overspend attempts on-chain. Check the <strong>Tx Logs</strong> tab to see approved/rejected entries.</p>
              <div style={{marginTop: 32}}>
                <a href="https://explorer.solana.com/address/FRuK1VzhqjybBMhp8UGVipJ9jkyuT9Dy7YJHAREwSApw?cluster=devnet" target="_blank" className="btn btn-primary">
                  View on Solana Explorer ↗
                </a>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Toast */}
      {toast && <div className={`toast toast-${toast.type}`}>{toast.message}</div>}
    </div>
  );
};

export default Dashboard;
