use soroban_sdk::{Address, Env, IntoVal, Map, Symbol, TryFromVal, Val};

const ADMIN_KEY: &str = "Admin";
const OWNER_KEY: &str = "Owner";
const EMITTER_KEY: &str = "Emitter";
const BACKSTOP_KEY: &str = "Backstop";
const BACKSTOP_TOKEN_KEY: &str = "BackstopToken";
const IS_INIT_KEY: &str = "IsInit";
const UNLOCKS_KEY: &str = "Unlocks";

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average
const LEDGER_THRESHOLD_SHARED: u32 = 14 * ONE_DAY_LEDGERS;
const LEDGER_BUMP_SHARED: u32 = 15 * ONE_DAY_LEDGERS;

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD_SHARED, LEDGER_BUMP_SHARED);
}

/// Fetch an entry in persistent storage that has a default value if it doesn't exist
fn get_persistent_default<K: IntoVal<Env, Val>, V: TryFromVal<Env, Val>>(
    e: &Env,
    key: &K,
    default: V,
    bump_threshold: u32,
    bump_amount: u32,
) -> V {
    if let Some(result) = e.storage().persistent().get::<K, V>(key) {
        e.storage()
            .persistent()
            .extend_ttl(key, bump_threshold, bump_amount);
        result
    } else {
        default
    }
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

/// Get the emitter address
pub fn get_emitter(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, EMITTER_KEY))
        .unwrap()
}

/// Set the emitter address
pub fn set_emitter(e: &Env, emitter: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, EMITTER_KEY), &emitter);
}

/// Get the backstop address
pub fn get_backstop(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, BACKSTOP_KEY))
        .unwrap()
}

/// Set the backstop address
pub fn set_backstop(e: &Env, backstop: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, BACKSTOP_KEY), &backstop);
}

/// Get the backstop token address
pub fn get_backstop_token(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, BACKSTOP_TOKEN_KEY))
        .unwrap()
}

/// Set the backstop token address
pub fn set_backstop_token(e: &Env, backstop_token: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, BACKSTOP_TOKEN_KEY), &backstop_token);
}

/********** Persistent **********/

/// Get the mapping of sequence to unlock percentage
pub fn get_unlocks(e: &Env) -> Map<u32, u64> {
    let unlocks: Map<u32, u64> = get_persistent_default(
        e,
        &Symbol::new(e, UNLOCKS_KEY),
        Map::new(e),
        LEDGER_THRESHOLD_SHARED,
        LEDGER_BUMP_SHARED,
    );
    unlocks
}

/// Set the mapping of sequence to unlock percentage
pub fn set_unlocks(e: &Env, unlocks: &Map<u32, u64>) {
    e.storage()
        .persistent()
        .set::<Symbol, Map<u32, u64>>(&Symbol::new(e, UNLOCKS_KEY), &unlocks);
}
