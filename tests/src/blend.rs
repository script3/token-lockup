use sep_41_token::testutils::MockTokenClient;
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    vec, Address, BytesN, Env, Map, Symbol,
};

use crate::common::create_stellar_token;
use pool_factory::PoolInitMeta;
pub mod emitter {
    soroban_sdk::contractimport!(file = "../dependencies/emitter.wasm");
}
pub mod backstop {
    soroban_sdk::contractimport!(file = "../dependencies/backstop.wasm");
}
pub mod pool {
    soroban_sdk::contractimport!(file = "../dependencies/pool.wasm");
}
pub mod pool_factory {
    soroban_sdk::contractimport!(file = "../dependencies/pool_factory.wasm");
}
pub mod comet {
    soroban_sdk::contractimport!(file = "../dependencies/comet.wasm");
}

pub struct BlendContracts<'a> {
    pub backstop: backstop::Client<'a>,
    pub emitter: emitter::Client<'a>,
    pub backstop_token: comet::Client<'a>,
    pub pool_factory: pool_factory::Client<'a>,
    pub pool: pool::Client<'a>,
    pub usdc: MockTokenClient<'a>,
    pub blnd: MockTokenClient<'a>,
}

pub fn create_emitter_wasm<'a>(
    e: &Env,
    emitter: &Address,
    backstop: &Address,
    blnd: &Address,
    backstop_token: &Address,
) -> emitter::Client<'a> {
    let emitter_address = e.register_contract_wasm(Some(emitter), emitter::WASM);
    let emitter_client: emitter::Client<'a> = emitter::Client::new(&e, &emitter_address);
    emitter_client.initialize(blnd, &backstop, &backstop_token);

    emitter_client
}

pub fn create_backstop_wasm<'a>(
    e: &Env,
    backstop: &Address,
    backstop_token: &Address,
    emitter: &Address,
    usdc: &Address,
    blnd: &Address,
    pool_factory: &Address,
) -> backstop::Client<'a> {
    e.register_contract_wasm(backstop, backstop::WASM);
    let backstop_client: backstop::Client<'a> = backstop::Client::new(&e, &backstop);
    backstop_client.initialize(
        backstop_token,
        emitter,
        &usdc,
        &blnd,
        &pool_factory,
        &Map::new(&e),
    );
    backstop_client
}

pub fn create_comet_wasm<'a>(
    e: &Env,
    admin: &Address,
    usdc: &Address,
    blnd: &Address,
) -> comet::Client<'a> {
    let comet = e.register_contract_wasm(None, comet::WASM);
    let comet_client: comet::Client<'a> = comet::Client::new(&e, &comet);

    let usdc_client = MockTokenClient::new(&e, &usdc);
    let blnd_client = MockTokenClient::new(&e, &blnd);
    usdc_client.mint(&admin, &1_000_0000000);
    blnd_client.mint(&admin, &25_0000000);
    comet_client.init(&Address::generate(&e), &admin);
    comet_client.bundle_bind(
        &vec![&e, usdc.clone(), blnd.clone()],
        &vec![&e, 1_000_0000000, 25_0000000],
        &vec![&e, 8_0000000, 2_0000000],
    );
    comet_client.set_swap_fee(&30000_i128, &admin);
    comet_client.set_public_swap(&admin, &true);
    comet_client.finalize();

    comet_client
}

pub fn create_blend_contracts<'a>(e: &Env, admin: &Address) -> BlendContracts<'a> {
    let backstop = e.register_contract_wasm(None, backstop::WASM);
    let emitter = e.register_contract_wasm(None, emitter::WASM);
    let comet = e.register_contract_wasm(None, comet::WASM);
    let pool_factory = e.register_contract_wasm(None, pool_factory::WASM);

    let (usdc, usdc_client) = create_stellar_token(&e, &admin);
    let (blnd, blnd_client) = create_stellar_token(&e, &admin);
    blnd_client.mint(&admin, &1_000_0000000);
    usdc_client.mint(&admin, &25_0000000);

    let comet_client: comet::Client<'a> = comet::Client::new(&e, &comet);
    comet_client.init(&Address::generate(&e), &admin);

    comet_client.bundle_bind(
        &vec![&e, blnd.clone(), usdc.clone()],
        &vec![&e, 1_000_0000000, 25_0000000],
        &vec![&e, 8_0000000, 2_0000000],
    );
    comet_client.set_swap_fee(&30000_i128, &admin);
    comet_client.set_public_swap(&admin, &true);
    comet_client.finalize();

    blnd_client.set_admin(&emitter);
    let emitter_client: emitter::Client<'a> = emitter::Client::new(&e, &emitter);
    emitter_client.initialize(&blnd, &backstop, &comet);

    let backstop_client: backstop::Client<'a> = backstop::Client::new(&e, &backstop);
    backstop_client.initialize(&comet, &emitter, &usdc, &blnd, &pool_factory, &Map::new(&e));

    let pool_hash = e.deployer().upload_contract_wasm(pool::WASM);

    let pool_factory_client = pool_factory::Client::new(&e, &pool_factory);
    pool_factory_client.initialize(&PoolInitMeta {
        backstop,
        blnd_id: blnd.clone(),
        pool_hash,
        usdc_id: usdc.clone(),
    });

    let pool_address = pool_factory_client.deploy(
        &admin,
        &Symbol::new(&e, "test"),
        &BytesN::<32>::random(&e),
        &Address::generate(&e),
        &1000000,
        &6,
    );
    blnd_client.mint(&admin, &1_001_000_0000000);
    usdc_client.mint(&admin, &25000_0000000);
    comet_client.join_pool(
        &50_000_0000000,
        &vec![&e, 1_001_000_0000000, 25000_0000000],
        &admin,
    );

    backstop_client.deposit(&admin, &pool_address, &50_000_0000000);
    backstop_client.update_tkn_val();
    backstop_client.add_reward(&pool_address, &Address::generate(&e));
    BlendContracts {
        backstop: backstop_client,
        emitter: emitter_client,
        backstop_token: comet_client,
        pool_factory: pool_factory_client,
        pool: pool::Client::new(&e, &pool_address),
        usdc: usdc_client,
        blnd: blnd_client,
    }
}
