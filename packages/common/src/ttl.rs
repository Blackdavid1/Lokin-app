//! TTL (time-to-live) extension helpers shared by all Loki-app contracts.
//!
//! Soroban archives persistent and instance entries once their TTL expires,
//! so any contract that reads or writes them must keep extending that TTL.
//! The thresholds below are documented in `docs/TTL.md`.

use soroban_sdk::{Env, IntoVal, Val};

/// Approximate ledgers per day at the ~5 second network ledger close time.
pub const LEDGERS_PER_DAY: u32 = 17_280;

/// Persistent entries are extended once fewer than 30 days remain, back to 90 days.
pub const PERSISTENT_TTL_THRESHOLD: u32 = 30 * LEDGERS_PER_DAY;
pub const PERSISTENT_TTL_EXTEND_TO: u32 = 90 * LEDGERS_PER_DAY;

/// Instance storage is extended once fewer than 7 days remain, back to 30 days.
pub const INSTANCE_TTL_THRESHOLD: u32 = 7 * LEDGERS_PER_DAY;
pub const INSTANCE_TTL_EXTEND_TO: u32 = 30 * LEDGERS_PER_DAY;

/// Extends the TTL of a persistent entry.
///
/// No-op when the entry is absent, because `extend_ttl` traps the host on a missing key.
pub fn extend_persistent<K: IntoVal<Env, Val>>(env: &Env, key: &K) {
    let persistent = env.storage().persistent();
    if persistent.has(key) {
        persistent.extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    }
}

/// Extends the TTL of the contract instance entry and its code.
pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}
