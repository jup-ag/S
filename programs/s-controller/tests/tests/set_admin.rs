use s_controller_interface::{set_admin_ix, PoolState};
use s_controller_lib::{program::POOL_STATE_ID, try_pool_state, SetAdminFreeArgs};

use sanctum_solana_test_utils::{test_fixtures_dir, IntoAccount};
use solana_program_test::*;
use solana_readonly_account::sdk::KeyedAccount;
use solana_sdk::{
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};

use crate::common::*;

#[tokio::test]
async fn basic_set_admin() {
    let mock_auth_kp =
        read_keypair_file(test_fixtures_dir().join("s-controller-test-initial-authority-key.json"))
            .unwrap();
    let new_admin_kp = Keypair::new();
    let another_new_admin_kp = Keypair::new();

    let mut program_test = ProgramTest::default();

    program_test.add_program(
        "s_controller",
        s_controller_lib::program::ID,
        processor!(s_controller::entrypoint::process_instruction),
    );

    let pool_state_acc = MockPoolState(DEFAULT_POOL_STATE).into_account();

    program_test.add_account(POOL_STATE_ID, pool_state_acc.clone());

    let (mut banks_client, payer, last_blockhash) = program_test.start().await;

    // Change admin
    let ix = set_admin_ix(
        SetAdminFreeArgs {
            new_admin: new_admin_kp.pubkey(),
            pool_state: KeyedAccount {
                pubkey: POOL_STATE_ID,
                account: pool_state_acc.clone(),
            },
        }
        .resolve()
        .unwrap(),
    )
    .unwrap();

    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[&payer, &mock_auth_kp], last_blockhash);

    banks_client.process_transaction(tx).await.unwrap();

    let pool_state_acc = banks_client.get_pool_state_acc().await;
    let pool_state = try_pool_state(&pool_state_acc.data).unwrap();
    assert_eq!(
        *pool_state,
        PoolState {
            admin: new_admin_kp.pubkey(),
            ..*pool_state
        }
    );

    // Change admin again
    let ix2 = set_admin_ix(
        SetAdminFreeArgs {
            new_admin: another_new_admin_kp.pubkey(),
            pool_state: KeyedAccount {
                pubkey: POOL_STATE_ID,
                account: pool_state_acc.clone(),
            },
        }
        .resolve()
        .unwrap(),
    )
    .unwrap();

    let mut tx2 = Transaction::new_with_payer(&[ix2], Some(&payer.pubkey()));
    tx2.sign(&[&payer, &new_admin_kp], last_blockhash);

    banks_client.process_transaction(tx2).await.unwrap();

    let pool_state_acc = banks_client.get_pool_state_acc().await;
    let pool_state = try_pool_state(&pool_state_acc.data).unwrap();
    assert_eq!(
        *pool_state,
        PoolState {
            admin: another_new_admin_kp.pubkey(),
            ..*pool_state
        }
    );
}
