use soroban_sdk::{testutils::Address as _, Address, Env};
use tests::{
    blend::create_blend_contracts, env::EnvTestUtils, token_lockup::create_blend_lockup_wasm,
};
#[test]
fn test_initalize() {
    let e = Env::default();
    e.budget().reset_unlimited();
    e.set_default_info();
    e.mock_all_auths();

    let bombadil = Address::generate(&e);
    let frodo = Address::generate(&e);

    let contracts = create_blend_contracts(&e, &bombadil);
    let (_, blend_lockup_client) =
        create_blend_lockup_wasm(&e, &bombadil, &frodo, &contracts.emitter.address);

    assert_eq!(bombadil, blend_lockup_client.get_admin());
    assert_eq!(frodo, blend_lockup_client.get_owner());
    assert_eq!(contracts.emitter.address, blend_lockup_client.emitter());
    assert_eq!(contracts.backstop.address, blend_lockup_client.backstop());
    assert_eq!(
        contracts.backstop_token.address,
        blend_lockup_client.backstop_token()
    );

    let result = blend_lockup_client.try_initialize(&bombadil, &frodo, &contracts.emitter.address);
    assert_eq!(
        result.err(),
        Some(Ok(soroban_sdk::Error::from_contract_error(3)))
    );
}
