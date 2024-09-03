#![cfg(test)]

mod asset {
    soroban_sdk::contractimport!(
        file = "../HouseAsset/target/wasm32-unknown-unknown/release/house_asset.wasm"
    );
}

use super::{ HousePurchaseContract, HousePurchaseContractClient};
use soroban_sdk::{Env, testutils::Address as _, Address, token, String};
use token::Client as TokenClient;
use asset::Client;
use token::StellarAssetClient as TokenAdminClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(e, &contract_address),
        TokenAdminClient::new(e, &contract_address),
    )
}

fn create_asset(e: &Env) -> (Address, Client) {
    let asset_contract_id = e.register_contract_wasm(None, asset::WASM);
    let asset_contract = asset::Client::new(&e, &asset_contract_id);
    (
        asset_contract_id,
        asset_contract
    )
}

struct TestData<'a> {
    buyer: Address,
    asset: Address,
    asset_contract: Client<'a>,
    client:  HousePurchaseContractClient<'a>,
    sac_token: TokenClient<'a>
}

fn init_test_data(env: &Env) -> TestData {
    env.mock_all_auths();

    let contract_id = env.register_contract(None, HousePurchaseContract);
    let client = HousePurchaseContractClient::new(&env, &contract_id);

    let buyer: Address = Address::generate(&env);
    let owner: Address = Address::generate(&env);
    let (asset, asset_contract) = create_asset(&env);
    let asset_id = String::from_str(&env, "399fg7u6h69965h6");
    asset_contract.initialize(&owner, &asset_id);
    let token_admin = Address::generate(&env);

    let (sac_token, sac_token_admin) = create_token_contract(&env, &token_admin);
    sac_token_admin.mint(&buyer, &50000);

    TestData {
        buyer,
        asset,
        asset_contract,
        client,
        sac_token
    }
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let test_data = init_test_data(&env);

    assert_eq!(test_data.client.initialize(&test_data.asset, &test_data.buyer, &test_data.sac_token.address, &5000_i128, &45000_i128), true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_already_initialized() {
    let env = Env::default();
    let test_data = init_test_data(&env);

    test_data.client.initialize(&test_data.asset, &test_data.buyer, &test_data.sac_token.address, &5000_i128, &45000_i128);
    test_data.client.initialize(&test_data.asset, &test_data.buyer, &test_data.sac_token.address, &5000_i128, &45000_i128);
}

#[test]
fn test_transfer() {
    let env = Env::default();
    let test_data = init_test_data(&env);

    test_data.client.initialize(&test_data.asset, &test_data.buyer, &test_data.sac_token.address, &5000_i128, &45000_i128);
    test_data.client.transfer_first_payment();
    assert_eq!(test_data.sac_token.balance(&test_data.asset_contract.owner()), 5000);

    test_data.client.transfer_rest_of_payment();
    assert_eq!(test_data.sac_token.balance(&test_data.asset_contract.owner()), 45000);

    test_data.client.change_owner();
    assert_eq!(test_data.asset_contract.owner(), test_data.buyer);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_first_payment_contract_not_initialized() {
    let env = Env::default();
    let test_data = init_test_data(&env);
    test_data.client.transfer_first_payment();
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_first_payment_not_transferred() {
    let env = Env::default();
    let test_data = init_test_data(&env);
    test_data.client.initialize(&test_data.asset, &test_data.buyer, &test_data.sac_token.address, &5000_i128, &45000_i128);
    test_data.client.transfer_rest_of_payment();
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_change_owner_without_payment_transferred() {
    let env = Env::default();
    let test_data = init_test_data(&env);
    test_data.client.initialize(&test_data.asset, &test_data.buyer, &test_data.sac_token.address, &5000_i128, &45000_i128);
    test_data.client.transfer_first_payment();
    test_data.client.change_owner();
}