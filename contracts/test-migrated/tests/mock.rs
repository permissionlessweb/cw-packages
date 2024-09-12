mod common;

use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::Checksum;
use cw_blob::interface::CwBlob;
use cw_orch::{
    mock::{
        cw_multi_test::{AppBuilder, ChecksumGenerator, MockApiBech32, WasmKeeper},
        MockBech32, MockState,
    },
    prelude::*,
};

pub struct BlobChecksumGenerator {}

impl ChecksumGenerator for BlobChecksumGenerator {
    fn checksum(&self, _creator: &cosmwasm_std::Addr, code_id: u64) -> Checksum {
        // Should be first uploaded contract
        if code_id == 1 {
            <CwBlob<MockBech32> as Uploadable>::wasm(&ChainInfoOwned::default())
                .checksum()
                .unwrap()
        } else {
            // from SimpleChecksumGenerator https://docs.rs/cw-multi-test/2.1.1/src/cw_multi_test/checksums.rs.html#19-28
            Checksum::generate(format!("contract code {}", code_id).as_bytes())
        }
    }
}

pub fn blob_mock_bech32(prefix: &'static str) -> MockBech32 {
    let state = Rc::new(RefCell::new(MockState::new()));
    let checksum_generator = BlobChecksumGenerator {};
    let mut app = AppBuilder::new_custom()
        .with_api(MockApiBech32::new(prefix))
        .with_wasm(WasmKeeper::new().with_checksum_generator(checksum_generator))
        .build(|_, _, _| {});
    app.store_code(<CwBlob<MockBech32> as Uploadable>::wrapper());
    let app = Rc::new(RefCell::new(app));

    // We create an address internally
    let sender = app.borrow().api().addr_make("sender");

    MockBech32 { sender, state, app }
}

#[test]
fn mock() {
    let chain = blob_mock_bech32("cosm");

    common::test(chain, 1)
}
