#![no_std]

/// # Common Utilities for Loki-app
///
/// This crate provides shared utilities used across all Loki-app Soroban contracts:
/// - **math**: Safe arithmetic operations (checked add, sub, mul, div, fee calculation)
/// - **pause**: Emergency pause/unpause functionality
/// - **reentrancy**: Reentrancy guard for contract protection
/// - **vrf**: Verifiable Random Function for fair random selection
/// - **access**: Role-based access control
/// - **token_transfer**: Safe token transfer helpers
/// - **ttl**: Storage TTL extension for persistent and instance entries
/// - **upgrade**: Proxy/implementation upgrade pattern
/// - **types**: Shared type definitions
/// - **explorer**: Block explorer link generation
///
/// ## Example: Using safe math
///
/// ```
/// use common::math::{safe_add, safe_mul, apply_fee};
///
/// let a = 100i128;
/// let b = 200i128;
/// let sum = safe_add(a, b).unwrap(); // 300
/// let product = safe_mul(a, b).unwrap(); // 20000
/// let (net, fee) = apply_fee(1000, 100).unwrap(); // 990, 10 (1% fee)
/// assert_eq!(sum, 300);
/// assert_eq!(product, 20000);
/// assert_eq!(net, 990);
/// assert_eq!(fee, 10);
/// ```
///
/// ## Example: Using reentrancy guard
///
/// ```no_run
/// use soroban_sdk::Env;
/// use common::reentrancy::ReentrancyGuard;
///
/// let env = Env::default();
/// let guard = ReentrancyGuard::new(&env).expect("not reentrant");
/// // ... perform state changes ...
/// drop(guard); // releases the guard
/// ```
///
/// ## Example: Using pause functionality
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use common::pause::{pause, unpause, when_not_paused};
///
/// let env = Env::default();
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
///
/// pause(&env, &admin).unwrap();
/// // Contract is now paused - mutating functions will return ContractPaused
///
/// unpause(&env, &admin).unwrap();
/// // Contract is unpaused
///
/// when_not_paused(&env).unwrap(); // Returns Ok(()) when not paused
/// ```
pub mod access;
pub mod explorer;
pub mod math;
pub mod pause;
pub mod reentrancy;
pub mod token_transfer;
pub mod ttl;
pub mod types;
pub mod upgrade;
pub mod vrf;
