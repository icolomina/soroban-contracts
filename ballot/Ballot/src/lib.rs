#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, Env, Symbol, Map, Address, Vec};

mod storage;
mod validation;

mod token {
    soroban_sdk::contractimport!(
        file = "../BallotToken/target/wasm32-unknown-unknown/release/ballot_token.wasm"
    );
}

use storage::VCounter;

struct Voter<'a> {
    id: &'a Address
}

impl<'a> Voter<'a> {
    
    fn has_voted(&self, env: &Env) -> bool {
        let vts: Vec<Address> = storage::get_votes(env);
        vts.contains(self.id)
    }

    fn is_delegated(&self, env: &Env) -> bool {
        let token = storage::get_token(&env);
        let tk = token::Client::new(&env, &token);

        if tk.blocking(&self.id) {
            return true
        }

        false
    }

    fn has_delegated_vote(&self, env: &Env) -> bool {
        let token = storage::get_token(&env);
        let tk = token::Client::new(&env, &token);

        if tk.allowance(&self.id) {
            return true
        }

        false
    }
}

fn check_dates(env: &Env) -> bool {
    let cfg = storage::get_config(env);
    let mut valid = true;
    if cfg.from > 0 && cfg.to > 0 {
        valid = validation::is_valid_date(&env, &cfg.from, &cfg.to)
    }

    valid
}


#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    VoterHasHisVoteDelegated = 1,
    VoterHasAlreadyVoted = 2,
    VoterHasDelegatedVote = 3,
    VoterOriginHasAlreadyVotedAndCannotDelegate = 4,
    VoterTargetHasAlreadyVotedAndCannotDelegate = 5,
    BallotOutOfDate = 6,
    VoterDoesNotHoldToken = 7

}

#[contract]
pub struct Ballot;

#[contractimpl]
impl Ballot {

    pub fn configure(env: Env, admin: Address, token: Address, ts_start: u64, ts_end: u64) -> Result<bool, Error> {
        admin.require_auth();
        storage::store_config(&env, ts_start, ts_end, token);
        Ok(true)
    }

    pub fn vote(env: Env, voter: Address, candidate: Symbol) -> Result<bool, Error> {
        voter.require_auth();

        let token = storage::get_token(&env);
        let tk = token::Client::new(&env, &token);

        if tk.balance(&voter) < 1 {
            return Err(Error::VoterDoesNotHoldToken);
        }
        
        if !check_dates(&env) {
            return Err(Error::BallotOutOfDate);
        }

        let v: Voter = Voter { id: &voter };

        if v.is_delegated(&env) {
            return Err(Error::VoterHasHisVoteDelegated)
        }

        if v.has_voted(&env) {
            return Err(Error::VoterHasAlreadyVoted)
        }
        
        storage::store_party(&env, &candidate);

        let mut votes: Vec<Address> = storage::get_votes(&env);
        let candidate_key: VCounter = VCounter::Counter(candidate);
        let mut d_votes = 0;
        if v.has_delegated_vote(&env) {
            d_votes = 1;
        }

        let count = 1 + d_votes + storage::get_candidate_votes_count(&env, &candidate_key);
        votes.push_back(voter);
 
        storage::update_candidate_count(&env, candidate_key, count);
        storage::update_votes(&env, votes);

        Ok(true)
    }

    pub fn delegate(env: Env, o_voter: Address, d_voter: Address) -> Result<bool, Error> {

        o_voter.require_auth();
        let token = storage::get_token(&env);
        let tk = token::Client::new(&env, &token);

        if tk.balance(&o_voter) < 1 {
            return Err(Error::VoterDoesNotHoldToken);
        }

        if tk.balance(&d_voter) < 1 {
            return Err(Error::VoterDoesNotHoldToken);
        }

        if !check_dates(&env) {
            return Err(Error::BallotOutOfDate);
        }

        let ov: Voter = Voter { id: &o_voter };
        let dv: Voter = Voter { id: &d_voter };
    
        
        // Both ov and dv have not been voted yet    
        if ov.has_voted(&env) {
            return Err(Error::VoterOriginHasAlreadyVotedAndCannotDelegate)
        }

        if dv.has_voted(&env) {
            return Err(Error::VoterTargetHasAlreadyVotedAndCannotDelegate)
        }


        if ov.is_delegated(&env) {
            return Err(Error::VoterHasHisVoteDelegated)
        }

        if dv.has_delegated_vote(&env) {
            return Err(Error::VoterHasDelegatedVote)
        }

        let config = storage::get_config(&env);
        let expiration_ledger = (((config.to - config.from) / 5) + 60) as u32; // 5 seconds for every ledger. Add 5 extra minutes

        tk.approve(&o_voter, &d_voter, &expiration_ledger);

        Ok(true)

    }

    pub fn count(env: Env,  admin: Address) -> Map<Symbol, u32> {
        
        admin.require_auth();
        let pts = storage::get_candidates(&env);
        let mut count_map: Map<Symbol, u32>= Map::new(&env);
        for party in pts.iter() {
            let candidate_key = VCounter::Counter(party.clone());
            let candidate_count: u32 = storage::get_candidate_votes_count(&env, &candidate_key);
            count_map.set(party, candidate_count);
        }

        count_map
    }
}

mod test;
