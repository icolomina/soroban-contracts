
use soroban_sdk::{contracttype, contracterror, Address};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[contracttype]
pub struct Balance {
    pub deposited: i128,
    pub accumulated_interests: i128,
    pub total: i128
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
#[contracterror]
pub enum Error {
    AddressInsufficientBalance = 1,
    ContractInsufficientBalance = 2,
    ContractNotInitialized = 3,
    AmountLessOrEqualThan0 = 4,
    AmountLessOrThan0 = 5
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
#[contracttype]
pub enum State {
    Pending = 1,
    Initialized = 2
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Balance(Address),
    InterestRate,
    DepositStart(Address),
    Token,
    Admin,
    State
}

