# PolyShark Demo Script (3-4 minutes)

> **Optimized for MetaMask x Envio Hackathon Judges**

---

## Pre-Recording Setup

- Dashboard open at `dashboard/index.html`
- MetaMask extension ready
- Clear browser cache for clean demo
- Terminal ready (optional, for agent logs)

---

## Scene 1: The Problem (0:00–0:30)

**SCREEN:** Show the dashboard header briefly, then cut to concept slide.

**SAY:**
> "Every DeFi bot today makes you approve every single trade. That's fine for manual trading, but for an autonomous agent? Unusable. 
>
> PolyShark solves this with ERC-7715 Advanced Permissions—cryptographically enforced spending limits. Let me show you."

---

## Scene 2: Configure Permission (0:30–1:15)

**ACTION:**
1. Show dashboard Permission Configuration section
2. Adjust daily limit slider to $20
3. Select 30 days duration
4. Click "View JSON Config" → show the modal

**SAY:**
> "The user sets their risk tolerance: 20 USDC per day for 30 days. 
>
> This is the exact ERC-7715 permission object that gets sent to MetaMask. Human-readable, user-controlled, and most importantly—**cryptographically enforced**. Not trust-based. Not a promise. On-chain enforcement."

**KEY PHRASE:**
> 💡 "This is not a trust relationship. It's cryptographic enforcement."

---

## Scene 3: Grant Permission (1:15–1:45)

**ACTION:**
1. Click "Connect MetaMask" 
2. Show MetaMask popup (or simulated)
3. Approve the permission
4. Agent status changes to "RUNNING"
5. Strategy mode shows "AGGRESSIVE" (full allowance)

**SAY:**
> "**This is the last popup you'll see for the next thousand trades.**
>
> Once granted, PolyShark can trade autonomously—but it can never exceed this daily limit. The user approved once; the agent executes within bounds."

**KEY PHRASE:**
> 💡 "One popup. Then autonomous."

---

## Scene 4: Autonomous Trading + Envio Safety (1:45–2:30)

**ACTION:**
1. Point to Envio health panel: delay ~150ms, block height, 0 errors
2. Watch trades populate in the list
3. Observe allowance bar decreasing
4. (Optional) Simulate delay >5s → show agent enters HALTED mode

**SAY:**
> "Watch the Envio indicator. We're getting market data with 150 millisecond latency. 
>
> **Without Envio, autonomous trading is reckless.** Stale data leads to bad trades. But PolyShark monitors Envio health—if delay exceeds 5 seconds, the agent halts. No stale data, no blind trades.
>
> Notice the strategy mode adapting: as allowance drops, the agent gets more conservative. It's not just about limits—it's about intelligent budgeting."

**KEY PHRASE:**
> 💡 "Without Envio, this would be dangerous. With Envio + ERC-7715, it's safe."

---

## Scene 5: Instant Revocation (2:30–3:00)

**ACTION:**
1. Click "Revoke Permission"
2. Agent immediately stops → status changes to "IDLE"
3. No more trades execute

**SAY:**
> "The user is always in control. **One click—instant revocation.** 
>
> The agent respects it immediately. There's no delay, no pending transactions. This is trust-minimized automation."

**KEY PHRASE:**
> 💡 "User always in control. Instant stop."

---

## Scene 6: Summary & Reusability (3:00–3:30)

**SCREEN:** Dashboard overview showing total trades, PnL, win rate

**SAY:**
> "PolyShark demonstrates how ERC-7715 Advanced Permissions enable safe, autonomous agents. Combined with Envio's low-latency indexer, we get real-time trading without sacrificing user control.
>
> But this isn't just a Polymarket bot—it's a **pattern for any ERC-7715-powered automation**. Swap the market data source, replace the trading logic, keep the permission layer. DEX arbitrage, NFT sniping, game automation—the pattern works."

**KEY PHRASE:**
> 💡 "This is a reference pattern, not just a trading bot."

---

## Key Phrases Cheat Sheet

| Moment | Power Phrase |
|--------|--------------|
| Permission grant | "This is the last popup you'll see for the next thousand trades." |
| Enforcement | "Not trust-based—cryptographically enforced on-chain." |
| Envio | "Without Envio, autonomous trading is reckless. With Envio, it's safe." |
| Safety | "When data is stale, the agent stops. Period." |
| Revocation | "One click. Instant stop. User always in control." |
| Reusability | "This pattern works for DEXs, NFTs, games—any agent." |

---

## Video Production Notes

1. **Length:** Target 3:00–3:30. Judges have many submissions.
2. **Pacing:** Don't rush Scene 2 (permission config). Let judges see the JSON.
3. **Highlight Envio:** Make sure block height and delay metrics are visible.
4. **Record in 1080p** at minimum. Clean browser window, no bookmarks visible.
5. **Upload to YouTube** as unlisted. Add link to README.

---

## Post-Recording Checklist

- [ ] Video uploaded to YouTube/Loom
- [ ] Link added to README.md TL;DR section
- [ ] Thumbnail shows PolyShark logo + "ERC-7715 Demo"
- [ ] Video description includes project summary + links
