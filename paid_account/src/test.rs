#![cfg(test)]

use crate::contract::{PaidAccount, PaidAccountClient};
use crate::data::Balance;
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
    assert_eq!(test_data.client.init(&test_data.admin, &test_data.token.address, &500_u32, &30_u64), true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_init_fail_no_rate() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    assert_eq!(test_data.client.init(&test_data.admin, &test_data.token.address, &0_u32, &30_u64), true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_init_fail_no_claim() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    assert_eq!(test_data.client.init(&test_data.admin, &test_data.token.address, &50_u32, &0_u64), true);
}

#[test]
fn test_user_deposit() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    let another_user = Address::generate(&e);
    test_data.token_admin.mint(&test_data.user, &1000000);
    test_data.token_admin.mint(&test_data.admin, &1000000);
    test_data.token_admin.mint(&another_user, &1000000);

    test_data.client.init(&test_data.admin, &test_data.token.address, &500_u32, &6_u64);
    test_data.client.admin_deposit(&100000);

    let balance_test_user: Balance = test_data.client.user_deposit(&test_data.user, &500000);
    test_data.client.user_deposit(&another_user, &500000);

    assert_eq!(balance_test_user.deposited, 500000);
    assert_eq!(balance_test_user.accumulated_interests, 25000);
    assert_eq!(balance_test_user.total, 525000);
    assert_eq!(test_data.client.get_contract_balance(), 1100000);

    let current_ts = e.ledger().timestamp();
    e.ledger().with_mut(|l| {
        l.timestamp = current_ts + 604800 // Simulate It's been a week since user started the deposit
    });

    let withdrawn = test_data.client.user_withdrawal(&test_data.user);
    assert_eq!(withdrawn, 525000);
    assert_eq!(test_data.client.get_contract_balance(), 575000);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_user_deposit_financial_reached() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.user, &1000000);
    test_data.client.init(&test_data.admin, &test_data.token.address, &500_u32, &6_u64);
    let another_user = Address::generate(&e);
    test_data.token_admin.mint(&another_user, &1000000);

    test_data.client.user_deposit(&test_data.user, &500000);
    test_data.client.stop_deposits();

    test_data.client.user_deposit(&another_user, &500000);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #10)")]
fn test_user_already_deposited() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.user, &1000000);
    test_data.client.init(&test_data.admin, &test_data.token.address, &500_u32, &6_u64);
    test_data.client.user_deposit(&test_data.user, &500000);
    test_data.client.user_deposit(&test_data.user, &500000);
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

    test_data.client.init(&test_data.admin, &test_data.token.address, &5_u32, &30_u64);
    test_data.client.admin_deposit(&0);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #9)")]
fn test_unable_to_withdraw() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.user, &1000000);
    test_data.token_admin.mint(&test_data.admin, &1000000);

    test_data.client.init(&test_data.admin, &test_data.token.address, &500_u32, &50_u64);
    test_data.client.admin_deposit(&100000);
    let balance = test_data.client.user_deposit(&test_data.user, &500000);

    assert_eq!(balance.deposited, 500000);
    assert_eq!(balance.accumulated_interests, 25000);
    assert_eq!(balance.total, 525000);

    let current_ts = e.ledger().timestamp();
    e.ledger().with_mut(|l| {
        l.timestamp = current_ts + 604800 // Simulate It's been a week since user started the deposit
    });

    test_data.client.user_withdrawal(&test_data.user);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_user_withdrawal_contract_insufficient_balance() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    test_data.token_admin.mint(&test_data.admin, &100000);
    test_data.token_admin.mint(&test_data.user, &100000);

    test_data.client.init(&test_data.admin, &test_data.token.address, &500_u32, &1_u64);
    test_data.client.user_deposit(&test_data.user, &100000);

    let current_ts = e.ledger().timestamp();
    e.ledger().with_mut(|l| {
        l.timestamp = current_ts + 604800 // Simulate It's been a week since user started the deposit
    });

    test_data.client.user_withdrawal(&test_data.user);
}

#[test]
fn test_admin_withdrawal() {
    let e = Env::default();
    let test_data = init_test_data(&e);
    let another_user = Address::generate(&e);
    test_data.token_admin.mint(&test_data.admin, &100000);
    test_data.token_admin.mint(&test_data.user, &100000);

    test_data.client.init(&test_data.admin, &test_data.token.address, &500_u32, &1_u64);
    test_data.client.user_deposit(&test_data.user, &100000);

    let current_balance = test_data.client.admin_withdrawal(&another_user, &90000);
    assert_eq!(current_balance, 10000);
}