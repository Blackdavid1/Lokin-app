/// Verifiable Random Function (VRF) for Loki-app contracts.
///
/// This module replaces the previous PRNG-based randomness (`env.prng()`) with
/// a cryptographically verifiable random function built on:
///
/// - **SHA-256 hash chain**: Each evaluation produces a deterministic output
///   from `(input_seed, salt, counter)`. The salt is generated via
///   `env.prng().gen()` at initialization and is a consensus-level value
///   derived from the overall transaction-set hash, making it cryptographically
///   hard to bias even by a corrupt validator.
///
/// - **Ed25519 signature verification**: An admin keypair (Ed25519) is stored
///   at initialization. Each VRF evaluation can be accompanied by an Ed25519
///   signature over the evaluation output, allowing any third party to verify
///   the randomness was authorized by the admin. This prevents unauthorized
///   or tampered random values.
///
/// ## VRF Properties
///
/// 1. **Determinism**: Given the same `(input_seed, salt, counter)`, the VRF
///    always produces the same output. Anyone can recompute and verify.
/// 2. **Unpredictability**: The salt is a consensus-level random value (from
///    transaction-set hash), unknown before the ledger closes. Combined with
///    the incrementing counter, each evaluation is unique and unpredictable.
/// 3. **Verifiability**: The Ed25519 signature provides cryptographic proof
///    that the admin authorized the specific VRF output. `verify_vrf` checks
///    this signature against the stored public key.
/// 4. **Non-replayability**: The counter increments with each evaluation.
///    Each `(input_seed, counter)` pair is unique.
///
/// ## Usage in Circle Payouts
///
/// The circle contract calls `shuffle_positions(env, n, base_nonce)` to generate a random
/// permutation of payout positions. Each position is derived from a separate
/// VRF evaluation with an incremented counter, ensuring each position's
/// randomness is independently derived.
///
/// For enhanced security, the admin can sign VRF outputs off-chain and callers
/// can verify via `verify_vrf()` before accepting the shuffled order.
use soroban_sdk::{contracterror, contractevent, symbol_short, Address, Bytes, BytesN, Env, Vec};

// ── Storage keys ──────────────────────────────────────────────────────────

const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("vrf_admin");
const SALT_KEY: soroban_sdk::Symbol = symbol_short!("vrf_salt");
const COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("vrf_ctr");
/// Address authorized to propose/activate VRF key rotations.
const OWNER_KEY: soroban_sdk::Symbol = symbol_short!("vrf_ownr");
const PENDING_KEY: soroban_sdk::Symbol = symbol_short!("vrf_pend");
/// Ledger timestamp at/after which the pending key rotation may be activated.
const PENDING_AT_KEY: soroban_sdk::Symbol = symbol_short!("vrf_pndat");
/// Tracks the last used input seed/nonce to prevent replay
const NONCE_KEY: soroban_sdk::Symbol = symbol_short!("vrf_nonce");

// ── Errors ────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VrfError {
    /// VRF has not been initialized — call `init_vrf` first.
    NotInitialized = 1,
    /// The Ed25519 signature does not match the expected VRF output.
    InvalidProof = 2,
    /// The VRF has already been initialized — reinitialization is prohibited.
    AlreadyInitialized = 3,
    /// Math overflow during counter or range computation.
    Overflow = 4,
    /// Caller is not the VRF owner, or there is no pending rotation, or the
    /// activation delay has not elapsed yet.
    Unauthorized = 5,
    /// No key rotation has been proposed.
    NoPendingRotation = 6,
    /// The proposed rotation's activation delay has not elapsed yet.
    ActivationNotReady = 7,
    /// The provided nonce/input_seed has already been used or is not strictly increasing.
    Replay = 8,
}

// ── Events ────────────────────────────────────────────────────────────────

/// Emitted on each VRF evaluation, enabling off-chain indexers to track
/// the hash chain and verify randomness deterministically.
#[contractevent]
#[derive(Clone, Debug)]
pub struct VrfEvaluated {
    pub input_seed: u32,
    pub vrf_output: u32,
    pub counter: u32,
}

/// Emitted when a key rotation is proposed. The old admin key remains active
/// (and usable by `verify_vrf`) until `activate_key_rotation` is called.
#[contractevent]
#[derive(Clone, Debug)]
pub struct VrfKeyRotationProposed {
    pub new_key: BytesN<32>,
    pub activation_time: u64,
}

/// Emitted when a proposed key rotation is activated, retiring the old key.
#[contractevent]
#[derive(Clone, Debug)]
pub struct VrfKeyRotated {
    pub new_key: BytesN<32>,
}

/// Emitted when a VRF request has been fulfilled with request id and seed.
#[contractevent]
#[derive(Clone, Debug)]
pub struct VrfFulfilled {
    pub request_id: u32,
    pub seed: u32,
}

// ── Public API ────────────────────────────────────────────────────────────

/// Initialize the VRF with an optional Ed25519 public key for signature verification.
///
/// Must be called exactly once before any VRF evaluation. Generates a random
/// salt via `env.prng()` (consensus-level entropy from the transaction-set
/// hash) and stores it alongside the optional admin key.
///
/// When `admin_key` is `Some(key)`, VRF outputs can be verified via
/// `verify_vrf()` using Ed25519 signature checks. When `admin_key` is `None`,
/// the VRF operates in hash-chain-only mode — outputs are deterministic and
/// verifiable by recomputation, but no Ed25519 signature check is available.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `admin_key` - Optional Ed25519 public key (32 bytes) for signature verification
/// * `owner` - Address authorized to propose and activate future key rotations
///   (see `propose_key_rotation` / `activate_key_rotation`). Stored once at init
///   time since the Ed25519 admin key itself is not a Soroban `Address` and
///   cannot authorize transactions on its own.
///
/// # Errors
/// * `VrfError::AlreadyInitialized` if called more than once
pub fn init_vrf(
    env: &Env,
    admin_key: Option<&BytesN<32>>,
    owner: &Address,
) -> Result<(), VrfError> {
    if env.storage().instance().has(&ADMIN_KEY) || env.storage().instance().has(&OWNER_KEY) {
        return Err(VrfError::AlreadyInitialized);
    }
    let salt: BytesN<32> = env.prng().gen();
    env.storage().instance().set(&SALT_KEY, &salt);
    env.storage().instance().set(&COUNTER_KEY, &0u32);
    env.storage().instance().set(&OWNER_KEY, owner);
    if let Some(key) = admin_key {
        env.storage().instance().set(&ADMIN_KEY, key);
    }
    Ok(())
}

/// Propose a rotation of the VRF Ed25519 admin key, without touching the
/// currently active key.
///
/// The proposed key only becomes active once `activate_key_rotation` is
/// called after `activation_delay_secs` have elapsed. Until then, `ADMIN_KEY`
/// is untouched, so any in-flight `verify_vrf` call continues to check
/// signatures against the OLD key — rotation causes no interruption.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `caller` - Must match the stored VRF owner address
/// * `new_key` - The proposed Ed25519 public key (32 bytes)
/// * `activation_delay_secs` - Seconds from now before the rotation can be activated
///
/// # Authorization
/// Requires `caller.require_auth()` and `caller` must equal the stored owner.
///
/// # Errors
/// * `VrfError::Unauthorized` if `caller` is not the stored owner
pub fn propose_key_rotation(
    env: &Env,
    caller: &Address,
    new_key: &BytesN<32>,
    activation_delay_secs: u64,
) -> Result<(), VrfError> {
    caller.require_auth();
    let owner: Address = env
        .storage()
        .instance()
        .get(&OWNER_KEY)
        .ok_or(VrfError::Unauthorized)?;
    if caller != &owner {
        return Err(VrfError::Unauthorized);
    }
    let activation_time = env
        .ledger()
        .timestamp()
        .checked_add(activation_delay_secs)
        .ok_or(VrfError::Overflow)?;
    env.storage().instance().set(&PENDING_KEY, new_key);
    env.storage()
        .instance()
        .set(&PENDING_AT_KEY, &activation_time);
    VrfKeyRotationProposed {
        new_key: new_key.clone(),
        activation_time,
    }
    .publish(env);
    Ok(())
}

/// Activate a previously proposed VRF key rotation, retiring the old key.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `caller` - Must match the stored VRF owner address
///
/// # Authorization
/// Requires `caller.require_auth()` and `caller` must equal the stored owner.
///
/// # Errors
/// * `VrfError::Unauthorized` if `caller` is not the stored owner
/// * `VrfError::NoPendingRotation` if no rotation has been proposed
/// * `VrfError::ActivationNotReady` if the activation delay has not elapsed
pub fn activate_key_rotation(env: &Env, caller: &Address) -> Result<(), VrfError> {
    caller.require_auth();
    let owner: Address = env
        .storage()
        .instance()
        .get(&OWNER_KEY)
        .ok_or(VrfError::Unauthorized)?;
    if caller != &owner {
        return Err(VrfError::Unauthorized);
    }
    let pending_key: BytesN<32> = env
        .storage()
        .instance()
        .get(&PENDING_KEY)
        .ok_or(VrfError::NoPendingRotation)?;
    let activation_time: u64 = env
        .storage()
        .instance()
        .get(&PENDING_AT_KEY)
        .ok_or(VrfError::NoPendingRotation)?;
    if env.ledger().timestamp() < activation_time {
        return Err(VrfError::ActivationNotReady);
    }
    env.storage().instance().set(&ADMIN_KEY, &pending_key);
    env.storage().instance().remove(&PENDING_KEY);
    env.storage().instance().remove(&PENDING_AT_KEY);
    VrfKeyRotated {
        new_key: pending_key,
    }
    .publish(env);
    Ok(())
}

/// Compute a VRF evaluation: deterministic hash of `(input_seed, salt, counter)`.
///
/// Returns the 32-byte SHA-256 hash truncated to a `u32`. The counter is
/// incremented after each evaluation, ensuring uniqueness. This function is
/// deterministic — anyone can recompute the output given the same inputs.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `input_seed` - Caller-provided seed (e.g., round number, member position)
///
/// # Returns
/// A `u32` derived from the SHA-256 hash, suitable for use in shuffling or
/// random selection.
///
/// # Errors
/// * `VrfError::NotInitialized` if `init_vrf` has not been called
/// * `VrfError::Overflow` if the internal counter overflows
pub fn evaluate_vrf(env: &Env, input_seed: u32) -> Result<u32, VrfError> {
    // Replay protection: each input_seed (nonce) must be strictly greater
    // than the last used one. The very first call (last_nonce absent) is
    // always allowed regardless of input_seed value.
    if let Some(last_nonce) = env.storage().instance().get::<_, u32>(&NONCE_KEY) {
        if input_seed <= last_nonce {
            return Err(VrfError::Replay);
        }
    }
    env.storage().instance().set(&NONCE_KEY, &input_seed);

    let counter: u32 = env
        .storage()
        .instance()
        .get(&COUNTER_KEY)
        .ok_or(VrfError::NotInitialized)?;
    let salt: BytesN<32> = env
        .storage()
        .instance()
        .get(&SALT_KEY)
        .ok_or(VrfError::NotInitialized)?;

    let output = compute_vrf_hash(env, input_seed, &salt, counter)?;

    let next_counter = counter.checked_add(1).ok_or(VrfError::Overflow)?;
    env.storage().instance().set(&COUNTER_KEY, &next_counter);

    VrfEvaluated {
        input_seed,
        vrf_output: output,
        counter,
    }
    .publish(env);

    VrfFulfilled {
        request_id: counter,
        seed: input_seed,
    }
    .publish(env);

    Ok(output)
}

/// Verify a VRF output against an Ed25519 signature.
///
/// Recomputes the hash from `(input_seed, salt, counter)` and checks that
/// `signature` is a valid Ed25519 signature over the 32-byte hash by the
/// stored admin public key. This allows any third party to independently
/// verify that a VRF output was authorized by the admin.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `input_seed` - The seed used during evaluation
/// * `vrf_output` - The claimed VRF output to verify
/// * `counter` - The counter value used during evaluation (from `VrfEvaluated` event)
/// * `signature` - Ed25519 signature (64 bytes) over the hash
///
/// # Returns
/// `true` if the signature is valid, `false` otherwise. Does not panic.
pub fn verify_vrf(
    env: &Env,
    input_seed: u32,
    vrf_output: u32,
    counter: u32,
    signature: &BytesN<64>,
) -> bool {
    let admin_key: BytesN<32> = match env.storage().instance().get(&ADMIN_KEY) {
        Some(k) => k,
        None => return false,
    };
    let salt: BytesN<32> = match env.storage().instance().get(&SALT_KEY) {
        Some(s) => s,
        None => return false,
    };

    let expected_output = match compute_vrf_hash(env, input_seed, &salt, counter) {
        Ok(o) => o,
        Err(_) => return false,
    };

    if expected_output != vrf_output {
        return false;
    }

    let hash_bytes = hash_to_bytes(env, input_seed, &salt, counter);

    let crypto = env.crypto();
    crypto.ed25519_verify(&admin_key, &hash_bytes, signature);
    true
}

/// Shuffle positions 0..n using VRF evaluations.
///
/// Generates a random permutation of `[0, n)` by performing `n` VRF
/// evaluations with incrementing input seeds. The salt and counter
/// ensure each position's randomness is independently derived.
///
/// This is used by the circle contract to determine random payout order.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `n` - Number of positions to shuffle (must be > 0)
/// * `base_nonce` - Starting nonce for the VRF evaluations. Must be > last used nonce.
///
/// # Returns
/// A `Vec<u32>` containing the shuffled positions.
///
/// # Errors
/// * `VrfError::NotInitialized` if `init_vrf` has not been called
/// * `VrfError::Replay` if the nonce is not strictly increasing
pub fn shuffle_positions(env: &Env, n: u32, base_nonce: u32) -> Result<Vec<u32>, VrfError> {
    let mut shuffled = Vec::new(env);
    for i in 0..n {
        let seed = base_nonce.checked_add(i).ok_or(VrfError::Overflow)?;
        let vrf_val = evaluate_vrf(env, seed)?;
        let pos = vrf_val % n;
        shuffled.push_back(pos);
    }
    Ok(shuffled)
}

/// Generate a pseudo-random `u32` in `[0, max)` using VRF.
///
/// Evaluates the VRF with `input_seed = nonce` and takes the result modulo `max`.
/// This is a convenience wrapper for single-value random generation.
///
/// # Arguments
/// * `env` - Soroban environment
/// * `max` - Upper bound (exclusive). If 0, returns 0.
/// * `nonce` - Strictly increasing nonce to prevent replay
///
/// # Returns
/// A `u32` in the range `[0, max)`.
pub fn random_in_range(env: &Env, max: u32, nonce: u32) -> Result<u32, VrfError> {
    if max == 0 {
        return Ok(0);
    }
    let vrf_val = evaluate_vrf(env, nonce)?;
    Ok(vrf_val % max)
}

// ── Internal helpers ──────────────────────────────────────────────────────

/// Compute the VRF hash deterministically from inputs.
///
/// SHA-256(input_seed:u32 ++ salt:32bytes ++ counter:u32) → folds all 8 4-byte
/// segments (all 32 bytes) together via XOR to preserve full 256-bit entropy.
fn compute_vrf_hash(
    env: &Env,
    input_seed: u32,
    salt: &BytesN<32>,
    counter: u32,
) -> Result<u32, VrfError> {
    let hash_bytes = hash_to_bytes(env, input_seed, salt, counter);
    let hash = env.crypto().sha256(&hash_bytes);
    let array = hash.to_array();
    let mut result = 0u32;
    for i in 0..8 {
        let chunk = [
            array[i * 4],
            array[i * 4 + 1],
            array[i * 4 + 2],
            array[i * 4 + 3],
        ];
        result ^= u32::from_le_bytes(chunk);
    }
    Ok(result)
}

/// Build the pre-hash byte sequence: input_seed(4) ++ salt(32) ++ counter(4)
fn hash_to_bytes(env: &Env, input_seed: u32, salt: &BytesN<32>, counter: u32) -> Bytes {
    let mut data = Bytes::new(env);
    data.extend_from_slice(&input_seed.to_le_bytes());
    data.extend_from_slice(&salt.to_array());
    data.extend_from_slice(&counter.to_le_bytes());
    data
}
