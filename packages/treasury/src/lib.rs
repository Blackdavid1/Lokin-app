#![cfg_attr(not(test), no_std)]
mod contract;
#[cfg(test)]
mod test;
mod types;
use soroban_sdk::{contract, contractimpl, Address, Env};

/// # Treasury Contract
///
/// Fee collection and fund management for Loki-app circles.
/// Collects fees from circles and allows admin withdrawals.
///
/// ## Example: Initialize the treasury
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use treasury::TreasuryClient;
///
/// let env = Env::default();
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
/// let token = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let treasury_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATREASURY..."));
///
/// let client = TreasuryClient::new(&env, &treasury_id);
/// client.init(&admin, &token);
/// ```
///
/// ## Example: Deposit fee from a circle
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use treasury::TreasuryClient;
///
/// let env = Env::default();
/// let treasury_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATREASURY..."));
/// let from = Address::from_string(&soroban_sdk::String::from_str(&env, "GAFROM..."));
/// let circle_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GACIRCLE..."));
/// let amount = 100_0000000i128;
///
/// let client = TreasuryClient::new(&env, &treasury_id);
/// client.deposit_fee(&from, &amount, &circle_id);
/// ```
///
/// ## Example: Admin withdraw funds
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use treasury::TreasuryClient;
///
/// let env = Env::default();
/// let treasury_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATREASURY..."));
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
/// let to = Address::from_string(&soroban_sdk::String::from_str(&env, "GATO..."));
/// let amount = 50_0000000i128;
///
/// let client = TreasuryClient::new(&env, &treasury_id);
/// client.withdraw(&admin, &to, &amount);
/// ```
///
/// ## Example: Get treasury balance
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use treasury::TreasuryClient;
///
/// let env = Env::default();
/// let treasury_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATREASURY..."));
///
/// let client = TreasuryClient::new(&env, &treasury_id);
/// let balance = client.get_balance();
/// ```
///
/// ## Example: Get all deposits
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use treasury::TreasuryClient;
///
/// let env = Env::default();
/// let treasury_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATREASURY..."));
///
/// let client = TreasuryClient::new(&env, &treasury_id);
/// let deposits = client.get_deposits();
/// ```
///
/// ## Example: Rescue tokens (admin only)
///
/// ```no_run
/// use soroban_sdk::{Address, Env, String};
/// use treasury::TreasuryClient;
///
/// let env = Env::default();
/// let treasury_id = Address::from_string(&soroban_sdk::String::from_str(&env, "GATREASURY..."));
/// let admin = Address::from_string(&soroban_sdk::String::from_str(&env, "GAADMIN..."));
/// let to = Address::from_string(&soroban_sdk::String::from_str(&env, "GATO..."));
/// let token = Address::from_string(&soroban_sdk::String::from_str(&env, "GATOKEN..."));
/// let amount = 100_0000000i128;
///
/// let client = TreasuryClient::new(&env, &treasury_id);
/// client.rescue_tokens(&admin, &to, &token, &amount);
/// ```
#[contract]
pub struct Treasury;
#[contractimpl]
impl Treasury {
    pub fn init(env: Env, admin: Address, token: Address) -> Result<(), types::TreasuryError> {
        contract::init(&env, &admin, &token)
    }
    pub fn deposit_fee(
        env: Env,
        from: Address,
        amount: i128,
        circle_id: Address,
    ) -> Result<(), types::TreasuryError> {
        contract::deposit(&env, &from, amount, &circle_id)
    }
    pub fn withdraw(
        env: Env,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), types::TreasuryError> {
        contract::withdraw(&env, &admin, &to, amount)
    }
    pub fn get_balance(env: Env) -> i128 {
        contract::get_balance(&env)
    }
    pub fn get_deposits(env: Env) -> soroban_sdk::Vec<types::Deposit> {
        contract::get_deposits(&env)
    }
    pub fn rescue_tokens(
        env: Env,
        admin: Address,
        to: Address,
        token: Address,
        amount: i128,
    ) -> Result<(), types::TreasuryError> {
        contract::rescue_tokens(&env, &admin, &to, &token, amount)
    }
    pub fn pause(env: Env, admin: Address) -> Result<(), types::TreasuryError> {
        contract::pause(&env, &admin)
    }
    pub fn unpause(env: Env, admin: Address) -> Result<(), types::TreasuryError> {
        contract::unpause(&env, &admin)
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
