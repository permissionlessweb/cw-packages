use cosmwasm_std::{entry_point, DepsMut, Empty, Env, MessageInfo, Never, Response};

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, Never> {
    deps.storage.set(&msg.key, &msg.value);
    Ok(Response::new())
}

#[entry_point]
pub fn migrate(deps: DepsMut, env: Env, migrate_msg: InstantiateMsg) -> Result<Response, Never> {
    let contract_info = deps
        .querier
        .query_wasm_contract_info(&env.contract.address)
        .unwrap();
    // Safe assumption that sender is the admin, as only admin can call migrate on contract
    let sender = contract_info.admin.unwrap();
    let message_info = MessageInfo {
        sender,
        funds: vec![],
    };
    instantiate(deps, env, message_info, migrate_msg)
}

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use super::*;

    use cw_orch::{interface, prelude::*};

    #[interface(InstantiateMsg, Empty, Empty, InstantiateMsg)]
    pub struct MigratedBlob;

    impl<T: CwEnv> Uploadable for MigratedBlob<T> {
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path("cw_test_migrate")
                .unwrap()
        }

        fn wrapper() -> Box<dyn MockContract<Empty, Empty>> {
            Box::new(
                ContractWrapper::new_with_empty(
                    |_, _, _, _: Empty| -> Result<Response, Never> { unreachable!() },
                    super::instantiate,
                    |_, _, _: Empty| -> Result<cosmwasm_std::Binary, Never> { unreachable!() },
                )
                .with_migrate(super::migrate),
            )
        }
    }

    impl<Chain: CwEnv> cw_blob::interface::DeterministicInstantiation<Chain> for MigratedBlob<Chain> {}
}
