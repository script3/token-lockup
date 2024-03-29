use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Symbol,
};
use tests::{env::EnvTestUtils, token_lockup::create_standard_lockup};
#[test]
fn test_set_admin() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (standard_lockup_address, standard_lockup_client) =
        create_standard_lockup(&e, &admin, &owner);
    assert_eq!(standard_lockup_client.get_admin(), admin);

    let new_admin = Address::generate(&e);
    standard_lockup_client.update_admin(&new_admin);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    standard_lockup_address,
                    Symbol::new(&e, "update_admin"),
                    vec![&e, new_admin.to_val()]
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(standard_lockup_client.get_admin(), new_admin);
}
