use crate::{
    errors::TokenLockupError,
    storage,
    token_lockup::TokenLockupTrait,
    unlocks::{find_unlock_with_sequence, remove_past_unlocks},
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, token::TokenClient, unwrap::UnwrapOptimized, Address,
    Env, Map, Vec,
};

#[cfg(feature = "standard")]
use crate::token_lockup::Standard;

#[cfg(feature = "blend")]
use crate::dependencies::{BackstopClient, CometClient, EmitterClient};
#[cfg(feature = "blend")]
use crate::token_lockup::Blend;
#[cfg(feature = "blend")]
use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    vec, FromVal, IntoVal, Symbol, Val,
};

#[contract]
pub struct TokenLockup;

#[contractimpl]
impl TokenLockupTrait for TokenLockup {
    fn update_owner(e: Env, new_owner: Address) {
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::set_owner(&e, &new_owner);
    }

    fn get_owner(e: Env) -> Address {
        storage::get_owner(&e)
    }

    fn update_admin(e: Env, new_admin: Address) {
        let admin = storage::get_admin(&e);
        admin.require_auth();
        storage::set_admin(&e, &new_admin);
    }

    fn get_admin(e: Env) -> Address {
        storage::get_admin(&e)
    }

    fn add_unlock(e: Env, sequence: u32, percent: u64) {
        let admin = storage::get_admin(&e);
        admin.require_auth();
        if percent > 10000 {
            panic_with_error!(&e, TokenLockupError::InvalidPercentError);
        }

        let mut unlocks: Map<u32, u64> = storage::get_unlocks(&e);
        unlocks.set(sequence, percent);
        storage::set_unlocks(&e, &unlocks);
    }

    fn remove_unlock(e: Env, sequence: u32) {
        let admin = storage::get_admin(&e);
        admin.require_auth();
        let mut unlocks = storage::get_unlocks(&e);
        if unlocks.contains_key(sequence) {
            unlocks.remove(sequence);
        } else {
            panic_with_error!(&e, TokenLockupError::InvalidUnlockSequenceError);
        }
        storage::set_unlocks(&e, &unlocks);
    }

    fn get_unlock(e: Env, sequence: u32) -> Option<u64> {
        let unlocks = storage::get_unlocks(&e);
        if unlocks.contains_key(sequence) {
            Some(unlocks.get(sequence).unwrap_optimized())
        } else {
            None
        }
    }

    fn claim(e: Env, sequence: u32, asset_ids: Vec<Address>) {
        let owner = storage::get_owner(&e);
        owner.require_auth();

        let mut unlocks: Map<u32, u64> = storage::get_unlocks(&e);
        let index = find_unlock_with_sequence(sequence, &unlocks.keys());
        if sequence > e.ledger().sequence() || index.is_none() {
            panic_with_error!(&e, TokenLockupError::InvalidUnlockSequenceError);
        }

        let index = index.unwrap_optimized();
        let percentage_vec = unlocks.values().slice(0..index + 1);
        remove_past_unlocks(index, &mut unlocks);
        storage::set_unlocks(&e, &unlocks);

        for asset_id in asset_ids.iter() {
            let mut claim_amount = 0;
            let token_client = TokenClient::new(&e, &asset_id);
            let mut balance = token_client.balance(&e.current_contract_address());
            for percentage in percentage_vec.iter() {
                let transfer_amount = balance * percentage as i128 / 10000;
                balance -= transfer_amount;
                claim_amount += transfer_amount;
            }
            token_client.transfer(&e.current_contract_address(), &owner, &claim_amount);
        }
    }
}

#[cfg(feature = "standard")]
#[contractimpl]
impl Standard for TokenLockup {
    fn initialize(e: Env, admin: Address, owner: Address) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, TokenLockupError::AlreadyInitializedError);
        }
        storage::extend_instance(&e);
        storage::set_is_init(&e);
        storage::set_admin(&e, &admin);
        storage::set_owner(&e, &owner);
    }
}

#[cfg(all(feature = "blend", not(feature = "standard")))]
#[contractimpl]
impl Blend for TokenLockup {
    fn initialize(e: Env, admin: Address, owner: Address, emitter: Address) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, TokenLockupError::AlreadyInitializedError);
        }
        let backstop = EmitterClient::new(&e, &emitter).get_backstop();
        let backstop_token = BackstopClient::new(&e, &backstop).backstop_token();
        storage::extend_instance(&e);
        storage::set_is_init(&e);
        storage::set_admin(&e, &admin);
        storage::set_owner(&e, &owner);
        storage::set_emitter(&e, &emitter);
        storage::set_backstop(&e, &backstop);
        storage::set_backstop_token(&e, &backstop_token);
    }

    fn emitter(e: Env) -> Address {
        storage::get_emitter(&e)
    }

    fn backstop(e: Env) -> Address {
        storage::get_backstop(&e)
    }

    fn backstop_token(e: Env) -> Address {
        storage::get_backstop_token(&e)
    }

    fn update_backstop(e: Env) -> Address {
        let owner = storage::get_owner(&e);
        owner.require_auth();

        let emitter = storage::get_emitter(&e);
        let backstop = EmitterClient::new(&e, &emitter).get_backstop();
        storage::set_backstop(&e, &backstop);
        backstop
    }

    fn update_backstop_token(e: Env) -> Address {
        let owner = storage::get_owner(&e);
        owner.require_auth();

        let backstop = storage::get_backstop(&e);
        let backstop_token = BackstopClient::new(&e, &backstop).backstop_token();
        storage::set_backstop_token(&e, &backstop_token);
        backstop_token
    }

    fn execute_backstop_fn(e: Env, fn_name: Symbol, args: Vec<Val>) {
        let owner = storage::get_owner(&e);
        owner.require_auth();
        let backstop_address = storage::get_backstop(&e);

        if fn_name == Symbol::new(&e, "deposit") {
            e.authorize_as_current_contract(vec![
                &e,
                InvokerContractAuthEntry::Contract(SubContractInvocation {
                    context: ContractContext {
                        contract: storage::get_backstop_token(&e),
                        fn_name: Symbol::new(&e, "transfer"),
                        args: vec![
                            &e,
                            e.current_contract_address().into_val(&e),
                            storage::get_backstop(&e).into_val(&e),
                            args.get_unchecked(2),
                        ],
                    },
                    sub_invocations: vec![&e],
                }),
            ]);
        } else if fn_name == Symbol::new(&e, "claim") {
            let to = Address::from_val(&e, &args.get_unchecked(2));
            if to != e.current_contract_address() {
                panic_with_error!(&e, TokenLockupError::InvalidClaimToError);
            }
        }
        e.invoke_contract::<Val>(&backstop_address, &fn_name, args);
    }

    fn mint_backstop_token(e: Env, mint_amount: i128, deposit_amounts: Vec<i128>) {
        let owner = storage::get_owner(&e);
        owner.require_auth();

        let comet = CometClient::new(&e, &&storage::get_backstop_token(&e));
        let comet_tokens = comet.get_tokens();
        let mut auths = vec![&e];
        for index in 0..comet_tokens.len() {
            let amount = deposit_amounts.get(index).unwrap_optimized();
            let token_address = comet_tokens.get(index).unwrap_optimized();
            let approval_ledger = (e.ledger().sequence() / 100000 + 1) * 100000;
            auths.push_back(InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token_address,
                    fn_name: Symbol::new(&e, "approve"),
                    args: vec![
                        &e,
                        e.current_contract_address().into_val(&e),
                        storage::get_backstop_token(&e).into_val(&e),
                        amount.into_val(&e),
                        approval_ledger.into_val(&e),
                    ],
                },
                sub_invocations: vec![&e],
            }));
        }
        e.authorize_as_current_contract(auths);
        comet.join_pool(
            &mint_amount,
            &deposit_amounts,
            &e.current_contract_address(),
        );
    }

    fn withdraw_backstop_token(e: Env, burn_amount: i128, withdraw_amounts: Vec<i128>) {
        let owner = storage::get_owner(&e);
        owner.require_auth();

        let comet = CometClient::new(&e, &&storage::get_backstop_token(&e));
        comet.exit_pool(
            &burn_amount,
            &withdraw_amounts,
            &e.current_contract_address(),
        )
    }
}
