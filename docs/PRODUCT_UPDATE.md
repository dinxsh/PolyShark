# PolyShark V2 Product Update

> **Permission-Safe Arbitrage Agent for Polymarket**  
> MetaMask x Envio Hackathon Submission

---

## 🎯 TL;DR for Judges

| What Makes PolyShark Special |
|------------------------------|
| 🔐 **ERC-7715 Permission System** — Cryptographically enforced daily USDC limits |
| 📡 **Envio-Powered Data** — Low-latency HyperIndex enables safe automation |
| 🤖 **Zero-Popup Trading** — Autonomous trades after one-time permission grant |
| 🛡️ **Adaptive Safety** — Strategy modes adjust based on remaining allowance |

---

## Overview

PolyShark is an **autonomous trading agent** that detects logical arbitrage opportunities on Polymarket and executes trades automatically within user-defined permission bounds using **ERC-7715 Advanced Permissions**.

### Core Philosophy
> *"If markets contradict themselves, eat the contradiction."*

When prediction market prices violate logical constraints (e.g., YES + NO ≠ 1), PolyShark identifies and executes profitable trades—all without requiring wallet confirmations after the initial permission grant.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       USER                                   │
│              ↓ Grant Permission (once)                       │
├─────────────────────────────────────────────────────────────┤
│            MetaMask Smart Account (ERC-7715)                 │
│                    ↓ Enforced Daily Limit                    │
├─────────────────────────────────────────────────────────────┤
│               PolyShark Agent (Rust)                         │
│    ┌──────────────┬──────────────┬──────────────┐           │
│    │ Constraint   │ Arbitrage    │ Execution    │           │
│    │ Engine       │ Detector     │ Engine       │           │
│    └──────────────┴──────────────┴──────────────┘           │
│                    ↓                ↑                        │
├─────────────────────────────────────────────────────────────┤
│              Polymarket Contracts                            │
│                    ↑                                         │
├─────────────────────────────────────────────────────────────┤
│              Envio HyperIndex                                │
│         (Low-latency market state)                           │
└─────────────────────────────────────────────────────────────┘
```

---

## ERC-7715 Permission System

### Permission Configuration

Users configure their permission through the dashboard:

| Parameter | Options | Default |
|-----------|---------|---------|
| **Token** | USDC, USDT, DAI | USDC |
| **Daily Limit** | 5 - 50 USDC | 10 USDC |
| **Duration** | 7, 30, 90 days | 30 days |

### Permission JSON Object

```json
{
  "erc7715:permission": {
    "type": "spend",
    "token": {
      "symbol": "USDC",
      "address": "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"
    },
    "limit": { "amount": 10.0, "period": "day" },
    "duration": { "days": 30 },
    "scope": { "protocol": "polymarket" },
    "metadata": {
      "title": "PolyShark Trading Permission",
      "description": "PolyShark may automatically trade up to 10 USDC per day."
    }
  }
}
```

### Permission Lifecycle

```
1. SETUP     → Connect MetaMask Smart Account
2. REQUEST   → Generate config, user approves in MetaMask
3. REDEEM    → Agent trades autonomously (zero popups)
4. MONITOR   → Dashboard shows real-time allowance consumption
5. REVOKE    → User clicks revoke, agent stops immediately
```

---

## Adaptive Strategy Modes

The agent adapts its behavior based on remaining daily allowance:

| Mode | Condition | Behavior |
|------|-----------|----------|
| **Conservative** | < 30% remaining | High-edge trades only (≥5% edge) |
| **Normal** | 30-70% remaining | Standard trading (≥2% edge) |
| **Aggressive** | > 70% remaining | Frequent trades (≥1% edge) |

This ensures the agent intelligently budgets within the permission bounds.

---

## Why Envio Matters

Envio HyperIndex is not just a data source—it's what enables **safe** autonomous trading:

| Benefit | Impact |
|---------|--------|
| **~150ms latency** | Near-real-time market state |
| **Reliability** | Decoupled from Polymarket API limits |
| **Replay capability** | Historical data for backtesting |

### Example Query

```graphql
query GetMarketState($conditionId: String!) {
  Market(where: { conditionId: { _eq: $conditionId } }) {
    question
    outcomes
    outcomePrices
    volume
    liquidity
  }
}
```

---

## Safety & Failure Handling

### Agent Status

| Status | Meaning |
|--------|---------|
| 🟢 **RUNNING** | Normal operation |
| 🟡 **SAFE_MODE** | Suspended due to failures |
| 🔴 **PERMISSION_EXPIRED** | Duration ended |
| ⚪ **IDLE** | Not trading |

### Failure Scenarios

| Condition | Response |
|-----------|----------|
| Data delay > 5s | Suspend trading |
| 3+ API failures | Enter safe mode (5 min cooldown) |
| Permission query fails | Assume 0 allowance |

---

## Dashboard Features

The dashboard (`dashboard/index.html`) provides:

- **Permission Configuration** — Adjust limit, duration, token
- **Permission Center** — View status, request more, tighten, revoke
- **Envio Health** — Index delay, connection status
- **Strategy Mode** — Current mode indicator
- **Agent Status** — Real-time operational state
- **Dry-Run Toggle** — Simulate without real transactions
- **JSON Viewer** — See exact permission configuration

---

## Module Structure

```
src/
├── metamask.rs    → ERC-7715 client, StrategyMode, AgentStatus
├── wallet.rs      → Permission-aware adapter
├── market.rs      → Envio-sourced market data
├── constraint.rs  → Logical arbitrage constraints
├── arb.rs         → Arbitrage detection
├── engine.rs      → Main loop with safety handling
├── execution.rs   → Trade execution
└── config.rs      → Configuration system
```

---

## Configuration

```toml
[permission]
daily_limit_usdc = 10.0
duration_days = 30
token = "USDC"

[strategy]
conservative_threshold = 0.30
aggressive_threshold = 0.70
conservative_min_edge = 0.05
normal_min_edge = 0.02
aggressive_min_edge = 0.01

[safety]
max_data_delay_ms = 5000
max_consecutive_failures = 3
safe_mode_cooldown_secs = 300
```

---

## Demo Script (3-4 minutes)

1. **Connect & Configure** (1 min) — Open dashboard, adjust limits, show JSON config
2. **Grant Permission** (30s) — Connect MetaMask, approve permission
3. **Autonomous Trading** (1 min) — Watch trades, allowance consumption, strategy mode
4. **Revoke** (30s) — Click revoke, agent stops immediately

---

## Tech Stack

| Component | Technology |
|-----------|------------|
| Agent | Rust |
| Wallet | MetaMask Smart Account |
| Permissions | ERC-7715 |
| Market Data | Envio HyperIndex |
| Target Chain | Polygon (137) |

---

## Related Resources

- [Delegation Toolkit](https://docs.metamask.io/smart-accounts/delegation-toolkit)
- [Smart Accounts Kit](https://docs.metamask.io/smart-accounts)
- [ERC-7715 Spec](https://eips.ethereum.org/EIPS/eip-7715)
- [create-gator-app](https://github.com/MetaMask/create-gator-app)

---

## License

MIT License — See [LICENSE](../LICENSE) for details.
