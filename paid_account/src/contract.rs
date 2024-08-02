use core::u64;

use soroban_sdk::token::TokenClient;
use soroban_sdk::{contract, contractimpl, token, Address, Env};
use crate::balance::{reset_balance, write_balance};
use crate::data::{
    Balance, DataKey, Error, State, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD
};
use crate::claim::{set_withdrawal_ts, check_address_claimable};

fn get_state(env: &Env) -> State {
    let state_key = DataKey::State;
    if let Some(s) = env.storage().instance().get(&state_key) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        return s;
    }
    return State::Pending;
}

fn check_contract_initialized(e: &Env) -> bool {
    let state = get_state(&e);
    if state == State::Initialized || state == State::FinancingReached {
        return true;
    }

    false
}

fn get_token(env: &Env) -> TokenClient {
    let token_key = DataKey::Token;
    let token_addr = env.storage().instance().get(&token_key).unwrap();
    let tk = token::Client::new(&env, &token_addr);
    tk
}


#[contract]
pub struct PaidAccount;

#[contractimpl]
impl PaidAccount {

    pub fn init(env: Env, admin_addr: Address, token_addr: Address, i_rate: u32, claim_block_days: u64) -> Result<bool, Error>{

        if check_contract_initialized(&env) {
            return Err(Error::ContractAlreadyInitialized);
        }

        if i_rate == 0 {
            return Err(Error::RateMustBeGreaterThan0);
        }

        if claim_block_days == 0 {
            return Err(Error::DepositTtlMustBeGreaterThan0);
        }

        admin_addr.require_auth();

        let interest_rate_key = DataKey::InterestRate;
        let admin_addr_key = DataKey::Admin;
        let token_key = DataKey::Token;
        let state_key = DataKey::State;
        let claim_block_days_key = DataKey::ClaimBlockDays;

        env.storage().instance().set(&admin_addr_key, &admin_addr);
        env.storage().instance().set(&token_key, &token_addr);
        env.storage().instance().set(&state_key, &State::Initialized);
        env.storage().instance().set(&interest_rate_key, &i_rate);
        env.storage().instance().set(&claim_block_days_key, &claim_block_days);

        Ok(true)
    }

    pub fn admin_deposit(env: Env, amount: i128) -> Result<i128, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        if amount <= 0  {
            return Err(Error::AmountLessOrEqualThan0);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let admin_addr_key = DataKey::Admin;
        let admin_addr: Address = env.storage().instance().get(&admin_addr_key).unwrap();
        admin_addr.require_auth();

        let tk = get_token(&env);
        tk.transfer(&admin_addr, &env.current_contract_address(), &amount);

        let contract_address_balance = tk.balance(&env.current_contract_address());
        Ok(contract_address_balance)
    }
    

    
    pub fn user_deposit(env: Env, addr: Address, amount: i128) -> Result<Balance, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        if amount <= 0  {
            return Err(Error::AmountLessOrEqualThan0);
        }

        let state: State = env.storage().instance().get(&DataKey::State).unwrap();
        if state == State::FinancingReached {
            return Err(Error::ContractFinancingReached);
        }

        let addr_status_key = DataKey::AddressStatus(addr.clone());
        let addr_status = env.storage().persistent().get(&addr_status_key).unwrap_or_else(|| State::NoDeposited);
        if addr_status == State::Deposited {
            return Err(Error::AddressAlreadyDeposited);
        }

        addr.require_auth();
        set_withdrawal_ts(&env, addr.clone());

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let tk = get_token(&env);
        tk.transfer(&addr, &env.current_contract_address(), &amount);

        let balance = write_balance(&env, addr, amount);
        Ok(balance)
    }

    pub fn get_contract_balance(env: Env) -> Result<i128, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        let admin_key = DataKey::Admin;
        let admin_addr: Address = env.storage().instance().get(&admin_key).unwrap();
        admin_addr.require_auth();

        let tk = get_token(&env);
        let contract_balance = tk.balance(&env.current_contract_address());
        Ok(contract_balance)
    }

    pub fn user_withdrawal(env: Env, addr: Address) -> Result<i128, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        if !check_address_claimable(&env, addr.clone()) {
            return Err(Error::AddressNotClaimableYet);
        }

        addr.require_auth();
        let addr_balance: Balance = env.storage().persistent().get(&DataKey::Balance(addr.clone())).unwrap();
        let tk = get_token(&env);

        if tk.balance(&env.current_contract_address()) < addr_balance.total {
            return Err(Error::ContractInsufficientBalance);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        tk.transfer(&env.current_contract_address(), &addr, &addr_balance.total);
        reset_balance(&env, addr);
        Ok(addr_balance.total)
    }

    pub fn admin_withdrawal(env: Env, amount: i128) -> Result<i128, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        if amount <= 0  {
            return Err(Error::AmountLessOrEqualThan0);
        }

        let admin_key = DataKey::Admin;
        let admin_addr: Address = env.storage().instance().get(&admin_key).unwrap();
        admin_addr.require_auth();

        let tk = get_token(&env);
        if tk.balance(&env.current_contract_address()) < amount {
            return Err(Error::ContractInsufficientBalance);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        tk.transfer(&env.current_contract_address(), &admin_addr, &amount);
        Ok(tk.balance(&env.current_contract_address()))

    }

    pub fn stop_deposits(env: Env) -> Result<bool, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        let admin_key = DataKey::Admin;
        let admin_addr: Address = env.storage().instance().get(&admin_key).unwrap();
        admin_addr.require_auth();

        let state_key = DataKey::State;
        env.storage().instance().set(&state_key, &State::FinancingReached);

        Ok(true)
    }
}