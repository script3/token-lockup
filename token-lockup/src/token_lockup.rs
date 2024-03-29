use soroban_sdk::{contractclient, Address, Env, Vec};

#[cfg(feature = "blend")]
use soroban_sdk::{Symbol, Val};

#[contractclient(name = "TokenLockupClient")]
pub trait TokenLockupTrait {
    /// Update the owner address
    ///
    /// # Arguments
    /// * `new_owner` - The new owner address
    fn update_owner(e: Env, new_owner: Address);

    /// Get the owner address
    fn get_owner(e: Env) -> Address;

    /// Update the admin address
    ///
    /// # Arguments
    /// * `new_admin` - The new admin address
    fn update_admin(e: Env, new_admin: Address);

    /// Get the admin address
    fn get_admin(e: Env) -> Address;

    /// Add a new unlock time and percentage unlocked if unlock already exists the percentage is updated
    ///
    /// # Arguments
    /// * `sequence` - The sequence number to unlock at
    /// * `percent` - The percentage of the total amount to unlock (expressed in basis points, 1/100th of a percent)
    fn add_unlock(e: Env, sequence: u32, percent: u64);

    /// Remove an unlock time and percentage unlocked
    /// If the unlock does not exist, contract will panic
    ///
    /// # Arguments
    /// * `sequence` - The sequence number to unlock at
    fn remove_unlock(e: Env, sequence: u32);

    /// Get the unlock percentage for a given sequence number
    ///
    /// # Arguments
    /// * `sequence` - The sequence number to unlock
    fn get_unlock(e: Env, sequence: u32) -> Option<u64>;

    /// Claim unlocked assets at a given sequence number
    ///
    /// # Arguments
    /// * `sequence` - The sequence number to claim
    /// * `asset_ids` - The asset addresses to claim
    fn claim(e: Env, sequence: u32, asset_ids: Vec<Address>);
}

#[cfg(feature = "standard")]
pub trait Standard {
    /// Initialize the contract with the admin and owner addresses
    ///
    /// # Arguments
    /// * `admin` - The admin address
    /// * `owner` - The owner address
    fn initialize(e: Env, admin: Address, owner: Address);
}

#[cfg(feature = "blend")]
pub trait Blend {
    /// Initialize the contract with the admin and owner addresses
    ///
    /// # Arguments
    /// * `admin` - The admin address
    /// * `owner` - The owner address
    /// * `emitter` - The emitter address
    fn initialize(e: Env, admin: Address, owner: Address, emitter: Address);

    /// Get the emitter address
    fn emitter(e: Env) -> Address;

    /// Get the backstop address
    fn backstop(e: Env) -> Address;

    /// Get the backstop token address
    fn backstop_token(e: Env) -> Address;

    /// Update the backstop address from the emitter
    fn update_backstop(e: Env) -> Address;

    /// Update the backstop token address from the backstop
    fn update_backstop_token(e: Env) -> Address;

    /// Execute a backstop function
    ///
    /// # Arguments
    /// * `fn_name` - The name of the function to execute
    /// * `args` - The arguments to pass to the function
    fn execute_backstop_fn(e: Env, fn_name: Symbol, args: Vec<Val>);

    /// Mint backstop tokens
    ///
    /// # Arguments
    /// * `mint_amount` - The amount of tokens to mint
    /// * `deposit_amounts` - The vector of amounts to deposit
    fn mint_backstop_token(e: Env, mint_amount: i128, deposit_amounts: Vec<i128>);

    /// Withdraw backstop tokens
    ///
    /// # Arguments
    /// * `burn_amount` - The amount of tokens to burn
    /// * `withdraw_amounts` - The vector of amounts to withdraw
    fn withdraw_backstop_token(e: Env, burn_amount: i128, withdraw_amounts: Vec<i128>);
}
