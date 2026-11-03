# Loki-app — Agent Guidelines

You are an elite Rust/Soroban and Go engineer building the Loki-app smart contract ecosystem. The contracts are deployed to Stellar's Soroban platform and consumed by the Go backend.

Follow these directives strictly. Every deviation must be explicitly justified.

---

## 1. PRIME DIRECTIVES

### 1.1 Enterprise Quality Only
- Every line of code must be production-grade. No stubs, no placeholders, no TODOs unless part of a documented phase boundary.
- Every function must handle errors explicitly. No `unwrap()` in production code.
- Every error must be typed (not string-based). Use proper error enums with variants.
- Every storage read/write must have explicit lifetime considerations (instance vs persistent vs temporary).

### 1.2 Test-Driven Everything
- Write tests BEFORE implementation logic where possible.
- Every contract function must have: happy path test, edge case test, failure mode test, access control test.
- Target 95%+ line coverage on contracts.
- Use Soroban SDK's test framework (not mocks for contracts — test against real local ledger).

### 1.3 Security First
- Every mutating function MUST have access control checks FIRST, before any state changes.
- Use the common access control module. No inline auth checks.
- Validate ALL inputs before touching storage (check → compute → write pattern).
- Integer overflow protection: use Soroban SDK's safe math or explicit overflow checks.
- Reentrancy protection: never invoke external contracts during state mutation.
- Emergency pause: every contract must support pause/unpause via admin.

### 1.4 Gas Optimization
- Minimize storage operations (reads + writes). Batch where possible.
- Use instance storage for small, frequently-accessed data (< 64 bytes).
- Use persistent storage for large data (member lists, round history).
- Pre-compute derived values at write time, not read time.
- Short field names in events to reduce log gas costs.

### 1.5 Immutability + Upgradability
- Contracts deploy once. State persists forever.
- Use the proxy + implementation pattern from `common/upgrade.rs`.
- Admin auth required for upgrades.
- Never lose state during upgrades — store state in proxy, not implementation.

---

## 2. CODE STANDARDS

### 2.1 Rust Standards
```rust
// ✓ GOOD — typed errors, access control, safe math
pub fn contribute(env: Env, member: Address, amount: i128, round: u32) -> Result<(), CircleError> {
    member.require_auth();
    let circle = load_circle(&env)?;
    validate_active(&circle)?;
    validate_member(&env, &member, &circle)?;
    validate_round(&circle, round)?;
    validate_amount(amount, circle.contribution_amount)?;
    
    record_contribution(&env, &member, round, amount)?;
    
    env.events().publish((circle.id.clone(),), ContributionReceived {
        member: member.clone(),
        round,
        amount,
    });
    Ok(())
}

// ✗ BAD — no error type, no auth, no validation, unwrap
pub fn contribute(env: Env, amount: i128) {
    let c: Circle = env.storage().instance().get(&DataKey::Circle).unwrap();
    c.contributions.push(amount);
    env.storage().instance().set(&DataKey::Circle, &c);
}
```

### 2.2 Error Enums — Always Typed
```rust
#[derive(Debug)]
#[contracterror]
pub enum CircleError {
    NotActive = 1,
    CircleFull = 2,
    AlreadyMember = 3,
    NotMember = 4,
    InsufficientMoiScore = 5,
    RoundNotCurrent = 6,
    InvalidAmount = 7,
    PaymentDeadlinePassed = 8,
    MaxStrikesReached = 9,
    NotOrganizer = 10,
    ContractPaused = 11,
    InvalidInviteCode = 12,
    AuctionAlreadyResolved = 13,
    VoteQuorumNotMet = 14,
    // ... every possible failure mode gets a variant
}
```

### 2.3 Event Emissions
```rust
// Every state change emits an event. Events are the source of truth for the indexer.
// Format: (topic_symbol, data), where topic is contract-address scoped.

env.events().publish(
    (symbol_short!("contribute"),), 
    ContributionReceived { member, round, amount }
);

// Always use short symbol names — saves gas.
// Events must carry enough data for the indexer to reconstruct state.
```

### 2.4 Storage Layout
```rust
// Explicit storage key enums — never use string keys
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    CircleConfig,       // CircleConfig — once, at init
    Members,            // Vec<Member> — persistent, grows
    Contributions,      // Vec<Contribution> — persistent, grows per round
    PayoutBitmap,       // u128 bitmask — which positions have been paid
    CurrentRound,       // u32 — instance storage
    Collateral,         // Map<Address, i128> — member stakes
    Paused,            // bool — emergency
}
```

---

## 3. IMPLEMENTATION ORDER (MUST FOLLOW)

### Phase 1: Common Library First
1. `common/` — Build shared types, VRF, math, access control, pause, upgrade proxy
2. All contracts depend on `common`. Nothing else is built first.

### Phase 2: Core Contracts
3. `circle-factory/` — Factory deploys circles. Frame the registry.
4. `circle/` — The big one. All 12 functions. All 4 payout types. Heavy test coverage.
5. `reputation-registry/` — MoiScore calculation. Depends on circle events via callback.

### Phase 3: Supporting Contracts  
6. `treasury/` — Fee collection. Depends on factory fee config.

### Phase 4: Deployment + Bindings
8. Build → optimize → deploy to testnet
9. Generate Go bindings from deployed contract specs
10. Verify all contracts on-chain

---

## 4. TESTING STANDARDS

### 4.1 Contract Unit Tests (cargo test)
```rust
#[test]
fn test_contribute_happy_path() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let member = Address::generate(&env);
    
    // Setup: deploy circle with 1 member
    let contract = deploy_test_circle(&env, &admin, 5);
    join_as_member(&env, &contract, &member, 1);
    
    // Act
    let result = contract.contribute(&member, 100_0000000, 1);
    
    // Assert
    assert!(result.is_ok());
    let contributions = contract.get_contributions(&member);
    assert_eq!(contributions.len(), 1);
    assert_eq!(contributions[0].amount, 100_0000000);
}

#[test]
fn test_contribute_wrong_amount() {
    // Contribution must match circle config exactly
    let contract = deploy_test_circle(&env, &admin, 5);
    let result = contract.contribute(&member, 50_0000000, 1);  // wrong amount
    assert_eq!(result, Err(Ok(CircleError::InvalidAmount)));
}

#[test]
fn test_contribute_not_member() { /* ... */ }
#[test]
fn test_contribute_round_mismatch() { /* ... */ }
#[test]
fn test_contribute_past_deadline() { /* ... */ }
```

### 4.2 Integration Tests (multi-contract)
```rust
#[test]
fn test_full_circle_lifecycle() {
    let env = Env::default();
    
    // 1. Deploy factory
    let factory = deploy_factory(&env);
    
    // 2. Create circle via factory
    let config = CircleConfig { /* ... */ };
    let circle_id = factory.deploy_circle(&admin, config);
    let circle = CircleClient::new(&env, &circle_id);
    
    // 3. 5 members join
    for i in 0..5 {
        let member = Address::generate(&env);
        circle.join(&member, None);
    }
    
    // 4. Each member contributes for 5 rounds
    for round in 0..5 {
        for member in members.iter() {
            circle.contribute(member, config.amount, round);
        }
        circle.trigger_payout(round);
    }
    
    // 5. Verify circle completed
    assert_eq!(circle.get_status(), CircleStatus::Completed);
    assert_eq!(circle.get_completed_rounds(), 5);
}
```

### 4.3 Fuzz Testing
```rust
#[test]
fn fuzz_contribution_amounts() {
    // Fuzz random contribution amounts — must NOT panic
    // Must return proper errors for invalid amounts
    proptest!(|(amount: i128)| {
        let result = contract.contribute(&member, amount, 1);
        // Must never panic — always return Result
    });
}
```

---

## 5. Go BACKEND INTEGRATION

### 5.1 Contract IDs in Config
```yaml
# loki-app-backend/config/config.yaml
stellar:
  testnet:
    circle_factory_contract_id: "CDEF..."
    circle_wasm_hash: "abc123..."
    reputation_contract_id: "GHIJ..."
    treasury_contract_id: "OPQR..."
```

### 5.2 Error Classification
```go
// Map Soroban contract errors to Go domain errors
func ClassifySorobanError(scErr *SorobanContractError) error {
    switch scErr.Code {
    case 1:  return apperrors.ErrCircleNotActive
    case 2:  return apperrors.ErrCircleFull
    case 3:  return apperrors.ErrAlreadyMember
    case 5:  return apperrors.ErrMoiScoreTooLow
    case 10: return apperrors.ErrNotOrganizer
    case 11: return apperrors.ErrContractPaused
    default: return fmt.Errorf("contract error %d: %s", scErr.Code, scErr.Message)
    }
}
```

### 5.3 NEVER Do This
```go
// ✗ NEVER store secret keys in Go source code
const masterSecret = "SDBB..."

// ✗ NEVER hardcode contract IDs
circleContract := "CDEF..."

// ✓ Use config
cfg.Stellar.MasterSecretKey  // from env var
cfg.Stellar.CircleFactoryContractID  // from config.yaml
```

---

## 6. PERFORMANCE BUDGETS

| Operation | Gas Budget (testnet) | Time Budget |
|---|---|---|
| Deploy circle | < 500,000 | < 5s |
| Join circle | < 100,000 | < 2s |
| Contribute | < 80,000 | < 2s |
| Payout (random) | < 150,000 | < 3s |
| Payout (auction) | < 200,000 | < 3s |
| Get status (read) | < 5,000 | < 0.5s |

---

## 7. PHASE GATES — DO NOT PROCEED WITHOUT

| Phase | Gate | Must Pass |
|---|---|---|
| 1→2 | All contracts compile + pass unit tests | `cargo test --workspace` |
| 2→3 | Contracts deployed to testnet | `scripts/deploy.sh testnet` succeeds |
| 3→4 | Go client makes real testnet calls | `go test ./pkg/stellar/soroban/...` against testnet |
| 4→5 | Load test passes | 100 concurrent users, < 1% failure |
| 5 | All deliverables complete | See PLANS.md Phase 5 checklist |

---

## 8. AGGRESSIVE PHASE TESTING — MANDATORY GATE

**No phase is complete until every component implemented in that phase passes extreme aggressive testing. Do NOT mark any phase as complete without this.**

### 8.1 What "Extreme Aggressive Testing" Means

For every component (contract, function, Go module, indexer unit), you MUST test:

| Dimension | Minimum Required |
|---|---|
| **Happy path** | All valid inputs produce expected outputs |
| **Edge cases** | Zero values, max values, empty collections, boundary conditions |
| **Failure modes** | Every error variant triggered and verified |
| **Access control** | Unauthorized callers rejected for every mutating function |
| **Concurrency** | Race conditions tested (Go: `-race` flag) |
| **State corruption** | Invalid state transitions rejected |
| **Gas limits** | Every contract function verified within budget |
| **Input fuzzing** | Random/malformed inputs must NEVER panic — always return typed errors |
| **Network failures** | Simulated RPC timeouts, connection resets, reorgs |
| **Resource exhaustion** | Large inputs (100 members, 100 rounds), deep nesting |
| **Regression** | Previous passing tests must still pass |

### 8.2 Phase-Specific Testing Requirements

| Phase | Components | Aggressive Test Protocol |
|---|---|---|
| **Phase 1** | All 5 Rust contracts | `cargo test --workspace` must pass with 95%+ coverage. Every function must have: happy path ×1, edge case ×2, failure mode ×3, access control ×1. Fuzz test contribution amounts and member counts. Test all 4 payout types with 10-member circles. Deploy each contract to local Soroban ledger and run full lifecycle. |
| **Phase 2** | Go contract client (15 files) | `go test ./pkg/stellar/... -race -count=5` must pass 5 consecutive times with zero race conditions. Test against real testnet with your account. Simulate: network timeout, invalid sequence, insufficient balance, contract error, transaction expired. Submit 50 rapid-fire transactions — verify zero sequence gaps. |
| **Phase 3** | Indexer engine (7 files) | Replay 100 ledgers of known events — verify 100% captured, zero duplicates. Test cursor rollback on chain reorg. Test reconciler gap detection. Test DB connection loss + recovery. Test RabbitMQ disconnect + reconnect. Run for 1 hour continuously — verify memory stable, no goroutine leaks. |
| **Phase 4** | Hardening (upgrade, pause, load, soak) | Deploy v1 → upgrade to v2 → verify state preserved. Pause contract → verify all mutating functions reject → unpause → verify resume. Load test: k6 with 100 VUs for 5 minutes, p95 < 2s, < 1% failures. Soak test: 24-hour run, log memory/goroutine/connection pool every hour. |

### 8.3 Test Execution Rules

```
1. Tests run in FRESH environment — no cached state from previous runs
2. Tests run with RACE DETECTOR enabled (go test -race, cargo test with sanitizers)
3. Tests run MULTIPLE TIMES — flaky tests are broken tests
4. Tests ASSERT, not print — every check is an assertion that fails the build
5. Tests output COVERAGE — must meet 95%+ for contracts, 85%+ for Go
6. Test failures BLOCK the phase — no proceeding until all pass
7. Test results are DOCUMENTED in the phase completion report
```

### 8.4 Phase Completion Report Template

Before marking any phase complete, produce a report containing:

```
Phase X Completion Report
=========================
Date: YYYY-MM-DD
Components Tested: [list every file]
Total Tests: N
Passed: N
Failed: 0
Coverage: XX%
Race Detector: CLEAN

Per-Component Breakdown:
  component_a:
    - Happy path: ✓ (N tests)
    - Edge cases: ✓ (N tests)  
    - Failure modes: ✓ (N tests)
    - Access control: ✓ (N tests)
    - Concurrency: ✓ (N tests, -race clean)
    - Gas within budget: ✓ (max: X, budget: Y)

  component_b:
    [same breakdown]

Issues Found: 0
Blockers: None
Phase Status: COMPLETE ✓
```

### 8.5 NEVER Mark a Phase Complete If

- Any test fails (no skipping, no `#[ignore]`)
- Coverage is below the threshold
- Race detector shows any warnings
- A function was implemented but not tested
- Error paths are untested ("return err" with no test triggering it)
- The phase completion report has not been produced
- Any component shows a gas usage above budget without explanation

---

## 9. CRITICAL ANTI-PATTERNS

| Anti-Pattern | Why It's Dangerous | Correct Approach |
|---|---|---|
| `unwrap()` in contract code | Panics destroy state and lock funds permanently | Return `Result<_, ContractError>` always |
| String-based error messages | Not machine-readable, no classification | Use typed error enums with numeric codes |
| Hardcoded addresses | Breaks across networks | Use config/constructor injection |
| Storing large arrays in instance storage | Instance storage is expensive and limited | Use persistent storage for collections |
| No replay protection on contributions | Double-spend risk | Check contribution uniqueness per round |
| Skipping simulation before submit | Wastes gas on broken transactions | Always simulate → apply → sign → submit |
| No sequence manager | Sequence gaps break all subsequent transactions | Use AccountManager with mutex |

---

## 10. REFERENCE: Quick Commands

```bash
# Contracts
cargo build --target wasm32-unknown-unknown --release
cargo test --workspace
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/circle.wasm
soroban contract deploy --wasm circle.optimized.wasm --source admin --network testnet

# Backend
cd ../loki-app-backend
go test ./pkg/stellar/soroban/... -v -count=1
go test ./internal/indexer/... -v -count=1
go test ./... -count=1
```

---

## 11. FINAL RULE

**If you don't know the exact Soroban SDK API, look it up. Never guess.**

The SDK changes. The Stellar docs at https://developers.stellar.org/docs/smart-contracts are the source of truth. Read the actual SDK source if needed. A wrong API call that compiles but fails at runtime on-chain wastes real testnet XLM and developer time.
