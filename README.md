# SolanaGuard 🛡️

**On-chain risk enforcement layer that prevents AI agents from exceeding user-defined spending limits on Solana.**

[![Built with Anchor](https://img.shields.io/badge/Built%20with-Anchor-blue)](https://www.anchor-lang.com/)
[![Solana](https://img.shields.io/badge/Solana-Devnet-green)](https://solana.com/)
[![Hackathon](https://img.shields.io/badge/Colosseum-Frontier%202026-purple)](https://www.colosseum.org/)

## The Problem

AI agents are getting access to user wallets, but there's no on-chain mechanism to enforce spending limits. If an AI hallucinates, gets prompt-injected, or simply miscalculates — your funds are at risk. Current solutions rely on backend checks that can be bypassed.

## The Solution

SolanaGuard is a Solana smart contract (built with Anchor/Rust) that acts as a **deterministic firewall** between AI agents and your funds. Users register an agent, fund a program-controlled vault, set policies (max spend per tx, daily limit, daily tx cap, allowed protocols, and slippage limit), and the contract enforces these rules while executing each guarded transfer from the vault itself.

**No agent, no backend, and no developer can override them — enforcement happens at the blockchain level.**

## Features

- 🔐 **Agent Registration** — Bind AI agents to your wallet with PDA-based identity
- 🏦 **Program Vault** — Funds live in a PDA-controlled vault instead of an agent wallet
- 📊 **Per-Transaction Limits** — Cap the maximum any single transaction can spend
- 📅 **Daily Spending Limits** — Automatic 24-hour rolling reset
- 🔢 **Daily Transaction Caps** — Limit how many approved actions an agent can take per day
- ✅ **Protocol Allowlisting** — Whitelist only the programs your agent can interact with
- 📉 **Slippage Limits** — Reject actions when reported slippage exceeds policy
- 🚨 **Emergency Kill Switch** — Instantly pause any agent with one transaction
- 📝 **On-chain Audit Trail** — Every attempt is recorded as a PDA log; rejected attempts also emit events
- 🔄 **Partial Policy Updates** — Modify individual policy fields without resetting everything

Policy denials are recorded as successful on-chain audit entries with `was_approved = false` and a rejection reason code. This preserves a durable audit trail without moving funds from the guarded vault.

## Architecture

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   AI Agent   │────▶│   SolanaGuard    │────▶│  Target Account │
│  (e.g. GPT)  │     │  Smart Contract  │     │  (e.g. Jupiter) │
└─────────────┘     │                  │     └─────────────────┘
                    │  ✓ Agent active? │
                    │  ✓ Under tx max? │
                    │  ✓ Under daily?  │
                    │  ✓ Under tx/day? │
                    │  ✓ Protocol OK?  │
                    │  ✓ Slippage OK?  │
                    │                  │
                    │  ❌ REJECT or    │
                    │  ✅ EXECUTE      │
                    └──────────────────┘
```

## Program Instructions

| Instruction | Who Calls | Description |
|---|---|---|
| `register_agent` | Owner | Register an AI agent under your ownership |
| `fund_vault` | Owner | Deposit SOL into the guarded vault |
| `set_policy` | Owner | Define spending, tx-count, protocol, and slippage limits |
| `validate_and_execute` | Agent | Enforce policy and execute the guarded transfer from the vault |
| `toggle_agent` | Owner | Pause/unpause an agent (kill switch) |
| `update_policy` | Owner | Partially update policy parameters |
| `withdraw_vault` | Owner | Withdraw unused SOL back from the vault |

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solanalabs.com/cli/install)
- [Anchor](https://www.anchor-lang.com/docs/installation)
- [Node.js](https://nodejs.org/) v18+

### Build & Test

```bash
# Clone
git clone https://github.com/YOUR_USERNAME/solana-guard.git
cd solana-guard

# Build
anchor build

# Test
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

## Tech Stack

- **Smart Contract**: Rust + Anchor Framework
- **Client SDK**: TypeScript (coming soon)
- **Dashboard**: React (coming soon)
- **Network**: Solana Devnet → Mainnet

## Roadmap

- [x] Core Anchor program (register, policy, validate, toggle, update)
- [ ] TypeScript SDK for agent integration
- [ ] Agent demo with Jupiter swap guardrails
- [ ] React dashboard for policy management
- [ ] Demo video and Colosseum submission

## Hackathon

Built for the **Colosseum Frontier Hackathon** (April–May 2026).

**Tracks**: Security • Wallet • Infrastructure

## License

MIT
