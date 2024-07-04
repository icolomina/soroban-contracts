#![cfg(test)]

use super::{token, Ballot, BallotClient};
use soroban_sdk::{symbol_short, testutils::{Address as _, Ledger}, Address, Env, Vec};

struct TestData<'a> {
    admin: Address,
    voters: Vec<Address>,
    token: token::Client<'a>
}

fn get_test_data(env: &Env, num_voters: u8) -> TestData {
    let admin = Address::generate(&env);
    let mut voters = Vec::new(&env);
    let token_address = env.register_contract_wasm(None, token::WASM);
    let token = token::Client::new(&env, &token_address);
    token.initialize(&admin);

    for _i in 0..num_voters {
        let addr: Address = Address::generate(&env);
        token.mint(&addr);
        voters.push_back(addr);
    }

    TestData {
        admin,
        voters,
        token
    }
}

#[test]
fn vote_test() {
    let env = Env::default();
    env.ledger().with_mut(|l| {l.timestamp = 1689238844;});
    let client = create_client(&env);
    let test_data = get_test_data(&env, 5);

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59

    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    assert_eq!(client.vote(&test_data.voters.get(0).unwrap(), &symbol_short!("Laborist")), true);
    assert_eq!(client.vote(&test_data.voters.get(1).unwrap(), &symbol_short!("Conserv")), true);
    assert_eq!(client.vote(&test_data.voters.get(2).unwrap(), &symbol_short!("Conserv")), true);

    let count = client.count(&test_data.admin);

    assert_eq!(count.get(symbol_short!("Laborist")).unwrap(), 1);
    assert_eq!(count.get(symbol_short!("Conserv")).unwrap(), 2);

    client.delegate(&test_data.voters.get(3).unwrap(), &test_data.voters.get(4).unwrap());
    assert_eq!(client.vote(&&test_data.voters.get(4).unwrap(), &symbol_short!("Conserv")), true);

    let count = client.count(&test_data.admin);

    assert_eq!(count.get(symbol_short!("Laborist")).unwrap(), 1);
    assert_eq!(count.get(symbol_short!("Conserv")).unwrap(),4);

}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn vote_out_of_dates_test() {
    let env = Env::default();
    let client = create_client(&env);
    let test_data = get_test_data(&env, 1);

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59

    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    client.vote(&test_data.voters.get(0).unwrap(), &symbol_short!("Laborist"));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn vote_test_already_voted() {
    let env = Env::default();
    let client = create_client(&env);
    let test_data = get_test_data(&env, 1);
    env.ledger().with_mut(|l| {l.timestamp = 1689238844;});

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59

    let voter = test_data.voters.get(0).unwrap();
    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    client.vote(&voter, &symbol_short!("Laborist"));
    client.vote(&voter, &symbol_short!("Laborist"));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn voter_cannot_delegate_since_does_not_hold_token() {
    let env = Env::default();
    let client = create_client(&env);
    let test_data = get_test_data(&env, 0);
    env.ledger().with_mut(|l| {l.timestamp = 1689238844;});

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    client.delegate(&voter1, &voter2);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn voter_cannot_delegate_since_target_does_not_hold_token() {
    let env = Env::default();
    let client = create_client(&env);
    let test_data = get_test_data(&env, 0);
    env.ledger().with_mut(|l| {l.timestamp = 1689238844;});

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    
    test_data.token.mint(&voter1);
    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    client.delegate(&voter1, &voter2);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn voter_cannot_delegate_since_it_has_voted() {
    let env = Env::default();
    let client = create_client(&env);
    let test_data = get_test_data(&env, 2);
    env.ledger().with_mut(|l| {l.timestamp = 1689238844;});

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59
    let voter1 = test_data.voters.get(0).unwrap();
    let voter2 = test_data.voters.get(1).unwrap();

    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    client.vote(&voter1, &symbol_short!("Laborist"));
    client.delegate(&voter1, &voter2);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn vote_cannot_be_delegated_since_target_has_voted() {
    let env = Env::default();
    let client = create_client(&env);
    let test_data = get_test_data(&env, 2);
    env.ledger().with_mut(|l| {l.timestamp = 1689238844;});

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59
    let voter1 = test_data.voters.get(0).unwrap();
    let voter2 = test_data.voters.get(1).unwrap();

    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    client.vote(&voter2, &symbol_short!("Laborist"));
    client.delegate(&voter1, &voter2);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn voter_has_already_delegated_its_vote() {
    let env = Env::default();
    let client = create_client(&env);
    let test_data = get_test_data(&env, 2);
    env.ledger().with_mut(|l| {l.timestamp = 1689238844;});

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59
    let voter1 = test_data.voters.get(0).unwrap();
    let voter2 = test_data.voters.get(1).unwrap();

    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    client.delegate(&voter1, &voter2);
    client.delegate(&voter1, &voter2);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn voter_target_has_already_a_delegated_vote() {
    let env = Env::default();
    let client = create_client(&env);
    let test_data = get_test_data(&env, 3);
    env.ledger().with_mut(|l| {l.timestamp = 1689238844;});

    let ts_start: u64 = 1689238800; // 2023-07-13 09:00:00
    let ts_end: u64 = 1689551999; // 2023-07-16 23:59:59
    let voter1 = test_data.voters.get(0).unwrap();
    let voter2 = test_data.voters.get(1).unwrap();
    let voter3 = test_data.voters.get(2).unwrap();

    client.configure(&test_data.admin, &test_data.token.address, &ts_start, &ts_end);
    client.delegate(&voter1, &voter2);
    client.delegate(&voter3, &voter2);
}

fn create_client(env: &Env) -> BallotClient{
    env.mock_all_auths();

    let contract_id = env.register_contract(None, Ballot);
    let client = BallotClient::new(&env, &contract_id);
    client
}