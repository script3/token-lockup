#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, IntoVal, Symbol, Vec,
};

use crate::{
    testutils::{create_token_lockup_wasm, EnvTestUtils},
    types::Unlock,
};

#[test]
fn test_lockup_claim() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let token_1_id = e.register_stellar_asset_contract(bombadil.clone());
    let token_1_admin_client = StellarAssetClient::new(&e, &token_1_id);
    let token_1_client = TokenClient::new(&e, &token_1_id);

    let now = e.ledger().timestamp();
    let unlocks: Vec<Unlock> = vec![
        &e,
        Unlock {
            time: now + 100,
            percent: 5000,
        },
        Unlock {
            time: now + 200,
            percent: 10000,
        },
    ];

    let (lockup_id, lockup_client) = create_token_lockup_wasm(&e, &bombadil, &frodo, &unlocks);

    // send tokens to lockup
    let token_1_total: i128 = 3_000 * 10i128.pow(7);
    token_1_admin_client.mint(&lockup_id, &token_1_total);

    e.jump_time_only(150); // t = 150

    // claim first unlock
    lockup_client.claim(&vec![&e, token_1_id.clone()]);
    assert_eq!(token_1_client.balance(&frodo), token_1_total / 2);
    assert_eq!(token_1_client.balance(&lockup_id), token_1_total / 2);

    // verify set_unlocks validates unlocks
    let invalid_unlocks: Vec<Unlock> = vec![
        &e,
        Unlock {
            time: now + 100,
            percent: 100,
        },
        Unlock {
            time: now + 200,
            percent: 5000,
        },
        Unlock {
            time: now + 300,
            percent: 10000,
        },
    ];
    let result = lockup_client.try_set_unlocks(&invalid_unlocks);
    assert_eq!(
        result.err(),
        Some(Ok(soroban_sdk::Error::from_contract_error(102)))
    );

    // set valid unlocks
    let valid_unlocks: Vec<Unlock> = vec![
        &e,
        Unlock {
            time: now + 100,
            percent: 5000,
        },
        Unlock {
            time: now + 200,
            percent: 5000,
        },
        Unlock {
            time: now + 300,
            percent: 10000,
        },
    ];
    lockup_client.set_unlocks(&valid_unlocks);

    // validate admin is authenticated
    assert_eq!(
        e.auths()[0],
        (
            bombadil.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    lockup_client.address.clone(),
                    Symbol::new(&e, "set_unlocks"),
                    vec![&e, valid_unlocks.into_val(&e),]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let unlocks = lockup_client.unlocks();
    assert_eq!(unlocks.len(), valid_unlocks.len());
    for i in 0..unlocks.len() {
        assert_eq!(
            unlocks.get_unchecked(i).time,
            valid_unlocks.get_unchecked(i).time
        );
        assert_eq!(
            unlocks.get_unchecked(i).percent,
            valid_unlocks.get_unchecked(i).percent
        );
    }

    // claim second unlock
    e.jump_time_only(100); // t = 250
    lockup_client.claim(&vec![&e, token_1_id.clone()]);
    assert_eq!(
        token_1_client.balance(&frodo),
        token_1_total / 2 + token_1_total / 4
    );
    assert_eq!(token_1_client.balance(&lockup_id), token_1_total / 4);

    // claim final unlock
    e.jump_time_only(50); // t = 300
    lockup_client.claim(&vec![&e, token_1_id.clone()]);
    assert_eq!(token_1_client.balance(&frodo), token_1_total);
    assert_eq!(token_1_client.balance(&lockup_id), 0);
}
