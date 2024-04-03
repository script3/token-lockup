#![cfg(test)]

use crate::{contract::TokenLockupClient, types::Unlock};
use soroban_sdk::{
    testutils::{Ledger as _, LedgerInfo},
    Address, Env, Vec,
};
mod contract {
    soroban_sdk::contractimport!(
        file = "./target/wasm32-unknown-unknown/optimized/token_lockup.wasm"
    );
}

/// Create a blend lockup contract via wasm
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `owner` - The address of the owner
/// * `unlocks` - The unlock ledger time (in seconds)
pub fn create_token_lockup_wasm<'a>(
    e: &Env,
    admin: &Address,
    owner: &Address,
    unlocks: &Vec<Unlock>,
) -> (Address, TokenLockupClient<'a>) {
    let token_lockup_address = e.register_contract_wasm(None, contract::WASM);
    let token_lockup_client: TokenLockupClient<'a> =
        TokenLockupClient::new(&e, &token_lockup_address);
    token_lockup_client.initialize(admin, owner, unlocks);
    (token_lockup_address, token_lockup_client)
}

/***** Env Utils *****/

pub const ONE_DAY_LEDGERS: u32 = 17280;

pub trait EnvTestUtils {
    /// Jump the env by the given amount of ledgers. Assumes 5 seconds per ledger.
    fn jump(&self, ledgers: u32);

    /// Jump the env by the given amount of seconds. Does not chance the sequence number.
    fn jump_time_only(&self, seconds: u64);

    /// Set the ledger to the default LedgerInfo
    ///
    /// Time -> 1441065600 (Sept 1st, 2015 12:00:00 AM UTC)
    /// Sequence -> 100
    fn set_default_info(&self);
}

impl EnvTestUtils for Env {
    fn jump(&self, ledgers: u32) {
        self.ledger().set(LedgerInfo {
            timestamp: self.ledger().timestamp().saturating_add(ledgers as u64 * 5),
            protocol_version: 20,
            sequence_number: self.ledger().sequence().saturating_add(ledgers),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1 * ONE_DAY_LEDGERS,
            min_persistent_entry_ttl: 120 * ONE_DAY_LEDGERS,
            max_entry_ttl: 365 * ONE_DAY_LEDGERS,
        });
    }

    fn jump_time_only(&self, seconds: u64) {
        self.ledger().set(LedgerInfo {
            timestamp: self.ledger().timestamp().saturating_add(seconds),
            protocol_version: 20,
            sequence_number: self.ledger().sequence(),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1 * ONE_DAY_LEDGERS,
            min_persistent_entry_ttl: 120 * ONE_DAY_LEDGERS,
            max_entry_ttl: 365 * ONE_DAY_LEDGERS,
        });
    }

    fn set_default_info(&self) {
        self.ledger().set(LedgerInfo {
            timestamp: 1441065600, // Sept 1st, 2015 12:00:00 AM UTC
            protocol_version: 20,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 1 * ONE_DAY_LEDGERS,
            min_persistent_entry_ttl: 120 * ONE_DAY_LEDGERS,
            max_entry_ttl: 365 * ONE_DAY_LEDGERS,
        });
    }
}
