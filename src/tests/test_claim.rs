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
    let token_2_id = e.register_stellar_asset_contract(bombadil.clone());
    let token_1_admin_client = StellarAssetClient::new(&e, &token_1_id);
    let token_2_admin_client = StellarAssetClient::new(&e, &token_2_id);
    let token_1_client = TokenClient::new(&e, &token_1_id);
    let token_2_client = TokenClient::new(&e, &token_2_id);

    let now = e.ledger().timestamp();
    // half of tokens at first traunch after 10k seconds
    // split the remaining half into 10 equal installments every 1k seconds
    // for a total of 11 traunches
    let unlocks: Vec<Unlock> = vec![
        &e,
        Unlock {
            time: now + 10000,
            percent: 5000,
        },
        Unlock {
            time: now + 11000,
            percent: 10000 / 10,
        },
        Unlock {
            time: now + 12000,
            percent: 10000 / 9,
        },
        Unlock {
            time: now + 13000,
            percent: 10000 / 8,
        },
        Unlock {
            time: now + 14000,
            percent: 10000 / 7,
        },
        Unlock {
            time: now + 15000,
            percent: 10000 / 6,
        },
        Unlock {
            time: now + 16000,
            percent: 10000 / 5,
        },
        Unlock {
            time: now + 17000,
            percent: 10000 / 4,
        },
        Unlock {
            time: now + 18000,
            percent: 10000 / 3,
        },
        Unlock {
            time: now + 19000,
            percent: 10000 / 2,
        },
        Unlock {
            time: now + 20000,
            percent: 10000,
        },
    ];

    let (lockup_id, lockup_client) = create_token_lockup_wasm(&e, &bombadil, &frodo, &unlocks);

    // send tokens to lockup
    let token_1_total: i128 = 1_000 * 10i128.pow(7);
    let token_2_total: i128 = 400_000 * 10i128.pow(7);
    token_1_admin_client.mint(&lockup_id, &token_1_total);
    token_2_admin_client.mint(&lockup_id, &token_2_total);

    e.jump_time_only(5000);

    // verify owner cannot claim early
    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);

    // validate claim requires owner signature
    assert_eq!(
        e.auths()[0],
        (
            frodo.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    lockup_client.address.clone(),
                    Symbol::new(&e, "claim"),
                    vec![
                        &e,
                        vec![&e, token_1_id.clone(), token_2_id.clone()].into_val(&e),
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );

    // validate no tokens moved
    assert_eq!(token_1_client.balance(&frodo), 0);
    assert_eq!(token_2_client.balance(&frodo), 0);
    assert_eq!(token_1_client.balance(&lockup_id), token_1_total);
    assert_eq!(token_2_client.balance(&lockup_id), token_2_total);

    // claim after first traunch
    e.jump_time_only(5000); // t = 10000

    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);
    assert_eq!(token_1_client.balance(&frodo), token_1_total / 2);
    assert_eq!(token_2_client.balance(&frodo), token_2_total / 2);
    assert_eq!(token_1_client.balance(&lockup_id), token_1_total / 2);
    assert_eq!(token_2_client.balance(&lockup_id), token_2_total / 2);

    // verify duplicate claims within the same traunch don't send tokens
    e.jump_time_only(999); // t = 10999
    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);
    assert_eq!(token_1_client.balance(&frodo), token_1_total / 2);
    assert_eq!(token_2_client.balance(&frodo), token_2_total / 2);
    assert_eq!(token_1_client.balance(&lockup_id), token_1_total / 2);
    assert_eq!(token_2_client.balance(&lockup_id), token_2_total / 2);

    // claim after second traunch and verify individual token claims
    e.jump_time_only(100); // t = 11099
    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);
    assert_eq!(
        token_1_client.balance(&frodo),
        token_1_total / 2 + token_1_total / 20
    );
    assert_eq!(
        token_2_client.balance(&frodo),
        token_2_total / 2 + token_2_total / 20
    );
    assert_eq!(
        token_1_client.balance(&lockup_id),
        token_1_total / 2 - token_1_total / 20
    );
    assert_eq!(
        token_2_client.balance(&lockup_id),
        token_2_total / 2 - token_2_total / 20
    );

    // claim only token 1 for third traunch and verify token 2 does not miss payment of third traunch
    // when the fourth traunch is claimed
    e.jump_time_only(1000); // t = 12099
    lockup_client.claim(&vec![&e, token_1_id.clone()]);
    assert_eq!(
        token_1_client.balance(&frodo),
        token_1_total / 2 + 2 * token_1_total / 20 - 50000
    ); // rounding of 10000 / 9
    assert_eq!(
        token_2_client.balance(&frodo),
        token_2_total / 2 + token_2_total / 20
    );
    assert_eq!(
        token_1_client.balance(&lockup_id),
        token_1_total / 2 - 2 * token_1_total / 20 + 50000
    ); // rounding of 10000 / 9
    assert_eq!(
        token_2_client.balance(&lockup_id),
        token_2_total / 2 - token_2_total / 20
    );

    // claim both tokens for 4th traunch and verify total amount is the same (rounding of 10000 / 9 still has numbers off)
    e.jump_time_only(1000); // t = 13099
    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);
    assert_eq!(
        token_1_client.balance(&frodo),
        token_1_total / 2 + 3 * token_1_total / 20 - 43750
    );
    assert_eq!(
        token_2_client.balance(&frodo),
        token_2_total / 2 + 3 * token_2_total / 20 - 17500000
    );
    assert_eq!(
        token_1_client.balance(&lockup_id),
        token_1_total / 2 - 3 * token_1_total / 20 + 43750
    );
    assert_eq!(
        token_2_client.balance(&lockup_id),
        token_2_total / 2 - 3 * token_2_total / 20 + 17500000
    );

    // claim the next 5 traunches (to the 9th traunch) at once
    e.jump_time_only(5000); // t = 18099
    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);
    assert_eq!(
        token_1_client.balance(&frodo),
        token_1_total / 2 + 8 * token_1_total / 20 - 209183
    );
    assert_eq!(
        token_2_client.balance(&frodo),
        token_2_total / 2 + 8 * token_2_total / 20 - 83672718
    );
    assert_eq!(
        token_1_client.balance(&lockup_id),
        token_1_total / 2 - 8 * token_1_total / 20 + 209183
    );
    assert_eq!(
        token_2_client.balance(&lockup_id),
        token_2_total / 2 - 8 * token_2_total / 20 + 83672718
    );

    // inflate token 2 balance by 5000. There are two claims left, with the next being 50% of remaining, and the last being everything
    let inflation_amount = 5000 * 10i128.pow(7);
    token_2_admin_client.mint(&lockup_id, &inflation_amount);
    // claim the next traunch
    e.jump_time_only(1000); // t = 19099
    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);
    assert_eq!(
        token_1_client.balance(&frodo),
        token_1_total / 2 + 9 * token_1_total / 20 - 104592
    );
    assert_eq!(
        token_2_client.balance(&frodo),
        token_2_total / 2 + 9 * token_2_total / 20 + inflation_amount / 2 - 41836359
    );
    assert_eq!(
        token_1_client.balance(&lockup_id),
        token_1_total / 2 - 9 * token_1_total / 20 + 104592
    );
    assert_eq!(
        token_2_client.balance(&lockup_id),
        token_2_total / 2 - 9 * token_2_total / 20 + inflation_amount / 2 + 41836359
    );

    // validate everything included rounding losses is claimed after all unlocks are done
    e.jump_time_only(1000 - 99); // t = 20000
    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);
    assert_eq!(token_1_client.balance(&frodo), token_1_total);
    assert_eq!(
        token_2_client.balance(&frodo),
        token_2_total + inflation_amount
    );
    assert_eq!(token_1_client.balance(&lockup_id), 0);
    assert_eq!(token_2_client.balance(&lockup_id), 0);
}

#[test]
fn test_lockup_claim_at_end() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let token_1_id = e.register_stellar_asset_contract(bombadil.clone());
    let token_2_id = e.register_stellar_asset_contract(bombadil.clone());
    let token_1_admin_client = StellarAssetClient::new(&e, &token_1_id);
    let token_2_admin_client = StellarAssetClient::new(&e, &token_2_id);
    let token_1_client = TokenClient::new(&e, &token_1_id);
    let token_2_client = TokenClient::new(&e, &token_2_id);

    let now = e.ledger().timestamp();
    // half of tokens at first traunch after 10k seconds
    // split the remaining half into 10 equal installments every 1k seconds
    // for a total of 11 traunches
    let unlocks: Vec<Unlock> = vec![
        &e,
        Unlock {
            time: now + 10000,
            percent: 5000,
        },
        Unlock {
            time: now + 11000,
            percent: 10000 / 10,
        },
        Unlock {
            time: now + 12000,
            percent: 10000 / 9,
        },
        Unlock {
            time: now + 13000,
            percent: 10000 / 8,
        },
        Unlock {
            time: now + 14000,
            percent: 10000 / 7,
        },
        Unlock {
            time: now + 15000,
            percent: 10000 / 6,
        },
        Unlock {
            time: now + 16000,
            percent: 10000 / 5,
        },
        Unlock {
            time: now + 17000,
            percent: 10000 / 4,
        },
        Unlock {
            time: now + 18000,
            percent: 10000 / 3,
        },
        Unlock {
            time: now + 19000,
            percent: 10000 / 2,
        },
        Unlock {
            time: now + 20000,
            percent: 10000,
        },
    ];

    let (lockup_id, lockup_client) = create_token_lockup_wasm(&e, &bombadil, &frodo, &unlocks);

    // send tokens to lockup
    let token_1_total: i128 = 1_000 * 10i128.pow(7);
    let token_2_total: i128 = 400_000 * 10i128.pow(7);
    token_1_admin_client.mint(&lockup_id, &token_1_total);
    token_2_admin_client.mint(&lockup_id, &token_2_total);

    e.jump_time_only(20000);

    // validate everything gets claimed
    lockup_client.claim(&vec![&e, token_1_id.clone(), token_2_id.clone()]);
    assert_eq!(token_1_client.balance(&frodo), token_1_total);
    assert_eq!(token_2_client.balance(&frodo), token_2_total);
    assert_eq!(token_1_client.balance(&lockup_id), 0);
    assert_eq!(token_2_client.balance(&lockup_id), 0);

    // time travel and validate any future tokens can be claimed by the owner
    e.jump_time_only(999999999999);
    let inflation_amount: i128 = 1_000 * 10i128.pow(7);
    token_1_admin_client.mint(&lockup_id, &inflation_amount);
    lockup_client.claim(&vec![&e, token_1_id.clone()]);
    assert_eq!(
        token_1_client.balance(&frodo),
        token_1_total + inflation_amount
    );
}
