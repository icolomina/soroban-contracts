use soroban_sdk::{Address, Env};

use crate::data::{Balance, DataKey, State, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

pub fn write_balance(env: &Env, addr: Address, amount: i128) -> Balance {
    
    let key = DataKey::Balance(addr.clone());
    let interest_rate_key = DataKey::InterestRate;
    let interest_rate: u32 = env.storage().instance().get(&interest_rate_key).unwrap();
    let current_interest = (amount * (interest_rate as i128 / 100)) / 100;

    let balance = Balance {
        deposited: amount, 
        accumulated_interests: current_interest, 
        total: amount + current_interest
    };

    env.storage().persistent().set(&key, &balance);
    env.storage().persistent().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

    balance
}

pub fn reset_balance(env: &Env, addr: Address) {
    let key = DataKey::Balance(addr.clone());
    let deposit_start_key = DataKey::DepositStart(addr.clone());
    let addr_status_key = DataKey::AddressStatus(addr);

    env.storage().persistent().remove(&key);
    env.storage().persistent().remove(&deposit_start_key);
    env.storage().persistent().set(&addr_status_key, &State::Withdrawn);

    env.storage().persistent().extend_ttl(&addr_status_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

}