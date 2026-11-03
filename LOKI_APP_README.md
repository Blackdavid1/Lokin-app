# Loki-app Smart Contracts

Rust/Soroban smart contracts implementing ROSCA (Rotating Savings and Credit Association) protocol for 1.7B+ unbanked adults on Stellar.

## Business Model

Smart contracts hold circle funds in escrow, enforce contribution rules, distribute payouts automatically, and maintain on-chain MoiScore reputation. No central party controls funds.

### Contract Architecture (5 Contracts)

```
contracts/
├── circle-factory/         # Deploys new Circle instances, maintains registry
├── circle/                 # Core ROSCA engine (per-circle contract)
├── reputation-registry/    # On-chain MoiScore calculation/storage
└── treasury/               # Collects 0.5% protocol fee
```

### Revenue Collection
Treasury contract collects 0.5% fee on every payout distribution. Fee split across platform sustainability and future DAO treasury.

## Circle Contract (Core Engine)

### State Machine
```
[Created] → [Funding] → [Active] → [Completed]
     ↓          ↓           ↓
  Cancelled   Defaulted    Paused
```

### ROSCA Parameters
| Parameter | Description | Configurable |
|-----------|-------------|--------------|
| Contribution Amount | USDC/XLM per cycle | Yes |
| Currency | USDC or XLM | Yes |
| Frequency | Daily/Weekly/Biweekly/Monthly | Yes |
| Max Members | Circle capacity | Yes |
| Late Fee | Penalty percentage | Yes (default 5%) |
| Grace Period | Hours before late | Yes (1-168h, default 24h) |
| Max Strikes | Removal threshold | Yes (1-10, default 3) |
| Payout Type | Random/Fixed/Auction/Vote | Yes |

### Payout Types
| Type | Function | Algorithm |
|------|----------|-----------|
| Random | `payout_random()` | VRF selection, provably fair |
| Fixed | `payout_fixed()` | Sequential by member index |
| Auction | `payout_auction()` | Lowest discount bidder wins |
| Merit | `payout_merit()` | Reputation-weighted selection |

### Member Lifecycle
| Status | Trigger |
|--------|---------|
| pending | Joined, awaiting quorum |
| active | Circle started, in rounds |
| late | Missed grace period |
| removed | Exceeded max strikes |

### Events
```rust
emit_member_joined(member: Address, deposit: u128);
emit_contribution_submitted(member: Address, amount: u128);
emit_payout_processed(payout_type: PayoutType, winner: Address);
emit_penalty_applied(member: Address, amount: u128);
```

## Reputation Registry Contract

### MoiScore Formula (0-1000)
```
MoiScore = Streak×0.35 + Completions×0.30 + Volume×0.20 + Recency×0.15
```

### Score Tiers
| Range | Tier | Benefits |
|-------|------|----------|
| 0-200 | Bronze | Basic access |
| 201-400 | Silver | Higher limits |
| 401-600 | Gold | Premium access |
| 601-800 | Platinum | Governance voting |
| 801-1000 | Diamond | Zero collateral |

### Functions
- `calculate_score(address: Address)`: Compute current MoiScore
- `record_contribution(address, amount, cycle)`: Add to user score
- `get_score(address)`: Retrieve stored MoiScore

## Treasury Contract

### Functions
- `collect_fee(amount, circle_id)`: Take 0.5% fee
- `withdraw(amount)`: Admin withdrawal
- `get_balance()`: Current treasury holdings



## Security Architecture

### Access Control
```rust
enum Role {
    Admin,
    Organizer,
    Member,
}
```

### Safe Math
All arithmetic uses checked operations to prevent overflow/underflow.

### Security Features
| Feature | Implementation |
|---------|----------------|
| Authorization | Address-based caller verification |
| Pause Mechanism | Emergency stop via Admin |
| Input Validation | Contract-level checks |
| Integer Safety | Checked arithmetic |
| Reentrancy | Sequential execution model |
| No unwrap() | Panic-safe operations |

## Development

### Prerequisites
- Rust 1.70+
- Soroban CLI 21.x
- Stellar account with test XLM

### Build Commands
| Command | Purpose |
|---------|---------|
| `cargo build` | Compile WASM |
| `cargo test` | Run 45 unit tests |
| `make optimize` | Optimize for deployment |
| `make deploy-testnet` | Deploy to testnet |
| `make deploy-mainnet` | Deploy to mainnet |
| `make bindings` | Generate Go bindings |

### Gas Optimization
All contracts under 50KB WASM target size. Optimized for minimal on-chain storage and batch operations.

### Editor Config
Editor settings are standardized in [`.editorconfig`](.editorconfig) (Rust: 4 spaces, LF; matches `.rustfmt.toml` and `cargo fmt --check` in CI). See [CONTRIBUTING.md](CONTRIBUTING.md#code-style) for code style details.

## Deployment

### Testnet
```bash
soroban config identity --secret <secret-key>
make deploy-testnet
```

### Mainnet
```bash
make deploy-mainnet
sha256sum target/wasm32v1-none/release/circle.wasm
```

### Network Endpoints
| Network | RPC URL |
|---------|---------|
| Testnet | https://soroban-testnet.stellar.org |
| Mainnet | https://soroban-mainnet.stellar.org |

## Integration

Go bindings generated via `make bindings`:

```go
type Client struct {
    contractID string
    signer     keypair.KP
}

func (c *Client) Initialize(params InitializeParams) error
func (c *Client) Join(member string, amount int64) error
func (c *Client) Contribute(member string, amount int64) error
```

## Testing Matrix

| Test Type | Coverage | Tests |
|-----------|----------|-------|
| Unit | 92% | 45 tests |
| Property | 85% | 12 tests |
| Integration | 78% | 8 tests |
| Fuzz | 65% | 3 tests |

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the full text.