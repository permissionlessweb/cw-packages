use cosmwasm_std::{entry_point, DepsMut, Empty, Env, MessageInfo, Never, Response};

#[entry_point]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> Result<Response, Never> {
    Ok(Response::new())
}

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use super::*;

    use cosmwasm_std::{instantiate2_address, Binary, CanonicalAddr};
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
            Box::new(ContractWrapper::new_with_empty(
                |_, _, _, _: Empty| -> Result<Response, Never> { unreachable!() },
                super::instantiate,
                |_, _, _: Empty| -> Result<Binary, Never> { unreachable!() },
            ))
        }
    }

    impl<T: CwEnv> CwBlob<T> {
        /// Instantiate blob and migrate to your desired contract.
        /// It will upload your contract, if it's not uploaded already
        ///
        /// Checksum of the code id on chain should match [Uploadable::wasm()] of the [CwBlob]
        /// For cw-multi-test this can be achieved only with custom Checksum Generator
        pub fn upload_and_migrate<M>(
            chain: T,
            blob_code_id: u64,
            contract: &M,
            migrate_msg: &M::MigrateMsg,
            expected_addr: CanonicalAddr,
            salt: Binary,
        ) -> Result<(), CwOrchError>
        where
            M: ContractInstance<T> + CwOrchUpload<T> + MigratableContract,
        {
            let creator = chain.sender_addr();
            let blob_label = format!("{}_blob", contract.id());

            // Check stored checksum matches
            let on_chain_checksum = chain
                .wasm_querier()
                .code_id_hash(blob_code_id)
                .map_err(Into::into)?;
            let expected_checksum =
                <CwBlob<T> as Uploadable>::wasm(&ChainInfoOwned::default()).checksum()?;
            if on_chain_checksum != expected_checksum {
                return Err(CwOrchError::StdErr(format!(
                    "Expected blob checksum: {expected_checksum}, stored under given code_id: {on_chain_checksum}"
                )));
            }

            // Check incoming address of instantiated blob
            let account_id: cosmrs::AccountId = creator.as_str().parse().unwrap();
            let canon_creator = CanonicalAddr::from(account_id.to_bytes());
            let actual_addr = instantiate2_address(
                on_chain_checksum.as_slice(),
                &canon_creator,
                salt.as_slice(),
            )?;
            if actual_addr != expected_addr {
                return Err(CwOrchError::StdErr(
                    "Predicted blob address doesn't match to the expected".to_owned(),
                ));
            }

            let response = chain
                .instantiate2(
                    blob_code_id,
                    &cosmwasm_std::Empty {},
                    Some(&blob_label),
                    Some(&creator),
                    &[],
                    salt,
                )
                .map_err(Into::into)?;
            let blob_address = response.instantiated_contract_address()?;
            let blob_canon_account_id: cosmrs::AccountId = blob_address.as_str().parse().unwrap();
            if blob_canon_account_id.to_bytes() != expected_addr.as_slice() {
                // This shouldn't ever happen because we checked instantiate2 address before actually instantiating
                // But if it have different address then we have bad bug
                panic!("Unexpected error: Instantiated blob address doesn't match to the expected");
            }

            contract.upload_if_needed()?;
            let contract_code_id = contract.code_id()?;
            contract.set_address(&blob_address);
            contract.migrate(migrate_msg, contract_code_id)?;
            Ok(())
        }
    }
}
