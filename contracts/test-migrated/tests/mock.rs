mod common;

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
