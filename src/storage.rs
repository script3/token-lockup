use soroban_sdk::{Address, Env, Symbol, Vec};

use crate::types::Unlock;

/********** Ledger Thresholds **********/

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger

const LEDGER_BUMP: u32 = 120 * ONE_DAY_LEDGERS;
const LEDGER_THRESHOLD: u32 = LEDGER_BUMP - 20 * ONE_DAY_LEDGERS;

/********** Ledger Keys **********/

const OWNER_KEY: &str = "Owner";
const ADMIN_KEY: &str = "Admin";
const IS_INIT_KEY: &str = "IsInit";
const UNLOCKS_KEY: &str = "Unlocks";

/********** Ledger Thresholds **********/

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

/********** Instance **********/

/// Check if the contract has been initialized
pub fn get_is_init(e: &Env) -> bool {
    e.storage().instance().has(&Symbol::new(e, IS_INIT_KEY))
}

/// Set the contract as initialized
pub fn set_is_init(e: &Env) {
    e.storage()
        .instance()
        .set::<Symbol, bool>(&Symbol::new(e, IS_INIT_KEY), &true);
}

/// Get the owner address
pub fn get_owner(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, OWNER_KEY))
        .unwrap()
}

/// Set the owner address
pub fn set_owner(e: &Env, owner: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, OWNER_KEY), &owner);
}

/// Get the admin address
pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY))
        .unwrap()
}

/// Set the admin address
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY), &admin);
}

/********** Persistant **********/

/// Get the times of the lockup unlocks
pub fn get_unlocks(e: &Env) -> Option<Vec<Unlock>> {
    let key = Symbol::new(e, UNLOCKS_KEY);
    let result = e.storage().persistent().get::<Symbol, Vec<Unlock>>(&key);
    if result.is_some() {
        e.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
    }
    result
}

/// Set the times of the lockup unlocks
pub fn set_unlocks(e: &Env, unlocks: &Vec<Unlock>) {
    let key = Symbol::new(e, UNLOCKS_KEY);
    e.storage()
        .persistent()
        .set::<Symbol, Vec<Unlock>>(&key, unlocks);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

/// Get the last claim time for a token
pub fn get_last_claim(e: &Env, token: &Address) -> u64 {
    let result = e.storage().persistent().get::<Address, u64>(&token);
    match result {
        Some(last_claim) => {
            e.storage()
                .persistent()
                .extend_ttl(&token, LEDGER_THRESHOLD, LEDGER_BUMP);
            last_claim
        }
        None => 0,
    }
}

/// Set the last claim time for a token
pub fn set_last_claim(e: &Env, token: &Address, time: &u64) {
    e.storage().persistent().set::<Address, u64>(&token, time);
    e.storage()
        .persistent()
        .extend_ttl(&token, LEDGER_THRESHOLD, LEDGER_BUMP);
}
