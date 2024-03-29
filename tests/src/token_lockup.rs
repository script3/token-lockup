use soroban_sdk::{Address, Env};
use token_lockup::{TokenLockup, TokenLockupClient};
mod standard_lockup {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/optimized/standard_token_lockup.wasm"
    );
}
mod blend_lockup {
    soroban_sdk::contractimport!(
        file = "../target/wasm32-unknown-unknown/optimized/blend_token_lockup.wasm"
    );
}
use blend_lockup::{Client as BlendLockupClient, WASM as BlendLockupWASN};
use standard_lockup::{Client as StandardLockupClient, WASM as StandardLockupWASM};

/// Create a token lockup contract with the wasm contract
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `owner` - The address of the owner
pub fn create_standard_lockup<'a>(
    e: &Env,
    admin: &Address,
    owner: &Address,
) -> (Address, TokenLockupClient<'a>) {
    let token_lockup_address = e.register_contract(None, TokenLockup {});
    let token_lockup_client: TokenLockupClient<'a> =
        TokenLockupClient::new(&e, &token_lockup_address);

    token_lockup_client.initialize(admin, owner);
    (token_lockup_address, token_lockup_client)
}

/// Create a standard lockup contract via wasm
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `owner` - The address of the owner
pub fn create_standard_lockup_wasm<'a>(
    e: &Env,
    admin: &Address,
    owner: &Address,
) -> (Address, StandardLockupClient<'a>) {
    let token_lockup_address = e.register_contract_wasm(None, StandardLockupWASM);
    let token_lockup_client: StandardLockupClient<'a> =
        StandardLockupClient::new(&e, &token_lockup_address);
    token_lockup_client.initialize(admin, owner);
    (token_lockup_address, token_lockup_client)
}

/// Create a blend lockup contract via wasm
///
/// ### Arguments
/// * `admin` - The address of the admin
/// * `owner` - The address of the owner
pub fn create_blend_lockup_wasm<'a>(
    e: &Env,
    admin: &Address,
    owner: &Address,
    emitter: &Address,
) -> (Address, BlendLockupClient<'a>) {
    let token_lockup_address = e.register_contract_wasm(None, BlendLockupWASN);
    let token_lockup_client: BlendLockupClient<'a> =
        BlendLockupClient::new(&e, &token_lockup_address);
    token_lockup_client.initialize(admin, owner, emitter);
    (token_lockup_address, token_lockup_client)
}
