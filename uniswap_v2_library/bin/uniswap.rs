#![no_main]
#![no_std]
#![feature(slice_range)]

extern crate alloc;
use alloc::{collections::BTreeSet, format, vec, prelude::v1::Box};
use casper_contract::{
    contract_api::{runtime, storage},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    runtime_args, CLType, CLTyped, CLValue, EntryPoint, EntryPointAccess,
    Group, Key, Parameter, RuntimeArgs, URef, U256, EntryPointType,
    ContractHash, EntryPoints, api_error::{ApiError}
};
use crate::vec::Vec;

use contract_utils::{ContractContext, OnChainContractStorage};
use uniswap_v2_library::{self, UniswapV2Library};
use uniswap_v2_library::config::*;

#[derive(Default)]
struct uniswap(OnChainContractStorage);

impl ContractContext<OnChainContractStorage> for uniswap {
    fn storage(&self) -> &OnChainContractStorage {
        &self.0
    }
}

impl uniswap_v2_library<OnChainContractStorage> for uniswap {}

impl uniswap {
    fn constructor(&mut self, contract_hash:ContractHash) {
        uniswap_v2_library::init(self, contract_hash);
    }
}

#[no_mangle]
fn constructor() {
    let contract_hash: ContractHash = runtime::get_named_arg("contract_hash");
    uniswap::default().constructor(contract_hash);
}

#[no_mangle]
fn sort_tokens() {

    let token_a:ContractHash = runtime::get_named_arg("token_a");
    let token_b:ContractHash = runtime::get_named_arg("token_b");
    
    let (token_0, token_1) = uniswap::default().sort_tokens(token_a, token_b);
    runtime::ret(CLValue::from_t((token_0, token_1)).unwrap_or_revert())
}

#[no_mangle]
// calculates the CREATE2 address for a pair without making any external calls
fn pair_for() {
    
    let factory:ContractHash = runtime::get_named_arg("factory");
    let token_a:ContractHash = runtime::get_named_arg("token_a");
    let token_b:ContractHash = runtime::get_named_arg("token_b");
    
    // let pair = address(uint(keccak256(abi.encodePacked( hex'ff', factory, keccak256(abi.encodePacked(token0, token1)), hex'96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f' ))));
    // let hex = 
    // let pair:ContractHash = 
}

#[no_mangle]
fn get_reserves() {
    
    let factory:ContractHash = runtime::get_named_arg("factory");
    let token_a:ContractHash = runtime::get_named_arg("token_a");
    let token_b:ContractHash = runtime::get_named_arg("token_b");
    
    let (reserve_a, reserve_b) = uniswap::default().get_reserves(factory, token_a, token_b);
    runtime::ret(CLValue::from_t((reserve_a, reserve_b)).unwrap_or_revert())
}

#[no_mangle]
// given some amount of an asset and pair reserves, returns an equivalent amount of the other asset
fn quote() {
    
    let amount_a: U256 = runtime::get_named_arg("amount_a");
    let reserve_a: U256 = runtime::get_named_arg("reserve_a");
    let reserve_b: U256 = runtime::get_named_arg("reserve_b");
    
    let amount_b = uniswap::default().quote(amount_a, reserve_a, reserve_b);
    runtime::ret(CLValue::from_t(amount_b).unwrap_or_revert())
}

#[no_mangle]
// given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset
fn get_amount_out(){
    
    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let reserve_in: U256 = runtime::get_named_arg("reserve_in");
    let reserve_out: U256 = runtime::get_named_arg("reserve_out");
    
    let amount_out = uniswap::default().get_amount_out(amount_in, reserve_in, reserve_out);
    runtime::ret(CLValue::from_t(amount_out).unwrap_or_revert())
}

#[no_mangle]
// given an output amount of an asset and pair reserves, returns a required input amount of the other asset
fn get_amount_in() {
    
    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let reserve_in: U256 = runtime::get_named_arg("reserve_in");
    let reserve_out: U256 = runtime::get_named_arg("reserve_out");

    let amount_in = uniswap::default().get_amount_in(amount_out, reserve_in, reserve_out);
    runtime::ret(CLValue::from_t(amount_in).unwrap_or_revert())
}

#[no_mangle]
// performs chained getAmountOut calculations on any number of pairs
fn get_amounts_out(){

    let factory: ContractHash = runtime::get_named_arg("factory");
    let amount_in: U256 = runtime::get_named_arg("amount_in");
    let path: Vec<ContractHash> = runtime::get_named_arg("path");

    let amounts:Vec<U256> = uniswap::default().get_amounts_out(factory, amount_in, path);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert())
}

#[no_mangle]
// performs chained getAmountIn calculations on any number of pairs
fn get_amounts_in(){

    let factory: ContractHash = runtime::get_named_arg("factory");
    let amount_out: U256 = runtime::get_named_arg("amount_out");
    let path: Vec<ContractHash> = runtime::get_named_arg("path");

    let amounts:Vec<U256> = uniswap::default().get_amounts_in(factory, amount_out, path);
    runtime::ret(CLValue::from_t(amounts).unwrap_or_revert())
}



fn get_entry_points() -> EntryPoints {

    let mut entry_points = EntryPoints::new();
    
    entry_points.add_entry_point(EntryPoint::new(
        "constructor",
        vec![
            Parameter::new("contract_hash", ContractHash::cl_type()),
        ],
        <()>::cl_type(),
        EntryPointAccess::Groups(vec![Group::new("constructor")]),
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "quote",
        vec![
            Parameter::new("amount_a", Key::cl_type()),
            Parameter::new("reserve_a", Key::cl_type()),
            Parameter::new("reserve_b", Key::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_amount_out",
        vec![
            Parameter::new("amount_in", Key::cl_type()),
            Parameter::new("reserve_in", Key::cl_type()),
            Parameter::new("reserve_out", Key::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_amount_in",
        vec![
            Parameter::new("amount_out", Key::cl_type()),
            Parameter::new("reserve_in", Key::cl_type()),
            Parameter::new("reserve_out", Key::cl_type()),
        ],
        U256::cl_type(),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_amounts_out",
        vec![
            Parameter::new("factory", Key::cl_type()),
            Parameter::new("amount_in", Key::cl_type()),
            Parameter::new("path", Key::cl_type()),
        ],
        CLType::List(Box::new(CLType::U256)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));
    entry_points.add_entry_point(EntryPoint::new(
        "get_amounts_in",
        vec![
            Parameter::new("factory", Key::cl_type()),
            Parameter::new("amount_out", Key::cl_type()),
            Parameter::new("path", Key::cl_type()),
        ],
        CLType::List(Box::new(CLType::U256)),
        EntryPointAccess::Public,
        EntryPointType::Contract,
    ));

    entry_points
}

#[no_mangle]
fn call() {
    // Build new package with initial a first version of the contract.
    let (package_hash, access_token) = storage::create_contract_package_at_hash();
    let (contract_hash, _) =
        storage::add_contract_version(package_hash, get_entry_points(), Default::default());

    // Prepare constructor args
    let constructor_args = runtime_args! {
        "contract_hash" => contract_hash          // USING THIS FOR INTERNAL FUNCTION CALLS...
    };

    // Add the constructor group to the package hash with a single URef.
    let constructor_access: URef =
        storage::create_contract_user_group(package_hash, "constructor", 1, Default::default())
            .unwrap_or_revert()
            .pop()
            .unwrap_or_revert();

    // Call the constructor entry point
    let _: () =
        runtime::call_versioned_contract(package_hash, None, "constructor", constructor_args);

    // Remove all URefs from the constructor group, so no one can call it for the second time.
    let mut urefs = BTreeSet::new();
    urefs.insert(constructor_access);
    storage::remove_contract_user_group_urefs(package_hash, "constructor", urefs)
        .unwrap_or_revert();

    // Store contract in the account's named keys.
    let contract_name: alloc::string::String = runtime::get_named_arg("contract_name");
    runtime::put_key(
        &format!("{}_package_hash", contract_name),
        package_hash.into(),
    );
    runtime::put_key(
        &format!("{}_package_hash_wrapped", contract_name),
        storage::new_uref(package_hash).into(),
    );
    runtime::put_key(
        &format!("{}_contract_hash", contract_name),
        contract_hash.into(),
    );
    runtime::put_key(
        &format!("{}_contract_hash_wrapped", contract_name),
        storage::new_uref(contract_hash).into(),
    );
    runtime::put_key(
        &format!("{}_package_access_token", contract_name),
        access_token.into(),
    );
}