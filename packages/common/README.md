# common — Shared Soroban Library

Purpose: Shared, audit-friendly primitives used by every Loki-app contract. Not a deployable contract itself — it is an `rlib` dependency (no `cdylib`). Provides access control, pause/emergency-stop, reentrancy guards, upgrade proxy helpers, VRF (SHA-256 hash chain + Ed25519 proofs), safe/fixed-point math, token transfer helpers, and typed error envelopes. All contracts must route auth, pause, and reentrancy through this module rather than maintaining local copies.

## Storage keys
All keys live in **instance storage** (small, frequently read):

| Key | Type | Meaning |
|-----|------|---------|
| `paused` | `bool` | Emergency pause flag (common::pause) |
| `impl` | `Address` | Proxy implementation address (common::upgrade) |
| `vrf_admin` / `vrf_salt` / `vrf_ctr` / `vrf_ownr` / `vrf_pend` / `vrf_pndat` / `vrf_nonce` | various | VRF state (common::vrf) |
| `reent` | `bool` | Reentrancy lock in **temporary** storage with 2-ledger TTL (common::reentrancy) |

## Entry points
This crate exposes no Soroban entry points. Its public API is Rust functions called by dependent contracts:

- `common::access::require_self_or_admin(env, addr, stored_admin)` — self-or-admin authorization
- `common::pause::{pause, unpause, is_paused, when_not_paused}` — emergency stop
- `common::reentrancy::{ReentrancyGuard, with_reentrancy_guard}` — reentrancy protection
- `common::upgrade::{get_implementation, set_implementation, upgrade_contract}` — proxy + in-place WASM upgrade
- `common::vrf::{init_vrf, evaluate_vrf, shuffle_positions, random_in_range, verify_vrf, propose_key_rotation, activate_key_rotation}` — verifiable randomness
- `common::math::{safe_add, safe_sub, safe_mul, safe_div, calculate_percentage, apply_fee, calculate_penalty, fp_*, convert_shares}` — checked/fixed-point arithmetic
- `common::token_transfer::{safe_transfer, transfer_from_contract, transfer_to_contract}` — explicit token transfers
- `common::explorer::{get_explorer_base_url, construct_tx_explorer_url}` — explorer URL helpers

## Events
- `paused { by: Address }` — contract paused
- `unpaused { by: Address }` — contract unpaused
- `upgraded { by: Address, new_impl: Address }` — implementation address updated
- `ContractUpgraded { by: Address, new_wasm_hash: BytesN<32> }` — WASM upgraded in place
- `VrfEvaluated { input_seed, vrf_output, counter }` — VRF evaluation
- `VrfKeyRotationProposed { new_key, activation_time }` — key rotation proposed
- `VrfKeyRotated { new_key }` — key rotation activated
- `VrfFulfilled { request_id, seed }` — VRF request fulfilled

## Invariants
- Pause/unpause are idempotent (no-op + success when already in target state).
- Upgrade may proceed while paused (an upgrade may be the only way to fix a paused contract).
- Reentrancy lock uses temporary storage with TTL so a panic never permanently locks the contract.
- All arithmetic is checked — no panics on overflow/underflow/div-by-zero; typed `MathError` returned instead.
- VRF input seeds must be strictly increasing (replay protection).

## Test command
```bash
# From repo root
cargo test -p common
cargo test -p common --doc
```
