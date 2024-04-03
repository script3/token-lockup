use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct Unlock {
    /// The ledger time (in seconds) the unlock occurs
    pub time: u64,
    /// The amount of current tokens (in bps) to unlock
    pub percent: u32,
}
