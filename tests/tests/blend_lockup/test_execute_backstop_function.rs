use blend::backstop::Q4W;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, Symbol,
};
use tests::{
    blend::{self, create_blend_contracts},
    env::EnvTestUtils,
    token_lockup::create_blend_lockup_wasm,
};

#[test]
fn test_execute_backstop() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let contracts = create_blend_contracts(&e, &bombadil);
    let (_, blend_lockup_client) =
        create_blend_lockup_wasm(&e, &bombadil, &frodo, &contracts.emitter.address);

    let lp_mint_amount = 100_0000000;
    contracts
        .blnd
        .mint(&blend_lockup_client.address, &100000_0000000);
    contracts
        .usdc
        .mint(&blend_lockup_client.address, &2500_0000000);

    contracts.backstop_token.join_pool(
        &lp_mint_amount,
        &vec![&e, 100000_0000000, 2500_0000000],
        &blend_lockup_client.address,
    );

    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "deposit"),
        &vec![
            &e,
            blend_lockup_client.address.into_val(&e),
            contracts.pool.address.into_val(&e),
            lp_mint_amount.into_val(&e),
        ],
    );
    assert_eq!(
        e.auths()[0],
        (
            frodo.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    blend_lockup_client.address.clone(),
                    Symbol::new(&e, "execute_backstop_fn"),
                    vec![
                        &e,
                        Symbol::new(&e, "deposit").into_val(&e),
                        (
                            blend_lockup_client.address.clone(),
                            contracts.pool.address.clone(),
                            (lp_mint_amount),
                        )
                            .into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    assert_eq!(
        lp_mint_amount,
        contracts
            .backstop
            .user_balance(&contracts.pool.address, &blend_lockup_client.address)
            .shares
    );

    e.jump(6000);
    contracts.emitter.distribute();
    contracts.backstop.gulp_emissions();

    e.jump(6000);
    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "claim"),
        &vec![
            &e,
            blend_lockup_client.address.into_val(&e),
            vec![&e, contracts.pool.address.clone()].into_val(&e),
            blend_lockup_client.address.into_val(&e),
        ],
    );
    let new_shares = contracts
        .backstop
        .user_balance(&contracts.pool.address, &blend_lockup_client.address)
        .shares;
    assert!(new_shares > lp_mint_amount);
    let claim_amount = new_shares - lp_mint_amount;

    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "queue_withdrawal"),
        &vec![
            &e,
            blend_lockup_client.address.into_val(&e),
            contracts.pool.address.into_val(&e),
            (lp_mint_amount + claim_amount).into_val(&e),
        ],
    );

    assert_eq!(
        Some(Q4W {
            amount: lp_mint_amount + claim_amount,
            exp: e.ledger().timestamp() + 21 * 24 * 60 * 60
        }),
        contracts
            .backstop
            .user_balance(&contracts.pool.address, &blend_lockup_client.address)
            .q4w
            .get(0)
    );

    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "dequeue_withdrawal"),
        &vec![
            &e,
            blend_lockup_client.address.into_val(&e),
            contracts.pool.address.into_val(&e),
            (lp_mint_amount + claim_amount).into_val(&e),
        ],
    );
    assert_eq!(
        None,
        contracts
            .backstop
            .user_balance(&contracts.pool.address, &blend_lockup_client.address)
            .q4w
            .get(0)
    );

    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "queue_withdrawal"),
        &vec![
            &e,
            blend_lockup_client.address.into_val(&e),
            contracts.pool.address.into_val(&e),
            (lp_mint_amount + claim_amount).into_val(&e),
        ],
    );
    e.jump(21 * 24 * 60 * 60 / 5);

    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "withdraw"),
        &vec![
            &e,
            blend_lockup_client.address.into_val(&e),
            contracts.pool.address.into_val(&e),
            (lp_mint_amount + claim_amount).into_val(&e),
        ],
    );

    assert_eq!(
        0,
        contracts
            .backstop
            .user_balance(&contracts.pool.address, &blend_lockup_client.address)
            .shares
    );
    assert_eq!(
        lp_mint_amount + claim_amount,
        contracts
            .backstop_token
            .balance(&blend_lockup_client.address)
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn test_execute_backstop_claim_invalid_to() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let contracts = create_blend_contracts(&e, &bombadil);
    let (_, blend_lockup_client) =
        create_blend_lockup_wasm(&e, &bombadil, &frodo, &contracts.emitter.address);

    let lp_mint_amount = 100_0000000;
    contracts
        .blnd
        .mint(&blend_lockup_client.address, &100000_0000000);
    contracts
        .usdc
        .mint(&blend_lockup_client.address, &2500_0000000);

    contracts.backstop_token.join_pool(
        &lp_mint_amount,
        &vec![&e, 100000_0000000, 2500_0000000],
        &blend_lockup_client.address,
    );

    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "deposit"),
        &vec![
            &e,
            blend_lockup_client.address.into_val(&e),
            contracts.pool.address.into_val(&e),
            lp_mint_amount.into_val(&e),
        ],
    );
    assert_eq!(
        lp_mint_amount,
        contracts
            .backstop
            .user_balance(&contracts.pool.address, &blend_lockup_client.address)
            .shares
    );

    e.jump(6000);
    contracts.emitter.distribute();
    contracts.backstop.gulp_emissions();

    e.jump(6000);
    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "claim"),
        &vec![
            &e,
            blend_lockup_client.address.into_val(&e),
            vec![&e, contracts.pool.address.clone()].into_val(&e),
            Address::generate(&e).into_val(&e),
        ],
    );
}

#[test]
#[should_panic(expected = "Error(WasmVm, MissingValue)")]
fn test_execute_backstop_invalid_function() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);
    let contracts = create_blend_contracts(&e, &bombadil);
    let (_, blend_lockup_client) =
        create_blend_lockup_wasm(&e, &bombadil, &frodo, &contracts.emitter.address);

    blend_lockup_client.execute_backstop_fn(
        &Symbol::new(&e, "InvalidFunction"),
        &vec![&e, 1.into_val(&e)],
    );
}
