use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, IntoVal, Symbol,
};
use tests::{
    common::create_stellar_token, env::EnvTestUtils, token_lockup::create_standard_lockup,
};

#[test]
fn single_token_single_unlock() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (standard_lockup_address, standard_lockup_client) =
        create_standard_lockup(&e, &admin, &owner);
    let (token_address, token_client) = create_stellar_token(&e, &admin);

    let token_amount: i128 = 1_000_000 * 10i128.pow(7);
    token_client.mint(&standard_lockup_address, &token_amount);

    let sequence = e.ledger().sequence() + 100;
    let percent_unlocked = 500;
    standard_lockup_client.add_unlock(&sequence, &percent_unlocked);

    e.jump(100);
    let claim_amount = token_amount * percent_unlocked as i128 / 10000;
    standard_lockup_client.claim(&sequence, &vec![&e, token_address.clone()]);
    assert_eq!(
        e.auths(),
        std::vec![(
            owner.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    standard_lockup_address,
                    Symbol::new(&e, "claim"),
                    vec![
                        &e,
                        sequence.into_val(&e),
                        vec![&e, token_address.to_val()].into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token_client.balance(&owner), claim_amount);
    assert_eq!(standard_lockup_client.get_unlock(&sequence), None);
}

#[test]
fn single_token_multi_unlock() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (standard_lockup_address, standard_lockup_client) =
        create_standard_lockup(&e, &admin, &owner);
    let (token_address, token_client) = create_stellar_token(&e, &admin);

    let token_amount: i128 = 1_000_000 * 10i128.pow(7);
    token_client.mint(&standard_lockup_address, &token_amount);

    let first_sequence = e.ledger().sequence() + 100;
    let first_percent_unlocked = 500;
    standard_lockup_client.add_unlock(&first_sequence, &first_percent_unlocked);

    let second_sequence = e.ledger().sequence() + 200;
    let second_percent_unlocked = 500;
    standard_lockup_client.add_unlock(&second_sequence, &second_percent_unlocked);

    e.jump(second_sequence);
    let first_claim_amount = token_amount * first_percent_unlocked as i128 / 10000;
    let second_claim_amount =
        (token_amount - first_claim_amount) * second_percent_unlocked as i128 / 10000;
    standard_lockup_client.claim(&second_sequence, &vec![&e, token_address.clone()]);
    assert_eq!(
        e.auths(),
        std::vec![(
            owner.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    standard_lockup_address,
                    Symbol::new(&e, "claim"),
                    vec![
                        &e,
                        second_sequence.into_val(&e),
                        vec![&e, token_address.to_val()].into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(
        token_client.balance(&owner),
        first_claim_amount + second_claim_amount
    );
    assert_eq!(standard_lockup_client.get_unlock(&first_sequence), None);
    assert_eq!(standard_lockup_client.get_unlock(&second_sequence), None);
}

#[test]
fn multi_token_single_unlock() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (standard_lockup_address, standard_lockup_client) =
        create_standard_lockup(&e, &admin, &owner);
    let (x_token_address, x_token_client) = create_stellar_token(&e, &admin);
    let (y_token_address, y_token_client) = create_stellar_token(&e, &admin);

    let token_amount: i128 = 1_000_000 * 10i128.pow(7);
    x_token_client.mint(&standard_lockup_address, &token_amount);
    y_token_client.mint(&standard_lockup_address, &token_amount);

    let sequence = e.ledger().sequence() + 100;
    let percent_unlocked = 500;
    standard_lockup_client.add_unlock(&sequence, &percent_unlocked);

    e.jump(100);
    let claim_amount = token_amount * percent_unlocked as i128 / 10000;
    standard_lockup_client.claim(
        &sequence,
        &vec![&e, x_token_address.clone(), y_token_address.clone()],
    );
    assert_eq!(
        e.auths(),
        std::vec![(
            owner.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    standard_lockup_address,
                    Symbol::new(&e, "claim"),
                    vec![
                        &e,
                        sequence.into_val(&e),
                        vec![&e, x_token_address.to_val(), y_token_address.to_val()].into_val(&e)
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(x_token_client.balance(&owner), claim_amount);
    assert_eq!(y_token_client.balance(&owner), claim_amount);
    assert_eq!(standard_lockup_client.get_unlock(&sequence), None);
}

#[test]
fn test_claim_early() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (standard_lockup_address, standard_lockup_client) =
        create_standard_lockup(&e, &admin, &owner);
    let (x_token_address, x_token_client) = create_stellar_token(&e, &admin);

    let token_amount: i128 = 1_000_000 * 10i128.pow(7);
    x_token_client.mint(&standard_lockup_address, &token_amount);

    let sequence = e.ledger().sequence() + 100;
    let percent_unlocked = 500;
    standard_lockup_client.add_unlock(&sequence, &percent_unlocked);

    e.jump(99);

    let result = standard_lockup_client.try_claim(&sequence, &vec![&e, x_token_address.clone()]);
    assert_eq!(
        result.err(),
        Some(Ok(soroban_sdk::Error::from_contract_error(101)))
    );
}

#[test]
fn test_claim_invalid_sequence() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let owner = Address::generate(&e);
    let (standard_lockup_address, standard_lockup_client) =
        create_standard_lockup(&e, &admin, &owner);
    let (x_token_address, x_token_client) = create_stellar_token(&e, &admin);

    let token_amount: i128 = 1_000_000 * 10i128.pow(7);
    x_token_client.mint(&standard_lockup_address, &token_amount);

    let sequence = e.ledger().sequence() + 100;
    let percent_unlocked = 500;
    standard_lockup_client.add_unlock(&sequence, &percent_unlocked);

    e.jump(100);
    let result =
        standard_lockup_client.try_claim(&(sequence - 1), &vec![&e, x_token_address.clone()]);
    assert_eq!(
        result.err(),
        Some(Ok(soroban_sdk::Error::from_contract_error(101)))
    );
}
