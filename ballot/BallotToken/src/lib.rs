#![no_std]

use soroban_sdk::{contract, contracttype, contractimpl, contracterror, symbol_short, Address, Env, Symbol};
pub const TOKEN_ADMIN: Symbol = symbol_short!("t_admin");

pub const DAY_IN_LEDGERS: u32 = 17280;
pub const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    TokenAlreadyInitialized = 1,
    AddressAlreadyHoldsToken = 2,
    AddressDoesNotHoldToken = 3,
    AddressAlreadyHasAllowance = 4,
    ExpirationLedgerLessThanCurrentLedger = 5
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Balance(Address),
    Allowance(Address),
    Blocking(Address)
}

fn has_admin(e: &Env) -> bool {
    let has_admin = e.storage().instance().has(&TOKEN_ADMIN);
    has_admin
}

fn get_balance(e: &Env, addr: Address)-> u32 {
    let key = DataKey::Balance(addr);
    if let Some(b) = e.storage().persistent().get::<DataKey, u32>(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

        return b;
    }

    0
}

fn get_allowance(e: &Env, addr: Address) -> bool {
    let allowance_key = DataKey::Allowance(addr);
    if let Some(_a) = e.storage().temporary().get::<_, Address>(&allowance_key) {
        return true;
    }

    false
}

fn get_blocking(e: &Env, addr: Address) -> bool {
    let blocking_key = DataKey::Blocking(addr);
    if let Some(_b) = e.storage().temporary().get::<_, Address>(&blocking_key) {
        return true;
    }

    false
}


#[contract]
pub struct BallotToken;

#[contractimpl]
impl BallotToken {

    pub fn initialize(e: Env, admin: Address) -> Result<bool, Error> {

        if has_admin(&e) {
            return Err(Error::TokenAlreadyInitialized);
        }

        e.storage().instance().set(&TOKEN_ADMIN, &admin);
        Ok(true)
        
    }

    pub fn mint(e: Env, addr: Address) -> Result<u32, Error> {
        
        let admin: Address = e.storage().instance().get(&TOKEN_ADMIN).unwrap();
        admin.require_auth();

        if get_balance(&e, addr.clone()) > 0 {
            return Err(Error::AddressAlreadyHoldsToken);
        }

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let key = DataKey::Balance(addr.clone());
        let amount: u32 = 1;
        e.storage().persistent().set(&key, &amount);
        Ok(amount)
    }

    pub fn balance(e: Env, addr: Address) -> u32 {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let b: u32 = get_balance(&e, addr);
        b
    }

    pub fn transfer(e: Env, from: Address, to: Address) -> Result<bool, Error> {
        from.require_auth();

        if get_balance(&e, from.clone()) == 0 {
            return Err(Error::AddressDoesNotHoldToken);
        }

        if get_balance(&e, to.clone()) > 0 {
            return Err(Error::AddressAlreadyHoldsToken);
        }

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let from_key = DataKey::Balance(from.clone());
        let to_key   = DataKey::Balance(to.clone());
        let amount: u32 = 1;

        e.storage().persistent().remove(&from_key); // from toklen is burned
        e.storage().persistent().set(&to_key, &amount);
        e.storage()
            .persistent()
            .extend_ttl(&to_key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);

        Ok(true)
    }

    pub fn approve(e: Env, from: Address, spender: Address, expiration: u32) -> Result<bool, Error> {
        from.require_auth();
        if expiration < e.ledger().sequence(){
            return Err(Error::ExpirationLedgerLessThanCurrentLedger);
        }

        if get_blocking(&e, from.clone()) {
            return Err(Error::AddressAlreadyHasAllowance);
        }

        if get_allowance(&e, spender.clone()) {
            return Err(Error::AddressAlreadyHasAllowance);
        }

        if get_balance(&e, from.clone()) < 1 {
            return Err(Error::AddressDoesNotHoldToken);
        }

        if get_balance(&e, spender.clone()) < 1 {
            return Err(Error::AddressDoesNotHoldToken);
        }

        let allowance_key = DataKey::Allowance(spender.clone());
        let blocking_key  = DataKey::Blocking(from.clone());
        e.storage().temporary().set(&allowance_key, &from);
        e.storage().temporary().set(&blocking_key, &spender);

        let live_for = expiration
            .checked_sub(e.ledger().sequence())
            .unwrap()
        ;

        e.storage().temporary().extend_ttl(&allowance_key, live_for, live_for);
        e.storage().temporary().extend_ttl(&blocking_key, live_for, live_for);

        Ok(true)
    }

    pub fn allowance(e: &Env, addr: Address) -> bool {

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let allowance = get_allowance(&e, addr);
        allowance
    }

    pub fn blocking(e: &Env, addr: Address) -> bool {

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let blocking = get_blocking(e, addr);
        blocking
    }

    pub fn burn(e: Env, addr: Address) {
        let admin: Address = e.storage().instance().get(&TOKEN_ADMIN).unwrap();
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        let from_key = DataKey::Balance(addr);
        e.storage().persistent().remove(&from_key);
    }

}

mod test;
