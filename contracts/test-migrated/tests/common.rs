use cosmrs::AccountId;
use cosmwasm_std::{instantiate2_address, CanonicalAddr};
use cw_blob::interface::CwBlob;
use cw_orch::prelude::*;
use cw_test_migrated::interface::MigratedBlob;

pub fn test<T: CwEnv>(chain: T) {
    let blob = CwBlob::new("blob", chain.clone());
    blob.upload().unwrap();
    let checksum = chain
        .wasm_querier()
        .code_id_hash(blob.code_id().unwrap())
        .unwrap();

    let migrated_blob = MigratedBlob::new("migrated_blob", chain.clone());
    migrated_blob.upload().unwrap();
    let account_id: AccountId = chain.sender_addr().as_str().parse().unwrap();
    let prefix = account_id.prefix();
    let canon = account_id.to_bytes();
    let canon_address = instantiate2_address(
        checksum.as_slice(),
        &CanonicalAddr::from(canon),
        b"cw20_base",
    )
    .unwrap();
    let contract_address =
        Addr::unchecked(AccountId::new(prefix, &canon_address).unwrap().to_string());
    migrated_blob.set_address(&contract_address);

    blob.instantiate2(
        &cosmwasm_std::Empty {},
        Some(&chain.sender_addr()),
        &[],
        cosmwasm_std::Binary::from(b"cw20_base"),
    )
    .unwrap();

    migrated_blob
        .migrate(
            &cw_test_migrated::InstantiateMsg {
                key: b"foo".to_vec(),
                value: b"bar".to_vec(),
            },
            migrated_blob.code_id().unwrap(),
        )
        .unwrap();
    let res = migrated_blob.raw_query(b"foo".to_vec()).unwrap();
    assert_eq!(res, b"bar")
}
