use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Symbol,
};
use tests::{
    blend::{create_backstop_wasm, create_blend_contracts, create_comet_wasm},
    env::EnvTestUtils,
    token_lockup::create_blend_lockup_wasm,
};
#[test]
fn test_update_backstop_token() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let contracts = create_blend_contracts(&e, &bombadil);
    let (_, blend_lockup_client) =
        create_blend_lockup_wasm(&e, &bombadil, &frodo, &contracts.emitter.address);

    assert_eq!(
        blend_lockup_client.backstop_token(),
        contracts.backstop_token.address
    );

    contracts.blnd.mint(&frodo, &1100000_0000000);
    contracts.usdc.mint(&frodo, &26000_0000000);
    contracts.backstop_token.join_pool(
        &51_000_0000000,
        &vec![&e, 1100000_0000000, 26000_0000000],
        &frodo,
    );

    let new_backstop = Address::generate(&e);

    contracts
        .backstop_token
        .transfer(&frodo, &new_backstop, &51_000_0000000);
    contracts
        .emitter
        .queue_swap_backstop(&new_backstop, &contracts.backstop.address);
    e.jump(60 * 60 * 24 * 31 / 5);
    contracts.emitter.swap_backstop();
    blend_lockup_client.update_backstop();

    let new_backstop_token_client = create_comet_wasm(
        &e,
        &bombadil,
        &contracts.usdc.address,
        &contracts.blnd.address,
    );
    create_backstop_wasm(
        &e,
        &new_backstop,
        &new_backstop_token_client.address,
        &contracts.emitter.address,
        &contracts.usdc.address,
        &contracts.blnd.address,
        &Address::generate(&e),
    );

    blend_lockup_client.update_backstop_token();
    assert_eq!(
        e.auths()[0],
        (
            frodo.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    blend_lockup_client.address.clone(),
                    Symbol::new(&e, "update_backstop_token"),
                    vec![&e,]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    assert_eq!(
        new_backstop_token_client.address,
        blend_lockup_client.backstop_token()
    );
}
