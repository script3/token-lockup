use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Symbol,
};
use tests::{env::EnvTestUtils, token_lockup::create_standard_lockup};
#[test]
fn test_set_owner() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (standard_lockup_address, standard_lockup_client) =
        create_standard_lockup(&e, &admin, &owner);
    assert_eq!(standard_lockup_client.get_owner(), owner);

    let new_owner = Address::generate(&e);
    standard_lockup_client.update_owner(&new_owner);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    standard_lockup_address,
                    Symbol::new(&e, "update_owner"),
                    vec![&e, new_owner.to_val()]
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(standard_lockup_client.get_owner(), new_owner);
}
