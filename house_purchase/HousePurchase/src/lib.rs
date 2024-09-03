#![no_std]
use soroban_sdk::{contract, contractimpl, contracterror, contracttype, token, Env, Address};

mod asset {
    soroban_sdk::contractimport!(
        file = "../HouseAsset/target/wasm32-unknown-unknown/release/house_asset.wasm"
    );
}

#[contracttype]
enum DataKey {
    Asset,
    State,
    Buyer,
    Token,
    Admin,
    FirstPaymentAmount,
    Amount
}

#[contracttype]
#[derive(Eq, PartialEq)]
pub enum State {
    Pending,
    FirstPaymentTransferred,
    RestOfPaymentTransferred,
    Finished
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ContractAlreadyInitialized = 1,
    ContractNotInitialized = 2,
    FirstPaymentNotTransferred = 3,
    RestOfPaymentNotTransferred = 4
}


#[contract]
pub struct HousePurchaseContract;

#[contractimpl]
impl HousePurchaseContract {

    pub fn initialize(e: Env, asset: Address, buyer: Address, token: Address, first_payment: i128, amount: i128) -> Result<bool, Error> {

        if let Some(_asset) = e.storage().instance().get::<DataKey, Address>(&DataKey::Asset) {
            return Err(Error::ContractAlreadyInitialized);
        } else {
            e.storage().instance().set(&DataKey::Asset, &asset);
            e.storage().instance().set(&DataKey::Buyer, &buyer);
            e.storage().instance().set(&DataKey::Token, &token);
            e.storage().instance().set(&DataKey::FirstPaymentAmount, &first_payment);
            e.storage().instance().set(&DataKey::Amount, &amount);
            e.storage().instance().set(&DataKey::State, &State::Pending);

            Ok(true)
        }
    }

    pub fn transfer_first_payment(e: Env) -> Result<bool, Error> {

        if let Some(asset) = e.storage().instance().get::<DataKey, Address>(&DataKey::Asset) {
            let token = e.storage().instance().get::<DataKey, Address>(&DataKey::Token).unwrap();
            let buyer = e.storage().instance().get::<DataKey, Address>(&DataKey::Buyer).unwrap();
            let first_payment_amount = e.storage().instance().get::<DataKey, i128>(&DataKey::FirstPaymentAmount).unwrap();
            let asset_contract = asset::Client::new(&e, &asset);

            buyer.require_auth();
            let tk = token::Client::new(&e, &token);
            tk.transfer(&buyer, &asset_contract.owner(), &first_payment_amount);
            e.storage().instance().set(&DataKey::State, &State::FirstPaymentTransferred);
            Ok(true)  

        } else {
            return Err(Error::ContractNotInitialized);
        }
        
    }

    pub fn transfer_rest_of_payment(e: Env) -> Result<bool, Error> {

        if let Some(asset) = e.storage().instance().get::<DataKey, Address>(&DataKey::Asset) {
            let state: State = e.storage().instance().get(&DataKey::State).unwrap();
            if state != State::FirstPaymentTransferred {
                return Err(Error::FirstPaymentNotTransferred);
            } 

            let token = e.storage().instance().get::<DataKey, Address>(&DataKey::Token).unwrap();
            let buyer = e.storage().instance().get::<DataKey, Address>(&DataKey::Buyer).unwrap();
            let first_payment_amount = e.storage().instance().get::<DataKey, i128>(&DataKey::FirstPaymentAmount).unwrap();
            let amount = e.storage().instance().get::<DataKey, i128>(&DataKey::Amount).unwrap();

            let asset_contract = asset::Client::new(&e, &asset);
            let rest_of_payment_amount = amount - first_payment_amount;

            buyer.require_auth();
            let tk = token::Client::new(&e, &token);
            tk.transfer(&buyer, &asset_contract.owner(), &rest_of_payment_amount);
            e.storage().instance().set(&DataKey::State, &State::RestOfPaymentTransferred);
            Ok(true)
        } else {
            return Err(Error::ContractNotInitialized);
        }
        
    }

    pub fn change_owner(e: Env) -> Result<bool, Error> {
        if let Some(asset) = e.storage().instance().get::<DataKey, Address>(&DataKey::Asset) {
            let state: State = e.storage().instance().get(&DataKey::State).unwrap();

            if state != State::RestOfPaymentTransferred {
                return Err(Error::RestOfPaymentNotTransferred);
            } 

            let asset_contract = asset::Client::new(&e, &asset);
            
            let buyer = e.storage().instance().get::<DataKey, Address>(&DataKey::Buyer).unwrap();
            asset_contract.owner().require_auth();
            asset_contract.transfer(&buyer); // change the asset owner
            e.storage().instance().set(&DataKey::State, &State::Finished);
            Ok(true)

        } else {
            return Err(Error::ContractNotInitialized);
        }
    }

    pub fn state(e: Env) -> Result<State, Error> {
        if let Some(state) = e.storage().instance().get::<DataKey, State>(&DataKey::State) {
            Ok(state)
        } else {
            return Err(Error::ContractNotInitialized);
        }
    }

}

mod test;