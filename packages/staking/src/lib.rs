#![cfg_attr(not(test), no_std)]

mod contract;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};

/// # Staking Contract
///
/// Token staking with time-weighted voting power for Loki-app governance.
/// Supports staking, unstaking, claiming rewards, and slashing.
///
/// ## Example: Initialize staking
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use staking::StakingClient;
///
/// let env = Env::default();
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
/// let token = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let staking_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GASTAKING..."));
///
/// let client = StakingClient::new(&env, &staking_id);
/// client.init(&admin, &token);
/// ```
///
/// ## Example: Stake tokens
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use staking::StakingClient;
///
/// let env = Env::default();
/// let staking_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GASTAKING..."));
/// let user = Address::from_string(&soroban_sdk::String::from_str(&env, "GAUSER..."));
/// let amount = 100_0000000i128;
/// let period_months = 12u32;
///
/// let client = StakingClient::new(&env, &staking_id);
/// client.stake(&user, &amount, &period_months);
/// ```
///
/// ## Example: Unstake tokens
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use staking::StakingClient;
///
/// let env = Env::default();
/// let staking_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GASTAKING..."));
/// let user = Address::from_string(&soroban_sdk::String::from_str(&env, "GAUSER..."));
///
/// let client = StakingClient::new(&env, &staking_id);
/// client.unstake(&user);
/// ```
///
/// ## Example: Claim rewards
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use staking::StakingClient;
///
/// let env = Env::default();
/// let staking_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GASTAKING..."));
/// let user = Address::from_string(&soroban_sdk::String::from_str(&env, "GAUSER..."));
///
/// let client = StakingClient::new(&env, &staking_id);
/// client.claim(&user);
/// ```
///
/// ## Example: Get voting power
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use staking::StakingClient;
///
/// let env = Env::default();
/// let staking_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GASTAKING..."));
/// let user = Address::from_string(&soroban_sdk::String::from_str(&env, "GAUSER..."));
///
/// let client = StakingClient::new(&env, &staking_id);
/// let voting_power = client.get_voting_power(&user);
/// ```
///
/// ## Example: Get stake position
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use staking::StakingClient;
///
/// let env = Env::default();
/// let staking_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GASTAKING..."));
/// let user = Address::from_string(&soroban_sdk::String::from_str(&env, "GAUSER..."));
///
/// let client = StakingClient::new(&env, &staking_id);
/// let stake = client.get_stake(&user);
/// ```
///
/// ## Example: Get total staked
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use staking::StakingClient;
///
/// let env = Env::default();
/// let staking_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GASTAKING..."));
///
/// let client = StakingClient::new(&env, &staking_id);
/// let total = client.get_total_staked();
/// ```
///
/// ## Example: Create slash notice
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use staking::StakingClient;
///
/// let env = Env::default();
/// let staking_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GASTAKING..."));
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
/// let user = Address::from_string(&soroban_sdk::String::from_str(&env, "GAUSER..."));
/// let amount = 100_0000000i128;
///
/// let client = StakingClient::new(&env, &staking_id);
/// client.create_slash_notice(&admin, &user, &amount);
/// ```
#[contract]
pub struct Staking;

#[contractimpl]
impl Staking {
    pub fn init(env: Env, admin: Address, token: Address) {
        contract::init(&env, &admin, &token);
    }

    pub fn stake(
        env: Env,
        user: Address,
        amount: i128,
        period_months: u32,
    ) -> Result<(), types::StakingError> {
        contract::stake(&env, &user, amount, period_months)
    }

    pub fn unstake(env: Env, user: Address) -> Result<(), types::StakingError> {
        contract::unstake(&env, &user)
    }

    pub fn claim(env: Env, user: Address) -> Result<(), types::StakingError> {
        contract::claim(&env, &user)
    }

    pub fn get_voting_power(env: Env, user: Address) -> i128 {
        contract::get_voting_power(&env, &user)
    }

    pub fn get_stake(env: Env, user: Address) -> Option<types::StakePosition> {
        contract::get_stake(&env, &user)
    }

    pub fn get_unbonding(env: Env, user: Address) -> Option<types::UnbondingPosition> {
        contract::get_unbonding(&env, &user)
    }

    pub fn get_total_staked(env: Env) -> i128 {
        contract::get_total_staked(&env)
    }

    /// Returns only the raw staked token amount for a user (0 if none).
    /// Convenience alternative to get_stake() for callers that don't need
    /// the full StakePosition.
    pub fn get_stake_amount(env: Env, user: Address) -> i128 {
        contract::get_stake_amount(&env, &user)
    }

    /// Returns the list of all addresses that currently hold an active stake.
    /// Maintained automatically by stake() / unstake().
    pub fn get_all_stakers(env: Env) -> soroban_sdk::Vec<Address> {
        contract::get_all_stakers(&env)
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), types::StakingError> {
        contract::pause(&env, &admin)
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), types::StakingError> {
        contract::unpause(&env, &admin)
    }

    pub fn update_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), types::StakingError> {
        contract::update_admin(&env, &current_admin, &new_admin)
    }

    pub fn top_up(env: Env, user: Address, amount: i128) -> Result<(), types::StakingError> {
        contract::top_up(&env, &user, amount)
    }

    pub fn get_time_weighted_amount(env: Env, user: Address) -> i128 {
        contract::get_time_weighted_amount(&env, &user)
    }
    pub fn get_slash_notice_period(env: Env) -> u64 {
        contract::get_slash_notice_period(&env)
    }

    pub fn set_slash_notice_period(
        env: Env,
        admin: Address,
        period_seconds: u64,
    ) -> Result<(), types::StakingError> {
        contract::set_slash_notice_period(&env, &admin, period_seconds)
    }

    pub fn get_slash_notice(env: Env, user: Address) -> Option<(i128, u64, Address)> {
        contract::get_slash_notice(&env, &user)
    }

    pub fn cancel_slash(
        env: Env,
        admin: Address,
        user: Address,
    ) -> Result<(), types::StakingError> {
        contract::cancel_slash(&env, &admin, &user)
    }

    pub fn execute_slash(
        env: Env,
        admin: Address,
        user: Address,
    ) -> Result<(), types::StakingError> {
        contract::execute_slash(&env, &admin, &user)
    }
    /// #523 — Slash a staker's collateral proportional to shortfall.
    /// Returns the actual amount slashed (min of shortfall and stake).
    pub fn slash(
        env: Env,
        admin: Address,
        user: Address,
        shortfall: i128,
    ) -> Result<i128, types::StakingError> {
        contract::slash(&env, &admin, &user, shortfall)
    }

    /// #467 — Open a slashing notice period; execute after the period elapses
    /// or cancel it while the notice is still active.
    pub fn create_slash_notice(
        env: Env,
        admin: Address,
        user: Address,
        amount: i128,
    ) -> Result<(), types::StakingError> {
        contract::create_slash_notice(&env, &admin, &user, amount)
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
        // Verify contract types compile correctly
        assert!(true);
    }
}
