use soroban_sdk::{contracterror, symbol_short, Env, Symbol};

const REENTRANCY_KEY: Symbol = symbol_short!("reent");

/// # Canonical Reentrancy Guard — use this, not a local copy
///
/// `ReentrancyGuard` is the **single, authoritative** reentrancy protection
/// implementation for all Loki-app contracts. Every contract that needs
/// reentrancy protection must import this from `common::reentrancy` rather
/// than maintaining its own local copy. This prevents security fixes from
/// silently diverging between contracts.
///
/// ## How it works
///
/// Uses **temporary** storage (not instance/persistent) so that if a
/// transaction panics or runs out of gas after acquiring the lock, the
/// flag auto-expires after the temporary TTL elapses, preventing a
/// permanent contract lock. A short TTL (2 ledgers) is set explicitly
/// so the contract is usable again within seconds.
///
/// ## Usage
/// ```rust,ignore
/// use common::reentrancy::ReentrancyGuard;
///
/// pub fn my_mutating_fn(env: &Env) -> Result<(), MyError> {
///     let _guard = ReentrancyGuard::new(env).map_err(|_| MyError::ReentrantCall)?;
///     // ... mutating logic ...
///     Ok(())
///     // _guard dropped here — lock released automatically
/// }
/// ```
///
/// ## Why not a local copy?
///
/// A local `reentrancy.rs` per-contract creates a maintenance hazard: if
/// this module receives a security fix (e.g., a TTL adjustment or storage
/// key collision fix), every copy must be updated independently. Any missed
/// update leaves a contract vulnerable. The common module ensures one fix
/// protects all contracts.
pub struct ReentrancyGuard {
    env: Env,
}

impl ReentrancyGuard {
    /// Acquires the reentrancy lock. Returns an error if already locked.
    pub fn new(env: &Env) -> Result<Self, ReentrancyError> {
        let locked: bool = env
            .storage()
            .temporary()
            .get(&REENTRANCY_KEY)
            .unwrap_or(false);
        if locked {
            return Err(ReentrancyError::ReentrantCall);
        }
        env.storage().temporary().set(&REENTRANCY_KEY, &true);
        // Extend TTL to 2 ledgers so the key survives the current tx but
        // auto-expires quickly if the tx panics before drop() runs.
        env.storage().temporary().extend_ttl(&REENTRANCY_KEY, 1, 2);
        Ok(Self { env: env.clone() })
    }
    /// Explicitly leaves / releases the reentrancy lock before drop.
    pub fn leave(&mut self) {
        self.env.storage().temporary().set(&REENTRANCY_KEY, &false);
    }
}

/// Executes closure `f` under the canonical reentrancy guard.
pub fn with_reentrancy_guard<F, R, E>(env: &Env, f: F) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E>,
    E: From<ReentrancyError>,
{
    let _guard = ReentrancyGuard::new(env)?;
    f()
}

impl Drop for ReentrancyGuard {
    fn drop(&mut self) {
        self.leave();
    }
}

#[contracterror]
#[derive(Debug)]
pub enum ReentrancyError {
    ReentrantCall = 1,
}
