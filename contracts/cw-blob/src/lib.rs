use cosmwasm_std::{entry_point, DepsMut, Empty, Env, MessageInfo, Never, Response};

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> Result<Response, Never> {
    Ok(Response::new())
}

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use super::*;

    use cw_orch::{interface, prelude::*};

    #[interface(Empty, Empty, Empty, Empty)]
    pub struct CwBlob;

    impl<T: CwEnv> Uploadable for CwBlob<T> {
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path("cw_blob")
                .unwrap()
        }

        fn wrapper() -> Box<dyn MockContract<Empty, Empty>> {
            Box::new(
                ContractWrapper::new_with_empty(
                    |_, _, _, _: Empty| -> Result<Response, Never> { unreachable!() },
                    super::instantiate,
                    |_, _, _: Empty| -> Result<cosmwasm_std::Binary, Never> { unreachable!() },
                )
                .with_migrate(|_, _, _: Empty| -> Result<Response, Never> {
                    panic!("why I'm here")
                }),
            )
        }
    }
}
