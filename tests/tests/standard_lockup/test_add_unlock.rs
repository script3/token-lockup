use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, Symbol,
};
use tests::{env::EnvTestUtils, token_lockup::create_standard_lockup};
#[test]
fn test_add_unlock() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (standard_lockup_address, standard_lockup_client) =
        create_standard_lockup(&e, &admin, &owner);

    let sequence = e.ledger().sequence() + 100;
    let percent_unlocked = 500;
    standard_lockup_client.add_unlock(&sequence, &percent_unlocked);
    assert_eq!(
        e.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    standard_lockup_address,
                    Symbol::new(&e, "add_unlock"),
                    vec![&e, sequence.into_val(&e), percent_unlocked.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(
        standard_lockup_client.get_unlock(&sequence),
        Some(percent_unlocked)
    );

    let changed_percent_unlocked = 1000;
    standard_lockup_client.add_unlock(&sequence, &changed_percent_unlocked);
    assert_eq!(
        standard_lockup_client.get_unlock(&sequence),
        Some(changed_percent_unlocked)
    );

    let percent_unlocked = 10001;
    let result = standard_lockup_client.try_add_unlock(&sequence, &percent_unlocked);
    assert_eq!(
        result.err(),
        Some(Ok(soroban_sdk::Error::from_contract_error(100)))
    );
}
