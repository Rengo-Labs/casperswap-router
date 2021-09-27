use casper_contract::{ contract_api::{runtime}};
use casper_engine_test_support::AccountHash;
use casper_types::{U256, Key, runtime_args, RuntimeArgs, contracts::{ContractHash}, ContractPackageHash};
use test_env::{Sender, TestEnv, TestContract};

use crate::uniswap_instance::UniswapInstance;


use more_asserts;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

const NAME: &str = "uniswap_router";

fn deploy_dummy_tokens(env: &TestEnv, owner: Option<AccountHash>) -> (TestContract, TestContract, TestContract) 
{
    let decimals: u8 = 18;
    let init_total_supply: U256 = 1000.into();

    let token1_owner = if owner.is_none() { env.next_user() } else { owner.unwrap()};
    let token1_contract = TestContract::new(
        &env,
        "token.wasm",
        "token1_contract",
        Sender(token1_owner),
        runtime_args! {
            "initial_supply" => init_total_supply,
            "name" => "token1",
            "symbol" => "tk1",
            "decimals" => decimals
        }
    );

    let token2_owner = if owner.is_none() { env.next_user() } else { owner.unwrap()};
    let token2_contract = TestContract::new(
        &env,
        "token.wasm",
        "token2_contract",
        Sender(token2_owner),
        runtime_args! {
            "initial_supply" => init_total_supply,
            "name" => "token2",
            "symbol" => "tk2",
            "decimals" => decimals
        }
    );

    let token3_owner = if owner.is_none() { env.next_user() } else { owner.unwrap()};
    let token3_contract = TestContract::new(
        &env,
        "token.wasm",
        "token3_contract",
        Sender(token3_owner),
        runtime_args! {
            "initial_supply" => init_total_supply,
            "name" => "token3",
            "symbol" => "tk3",
            "decimals" => decimals
        }
    );
    (token1_contract, token2_contract, token3_contract)
}

fn deploy_uniswap_router() -> (TestEnv, UniswapInstance, AccountHash, Key, TestContract, TestContract, TestContract, TestContract, TestContract) 
{
    let env = TestEnv::new();
    let owner = env.next_user();

    // deploy factory contract
    let owner_factory = env.next_user();
    let factory_contract = TestContract::new(
        &env,
        "factory.wasm",
        "factory",
        Sender(owner_factory),
        runtime_args! {
            "fee_to_setter" => Key::from(owner_factory)
            // contract_name is passed seperately, so we don't need to pass it here.
        }
    );
    
    // deploy wcspr contract
    let env_wcspr = TestEnv::new();
    let owner_wcspr = env_wcspr.next_user();
    let wcspr = TestContract::new(
        &env,
        "wcspr.wasm",
        "wcspr",
        Sender(owner_wcspr),
        runtime_args! {}
    );

    // deploy library contract
    let env_library = TestEnv::new();
    let owner_library = env_library.next_user();
    let library_contract = TestContract::new(
        &env,
        "library.wasm",
        "library",
        Sender(owner_library),
        runtime_args! {}
    );
    
    // deploy pair contract
    let pair_contract = TestContract::new(
        &env,
        "pair.wasm",
        "pair",
        Sender(owner),
        runtime_args! {
            "callee_contract_hash" => Key::from(owner),
            "factory_hash" => Key::Hash(factory_contract.contract_hash()),
        }
    );
    
    let (token1, token2, token3) = deploy_dummy_tokens(&env, Some(owner));             // deploy dummy tokens for pair initialize

    let args: RuntimeArgs = runtime_args!{
        "token_a" => Key::Hash(token1.contract_hash()),
        "token_b" => Key::Hash(token2.contract_hash()),
        "pair_hash" => Key::Hash(pair_contract.contract_hash())
    };
    factory_contract.call_contract(Sender(owner), "create_pair", args);                 // call factory's create_pair to set the pair using token1 and token2
 /*   
    // mint tokens in pair
    let args: RuntimeArgs = runtime_args!{
        "to" => Key::Hash(token1.contract_hash())
    };
    pair_contract.call_contract(Sender(owner), "mint", args);
    
    let args: RuntimeArgs = runtime_args!{
        "to" => Key::Hash(token2.contract_hash())
    };
    pair_contract.call_contract(Sender(owner), "mint", args);
*/

    let router_contract = TestContract::new(
        &env,
        "uniswap-v2-router.wasm",
        NAME,
        Sender(owner),
        runtime_args! {
            "factory" => Key::Hash(factory_contract.contract_hash()),
            "wcspr" => Key::Hash(wcspr.contract_hash()),
            "library" => Key::Hash(library_contract.contract_hash()),
            "pair" => Key::Hash(pair_contract.contract_hash()),
        },
    );
    let router_package_hash: ContractPackageHash = router_contract.query_named_key(String::from("package_hash"));
    let router_package_hash:Key = router_package_hash.into();

    let token = UniswapInstance::new(
        &env,
        Key::Hash(router_contract.contract_hash()),
        Sender(owner)
    );
    
    (env, token, owner, router_package_hash, pair_contract, token1, token2, token3, wcspr)
}


#[test]
fn test_uniswap_deploy()
{
    let (env, token, owner, _, _, _, _, _, _) = deploy_uniswap_router();
    let self_hash: Key = token.uniswap_contract_address();
    let package_hash: Key = token.uniswap_contract_package_hash();
    let uniswap_router_address: Key = token.uniswap_router_address();

    let zero_addr:Key = Key::from_formatted_str("hash-0000000000000000000000000000000000000000000000000000000000000000").unwrap();
    assert_ne!(self_hash, zero_addr);
    assert_ne!(package_hash, zero_addr);
    assert_ne!(uniswap_router_address, zero_addr);
}


#[test]
fn add_liquidity()                                              // Working
{
    let (env, uniswap, owner, router_package_hash, _, token1, token2, token3, _) = deploy_uniswap_router();

    let token_a = Key::Hash(token1.contract_hash());
    let token_b = Key::Hash(token2.contract_hash());
    let to = Key::Hash(token3.contract_hash());
    
    let mut rng = rand::thread_rng();
    let amount_a_desired: U256 = rng.gen_range(300..600).into();
    let amount_b_desired: U256 = rng.gen_range(300..600).into();
    let amount_a_min: U256 = rng.gen_range(1..250).into();
    let amount_b_min: U256 = rng.gen_range(1..250).into();
    
    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    // approve the router to spend tokens
    uniswap.approve(&token1, Sender(owner), router_package_hash, amount_a_desired);
    uniswap.approve(&token2, Sender(owner), router_package_hash, amount_b_desired);

    uniswap.add_liquidity(Sender(owner), token_a, token_b, amount_a_desired, amount_b_desired, amount_a_min, amount_b_min, to, deadline.into());
    let (amount_a, amount_b, _): (U256, U256, U256) = uniswap.add_liquidity_result();

    more_asserts::assert_ge!(amount_a, amount_a_min);
    more_asserts::assert_ge!(amount_b, amount_b_min);
}

#[test]
fn add_liquidity_cspr()                                     // Working
{
    let (env, uniswap, owner, router_package_hash, _, token1, token2, _, _) = deploy_uniswap_router();

    let to = Key::Hash(token2.contract_hash());

    let mut rng = rand::thread_rng();
    let token = Key::Hash(token1.contract_hash());
    let amount_token_desired: U256 = rng.gen_range(300..600).into();
    let amount_cspr_desired: U256 = rng.gen_range(300..600).into();
    let amount_token_min: U256 = rng.gen_range(1..250).into();
    let amount_cspr_min: U256 = rng.gen_range(1..250).into();

    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.approve(&token1, Sender(owner), router_package_hash, amount_token_desired);
    uniswap.add_liquidity_cspr(Sender(owner), token, amount_token_desired, amount_cspr_desired, amount_token_min, amount_cspr_min, to, deadline.into());

    let (amount_token, amount_cspr, _): (U256, U256, U256) = uniswap.add_liquidity_cspr_result();
    more_asserts::assert_ge!(amount_token, amount_token_min);
    more_asserts::assert_ge!(amount_cspr, amount_cspr_min);
}


#[test]
fn remove_liquidity()                                           // Working
{
    let (env, uniswap, owner, router_package_hash, pair_contract, token1, token2, token3, _) = deploy_uniswap_router();
    let mut rng = rand::thread_rng();    

    // NO need to create pair, because pair of token1 and token2 already created in deploy_uniswap_router() method above.
    // The remove_liquidity() call below should be able to find that pair.

    let token_a = Key::Hash(token1.contract_hash());
    let token_b = Key::Hash(token2.contract_hash());
    let liquidity:U256 = rng.gen_range(300..500).into();
    let amount_a_min:U256 = rng.gen_range(1..250).into();
    let amount_b_min:U256 = rng.gen_range(1..250).into();
    let to = Key::Hash(token3.contract_hash());

    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    // approve router on pair
    let args: RuntimeArgs = runtime_args!{
        "spender" => router_package_hash,
        "amount" => liquidity
    };
    pair_contract.call_contract(Sender(owner), "approve", args);
    
    uniswap.remove_liquidity(Sender(owner), token_a, token_b, liquidity, amount_a_min, amount_b_min, to, deadline.into());
    
    let (amount_a, amount_b):(U256, U256) = uniswap.remove_liquidity_result();
    more_asserts::assert_ge!(amount_a, amount_a_min);
    more_asserts::assert_ge!(amount_b, amount_b_min);
}

#[test]
fn remove_liquidity_cspr()
{
    let (env, uniswap, owner, router_package_hash, pair_contract, token1, token2, _, _) = deploy_uniswap_router();
    let mut rng = rand::thread_rng();

    // Here we do need to first create the pair, because pair for token1 and wcspr isn't created anywhere.
    // First Add liquidity
    let token = Key::Hash(token1.contract_hash());
    let amount_token_desired: U256 = rng.gen_range(300..600).into();
    let amount_cspr_desired: U256 = rng.gen_range(300..600).into();
    let amount_token_min: U256 = rng.gen_range(100..250).into();
    let amount_cspr_min: U256 = rng.gen_range(100..250).into();
    let to = Key::Hash(token2.contract_hash());

    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.approve(&token1, Sender(owner), router_package_hash, amount_token_desired);
    uniswap.add_liquidity_cspr(Sender(owner), token, amount_token_desired, amount_cspr_desired, amount_token_min, amount_cspr_min, to, deadline.into());


    // Remove liquidity
    let token: Key = Key::Hash(token1.contract_hash());
    let liquidity:U256 = rng.gen_range(50..100).into();
    let amount_token_min: U256 = rng.gen_range(0..50).into();
    let amount_cspr_min: U256 = rng.gen_range(0..50).into();
    let to = Key::Hash(token2.contract_hash());

    // approve router on pair
    let args: RuntimeArgs = runtime_args!{
        "spender" => router_package_hash,
        "amount" => liquidity
    };
    pair_contract.call_contract(Sender(owner), "approve", args);

    uniswap.remove_liquidity_cspr(Sender(owner), token, liquidity, amount_token_min, amount_cspr_min, to, deadline.into());
    
    let (amount_token, amount_cspr) : (U256, U256) = uniswap.remove_liquidity_cspr_result();
    more_asserts::assert_ge!(amount_token, amount_token_min);
    more_asserts::assert_ge!(amount_cspr, amount_cspr_min);
}


#[test]
pub fn remove_liquidity_with_permit()
{
    let (env, uniswap, owner, router_package_hash, pair_contract, token1, token2, token3, _) = deploy_uniswap_router();
    let mut rng = rand::thread_rng();

    // NO need to create pair, because pair of token1 and token2 already created in deploy_uniswap_router() method above.
    // The remove_liquidity() call below should be able to find that pair.

    let token_a = Key::Hash(token1.contract_hash());
    let token_b = Key::Hash(token2.contract_hash());
    let liquidity:U256 = rng.gen_range(50..100).into();
    let amount_a_min: U256 = rng.gen_range(0..50).into();
    let amount_b_min: U256 = rng.gen_range(0..50).into();
    let to = Key::Hash(token3.contract_hash());
    let approve_max = false;
    
    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    let blocktime: U256 = deadline.into();

    let data:String = format!("{}{}{}{}", Key::from(owner), router_package_hash, liquidity, blocktime);
    let (signature, public_key) : (String, String) = uniswap.calculate_signature(&data);
    println!("Returned Signature: {}", signature);
    println!("Returned Public-Key: {}", public_key);

    uniswap.remove_liquidity_with_permit(Sender(owner), token_a, token_b, liquidity, amount_a_min, amount_b_min, to, deadline.into(), approve_max, public_key, signature);

    let (amount_a, amount_b):(U256, U256) = uniswap.remove_liquidity_with_permit_result();
    more_asserts::assert_ge!(amount_a, amount_a_min);
    more_asserts::assert_ge!(amount_b, amount_b_min);
}

#[test]
fn remove_liquidity_cspr_with_permit()
{
    let (env, uniswap, owner, router_package_hash, _, token1, token2, _, _) = deploy_uniswap_router();
    let mut rng = rand::thread_rng();

    // Here we do need to first create the pair, because pair for token1 and wcspr isn't created anywhere.
    // First Add liquidity
    let token = Key::Hash(token1.contract_hash());
    let amount_token_desired: U256 = rng.gen_range(300..600).into();
    let amount_cspr_desired: U256 = rng.gen_range(300..600).into();
    let amount_token_min: U256 = rng.gen_range(1..250).into();
    let amount_cspr_min: U256 = rng.gen_range(1..250).into();
    let to = Key::Hash(token2.contract_hash());

    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.approve(&token1, Sender(owner), router_package_hash, amount_token_desired);
    uniswap.add_liquidity_cspr(Sender(owner), token, amount_token_desired, amount_cspr_desired, amount_token_min, amount_cspr_min, to, deadline.into());

    // Now remove liquidity
    let token = Key::Hash(token1.contract_hash());
    let liquidity:U256 = rng.gen_range(50..100).into();
    let amount_token_min: U256 = rng.gen_range(0..50).into();
    let amount_cspr_min: U256 = rng.gen_range(0..50).into();
    let to = Key::Hash(token2.contract_hash());
    let approve_max = false;

    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };
    let data:String = format!("{}{}{}{}", Key::from(owner), router_package_hash, liquidity, deadline);
    let (signature, public_key) : (String, String) = uniswap.calculate_signature(&data);
    println!("Returned Signature: {}", signature);
    println!("Returned Public-Key: {}", public_key);

    uniswap.remove_liquidity_cspr_with_permit(Sender(owner), token, liquidity, amount_token_min, amount_cspr_min, to, deadline.into(), approve_max, public_key, signature);

    let (amount_token, amount_cspr):(U256, U256) = uniswap.remove_liquidity_cspr_with_permit_result();
    more_asserts::assert_ge!(amount_token, amount_token_min);
    more_asserts::assert_ge!(amount_cspr, amount_cspr_min);
}

#[test]
fn swap_exact_tokens_for_tokens()
{
    let (env, uniswap, owner, router_package_hash, _, token1, token2, token3, _) = deploy_uniswap_router();
    
    let mut rng = rand::thread_rng();
    let amount_in: U256 = rng.gen_range(300..600).into();
    let amount_out_min: U256 = rng.gen_range(0..250).into();
    let path: Vec<Key> = vec![Key::Hash(token1.contract_hash()), Key::Hash(token2.contract_hash())];
    let to: Key = Key::Hash(token3.contract_hash());
    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.swap_exact_tokens_for_tokens(Sender(owner), amount_in, amount_out_min, path, to, deadline.into());
}

#[test]
fn swap_tokens_for_exact_tokens()
{
    let (env, uniswap, owner, router_package_hash, _, token1, token2, token3, _) = deploy_uniswap_router();
    
    let mut rng = rand::thread_rng();
    let amount_in_max: U256 = rng.gen_range(300..600).into();
    let amount_out: U256 = rng.gen_range(0..250).into();
    let path: Vec<Key> = vec![Key::Hash(token1.contract_hash()), Key::Hash(token2.contract_hash())];
    let to: Key = Key::Hash(token3.contract_hash());
    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.swap_tokens_for_exact_tokens(Sender(owner), amount_out, amount_in_max, path, to, deadline.into());
}

#[test]
fn swap_exact_cspr_for_tokens()
{
    let (env, uniswap, owner, router_package_hash, _, _, token2, token3, wcspr) = deploy_uniswap_router();
    
    let mut rng = rand::thread_rng();
    let amount_in: U256 = rng.gen_range(300..600).into();
    let amount_out_min: U256 = rng.gen_range(0..250).into();
    let path: Vec<Key> = vec![Key::Hash(wcspr.contract_hash()), Key::Hash(token2.contract_hash())];
    let to: Key = Key::Hash(token3.contract_hash());
    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.swap_exact_cspr_for_tokens(Sender(owner), amount_out_min, amount_in, path, to, deadline.into());
}

#[test]
fn swap_tokens_for_exact_cspr()
{
    let (env, uniswap, owner, router_package_hash, _, token1, _, token3, wcspr) = deploy_uniswap_router();
    
    let mut rng = rand::thread_rng();
    let amount_in_max: U256 = rng.gen_range(300..600).into();
    let amount_out: U256 = rng.gen_range(0..250).into();
    let path: Vec<Key> = vec![Key::Hash(token1.contract_hash()), Key::Hash(wcspr.contract_hash())];
    let to: Key = Key::Hash(token3.contract_hash());
    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.swap_tokens_for_exact_cspr(Sender(owner), amount_out, amount_in_max, path, to, deadline.into());
}

#[test]
fn swap_exact_tokens_for_cspr()
{
    let (env, uniswap, owner, router_package_hash, _, token1, _, token3, wcspr) = deploy_uniswap_router();
    
    let mut rng = rand::thread_rng();
    let amount_in: U256 = rng.gen_range(300..600).into();
    let amount_out_min: U256 = rng.gen_range(0..250).into();
    let path: Vec<Key> = vec![Key::Hash(token1.contract_hash()), Key::Hash(wcspr.contract_hash())];
    let to: Key = Key::Hash(token3.contract_hash());
    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.swap_exact_tokens_for_cspr(Sender(owner), amount_in, amount_out_min, path, to, deadline.into());
}

#[test]
fn swap_cspr_for_exact_tokens()
{
    let (env, uniswap, owner, router_package_hash, _, _, token2, token3, wcspr) = deploy_uniswap_router();
    
    let mut rng = rand::thread_rng();
    let amount_in_max: U256 = rng.gen_range(300..600).into();
    let amount_out: U256 = rng.gen_range(0..250).into();
    let path: Vec<Key> = vec![Key::Hash(wcspr.contract_hash()), Key::Hash(token2.contract_hash())];
    let to: Key = Key::Hash(token3.contract_hash());
    let deadline: u128 = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => n.as_millis() + (1000 * (30 * 60)),      // current epoch time in milisecond + 30 minutes
        Err(_) => 0
    };

    uniswap.swap_cspr_for_exact_tokens(Sender(owner), amount_out, amount_in_max, path, to, deadline.into());
}