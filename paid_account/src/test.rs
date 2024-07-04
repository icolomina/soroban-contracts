#![cfg(test)]

use crate::contract::{PaidAccount, PaidAccountClient};
use soroban_sdk::testutils::Ledger;
use soroban_sdk::{Env, testutils::Address as _, Address, token};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(e, &contract_address),
        TokenAdminClient::new(e, &contract_address),
    )
}

struct TestData<'a> {
    admin: Address,
    user: Address,
    client:  PaidAccountClient<'a>,
    token: TokenClient<'a>,
    token_admin: TokenAdminClient<'a>
}

fn init_test_data(e: &Env) -> TestData {
    e.mock_all_auths();

    let contract_id = e.register_contract(None, PaidAccount);
    let client = PaidAccountClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let user = Address::generate(&e);
    let (token, token_admin) = create_token_contract(&e, &admin);

    TestData {
        admin,
        user,
        client,
        token,
        token_admin
    }
}

#[test]
fn test_init() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    assert_eq!(test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32), true);
}

#[test]
fn test_user_deposit() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.user, &100);

    test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32);
    assert_eq!(test_data.client.user_deposit(&test_data.user, &50), 50);
    assert_eq!(test_data.client.user_deposit(&test_data.user, &50), 100);

    let balance = test_data.client.get_balance(&test_data.user);
    assert_eq!(balance.deposited, 100);
    assert_eq!(balance.accumulated_interests, 0);
    assert_eq!(balance.total, 100);

    let current_ts = e.ledger().timestamp();
    e.ledger().with_mut(|l| {
        l.timestamp = current_ts + 604800 // Simulate It's been a week since user started the deposit
    });

    let balance = test_data.client.get_balance(&test_data.user);
    assert_eq!(balance.deposited, 100);
    assert!(balance.accumulated_interests > 0);
    assert!(balance.total == balance.deposited + balance.accumulated_interests);
}

#[test]
fn test_admin_deposit() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.admin, &100);

    test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32);
    assert_eq!(test_data.client.admin_deposit(&50), 50);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_contract_non_initialized() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.client.user_deposit(&test_data.user, &50);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_zero_amount() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.admin, &100);

    test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32);
    test_data.client.admin_deposit(&0);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_user_withdrawal_insufficient_balance() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.admin, &1000);
    test_data.token_admin.mint(&test_data.user, &100);

    test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32);
    test_data.client.admin_deposit(&1000);
    test_data.client.user_deposit(&test_data.user, &100);

    test_data.client.user_withdrawal(&test_data.user, &105);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_user_withdrawal_contract_insufficient_balance() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.admin, &1000);
    test_data.token_admin.mint(&test_data.user, &100);

    test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32);
    test_data.client.admin_deposit(&2);
    test_data.client.user_deposit(&test_data.user, &100);
    test_data.client.admin_withdrawal(&80);

    test_data.client.user_withdrawal(&test_data.user, &90);
}

#[test]
fn test_user_withdrawal_all() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.admin, &1000);
    test_data.token_admin.mint(&test_data.user, &100);

    test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32);
    test_data.client.admin_deposit(&1000);
    test_data.client.user_deposit(&test_data.user, &100);

    let current_ts = e.ledger().timestamp();
    e.ledger().with_mut(|l| {
        l.timestamp = current_ts + 604800 // Simulate It's been a week since user started the deposit
    });

    let current_balance = test_data.client.get_balance(&test_data.user);
    let new_balance = test_data.client.user_withdrawal(&test_data.user, &current_balance.total);

    assert_eq!(new_balance.total, 0);
}

#[test]
fn test_user_withdrawal_partially() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.admin, &1000);
    test_data.token_admin.mint(&test_data.user, &100);

    test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32);
    test_data.client.admin_deposit(&1000);
    test_data.client.user_deposit(&test_data.user, &100);

    let current_ts = e.ledger().timestamp();
    e.ledger().with_mut(|l| {
        l.timestamp = current_ts + 604800 // Simulate It's been a week since user started the deposit
    });

    let current_balance = test_data.client.get_balance(&test_data.user);
    let amount_to_withdraw = current_balance.total / 2;
    let new_balance = test_data.client.user_withdrawal(&test_data.user, &amount_to_withdraw);

    assert_eq!(new_balance.deposited, current_balance.total - amount_to_withdraw);
    assert!(new_balance.total > 0);
    assert!(new_balance.total > new_balance.deposited);
    assert!(new_balance.accumulated_interests > 0);
}