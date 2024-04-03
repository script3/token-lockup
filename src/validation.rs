use soroban_sdk::{panic_with_error, Env, Vec};

use crate::{errors::TokenLockupError, storage, types::Unlock};

/// Validate the unlock times and unlock percents. If a current unlocks are already set, validates that
/// any unlocks that have already occured remain unchanged. A maximum of 48 unlock periods are supported.
///
/// Panic if the unlock times are not in ascending order, if the unlock percents are not valid, or if
/// there are more than 48 unlock periods.
pub fn require_valid_unlocks(e: &Env, unlocks: &Vec<Unlock>) {
    if unlocks.is_empty() || unlocks.len() > 48 || unlocks.last_unchecked().percent != 10000 {
        panic_with_error!(&e, TokenLockupError::InvalidUnlocks);
    }

    let prev_unlocks_opt = storage::get_unlocks(e);
    if let Some(ref prev_unlocks) = prev_unlocks_opt {
        // check if prev_unlocks are already unlocked
        if prev_unlocks.last_unchecked().time <= e.ledger().timestamp() {
            panic_with_error!(&e, TokenLockupError::AlreadyUnlocked);
        }
    }
    let mut last_time = 0;
    for (i, unlock) in unlocks.iter().enumerate() {
        if unlock.percent > 10000 || unlock.percent == 0 {
            panic_with_error!(&e, TokenLockupError::InvalidUnlocks);
        }

        if let Some(ref prev_unlocks) = prev_unlocks_opt {
            // validate that any old unlocks remain unchanged
            if let Some(prev_unlock) = prev_unlocks.get(i as u32) {
                if prev_unlock.time <= e.ledger().timestamp() && prev_unlock != unlock {
                    panic_with_error!(&e, TokenLockupError::AlreadyUnlocked);
                }
            }
        }

        if unlock.time <= last_time {
            panic_with_error!(&e, TokenLockupError::InvalidUnlocks);
        }
        last_time = unlock.time;
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::vec;

    use crate::testutils::EnvTestUtils;

    use super::*;

    #[test]
    fn test_require_valid_unlocks_first_time() {
        let e = Env::default();
        let unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 500,
                percent: 10000,
            },
        ];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            require_valid_unlocks(&e, &unlocks);
            assert!(true);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #100)")]
    fn test_require_valid_unlocks_empty() {
        let e = Env::default();
        let unlocks = vec![&e];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            require_valid_unlocks(&e, &unlocks);
            assert!(false);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #100)")]
    fn test_require_valid_unlocks_over_48() {
        let e = Env::default();
        let unlocks = vec![&e];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            require_valid_unlocks(&e, &unlocks);
            assert!(false);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #100)")]
    fn test_require_valid_unlocks_does_not_end_with_100_percent() {
        let e = Env::default();
        let unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 500,
                percent: 10000 - 1,
            },
        ];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            require_valid_unlocks(&e, &unlocks);
            assert!(false);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #100)")]
    fn test_require_valid_unlocks_invalid_percent() {
        let e = Env::default();
        let unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 500,
                percent: 10000 + 1,
            },
        ];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            require_valid_unlocks(&e, &unlocks);
            assert!(false);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #100)")]
    fn test_require_valid_unlocks_out_of_order() {
        let e = Env::default();
        let unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 195,
                percent: 100,
            },
            Unlock {
                time: 500,
                percent: 10000,
            },
        ];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            require_valid_unlocks(&e, &unlocks);
            assert!(false);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #100)")]
    fn test_require_valid_unlocks_duplicate_unlock_time() {
        let e = Env::default();
        let unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 200,
                percent: 100,
            },
            Unlock {
                time: 500,
                percent: 10000,
            },
        ];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            require_valid_unlocks(&e, &unlocks);
            assert!(false);
        });
    }

    #[test]
    fn test_require_valid_unlocks_replaces() {
        let e = Env::default();
        e.jump_time_only(300); // t = 300

        let old_unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 500,
                percent: 10000,
            },
        ];
        let new_unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 250,
                percent: 2500,
            },
            Unlock {
                time: 800,
                percent: 10000,
            },
        ];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            storage::set_unlocks(&e, &old_unlocks);
            require_valid_unlocks(&e, &new_unlocks);
            assert!(true);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #102)")]
    fn test_require_valid_unlocks_replaces_changes_old() {
        let e = Env::default();
        e.jump_time_only(300); // t = 300

        let old_unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 500,
                percent: 10000,
            },
        ];
        let new_unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 350,
                percent: 2500,
            },
            Unlock {
                time: 800,
                percent: 10000,
            },
        ];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            storage::set_unlocks(&e, &old_unlocks);
            require_valid_unlocks(&e, &new_unlocks);
            assert!(false);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #102)")]
    fn test_require_valid_unlocks_replaces_fully_unlocked() {
        let e = Env::default();
        e.jump_time_only(500); // t = 500

        let old_unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 500,
                percent: 10000,
            },
        ];
        let new_unlocks = vec![
            &e,
            Unlock {
                time: 100,
                percent: 5000,
            },
            Unlock {
                time: 200,
                percent: 2500,
            },
            Unlock {
                time: 500,
                percent: 10000,
            },
            Unlock {
                time: 800,
                percent: 10000,
            },
        ];

        let lockup = e.register_contract(None, crate::contract::TokenLockup {});

        e.as_contract(&lockup, || {
            storage::set_unlocks(&e, &old_unlocks);
            require_valid_unlocks(&e, &new_unlocks);
            assert!(false);
        });
    }
}
