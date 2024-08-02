use soroban_sdk::{Address, Env};

use crate::data::{DataKey, State, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD};

pub fn set_withdrawal_ts(env: &Env, addr: Address) -> u64 {
    
    let claim_ttl_key = DataKey::ClaimBlockDays;
    let addr_claim_key = DataKey::ClaimTime(addr.clone());
    let addr_status_key = DataKey::AddressStatus(addr.clone());

    let claim_ttl: u64 = env.storage().instance().get(&claim_ttl_key).unwrap();
    let withdrawal_ts =  env.ledger().timestamp() + (claim_ttl * 24 * 3600);
    env.storage().persistent().set(&addr_claim_key, &withdrawal_ts);
    env.storage().persistent().set(&addr_status_key, &State::Deposited);
    env.storage().persistent().extend_ttl(&addr_claim_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    return withdrawal_ts;
}

pub fn check_address_claimable(env: &Env, addr: Address) -> bool {
    let addr_claim_key = DataKey::ClaimTime(addr);
    let mut result = false;
    if let Some(ts) = env.storage().persistent().get(&addr_claim_key) {
        result = env.ledger().timestamp() >= ts;
        env.storage().persistent().extend_ttl(&addr_claim_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
    }
    
    result

}