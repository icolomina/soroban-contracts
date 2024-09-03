#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, contracterror, Env, String, Address};

#[contracttype]
struct Metadada {
    asset_id: String,
}

#[contracttype]
enum DataKey {
    Owner,
    Metadata,
    Allowance
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AssetAlreadyInitialized = 1,
    AssetNotInitialized = 2,
    AddressNotApproved = 3,
    AssetWithoutTransferAllowance = 4
}

#[contract]
pub struct Asset;

#[contractimpl]
impl Asset {

    pub fn initialize(e: Env, owner: Address, asset_id: String) -> Result<bool, Error> {
        if let Some(_owner) = e.storage().instance().get::<DataKey, Address>(&DataKey::Owner) {
            return Err(Error::AssetAlreadyInitialized);
        }

        let metadata = Metadada {
            asset_id,
        };

        e.storage().instance().set(&DataKey::Owner, &owner);
        e.storage().instance().set(&DataKey::Metadata, &metadata);
        Ok(true)
    }

    pub fn approve(e: Env, addr_to_allow: Address, ts: u64) -> Result<bool, Error> {
        
        if let Some(owner) = e.storage().instance().get::<DataKey, Address>(&DataKey::Owner) { 
            owner.require_auth();
            e.storage().temporary().set(&DataKey::Allowance, &addr_to_allow);
            let next_ledger = e.ledger().sequence() + (ts / 5) as u32;
            let live_for = next_ledger
                .checked_sub(e.ledger().sequence())
                .unwrap();

                e.storage().temporary().extend_ttl(&DataKey::Allowance, live_for, live_for);
                Ok(true)
        }
        else {
            return Err(Error::AssetNotInitialized);
        } 
    }
    
    pub fn transfer(e: Env, to: Address) -> Result<bool, Error>  {
        if let Some(owner) = e.storage().instance().get::<DataKey, Address>(&DataKey::Owner) {
            owner.require_auth();
            e.storage().instance().set(&DataKey::Owner, &to);
            Ok(true)
        }
        else{
            return Err(Error::AssetNotInitialized);
        }
    }

    pub fn transfer_from(e: Env, allowed_addr: Address, to: Address) -> Result<bool, Error>  {
        if let Some(_owner) = e.storage().instance().get::<DataKey, Address>(&DataKey::Owner) {
            if let Some(allowed_a) = e.storage().temporary().get(&DataKey::Allowance) {
                if allowed_addr != allowed_a {
                    return Err(Error::AddressNotApproved);
                }

                allowed_addr.require_auth();
                e.storage().instance().set(&DataKey::Owner, &to);
                Ok(true)
            } else {
                return Err(Error::AssetWithoutTransferAllowance);
            }
        } else {
            return Err(Error::AssetNotInitialized);
        }
    }

    pub fn owner(e: Env) -> Address {
        if let Some(owner) = e.storage().instance().get::<DataKey, Address>(&DataKey::Owner) {
            owner
        } else {
            panic!("Asset not Initialized");
        }
    }
}

mod test;
