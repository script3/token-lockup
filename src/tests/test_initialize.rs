#![cfg(test)]

use soroban_sdk::{testutils::Address as _, vec, Address, Env, Error, Vec};

use crate::{
    contract::{TokenLockup, TokenLockupClient},
    testutils::EnvTestUtils,
    types::Unlock,
};

#[test]
fn test_lockup_initialize_validates_unlocks() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let now = e.ledger().timestamp();
    // half of current tokens at first cliff, then equal payments of the remaining half in
    // 10 installments every 1k seconds
    let unlocks: Vec<Unlock> = vec![
        &e,
        Unlock {
            time: now + 10000,
            percent: 5000,
        },
        Unlock {
            time: now + 20000,
            percent: 10000 - 1,
        },
    ];

    let lockup_id = e.register_contract(None, TokenLockup {});
    let lockup_client = TokenLockupClient::new(&e, &lockup_id);

    let result = lockup_client.try_initialize(&bombadil, &frodo, &unlocks);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(100))));
}

#[test]
fn test_lockup_initialize_once() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let now = e.ledger().timestamp();
    // half of current tokens at first cliff, then equal payments of the remaining half in
    // 10 installments every 1k seconds
    let unlocks: Vec<Unlock> = vec![
        &e,
        Unlock {
            time: now + 10000,
            percent: 5000,
        },
        Unlock {
            time: now + 20000,
            percent: 10000,
        },
    ];

    let lockup_id = e.register_contract(None, TokenLockup {});
    let lockup_client = TokenLockupClient::new(&e, &lockup_id);

    lockup_client.initialize(&bombadil, &frodo, &unlocks);

    let new_unlocks: Vec<Unlock> = vec![
        &e,
        Unlock {
            time: now,
            percent: 10000,
        },
    ];
    let result = lockup_client.try_initialize(&bombadil, &frodo, &new_unlocks);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(3))));
}
