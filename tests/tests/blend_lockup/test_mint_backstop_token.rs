use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, Symbol,
};
use tests::{
    blend::create_blend_contracts, env::EnvTestUtils, token_lockup::create_blend_lockup_wasm,
};
#[test]
fn test_mint_backstop_token() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let contracts = create_blend_contracts(&e, &bombadil);
    let (_, blend_lockup_client) =
        create_blend_lockup_wasm(&e, &bombadil, &frodo, &contracts.emitter.address);
    contracts
        .usdc
        .mint(&blend_lockup_client.address, &10000_0000000);
    contracts
        .blnd
        .mint(&blend_lockup_client.address, &10000000_0000000);
    blend_lockup_client.mint_backstop_token(&1000_0000000, &vec![&e, 10050_0000000, 251_0000000]);
    assert_eq!(
        e.auths()[0],
        (
            frodo.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    blend_lockup_client.address.clone(),
                    Symbol::new(&e, "mint_backstop_token"),
                    vec![
                        &e,
                        1000_0000000_i128.into_val(&e),
                        vec![&e, 10050_0000000_i128, 251_0000000_i128].into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    assert_eq!(
        1000_0000000,
        contracts
            .backstop_token
            .balance(&blend_lockup_client.address)
    )
}

#[test]
fn test_mint_backstop_token_to_many_tokens() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let contracts = create_blend_contracts(&e, &bombadil);
    let (_, blend_lockup_client) =
        create_blend_lockup_wasm(&e, &bombadil, &frodo, &contracts.emitter.address);
    contracts
        .usdc
        .mint(&blend_lockup_client.address, &10000_0000000);
    contracts
        .blnd
        .mint(&blend_lockup_client.address, &10000000_0000000);
    blend_lockup_client.mint_backstop_token(
        &1000_0000000,
        &vec![&e, 10050_0000000, 251_0000000, 1000_0000000],
    );
    assert_eq!(
        e.auths()[0],
        (
            frodo.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    blend_lockup_client.address.clone(),
                    Symbol::new(&e, "mint_backstop_token"),
                    vec![
                        &e,
                        1000_0000000_i128.into_val(&e),
                        vec![&e, 10050_0000000_i128, 251_0000000_i128, 1000_0000000_i128]
                            .into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    assert_eq!(
        1000_0000000,
        contracts
            .backstop_token
            .balance(&blend_lockup_client.address)
    )
}
