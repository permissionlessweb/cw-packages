mod common;
use hex;

use cw_orch::{anyhow, mock::MockBech32};
use cw_orch_clone_testing::CloneTesting;
use cw_orch_daemon::networks;

#[test]
fn mock() -> anyhow::Result<()> {
    let chain = MockBech32::new("cosm");

    common::test(chain)
}

#[test]
fn clone_test() -> anyhow::Result<()> {
    let chain = CloneTesting::new(networks::JUNO_1)?;

    common::test(chain)
}


#[test]
fn create_hex() {
    let hex_string = "a9075255921026bcc49a6812f235f3f0c6f51cf5207db556daecefac74080a85";
    let bytes = hex::decode(hex_string).unwrap();
    let array: [u8; 32] = bytes.try_into().unwrap();
    println!("pub const CHECKSUM: [u8; 32] = {:?};", array);
}