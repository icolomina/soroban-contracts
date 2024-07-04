use soroban_sdk::{Address, Env};

use crate::data::{BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD, DataKey, Balance};
use crate::interest::calculate_interest;

pub fn write_balance(env: &Env, addr: Address, amount: i128) -> i128 {
    let key = DataKey::Balance(addr.clone());
    let deposit_start_key = DataKey::DepositStart(addr.clone());
    let balance: i128;
    if let Some(b) = env.storage().persistent().get::<DataKey, i128>(&key) {
        balance = b + amount;
    } else {
        env.storage().persistent().set(&deposit_start_key, &env.ledger().timestamp());
        balance = amount;
    }

    env.storage().persistent().set(&key, &balance);
    env.storage().persistent().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().extend_ttl(&deposit_start_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    balance
}

pub fn get_balance(env: &Env, addr: Address) -> Balance {
    let key = DataKey::Balance(addr.clone());
    let mut balance = Balance {deposited: 0, accumulated_interests: 0, total: 0};

    if let Some(b) = env.storage().persistent().get::<DataKey, i128>(&key) {

        env.storage().persistent().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

        let deposit_start_key = DataKey::DepositStart(addr.clone());
        let deposit_start_ts = env.storage().persistent().get::<DataKey, u64>(&deposit_start_key).unwrap();
        let current_ts = env.ledger().timestamp();
        let days = ((current_ts - deposit_start_ts) / 3600) / 24;
        if days > 0 {
            let interest_rate_key = DataKey::InterestRate;
            let interest_rate: u32 = env.storage().instance().get(&interest_rate_key).unwrap();
            let current_interest = calculate_interest(b, interest_rate, days);
            balance.deposited = b;
            balance.accumulated_interests = current_interest;
            balance.total = b + current_interest;
        } else {
            balance.deposited = b;
            balance.accumulated_interests = 0;
            balance.total = b;
        }
    } 

    balance

}

pub fn reset_balance(env: &Env, addr: Address) {
    let key = DataKey::Balance(addr.clone());
    let deposit_start_key = DataKey::DepositStart(addr.clone());

    env.storage().persistent().set(&key, &0_i128);
    env.storage().persistent().extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    env.storage().persistent().extend_ttl(&deposit_start_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}
