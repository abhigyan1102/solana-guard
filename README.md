# SolanaGuard 🛡️

**On-chain risk enforcement layer that prevents AI agents from exceeding user-defined spending limits on Solana.**

[![Built with Anchor](https://img.shields.io/badge/Built%20with-Anchor-blue)](https://www.anchor-lang.com/)
[![Solana](https://img.shields.io/badge/Solana-Devnet-green)](https://solana.com/)
[![Hackathon](https://img.shields.io/badge/Colosseum-Frontier%202026-purple)](https://www.colosseum.org/)

## The Problem

AI agents are getting access to user wallets, but there's no on-chain mechanism to enforce spending limits. If an AI hallucinates, gets prompt-injected, or simply miscalculates — your funds are at risk. Current solutions rely on backend checks that can be bypassed.

## The Solution

SolanaGuard is a Solana smart contract (built with Anchor/Rust) that acts as a **deterministic firewall** between AI agents and your funds. Users register an agent, set policies (max spend per tx, daily limit, allowed protocols), and the contract enforces these rules on every transaction.

**No agent, no backend, and no developer can override them — enforcement happens at the blockchain level.**

## Features

- 🔐 **Agent Registration** — Bind AI agents to your wallet with PDA-based identity
- 📊 **Per-Transaction Limits** — Cap the maximum any single transaction can spend
- 📅 **Daily Spending Limits** — Automatic 24-hour rolling reset
- ✅ **Protocol Allowlisting** — Whitelist only the programs your agent can interact with
- 🚨 **Emergency Kill Switch** — Instantly pause any agent with one transaction
- 📝 **On-chain Audit Trail** — Every transaction logged as a PDA for full transparency
- 🔄 **Partial Policy Updates** — Modify individual policy fields without resetting everything

## Architecture

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   AI Agent   │────▶│   SolanaGuard    │────▶│  Target Protocol│
│  (e.g. GPT)  │     │  Smart Contract  │     │  (e.g. Jupiter) │
└─────────────┘     │                  │     └─────────────────┘
                    │  ✓ Agent active? │
                    │  ✓ Under tx max? │
                    │  ✓ Under daily?  │
                    │  ✓ Protocol OK?  │
                    │                  │
                    │  ❌ REJECT or    │
                    │  ✅ APPROVE      │
                    └──────────────────┘
```

## Program Instructions

| Instruction | Who Calls | Description |
|---|---|---|
| `register_agent` | Owner | Register an AI agent under your ownership |
| `set_policy` | Owner | Define spending limits and allowed protocols |
| `validate_and_execute` | Agent | Check transaction against policy before executing |
| `toggle_agent` | Owner | Pause/unpause an agent (kill switch) |
| `update_policy` | Owner | Partially update policy parameters |

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
