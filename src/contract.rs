use crate::{errors::TokenLockupError, storage, types::Unlock, validation::require_valid_unlocks};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token::TokenClient, unwrap::UnwrapOptimized, Address,
    Env, Vec,
};

#[contract]
pub struct TokenLockup;

#[contractimpl]
impl TokenLockup {
    /********** Constructor **********/

    /// Initialize the contract
    ///
    /// ### Arguments
    /// * `admin` - The admin of the lockup contract
    /// * `owner` - The owner of the lockup contract
    /// * `token` - The token to lock up
    /// * `unlocks` - A vector of unlocks. Percentages represent the portion of the lockups token balance can be claimed
    ///               at the given unlock time. If multiple unlocks are claimed at once, the percentages are applied in order.
    ///
    /// ### Errors
    /// * AlreadyInitializedError - The contract has already been initialized
    /// * InvalidUnlocks - The unlock times do not represent a valid unlock sequence
    pub fn initialize(e: Env, admin: Address, owner: Address, unlocks: Vec<Unlock>) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, TokenLockupError::AlreadyInitializedError);
        }
        storage::extend_instance(&e);

        require_valid_unlocks(&e, &unlocks);
        storage::set_unlocks(&e, &unlocks);
        storage::set_admin(&e, &admin);
        storage::set_owner(&e, &owner);

        storage::set_is_init(&e);
    }

    /********** Read-Only **********/

    /// Get unlocks for the lockup
    pub fn unlocks(e: Env) -> Vec<Unlock> {
        storage::get_unlocks(&e).unwrap_optimized()
    }

    /// Get the admin address
    pub fn admin(e: Env) -> Address {
        storage::get_admin(&e)
    }

    /// Get the owner address
    pub fn owner(e: Env) -> Address {
        storage::get_owner(&e)
    }

    /********** Write **********/

    /// (Only admin) Set new unlocks for the lockup. The new unlocks must retain
    /// any existing unlocks that have already passed their unlock time.
    ///
    /// ### Arguments
    /// * `new_unlocks` - The new unlocks to set
    ///
    /// ### Errors
    /// * UnauthorizedError - The caller is not the admin
    /// * InvalidUnlocks - The unlock times do not represent a valid unlock sequence
    pub fn set_unlocks(e: Env, new_unlocks: Vec<Unlock>) {
        storage::get_admin(&e).require_auth();

        require_valid_unlocks(&e, &new_unlocks);

        storage::set_unlocks(&e, &new_unlocks);
    }

    /// (Only owner) Claim the unlocked tokens. The tokens are transferred to the owner.
    ///
    /// ### Arguments
    /// * `tokens` - A vector of tokens to claim
    ///
    /// ### Errors
    /// * UnauthorizedError - The caller is not the owner
    /// * NoUnlockedTokens - There are not tokens to claim for a given asset
    pub fn claim(e: Env, tokens: Vec<Address>) {
        let owner = storage::get_owner(&e);
        owner.require_auth();

        let unlocks = storage::get_unlocks(&e).unwrap_optimized();
        let is_fully_unlocked = unlocks.last_unchecked().time <= e.ledger().timestamp();

        for token in tokens.iter() {
            let mut claim_amount = 0;
            let token_client = TokenClient::new(&e, &token);
            let mut balance = token_client.balance(&e.current_contract_address());
            if is_fully_unlocked {
                claim_amount = balance;
            } else {
                let last_asset_claim = storage::get_last_claim(&e, &token);
                for unlock in unlocks.iter() {
                    if unlock.time > last_asset_claim && unlock.time <= e.ledger().timestamp() {
                        let transfer_amount = (balance * unlock.percent as i128) / 10000_i128;
                        balance -= transfer_amount;
                        claim_amount += transfer_amount;
                    }
                }
            }
            storage::set_last_claim(&e, &token, &e.ledger().timestamp());
            token_client.transfer(&e.current_contract_address(), &owner, &claim_amount);
        }
    }
}
