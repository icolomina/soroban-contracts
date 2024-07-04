#![cfg(test)]
extern crate std;

use crate::data::Meeting;

use super::{HousePurchaseContract, HousePurchaseContractClient};
use soroban_sdk::IntoVal;
use soroban_sdk::testutils::{Ledger, Events};
use soroban_sdk::{Env, symbol_short, testutils::Address as _, Address, token, vec};
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
    buyer: Address,
    seller: Address,
    client:  HousePurchaseContractClient<'a>,
    contract_id: Address,
    sac_token: TokenClient<'a>
}

fn init_test_data(env: &Env, mint: bool) -> TestData {
    env.mock_all_auths();

    let contract_id = env.register_contract(None, HousePurchaseContract);
    let client = HousePurchaseContractClient::new(&env, &contract_id);

    let buyer: Address = Address::generate(&env);
    let seller = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let (sac_token, sac_token_admin) = create_token_contract(&env, &token_admin);
    if mint {
        sac_token_admin.mint(&buyer, &50000);
        sac_token_admin.mint(&seller, &1);
    }

    TestData {
        buyer,
        seller,
        client,
        contract_id,
        sac_token
    }
}

#[test]
fn test_success() {
    let env = Env::default();
    let test_data = init_test_data(&env, true);
    env.ledger().with_mut(|li| { li.timestamp = 12345; });

    assert_eq!(test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &5000, &40000, &symbol_short!("256997415")), true);
    assert_eq!(test_data.client.transfer_first_payment(), true);
    assert_eq!(test_data.client.seller_propose_meeting(&12349), true);

    let meeting = Meeting { ts: 12349 };
    let last_events = vec![&env, env.events().all().pop_back().unwrap()];
    assert_eq!(
        last_events,
        vec![
            &env,
            (
                test_data.contract_id,
                (symbol_short!("MP"), ).into_val(&env),
                meeting.into_val(&env)
            )
        ]
    );

    assert_eq!(test_data.client.buyer_review_meeting(&12349, &true), true);

    env.ledger().with_mut(|li| { li.timestamp = 12354; });
    assert_eq!(test_data.client.transfer_rest_of_payment(), 35000);
    assert_eq!(test_data.sac_token.balance(&test_data.seller), 40001);

}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #8)")]
fn test_purchase_already_init() {
    let env = Env::default();
    let test_data = init_test_data(&env, false);
    test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &4000, &40000, &symbol_short!("256997415"));
    test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &4000, &40000, &symbol_short!("256997415"));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_first_payment_greather_or_equal_amount() {
    let env = Env::default();
    let test_data = init_test_data(&env, false);
    test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &40000, &40000, &symbol_short!("256997415"));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #7)")]
fn test_propose_meeting_without_having_sent_first_payment() {
    let env = Env::default();
    let test_data = init_test_data(&env, false);
    test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &5000, &40000, &symbol_short!("256997415"));
    test_data.client.seller_propose_meeting(&12349);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #6)")]
fn test_meeting_cannot_be_proposed_before_current_date() {
    let env = Env::default();
    let test_data = init_test_data(&env, true);
    env.ledger().with_mut(|li| { li.timestamp = 12345; });
    test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &5000, &40000, &symbol_short!("256997415"));
    test_data.client.transfer_first_payment();

    test_data.client.seller_propose_meeting(&12341);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_meeting_already_accepted() {
    let env = Env::default();
    let test_data = init_test_data(&env, true);

    env.ledger().with_mut(|li| { li.timestamp = 12345; });
    test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &5000, &40000, &symbol_short!("256997415"));
    test_data.client.transfer_first_payment();

    test_data.client.seller_propose_meeting(&12347);
    test_data.client.buyer_review_meeting(&12347, &true);
    test_data.client.seller_propose_meeting(&12348);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_transfer_rest_of_payment_before_meeting() {
    let env = Env::default();
    let test_data = init_test_data(&env, true);

    env.ledger().with_mut(|li: &mut soroban_sdk::testutils::LedgerInfo| { li.timestamp = 12345; });
    test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &5000, &40000, &symbol_short!("256997415"));
    test_data.client.transfer_first_payment();

    test_data.client.seller_propose_meeting(&12347);
    test_data.client.buyer_review_meeting(&12347, &true);

    test_data.client.transfer_rest_of_payment();

}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_transfer_rest_of_payment_without_meeting_accepted() {
    let env = Env::default();
    let test_data = init_test_data(&env, true);

    env.ledger().with_mut(|li| { li.timestamp = 12345; });
    test_data.client.initialize(&test_data.buyer, &test_data.seller, &test_data.sac_token.address, &5000, &40000, &symbol_short!("256997415"));
    test_data.client.transfer_first_payment();
    test_data.client.seller_propose_meeting(&12347);
    test_data.client.transfer_rest_of_payment();
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_transfer_first_payment_with_no_purchase_data_stored() {
    let env = Env::default();
    let test_data = init_test_data(&env, true);
    test_data.client.transfer_first_payment();
}