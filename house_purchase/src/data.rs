
use soroban_sdk::{contracttype, contracterror, symbol_short, Symbol, Address};

pub const PRUCHASE_TRADING: Symbol = symbol_short!("pt");
pub const CURRENT_STATE: Symbol = symbol_short!("c_state");
pub const PROPIETARY: Symbol = symbol_short!("prop");
pub const MEETING_ACCEPTED: Symbol = symbol_short!("meet_a");

pub const STATE_WAITING_FIRST_PAYMENT: Symbol = symbol_short!("WFP");
pub const STATE_FIRST_PAYMENT_SENT: Symbol = symbol_short!("FPS");
pub const STATE_REST_OF_PAYMENT_SENT: Symbol = symbol_short!("ROPS");

pub const DAY_IN_LEDGERS: u32 = 17280;
pub const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    FirstPaymentGreaterThanOrEqualAmount = 1,
    MeetingAlreadyAccepted = 2,
    CannotTransferAmountBeforeMeeting = 3,
    MeetingNotAcceptedYet = 4,
    PurchaseDataNotStored = 5,
    MeetingCannotBeProposedBeforeCurrentDate = 6,
    MeetingCanNotBeProposedIfFirstPaymentHaveNotBeenSent = 7,
    PurchaseDataAlreadyInit = 8
}

#[contracttype]
#[derive(Clone)]
pub struct PurchaseTrading {
    pub amount: i128,
    pub first_payment: i128,
    pub token: Address,
    pub key: Symbol,
    pub seller: Address,
    pub buyer: Address
}

#[contracttype]
pub struct Meeting {
    pub ts: u64
}

impl Meeting {
    pub fn is_meeting_taking_place(&self, current_ts: u64) -> bool{
        return self.ts < current_ts;
    }
}