mod common;

use cw_orch::mock::MockBech32;

#[test]
fn mock() {
    let chain = MockBech32::new("cosm");
    common::test(chain)
}

mod cw_multi_test {
    use cosmwasm_std::{Empty, Never, Response};
    use cw_multi_test::ContractWrapper;
    use cw_multi_test::{App, Executor};

    pub fn blob() -> Box<dyn cw_multi_test::Contract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                |_, _, _, _: Empty| -> Result<Response, Never> { unreachable!() },
                cw_blob::instantiate,
                |_, _, _: Empty| -> Result<cosmwasm_std::Binary, Never> { unreachable!() },
            )
            .with_migrate(|_, _, _: Empty| -> Result<Response, Never> { panic!("why I'm here") }),
        )
    }

    pub fn migrated_blob() -> Box<dyn cw_multi_test::Contract<Empty>> {
        Box::new(
            ContractWrapper::new_with_empty(
                |_, _, _, _: Empty| -> Result<Response, Never> { unreachable!() },
                cw_test_migrated::instantiate,
                |_, _, _: Empty| -> Result<cosmwasm_std::Binary, Never> { unreachable!() },
            )
            .with_migrate(cw_test_migrated::migrate),
        )
    }

    #[test]
    fn migrate() {
        let mut app = App::default();
        let sender = app.api().addr_make("sender");
        let blob_code_id = app.store_code_with_creator(sender.clone(), blob());
        let migrate_to_code_id = app.store_code_with_creator(sender.clone(), migrated_blob());

        let addr = app
            .instantiate2_contract(
                blob_code_id,
                sender.clone(),
                &cosmwasm_std::Empty {},
                &[],
                "blob",
                Some(sender.to_string()),
                b"blob".to_vec(),
            )
            .unwrap();

        app.migrate_contract(
            sender,
            addr.clone(),
            &cw_test_migrated::InstantiateMsg {
                key: b"foo".to_vec(),
                value: b"bar".to_vec(),
            },
            migrate_to_code_id,
        )
        .unwrap();

        let res = app.wrap().query_wasm_raw(addr, b"foo").unwrap().unwrap();
        assert_eq!(res, b"bar");
    }
}
