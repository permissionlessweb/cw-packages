use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Never, Response};

/// Checksum of the wasm
// Unused, so optimized out of the wasm
pub const CHECKSUM: [u8; 32] = [
    89, 178, 71, 166, 117, 182, 203, 76, 79, 113, 13, 221, 231, 111, 158, 232, 2, 192, 224, 164,
    210, 48, 131, 111, 30, 203, 245, 199, 163, 20, 125, 21,
];

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(_: DepsMut, _: Env, _: MessageInfo, _: Empty) -> Result<Response, Never> {
    Ok(Response::new())
}

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use super::*;

    use cosmwasm_std::{
        instantiate2_address, Binary, CanonicalAddr, Checksum, Instantiate2AddressError,
    };
    use cw_orch::{contract::Contract, prelude::*};

    // We don't want it to be manually instantiated/executed/etc, only uploaded. So not using cw_orch interface
    #[derive(Clone)]
    pub struct CwBlob<Chain>(Contract<Chain>);

    impl<Chain> CwBlob<Chain> {
        pub fn new(contract_id: impl ToString, chain: Chain) -> Self {
            Self(Contract::new(contract_id, chain))
        }
    }

    impl<Chain: ChainState> ContractInstance<Chain> for CwBlob<Chain> {
        fn as_instance(&self) -> &Contract<Chain> {
            &self.0
        }

        fn as_instance_mut(&mut self) -> &mut Contract<Chain> {
            &mut self.0
        }
    }

    impl<T: CwEnv> Uploadable for CwBlob<T> {
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            wasm_path()
        }

        fn wrapper() -> Box<dyn MockContract<Empty, Empty>> {
            Box::new(
                ContractWrapper::new_with_empty(
                    |_, _, _, _: Empty| -> Result<Response, Never> { unreachable!() },
                    super::instantiate,
                    |_, _, _: Empty| -> Result<Binary, Never> { unreachable!() },
                )
                .with_checksum(checksum()),
            )
        }
    }

    pub fn checksum() -> Checksum {
        Checksum::from(CHECKSUM)
    }

    pub(crate) fn wasm_path() -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("cw_blob")
            .unwrap()
    }

    pub trait DeterministicInstantiation<Chain: CwEnv>:
        ContractInstance<Chain> + CwOrchUpload<Chain> + MigratableContract
    {
        /// Instantiate blob and migrate to your desired contract.
        /// It will upload your contract, if it's not uploaded already
        ///
        /// Checksum of the uploaded blob_code_id on chain should match [CwBlob::checksum()]
        fn deterministic_instantiate(
            &self,
            migrate_msg: &Self::MigrateMsg,
            // Ensures blob is uploaded and avoid couple of redundant checks
            blob_code_id: u64,
            expected_addr: CanonicalAddr,
            salt: Binary,
        ) -> Result<(), CwOrchError> {
            let chain = self.environment();
            let on_chain_checksum = chain
                .wasm_querier()
                .code_id_hash(blob_code_id)
                .map_err(Into::into)?;
            let creator = chain.sender_addr();
            let label = self.id();

            // Check stored checksum matches
            {
                let expected_checksum = checksum();
                if on_chain_checksum != expected_checksum {
                    return Err(CwOrchError::StdErr(format!(
                "Expected blob checksum: {expected_checksum}, stored under given code_id: {on_chain_checksum}"
            )));
                }
            }

            // Check incoming address of instantiated blob
            {
                let actual_addr = self.deterministic_address(&salt)?;
                if actual_addr != expected_addr {
                    return Err(CwOrchError::StdErr(
                        "Predicted blob address doesn't match to the expected".to_owned(),
                    ));
                }
            }

            let response = chain
                .instantiate2(
                    blob_code_id,
                    &cosmwasm_std::Empty {},
                    Some(&label),
                    Some(&creator),
                    &[],
                    salt,
                )
                .map_err(Into::into)?;
            let blob_address = response.instantiated_contract_address()?;
            let blob_cosmrs_account_id: cosmrs::AccountId = blob_address.as_str().parse().unwrap();
            if blob_cosmrs_account_id.to_bytes() != expected_addr.as_slice() {
                // This shouldn't ever happen because we checked instantiate2 address before actually instantiating
                // But if it have different address then we have bad bug
                panic!("Unexpected error: Instantiated blob address doesn't match to the expected");
            }

            self.upload_if_needed()?;
            let contract_code_id = self.code_id()?;
            self.set_address(&blob_address);
            self.migrate(migrate_msg, contract_code_id)?;
            Ok(())
        }

        fn deterministic_address(
            &self,
            salt: &Binary,
        ) -> Result<CanonicalAddr, Instantiate2AddressError> {
            let creator = self.environment().sender_addr();
            let account_id: cosmrs::AccountId = creator.as_str().parse().unwrap();
            let canon_creator = CanonicalAddr::from(account_id.to_bytes());
            let actual_addr =
                instantiate2_address(checksum().as_slice(), &canon_creator, salt.as_slice())?;
            Ok(actual_addr)
        }
    }
}

#[cfg(test)]
mod test {
    use interface::{checksum, wasm_path};

    use super::*;

    #[test]
    fn test_checksum() {
        let checksum = checksum();
        let expected_checksum = wasm_path().checksum().unwrap();
        assert_eq!(checksum, expected_checksum);
    }
}
