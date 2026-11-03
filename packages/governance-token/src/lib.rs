#![cfg_attr(not(test), no_std)]

mod contract;
#[cfg(test)]
mod test;
pub mod types;

use crate::types::{AllowanceData, TokenError};
use soroban_sdk::{contract, contractimpl, Address, Env, MuxedAddress, String};

/// # Governance Token Contract
///
/// SEP-41 (Stellar Token Interface) compatible governance token with extra
/// administrative controls: mint, burn, clawback, freeze/unfreeze and
/// allowlist mode. Used for voting power in Loki-app governance.
///
/// The token surface follows SEP-41: `balance`, `allowance`, `approve`,
/// `transfer`, `transfer_from`, `burn`, `burn_from`, `decimals`, `name` and
/// `symbol` use the canonical names, arguments and semantics, and the token
/// emits the canonical Stellar token events through `soroban-token-sdk`.
/// Governance-only extensions (`mint`, `clawback`, `admin_burn_from`,
/// `allowance_data`, `total_supply`, freeze/allowlist/pause controls) are
/// layered on top.
///
/// ## Example: Initialize the token
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// client.initialize(
///     &admin,
///     &soroban_sdk::String::from_str(&env, "Loki-app Governance Token"),
///     &soroban_sdk::String::from_str(&env, "MOIST"),
///     &18,
/// );
/// ```
///
/// ## Example: Transfer tokens
///
/// ```no_run
/// use soroban_sdk::{Address, Env, MuxedAddress, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let from = Address::from_string(&soroban_sdk::String::from_str(&env, "GAFROM..."));
/// let to = MuxedAddress::from(Address::from_string(&soroban_sdk::String::from_str(&env, "GATO...")));
/// let amount = 100_0000000i128;
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// client.transfer(&from, &to, &amount);
/// ```
///
/// ## Example: Transfer from (with allowance)
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let spender = Address::from_string(&soroban_sdk::String::from_str(&env, "GASPENDER..."));
/// let from = Address::from_string(&soroban_sdk::String::from_str(&env, "GAFROM..."));
/// let to = Address::from_string(&soroban_sdk::String::from_str(&env, "GATO..."));
/// let amount = 100_0000000i128;
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// client.transfer_from(&spender, &from, &to, &amount);
/// ```
///
/// ## Example: Approve spending
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let owner = Address::from_string(&soroban_sdk::String::from_str(&env, "GAOWNER..."));
/// let spender = Address::from_string(&soroban_sdk::String::from_str(&env, "GASPENDER..."));
/// let amount = 100_0000000i128;
/// let expiration_ledger = 1000u32;
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// client.approve(&owner, &spender, &amount, &expiration_ledger);
/// ```
///
/// ## Example: Check balance
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let account = Address::from_string(&soroban_sdk::String::from_str(&env, "GAACCOUNT..."));
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// let balance = client.balance(&account);
/// ```
///
/// ## Example: Check allowance
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let owner = Address::from_string(&soroban_sdk::String::from_str(&env, "GAOWNER..."));
/// let spender = Address::from_string(&soroban_sdk::String::from_str(&env, "GASPENDER..."));
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// let allowance = client.allowance(&owner, &spender);
/// ```
///
/// ## Example: Mint tokens (admin only)
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
/// let to = Address::from_string(&soroban_sdk::String::from_str(&env, "GATO..."));
/// let amount = 100_0000000i128;
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// client.mint(&admin, &to, &amount);
/// ```
///
/// ## Example: Burn tokens
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let from = Address::from_string(&soroban_sdk::String::from_str(&env, "GAFROM..."));
/// let amount = 100_0000000i128;
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// client.burn(&from, &amount);
/// ```
///
/// ## Example: Get token metadata
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use governance_token::GovernanceTokenClient;
///
/// let env = Env::default();
/// let token_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
///
/// let client = GovernanceTokenClient::new(&env, &token_id);
/// let name = client.name();
/// let symbol = client.symbol();
/// let decimals = client.decimals();
/// let total_supply = client.total_supply();
/// ```
#[contract]
pub struct GovernanceToken;

#[contractimpl]
impl GovernanceToken {
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
    ) -> Result<(), TokenError> {
        contract::initialize(&env, &admin, &name, &symbol, decimals)
    }

    /// SEP-41 `transfer`: sends `amount` from `from` to the (possibly muxed)
    /// `to` address.
    pub fn transfer(
        env: Env,
        from: Address,
        to: MuxedAddress,
        amount: i128,
    ) -> Result<(), TokenError> {
        contract::transfer(&env, &from, &to, amount)
    }

    /// SEP-41 `transfer_from`: `spender` moves `amount` out of `from`'s
    /// balance, consuming its allowance.
    pub fn transfer_from(
        env: Env,
        spender: Address,
        from: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        contract::transfer_from(&env, &spender, &from, &to, amount)
    }

    /// SEP-41 `approve`: overwrites `spender`'s allowance on `from`.
    pub fn approve(
        env: Env,
        owner: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) -> Result<(), TokenError> {
        contract::approve(&env, &owner, &spender, amount, expiration_ledger)
    }

    /// SEP-41 `balance`: the amount of tokens held by `account`.
    pub fn balance(env: Env, account: Address) -> i128 {
        contract::balance(&env, &account)
    }

    /// SEP-41 `allowance`: the amount `spender` may still spend from `owner`.
    ///
    /// Expired allowances are reported as zero, as mandated by SEP-41.
    pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
        contract::allowance(&env, &owner, &spender)
    }

    /// Governance extension: the raw allowance record, including the ledger at
    /// which the allowance expires.
    pub fn allowance_data(env: Env, owner: Address, spender: Address) -> AllowanceData {
        contract::allowance_data(&env, &owner, &spender)
    }

    pub fn total_supply(env: Env) -> i128 {
        contract::total_supply(&env)
    }

    pub fn name(env: Env) -> String {
        contract::name(&env)
    }

    pub fn symbol(env: Env) -> String {
        contract::symbol(&env)
    }

    pub fn decimals(env: Env) -> u32 {
        contract::decimals(&env)
    }

    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) -> Result<(), TokenError> {
        contract::mint(&env, &admin, &to, amount)
    }

    /// SEP-41 `burn`: burns `amount` from `from`'s balance.
    pub fn burn(env: Env, from: Address, amount: i128) -> Result<(), TokenError> {
        contract::burn(&env, &from, amount)
    }

    /// SEP-41 `burn_from`: burns `amount` from `from`, consuming `spender`'s
    /// allowance.
    pub fn burn_from(
        env: Env,
        spender: Address,
        from: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        contract::burn_from(&env, &spender, &from, amount)
    }

    /// Admin-authorized burn (does not consume an allowance).
    pub fn admin_burn_from(
        env: Env,
        admin: Address,
        from: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        contract::admin_burn_from(&env, &admin, &from, amount)
    }

    pub fn clawback(
        env: Env,
        admin: Address,
        from: Address,
        amount: i128,
    ) -> Result<(), TokenError> {
        contract::clawback(&env, &admin, &from, amount)
    }

    pub fn freeze(env: Env, admin: Address, account: Address) -> Result<(), TokenError> {
        contract::freeze(&env, &admin, &account)
    }

    pub fn unfreeze(env: Env, admin: Address, account: Address) -> Result<(), TokenError> {
        contract::unfreeze(&env, &admin, &account)
    }

    pub fn is_frozen(env: Env, account: Address) -> bool {
        contract::is_frozen(&env, &account)
    }

    pub fn set_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), TokenError> {
        contract::set_admin(&env, &admin, &new_admin)
    }

    pub fn get_admin(env: Env) -> Result<Address, TokenError> {
        contract::get_admin(&env)
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), TokenError> {
        contract::pause(&env, &admin)
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), TokenError> {
        contract::unpause(&env, &admin)
    }

    pub fn set_allowlist_mode(env: Env, admin: Address, enabled: bool) -> Result<(), TokenError> {
        contract::set_allowlist_mode(&env, &admin, enabled)
    }

    pub fn is_allowlist_mode_enabled(env: Env) -> bool {
        contract::is_allowlist_mode_enabled(&env)
    }

    pub fn add_to_allowlist(env: Env, admin: Address, account: Address) -> Result<(), TokenError> {
        contract::add_to_allowlist(&env, &admin, &account)
    }

    pub fn remove_from_allowlist(
        env: Env,
        admin: Address,
        account: Address,
    ) -> Result<(), TokenError> {
        contract::remove_from_allowlist(&env, &admin, &account)
    }

    pub fn is_allowlisted(env: Env, account: Address) -> bool {
        contract::is_allowlisted(&env, &account)
    }
}
