mod common;

use cosmrs::proto::cosmos::authz::v1beta1::{GenericAuthorization, Grant, MsgGrant};
use cosmrs::proto::cosmwasm::wasm::v1::{
    ContractExecutionAuthorization, ContractMigrationAuthorization,
};
use cosmrs::proto::prost::Name;
use cosmwasm_std::to_json_binary;
use cw_orch::anyhow;
use cw_orch_daemon::{networks, Daemon, TxSender};

// From https://github.com/CosmosContracts/juno/blob/32568dba828ff7783aea8cb5bb4b8b5832888255/docker/test-user.env#L2
const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";

#[test]
fn daemon_local() -> anyhow::Result<()> {
    let daemon = Daemon::builder(networks::LOCAL_JUNO)
        .is_test(true)
        .mnemonic(LOCAL_MNEMONIC)
        .build()
        .unwrap();

    common::test(daemon)
}

#[test]
fn daemon_authz_test() -> anyhow::Result<()> {
    let network = networks::LOCAL_JUNO;

    // Create granter daemon (DAO/organization wallet)
    let granter_daemon = Daemon::builder(network.clone())
        .is_test(true)
        .mnemonic(LOCAL_MNEMONIC)
        .build()
        .unwrap();

    // Create grantee daemon (hot wallet) with different mnemonic
    let grantee_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let mut grantee_daemon = Daemon::builder(network.clone())
        .is_test(true)
        .mnemonic(grantee_mnemonic)
        .build()
        .unwrap();

    let grantee_wallet = grantee_daemon.sender();
    let granter_wallet = granter_daemon.sender();

    let granter = granter_wallet.pub_addr_str();
    let grantee = grantee_wallet.pub_addr_str();

    let msgs = vec![
        "/cosmwasm.wasm.v1.MsgExecuteContract",
        "/cosmwasm.wasm.v1.MsgMigrateContract",
        "/cosmwasm.wasm.v1.MsgStoreCode",
        "/cosmwasm.wasm.v1.MsgInstantiateContract",
    ];

    let mut grants = vec![];
    for msg in msgs {
        let grant = MsgGrant {
            granter: granter.to_string(),
            grantee: grantee.to_string(),
            grant: Some(Grant {
                authorization: Some(cosmrs::Any {
                    type_url: GenericAuthorization::type_url(),
                    value: to_json_binary(&msg.to_string()).unwrap().to_vec(),
                }),
                expiration: None,
            }),
        };
        grants.push(cosmrs::Any::from_msg(&grant)?);
    }

    let rt = granter_daemon.rt_handle.clone();

    // Grant all wasm permissions from granter to grantee
    rt.block_on(granter_wallet.commit_tx_any(grants, Some("Grant all wasm permissions")))?;

    println!("Granter address: {}", granter);
    println!("Grantee address: {}", grantee);
    println!("Testing authz deployment with accurate instantiate2 address calculation");

    // Clone the daemon first to avoid borrow checker conflicts
    let mut grantee_daemon_clone = grantee_daemon.clone();

    // Set up authz: grantee acts on behalf of granter
    grantee_daemon_clone
        .sender_mut()
        .set_authz_granter(&granter_wallet.address());

    common::test_with_authz(
        grantee_daemon_clone,
        granter_wallet.address(),
        grantee_wallet.address(),
    )
}
