use frame_system as system;
use frame_support::assert_ok;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use move_core_types::language_storage::StructTag;
use move_vm::data::*;
use move_vm_runtime::data_cache::RemoteCache;
use serde::Deserialize;
use sp_mvm::storage::MoveVmStorage;

mod mock;
use mock::*;

mod utils;
use utils::*;

fn store_module_bc() -> Vec<u8> {
    include_bytes!("assets/target/modules/1_Store.mv").to_vec()
}

fn script_tx() -> Vec<u8> {
    include_bytes!("assets/target/transactions/store_u64.mvt").to_vec()
}

#[derive(Deserialize)]
struct StoreU64 {
    pub val: u64,
}

fn call_publish_module(signer: <Test as system::Trait>::AccountId, bc: Vec<u8>, mod_name: &str) {
    let origin = Origin::signed(signer);
    // execute VM for publish module:
    let result = Mvm::publish(origin, bc.clone());
    eprintln!("publish_module result: {:?}", result);
    assert_ok!(result);

    // check storage:
    let module_id = ModuleId::new(to_move_addr(signer), Identifier::new(mod_name).unwrap());
    let storage = Mvm::move_vm_storage();
    let oracle = MockOracle(None);
    let state = State::new(storage, oracle);
    assert_eq!(bc, state.get_module(&module_id).unwrap().unwrap());
}

fn call_execute_script(origin: Origin) {
    let txbc = script_tx();

    // execute VM tx:
    let result = Mvm::execute(origin, txbc);
    eprintln!("execute_script result: {:?}", result);
    assert_ok!(result);

    // check storage:
    let store = Mvm::move_vm_storage();
    let oracle = MockOracle(None);
    let state = State::new(store, oracle);
    let tag = StructTag {
        address: origin_move_addr(),
        module: Identifier::new("Store").unwrap(),
        name: Identifier::new("U64").unwrap(),
        type_params: vec![],
    };
    let blob = state
        .get_resource(&origin_move_addr(), &tag)
        .unwrap()
        .unwrap();
    let store: StoreU64 = lcs::from_bytes(&blob).unwrap();
    assert_eq!(42, store.val);
}

#[test]
fn publish_module() {
    new_test_ext().execute_with(|| {
        let origin = origin_ps_acc();

        call_publish_module(origin, store_module_bc(), "Store");
    });
}

#[test]
fn execute_script() {
    new_test_ext().execute_with(|| {
        let origin = origin_ps_acc();
        let signer = Origin::signed(origin);

        call_publish_module(origin, store_module_bc(), "Store");
        call_execute_script(signer);
    });
}
