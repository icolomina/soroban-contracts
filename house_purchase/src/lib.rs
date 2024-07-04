#![no_std]

mod data;
use soroban_sdk::{contract, contractimpl, token, Env, Symbol, Address, symbol_short};
use data::{
    CURRENT_STATE,
    PRUCHASE_TRADING,
    PROPIETARY,
    MEETING_ACCEPTED,
    STATE_FIRST_PAYMENT_SENT,
    STATE_REST_OF_PAYMENT_SENT,
    STATE_WAITING_FIRST_PAYMENT,
    DAY_IN_LEDGERS,
    INSTANCE_LIFETIME_THRESHOLD,
    INSTANCE_BUMP_AMOUNT,
    PurchaseTrading,
    Error,
    Meeting
};

fn store_initial_data(env: &Env, purchase_trading: &PurchaseTrading, seller: &Address) {
    env.storage().instance().set(&PRUCHASE_TRADING, purchase_trading);
    env.storage().instance().set(&PROPIETARY, seller);
    env.storage().instance().set(&CURRENT_STATE, &STATE_WAITING_FIRST_PAYMENT);

    env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

}

fn get_purchase_trading(env: &Env) -> Option<PurchaseTrading> {
    let purchase_trading = env.storage().instance().get(&PRUCHASE_TRADING);
    purchase_trading
}

fn set_state_as_first_payment_sent(env: &Env) {
    env.storage().instance().set(&CURRENT_STATE, &STATE_FIRST_PAYMENT_SENT);
    env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn get_meeting_accepted(env: &Env) -> Option<Meeting> {
    let meeting = env.storage().temporary().get(&MEETING_ACCEPTED);
    meeting
}

fn store_final_data(env: &Env, buyer: &Address) {
    env.storage().instance().set(&CURRENT_STATE, &STATE_REST_OF_PAYMENT_SENT);
    env.storage().instance().set(&PROPIETARY, buyer);
    env.storage().temporary().remove(&MEETING_ACCEPTED);

    env.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

}

fn store_meeting_key(env: &Env, meeting: Meeting, ts: u64) {
    env.storage().temporary().set(&MEETING_ACCEPTED, &meeting);
    let ts_diff = ts - env.ledger().timestamp();
    let ts_days_diff = (ts_diff / 3600) as u32;
    let ttl;

    if ts_days_diff == 0 {
        ttl = DAY_IN_LEDGERS;
    }
    else{
        ttl = ts_days_diff * DAY_IN_LEDGERS + (DAY_IN_LEDGERS / 2);
    }
    
    env.storage().temporary().extend_ttl(&MEETING_ACCEPTED, ttl, ttl);
}

fn get_current_state(env: &Env) -> Symbol {
    let state = env.storage().instance().get(&CURRENT_STATE).unwrap_or(symbol_short!(""));
    state
}


#[contract]
pub struct HousePurchaseContract;

#[contractimpl]
impl HousePurchaseContract {

    pub fn initialize(env: Env, buyer: Address, seller: Address, token: Address, first_payment: i128, amount: i128, key: Symbol) -> Result<bool, Error> {

        if let Some(_purchase_trading) = get_purchase_trading(&env) {
            return Err(Error::PurchaseDataAlreadyInit);
        }

        if amount <= first_payment {
            return Err(Error::FirstPaymentGreaterThanOrEqualAmount);
        }

        let purchase_trading = PurchaseTrading {
            amount,
            first_payment, 
            token,
            key,
            seller,
            buyer
        };

        store_initial_data(&env, &purchase_trading, &purchase_trading.seller);
        Ok(true)
    }

    pub fn transfer_first_payment(env: Env) -> Result<bool, Error> {
        if let Some(purchase_trading) = get_purchase_trading(&env) {
            let tk = token::Client::new(&env, &purchase_trading.token);
            purchase_trading.buyer.require_auth();
            tk.transfer(&purchase_trading.buyer, &purchase_trading.seller, &purchase_trading.first_payment);
            set_state_as_first_payment_sent(&env);
            return Ok(true);
        } else {
            return Err(Error::PurchaseDataNotStored);
        }
        
    }

    pub fn seller_propose_meeting(env: Env, ts: u64) -> Result<bool, Error> {
        let state = get_current_state(&env);
        if state != STATE_FIRST_PAYMENT_SENT {
            return Err(Error::MeetingCanNotBeProposedIfFirstPaymentHaveNotBeenSent);
        }

        if env.storage().temporary().has(&MEETING_ACCEPTED) {
            return Err(Error::MeetingAlreadyAccepted);
        } else {

            if ts <= env.ledger().timestamp() {
                return Err(Error::MeetingCannotBeProposedBeforeCurrentDate);
            }

            let meeting = Meeting { ts };
            env.events().publish((symbol_short!("MP"), ), meeting);
            return Ok(true);
        }
    }

    pub fn buyer_review_meeting(env: Env, ts: u64, accept: bool) -> bool {
        if accept {
            let meeting = Meeting { ts };
            store_meeting_key(&env, meeting, ts);
            return true;
        } else {
            return false;
        }
    }

    pub fn transfer_rest_of_payment(env: Env) -> Result<i128, Error> {
        if let Some(mta) = get_meeting_accepted(&env) {
            let current_ts = env.ledger().timestamp();
            if !mta.is_meeting_taking_place(current_ts)  {
                return Err(Error::CannotTransferAmountBeforeMeeting);
            }
        } else {
            return Err(Error::MeetingNotAcceptedYet);
        }

        let purchase_trading: PurchaseTrading = get_purchase_trading(&env).unwrap(); 
        let rest_of_payment = purchase_trading.amount - purchase_trading.first_payment;

        let tk = token::Client::new(&env, &purchase_trading.token);
        purchase_trading.buyer.require_auth();
        tk.transfer(&purchase_trading.buyer, &purchase_trading.seller, &rest_of_payment);
        store_final_data(&env, &purchase_trading.buyer);
        return Ok(rest_of_payment);
        
    }

}

mod test;