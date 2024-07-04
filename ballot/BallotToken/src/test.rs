#![cfg(test)]

use super::{BallotToken, BallotTokenClient};
use soroban_sdk::{Env, Address, testutils::{Address as _, Ledger}};

#[test]
fn initialize() {
    let env = Env::default();
    let client = create_client(&env);

    let admin = Address::generate(&env);
    assert_eq!(client.initialize(&admin), true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn already_initialized() {
    let env = Env::default();
    let client = create_client(&env);

    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.initialize(&admin);
}

#[test]
fn mint() {
    let env = Env::default();
    let client = create_client(&env);

    let admin = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    assert_eq!(client.mint(&to), 1);
    assert_eq!(client.balance(&to), 1);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn already_minted() {

    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.mint(&to);
    client.mint(&to);
}

#[test]
fn transfer() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.mint(&from);
    client.transfer(&from, &to);

    assert_eq!(client.balance(&from), 0);
    assert_eq!(client.balance(&to), 1);

}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn transfer_from_does_not_hold_token() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.transfer(&from, &to);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn transfer_to_already_holds_token() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.initialize(&admin);
    client.mint(&from);
    client.mint(&to);
    client.transfer(&from, &to);
}

#[test]
fn no_allowance() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let addr = Address::generate(&env);

    client.initialize(&admin);
    assert_eq!(client.allowance(&addr), false);
}

#[test]
fn no_blocking() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let addr = Address::generate(&env);

    client.initialize(&admin);
    assert_eq!(client.blocking(&addr), false);
}

#[test]
fn approve_and_allowance() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let spender = Address::generate(&env);

    env.ledger().with_mut(|li| {li.sequence_number = 2499;});

    client.initialize(&admin);
    client.mint(&from);
    client.mint(&spender);

    client.approve(&from, &spender, &2500);
    assert_eq!(client.allowance(&spender), true);
    assert_eq!(client.blocking(&from), true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn approve_expiration_ledger_lower_than_current() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let spender = Address::generate(&env);

    env.ledger().with_mut(|li| {li.sequence_number = 2499;});
    client.initialize(&admin);
    client.mint(&from);
    client.mint(&spender);

    client.approve(&from, &spender, &2498);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn approve_from_already_has_allowance() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let spender = Address::generate(&env);
    env.ledger().with_mut(|li| {li.sequence_number = 2499;});

    client.initialize(&admin);
    client.mint(&from);
    client.mint(&spender);

    client.approve(&from, &spender, &2500);
    client.approve(&from, &spender, &2500);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn approve_spender_already_has_allowance() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let from1 = Address::generate(&env);
    let from2 = Address::generate(&env);
    let spender = Address::generate(&env);
    env.ledger().with_mut(|li| {li.sequence_number = 2499;});

    client.initialize(&admin);
    client.mint(&from1);
    client.mint(&from2);
    client.mint(&spender);

    client.approve(&from1, &spender, &2500);
    client.approve(&from2, &spender, &2500);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn approve_from_does_not_hold_token() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let spender = Address::generate(&env);
    env.ledger().with_mut(|li| {li.sequence_number = 2499;});

    client.initialize(&admin);
    client.mint(&spender);

    client.approve(&from, &spender, &2500);
}

#[test]
fn burn() {
    let env = Env::default();
    let client = create_client(&env);
    let admin = Address::generate(&env);
    let addr = Address::generate(&env);

    client.initialize(&admin);
    client.mint(&addr);
    assert_eq!(client.balance(&addr), 1);

    client.burn(&addr);
    assert_eq!(client.balance(&addr), 0);
}

fn create_client(env: &Env) -> BallotTokenClient{
    env.mock_all_auths();
    let contract_id = env.register_contract(None, BallotToken);
    let client = BallotTokenClient::new(&env, &contract_id);

    client
}