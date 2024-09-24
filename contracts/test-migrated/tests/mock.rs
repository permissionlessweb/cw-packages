mod common;

use cw_orch::{anyhow, mock::MockBech32};

#[test]
fn mock() -> anyhow::Result<()> {
    let chain = MockBech32::new("cosm");

    common::test(chain)
}
