use cosmrs::AccountId;
use cosmwasm_std::{Binary, CanonicalAddr};
use cw_blob::interface::{CwBlob, MigrationFromBlob};
use cw_orch::{anyhow, prelude::*};
use cw_test_migrated::interface::MigratedBlob;

pub fn test<T: CwEnv>(chain: T) -> anyhow::Result<()> {
    let blob = CwBlob::new("cw:blob", chain.clone());
    blob.upload_if_needed()?;
    let blob_code_id = blob.code_id()?;

    let first_migrated_blob = MigratedBlob::new("first_migrated", chain.clone());
    let first_salt = Binary::new(b"first".to_vec());

    let second_migrated_blob = MigratedBlob::new("second_migrated", chain.clone());
    let second_salt = Binary::new(b"second".to_vec());

    let expected_blob_addr = chain.wasm_querier().instantiate2_addr(
        blob_code_id,
        &chain.sender_addr(),
        first_salt.clone(),
    )?;
    let expected_blob_account_id: AccountId = expected_blob_addr.parse().unwrap();
    let expected_blob_canon_addr: CanonicalAddr =
        CanonicalAddr::from(expected_blob_account_id.to_bytes());

    first_migrated_blob.deterministic_instantiate(
        &cw_test_migrated::InstantiateMsg {
            key: b"foo".to_vec(),
            value: b"bar".to_vec(),
        },
        blob_code_id,
        expected_blob_canon_addr,
        first_salt.clone(),
    )?;

    let expected_blob_addr = chain.wasm_querier().instantiate2_addr(
        blob_code_id,
        &chain.sender_addr(),
        second_salt.clone(),
    )?;
    let expected_blob_account_id: AccountId = expected_blob_addr.parse().unwrap();
    let expected_blob_canon_addr: CanonicalAddr =
        CanonicalAddr::from(expected_blob_account_id.to_bytes());

    second_migrated_blob.deterministic_instantiate(
        &cw_test_migrated::InstantiateMsg {
            key: b"bar".to_vec(),
            value: b"foo".to_vec(),
        },
        blob_code_id,
        expected_blob_canon_addr,
        second_salt.clone(),
    )?;

    let res = first_migrated_blob.raw_query(b"foo".to_vec())?;
    assert_eq!(res, b"bar");

    let res = second_migrated_blob.raw_query(b"bar".to_vec())?;
    assert_eq!(res, b"foo");

    Ok(())
}
