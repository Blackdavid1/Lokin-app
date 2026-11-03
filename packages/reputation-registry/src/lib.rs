#![cfg_attr(not(test), no_std)]
mod contract;
pub mod scoring;
pub mod storage;
#[cfg(test)]
mod test;
mod types;
use soroban_sdk::{contract, contractimpl, Address, Env};

/// # Reputation Registry Contract
///
/// On-chain reputation system (MoiScore) for Loki-app circle members.
/// Tracks payment history, circle completions, defaults, and applies inactivity decay.
///
/// ## Example: Initialize the registry
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use reputation_registry::ReputationRegistryClient;
///
/// let env = Env::default();
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
/// let registry_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GAREGISTRY..."));
///
/// let client = ReputationRegistryClient::new(&env, &registry_id);
/// client.init(&admin);
/// ```
///
/// ## Example: Record on-time payment activity
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use reputation_registry::ReputationRegistryClient;
///
/// let env = Env::default();
/// let registry_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GAREGISTRY..."));
/// let member = Address::from_string(&soroban_sdk::String::from_str(&env, "GAMEMBER..."));
/// let circle_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GACIRCLE..."));
/// let amount = 100_0000000i128;
/// let round = 1u32;
///
/// let client = ReputationRegistryClient::new(&env, &registry_id);
/// let new_score = client.record_on_time_payment(&member, &circle_id, &amount, &round);
/// ```
///
/// ## Example: Record circle completion
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use reputation_registry::ReputationRegistryClient;
///
/// let env = Env::default();
/// let registry_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GAREGISTRY..."));
/// let member = Address::from_string(&soroban_sdk::String::from_str(&env, "GAMEMBER..."));
///
/// let client = ReputationRegistryClient::new(&env, &registry_id);
/// let new_score = client.record_circle_completion(&member);
/// ```
///
/// ## Example: Record default
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use reputation_registry::ReputationRegistryClient;
///
/// let env = Env::default();
/// let registry_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GAREGISTRY..."));
/// let member = Address::from_string(&soroban_sdk::String::from_str(&env, "GAMEMBER..."));
///
/// let client = ReputationRegistryClient::new(&env, &registry_id);
/// let new_score = client.record_default(&member);
/// ```
///
/// ## Example: Get member's MoiScore
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use reputation_registry::ReputationRegistryClient;
///
/// let env = Env::default();
/// let registry_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GAREGISTRY..."));
/// let member = Address::from_string(&soroban_sdk::String::from_str(&env, "GAMEMBER..."));
///
/// let client = ReputationRegistryClient::new(&env, &registry_id);
/// let score = client.get_moi_score(&member);
/// ```
///
/// ## Example: Get member's tier
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use reputation_registry::ReputationRegistryClient;
///
/// let env = Env::default();
/// let registry_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GAREGISTRY..."));
/// let member = Address::from_string(&soroban_sdk::String::from_str(&env, "GAMEMBER..."));
///
/// let client = ReputationRegistryClient::new(&env, &registry_id);
/// let tier = client.get_moi_tier(&member);
/// ```
///
/// ## Example: Apply inactivity decay
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use reputation_registry::ReputationRegistryClient;
///
/// let env = Env::default();
/// let registry_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GAREGISTRY..."));
/// let member = Address::from_string(&soroban_sdk::String::from_str(&env, "GAMEMBER..."));
/// let days_inactive = 30u64;
///
/// let client = ReputationRegistryClient::new(&env, &registry_id);
/// let new_score = client.apply_inactivity_decay(&member, &days_inactive);
/// ```
#[contract]
pub struct ReputationRegistry;
#[contractimpl]
impl ReputationRegistry {
    pub fn init(env: Env, admin: Address) -> Result<(), types::ReputationError> {
        contract::init(&env, &admin)
    }
    pub fn record_activity(
        env: Env,
        user: Address,
        activity_type: u32,
        score_impact: u32,
    ) -> Result<(), types::ReputationError> {
        contract::record(&env, &user, activity_type, score_impact)
    }
    pub fn get_score(env: Env, user: Address) -> Result<types::MoiScore, types::ReputationError> {
        contract::get_score(&env, &user)
    }
    pub fn get_history(
        env: Env,
        user: Address,
        page: u32,
        page_size: u32,
    ) -> soroban_sdk::Vec<types::Activity> {
        contract::get_history(&env, &user, page, page_size)
    }
    pub fn pause_registry(env: Env, admin: Address) -> Result<(), types::ReputationError> {
        contract::pause(&env, &admin)
    }
    pub fn unpause_registry(env: Env, admin: Address) -> Result<(), types::ReputationError> {
        contract::unpause(&env, &admin)
    }
    pub fn calc_collateral(env: Env, member: Address) -> u32 {
        scoring::calculate_collateral(&env, &member)
    }
    pub fn calc_max_size(env: Env, member: Address) -> u32 {
        scoring::max_circle_size(&env, &member)
    }
    pub fn calc_max_contrib(env: Env, member: Address) -> i128 {
        scoring::max_contribution(&env, &member)
    }

    // ── #53: enhanced on-chain scoring (tier-aware, streak/volume-aware) ──
    pub fn get_moi_score(env: Env, member: Address) -> u32 {
        crate::storage::get_score(&env, &member)
    }
    pub fn get_moi_tier(env: Env, member: Address) -> u32 {
        scoring::get_tier(crate::storage::get_score(&env, &member))
    }

    pub fn record_on_time_payment(
        env: Env,
        member: Address,
        circle_id: Address,
        amount: i128,
        round: u32,
    ) -> Result<u32, types::ReputationError> {
        common::pause::when_not_paused(&env).map_err(|_| types::ReputationError::ContractPaused)?;
        member.require_auth();
        Ok(scoring::record_on_time_payment(&env, &member, &circle_id, amount, round))
    }
    pub fn record_circle_completion(
        env: Env,
        member: Address,
    ) -> Result<u32, types::ReputationError> {
        common::pause::when_not_paused(&env).map_err(|_| types::ReputationError::ContractPaused)?;
        member.require_auth();
        Ok(scoring::record_circle_completion(&env, &member))
    }
    pub fn record_default(env: Env, member: Address) -> Result<u32, types::ReputationError> {
        common::pause::when_not_paused(&env).map_err(|_| types::ReputationError::ContractPaused)?;
        member.require_auth();
        Ok(scoring::record_default(&env, &member))
    }
    pub fn apply_inactivity_decay(
        env: Env,
        member: Address,
        days_inactive: u64,
    ) -> Result<u32, types::ReputationError> {
        common::pause::when_not_paused(&env).map_err(|_| types::ReputationError::ContractPaused)?;
        member.require_auth();
        Ok(scoring::apply_inactivity_decay(&env, &member, days_inactive))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_smoke_compile() {
        assert!(true);
    }
    #[test]
    fn test_types_compile() {
        assert!(true);
    }
}
