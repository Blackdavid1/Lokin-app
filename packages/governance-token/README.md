# governance-token — Governance Token (SEP-41 / ST-20-like)

Purpose: Fungible governance token with admin controls: mint/burn/clawback, freeze/unfreeze accounts, allowlist mode, and admin transfer. Used for governance voting power and membership-weighted actions across the Loki-app ecosystem.

The token implements the [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md) (Soroban Token Interface) surface — canonical function names, arguments and semantics — and it is built on top of [`soroban-token-sdk`](https://crates.io/crates/soroban-token-sdk):

- **Metadata** is stored through `soroban_token_sdk::TokenUtils`, i.e. in the canonical `METADATA` instance entry (`decimal` / `name` / `symbol`), exactly like every other SEP-41 token.
- **Events** are the canonical Stellar token events from `soroban_token_sdk::events`, so indexers and the Go backend decode governance-token activity with the same logic they use for Stellar Asset Contract (SAC) assets.

## Entry points

- `initialize(admin, name, symbol, decimals)` — initialize token metadata and admin
- `transfer(from, to, amount)` — SEP-41 transfer; `to` is a muxed address (a plain `G…` address is a valid muxed address)
- `transfer_from(spender, from, to, amount)` — spender transfers on owner's behalf, consuming its allowance
- `approve(owner, spender, amount, expiration_ledger)` — approve spender with expiration
- `burn(from, amount)` — burn tokens (owner)
- `burn_from(spender, from, amount)` — SEP-41 burn consuming `spender`'s allowance
- `mint(admin, to, amount)` — admin mints tokens
- `clawback(admin, from, amount)` — admin clawback
- `admin_burn_from(admin, from, amount)` — admin-authorized burn from any account (no allowance needed)
- `freeze(admin, account)` / `unfreeze(admin, account)` — freeze/unfreeze an account
- `set_admin(admin, new_admin)` — transfer admin role

Reads:
- `balance(account)` — i128 balance
- `allowance(owner, spender)` — i128 spendable amount (expired allowances read as `0`, per SEP-41)
- `allowance_data(owner, spender)` — `AllowanceData { amount, expiration_ledger }` raw record
- `total_supply()` — i128 total supply
- `name()` / `symbol()` / `decimals()` — metadata
- `get_admin()` — Result<Address> current admin
- `is_frozen(account)` — bool
- `is_allowlist_mode_enabled()` — bool
- `is_allowlisted(account)` — bool

Allowlist management:
- `set_allowlist_mode(admin, enabled)` — enable/disable allowlist mode
- `add_to_allowlist(admin, account)` / `remove_from_allowlist(admin, account)`

Pause:
- `pause(admin)` / `unpause(admin)` — emergency stop

## Storage keys (DataKey)
Defined in contract/src/types.rs and contract.rs (standard token storage: balances, allowances, metadata, admin, paused, allowlist). Metadata lives in the `soroban-token-sdk` `METADATA` instance entry; balances, allowances, frozen accounts and the allowlist are persistent entries.

## Events

| Event | Topics | Data | Meaning |
|-------|--------|------|---------|
| `transfer` | `("transfer", from, to)` | `{ amount, to_muxed_id }` (map) | tokens transferred to a muxed address |
| `transfer` | `("transfer", from, to)` | `amount` (single value) | tokens transferred to a plain address |
| `approve` | `("approve", from, spender)` | `[amount, expiration_ledger]` (vec) | approval granted |
| `mint` | `("mint", to)` | `amount` (single value) | tokens minted |
| `burn` | `("burn", from)` | `amount` (single value) | tokens burned |
| `clawback` | `("clawback", from)` | `amount` (single value) | tokens clawed back |
| `AccountFrozen { account }` | — | — | account frozen |
| `AccountUnfrozen { account }` | — | — | account unfrozen |
| `AllowlistModeChanged { enabled }` | — | — | allowlist mode toggled |
| `AllowlistAdded { account }` | — | — | account added to allowlist |
| `AllowlistRemoved { account }` | — | — | account removed from allowlist |

The five token events are the `soroban-token-sdk` definitions and are byte-for-byte identical to the ones the Stellar Asset Contract emits, so SAC-compatible indexers work out of the box.

## Invariants
- Transfer/burn amount must be positive and not exceed balance (`InsufficientBalance`, `InvalidAmount`).
- Only admin may mint, admin_burn_from, clawback, freeze, set_admin (`NotAdmin`/`Unauthorized`).
- Frozen accounts cannot transfer (`Frozen`).
- Allowance cannot exceed max / go negative (`AllowanceExceeded`, `NegativeAllowance`); expired allowances are invalid (`AllowanceExpired`) and read back as zero.
- `approve` overwrites the previous allowance (SEP-41) and may not point a live allowance at a past ledger (`InvalidExpirationLedger`).
- Arithmetic overflow/underflow returns typed errors (`Overflow`, `Underflow`), never panics.
- Cannot transfer to self (`CannotTransferToSelf`).
- Mutating functions rejected while paused (`ContractPaused`).

## SEP-41 deviations

| Deviation | Reason |
|---|---|
| Mutating functions return `Result<(), TokenError>` instead of panicking | Keeps the typed-error convention used across this workspace (`AGENTS.md`); the error code is still visible to clients as a contract error. |
| `expiration_ledger = 0` means "never expires" | Backwards compatible with the Go backend, which has always used `0` for non-expiring allowances. Any other value must be at or after the current ledger. |
| `mint`, `clawback`, `admin_burn_from` take an explicit `admin` argument | SEP-41 leaves mint/clawback naming and authorization free; the explicit admin argument makes the authorization visible in the call signature. |
| `allowance` returns `i128`, the raw record is `allowance_data` | SEP-41 mandates `i128`; the raw record is exposed as a governance extension for UIs that render expiry. |

## Test command
```bash
# From repo root
cargo test -p governance-token
cargo test -p governance-token --doc
# Full workspace
cargo test --workspace
```
