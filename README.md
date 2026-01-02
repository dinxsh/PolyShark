# 🦈 PolyShark

> *"If markets contradict themselves, eat the contradiction."*

**PolyShark** is a **permission-safe arbitrage agent** for Polymarket, built for the MetaMask x Envio Hackathon. It detects logical mispricing between linked prediction markets and executes trades automatically within user-defined limits using **ERC-7715 Advanced Permissions**.

---

## 🎯 TL;DR for Judges

| What Makes PolyShark Special |
|------------------------------|
| 🔐 **ERC-7715 Permission System** — Cryptographically enforced daily USDC limits, not just trust |
| 📡 **Envio-Powered Safety** — Low-latency HyperIndex enables safe automation; agent halts on stale data |
| 🤖 **Zero-Popup Trading** — Once granted, trades execute autonomously without wallet confirmations |
| 🛡️ **Adaptive Strategy** — Trading behavior adjusts based on remaining allowance |

> **[🎬 Demo Video](https://youtube.com/watch?v=YOUR_VIDEO_ID)** | **[🌐 Live Dashboard](https://your-username.github.io/polyshark-dashboard/)**

---

## ⚡ Before vs. After: The Permission Revolution

| | **Traditional Bots** | **PolyShark + ERC-7715** |
|---|----------------------|--------------------------|
| **Wallet Popups** | ❌ Every single trade | ✅ **One popup, then autonomous** |
| **Spend Control** | ❌ Trust-based limits | ✅ **Cryptographically enforced** |
| **User Control** | ❌ Must stop the bot | ✅ **Instant revocation** |
| **Risk Exposure** | ❌ Unlimited until stopped | ✅ **$10/day maximum** |
| **Data Freshness** | ❌ No guarantees | ✅ **Agent halts if data >5s stale** |

> 💡 **Key insight:** This is the last popup you'll see for the next thousand trades.

---

## 🛡️ Why Envio Makes This Safe (Not Reckless)

**Without Envio, autonomous trading is dangerous.** Stale data leads to bad trades.

PolyShark uses **Envio HyperIndex** as its safety gate:

| Protection | How It Works |
|------------|--------------|
| **Latency Monitoring** | Dashboard shows real-time index delay (~150ms typical) |
| **Automatic Halt** | Agent enters safe mode if delay exceeds 5 seconds |
| **Block Height Tracking** | Always know exactly how fresh your data is |
| **Error Counting** | Consecutive failures trigger cooldown |

```toml
# config.toml - Safety thresholds
[safety]
max_data_delay_ms = 5000         # Suspend trading if Envio delay exceeds this
max_consecutive_failures = 3     # Enter safe mode after N API failures
safe_mode_cooldown_secs = 300    # Wait 5 minutes before retrying
```

> ⚠️ **Critical:** PolyShark's safety depends on Envio's reliability. Without Envio + ERC-7715, this kind of bot would be dangerous. With them, it becomes safe.

---

## 🏆 Hackathon Highlights

| Feature | Implementation |
|---------|----------------|
| **Smart Accounts** | MetaMask Smart Account with ERC-7715 |
| **Advanced Permissions** | Daily USDC spend limits (configurable 5-50 USDC/day) |
| **Automation** | Zero-popup trading after permission grant |
| **On-Chain Integration** | Polymarket via Envio HyperIndex |
| **Adaptive Strategy** | Conservative/Normal/Aggressive modes |

> 📘 **Full Architecture:** [docs/metamask/v1.md](./docs/metamask/v1.md)

---

## 🎯 What It Does

1. **Detects** logical arbitrage (when YES + NO ≠ 1)
2. **Validates** against ERC-7715 permission allowance
3. **Executes** trades automatically (zero wallet popups)
4. **Adapts** strategy based on remaining budget
5. **Halts** if Envio data becomes stale

---

## 🧠 Architecture

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
│         (Low-latency market state + safety gate)             │
└─────────────────────────────────────────────────────────────┘
```

### Module Structure

```
src/
├── metamask.rs      → ERC-7715 client, strategy modes
├── wallet.rs        → Permission-aware adapter
├── market.rs        → Envio-sourced market data
├── constraint.rs    → Logical relationships
├── arb.rs           → Arbitrage detection
├── engine.rs        → Main trading loop with safety
└── config.rs        → Configuration system
```

---

## 📊 Permission Specification

PolyShark requests the following ERC-7715 permission:

| Property | Value |
|----------|-------|
| **Type** | Spend permission |
| **Token** | USDC (0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174) |
| **Limit** | 10 USDC per day (configurable) |
| **Duration** | 30 days (configurable) |
| **Scope** | Polymarket trading adapter |

### Permission JSON

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

> *"PolyShark may automatically trade up to 10 USDC per day on your behalf. You can revoke this permission at any time."*

---

## 📡 Envio Integration

### Example Query

```graphql
query GetMarketState($conditionId: String!) {
  Market(where: { conditionId: { _eq: $conditionId } }) {
    question
    outcomes
    outcomePrices
    volume
    liquidity
    lastTradePrice
    updatedAt
  }
}
```

### Dashboard Health Metrics

The dashboard shows live Envio status:
- **Index Delay** — Milliseconds behind chain head
- **Last Indexed Block** — Exact block number
- **API Errors (24h)** — Consecutive failure count
- **Connection Status** — Connected/Delayed/Offline

---

## 🔧 Use PolyShark as a Template

PolyShark is designed as a **reference implementation for ERC-7715-powered agents**, not just a Polymarket bot. The architecture cleanly separates:

- **Permission Layer** (`metamask.rs`, `wallet.rs`) — Handles Smart Account integration and daily limit enforcement via the Delegation Toolkit pattern
- **Data Layer** (`market.rs`, `websocket.rs`) — Consumes Envio HyperIndex; swap for any other indexer
- **Logic Layer** (`constraint.rs`, `arb.rs`) — Domain-specific arbitrage detection; replace with your DEX routing, NFT bidding, or game automation logic

### To Adapt for a Different Protocol

1. Replace `market.rs` with your data source (e.g., Uniswap subgraph)
2. Rewrite `constraint.rs` with your domain constraints
3. Keep the permission layer unchanged — it already handles ERC-7715 lifecycle
4. Update `config.toml` with your parameters

Developers using [`create-gator-app`](https://github.com/MetaMask/create-gator-app) can reference this architecture to build their own permissioned agents.

> See [`examples/gator-bridge.ts`](./examples/gator-bridge.ts) for a minimal TypeScript example using the Smart Accounts Kit.

---

## 🔧 Execution Realism

| Parameter | Description |
|-----------|-------------|
| **Fees** | Taker/maker fees from Polymarket API |
| **Slippage** | Non-linear price impact from order book |
| **Partial Fills** | Orders may not fully execute |
| **Latency** | Delay between signal and execution |
| **Position Sizing** | Dynamic sizing based on risk & liquidity |

---

## 📚 Documentation

| Doc | Purpose |
|-----|---------|
| [docs/metamask/v1.md](./docs/metamask/v1.md) | ERC-7715 architecture |
| [docs/spec.md](./docs/spec.md) | Technical specification |
| [docs/math.md](./docs/math.md) | Mathematical foundations |
| [docs/polymarket.md](./docs/polymarket.md) | API reference |
| [docs/implementation.md](./docs/implementation.md) | Build guide |
| [docs/demo-script.md](./docs/demo-script.md) | Demo walkthrough |

---

## 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| **Language** | Rust |
| **Wallet** | MetaMask Smart Account |
| **Permissions** | ERC-7715 |
| **Market Data** | Envio HyperIndex |
| **Target Chain** | Polygon Mainnet (Chain ID: 137) |

---

## 🚀 Quick Start

```bash
# Clone and configure
git clone https://github.com/your-username/polyshark
cd polyshark
cp .env.example .env  # Add your keys

# Build and run
cargo build --release
cargo run
```

Open `dashboard/index.html` in your browser to interact with the agent.

---

## 📄 License

MIT License — See [LICENSE](./LICENSE) for details.

---

## 🔗 Related Resources

- [Delegation Toolkit](https://docs.metamask.io/smart-accounts/delegation-toolkit)
- [Smart Accounts Kit](https://docs.metamask.io/smart-accounts)
- [ERC-7715 Spec](https://eips.ethereum.org/EIPS/eip-7715)
- [create-gator-app](https://github.com/MetaMask/create-gator-app)
- [Envio HyperIndex](https://docs.envio.dev/)
