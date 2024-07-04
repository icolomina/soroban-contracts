use soroban_sdk::token::TokenClient;
use soroban_sdk::{contract, contractimpl, token, Address, Env};
use crate::balance::{get_balance, reset_balance, write_balance};
use crate::data::{
    INSTANCE_LIFETIME_THRESHOLD,
    INSTANCE_BUMP_AMOUNT,  
    Balance,
    DataKey,
    State,
    Error
};

fn get_state(env: &Env) -> State {
    let state_key = DataKey::State;
    if let Some(s) = env.storage().instance().get(&state_key) {
        env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        return s;
    }
    return State::Pending;
}

fn check_contract_initialized(e: &Env) -> bool {
    if get_state(&e) == State::Initialized {
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

    pub fn init(env: Env, admin_addr: Address, token_addr: Address, i_rate: u32) -> Result<bool, Error>{

        admin_addr.require_auth();
        let interest_rate_key = DataKey::InterestRate;
        let admin_addr_key = DataKey::Admin;
        let token_key = DataKey::Token;
        let state_key = DataKey::State;

        env.storage().instance().set(&admin_addr_key, &admin_addr);
        env.storage().instance().set(&token_key, &token_addr);
        env.storage().instance().set(&state_key, &State::Initialized);
        env.storage().instance().set(&interest_rate_key, &i_rate);
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
    

    
    pub fn user_deposit(env: Env, addr: Address, amount: i128) -> Result<i128, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        if amount <= 0  {
            return Err(Error::AmountLessOrEqualThan0);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        addr.require_auth();
        let tk = get_token(&env);
        tk.transfer(&addr, &env.current_contract_address(), &amount);

        let new_balance = write_balance(&env, addr, amount);
        Ok(new_balance)
    }

    pub fn get_balance(env: Env, addr: Address) -> Result<Balance, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let balance = get_balance(&env, addr);
        Ok(balance)
    }

    pub fn user_withdrawal(env: Env, addr: Address, amount: i128) -> Result<Balance, Error> {

        if !check_contract_initialized(&env) {
            return Err(Error::ContractNotInitialized);
        }

        if amount <= 0  {
            return Err(Error::AmountLessOrEqualThan0);
        }

        addr.require_auth();
        let addr_balance: Balance = get_balance(&env, addr.clone());

        if addr_balance.total < amount {
            return Err(Error::AddressInsufficientBalance);
        }

        let tk = get_token(&env);
        if tk.balance(&env.current_contract_address()) <= amount {
            return Err(Error::ContractInsufficientBalance);
        }

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        tk.transfer(&env.current_contract_address(), &addr, &amount);
        let difference = addr_balance.total - amount;
        if difference == 0 {
            reset_balance(&env, addr.clone());

        } else {
            reset_balance(&env, addr.clone());
            write_balance(&env, addr.clone(), difference);
        }
        
        let current_balance = get_balance(&env, addr.clone());
        Ok(current_balance)
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
}