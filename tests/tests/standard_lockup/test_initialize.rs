use soroban_sdk::{testutils::Address as _, Address, Env, Error};
use tests::{env::EnvTestUtils, token_lockup::create_standard_lockup};
#[test]
fn test_initalize() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (_, standard_lockup_client) = create_standard_lockup(&e, &admin, &owner);
    assert_eq!(standard_lockup_client.get_admin(), admin);
    assert_eq!(standard_lockup_client.get_owner(), owner);

    let result = standard_lockup_client.try_initialize(&admin, &owner);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(3))));
}
