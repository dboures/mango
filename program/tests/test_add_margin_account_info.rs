// Tests related to editing MarginAccount info
#![cfg(feature = "test-bpf")]

mod helpers;

use helpers::*;
use solana_program::account_info::AccountInfo;
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::mem::size_of;

use mango::{
    entrypoint::process_instruction,
    instruction::{add_margin_account_info, init_margin_account},
    state::MarginAccount,
};

//test it works on empty
#[tokio::test]
async fn test_add_margin_account_info_succeeds() {
    // Test that the add_margin_account_info instruction succeeds for a MarginAccount with no info
    let program_id = Pubkey::new_unique();

    let mut test = ProgramTest::new("mango", program_id, processor!(process_instruction));

    // limit to track compute unit increase
    test.set_bpf_compute_max_units(50_000);

    // setup mango group
    let mango_group = add_mango_group_prodlike(&mut test, program_id);

    // setup user account
    let user = Keypair::new();
    test.add_account(
        user.pubkey(),
        Account::new(u32::MAX as u64, 0, &user.pubkey()),
    );

    // setup MarginAccount
    let margin_account_pk = Pubkey::new_unique();
    test.add_account(
        margin_account_pk,
        Account::new(u32::MAX as u64, size_of::<MarginAccount>(), &program_id),
    );

    let margin_account_info = [
        76, 101, 116, 39, 115, 32, 77, 97, 110, 103, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0,
    ];

    // setup test harness
    let (mut banks_client, payer, recent_blockhash) = test.start().await;

    {
        let mut transaction = Transaction::new_with_payer(
            &[
                mango_group.init_mango_group(&payer.pubkey()),
                init_margin_account(
                    &program_id,
                    &mango_group.mango_group_pk,
                    &margin_account_pk,
                    &user.pubkey(),
                )
                .unwrap(),
                add_margin_account_info(
                    &program_id,
                    &mango_group.mango_group_pk,
                    &margin_account_pk,
                    &user.pubkey(),
                    margin_account_info,
                )
                .unwrap(),
            ],
            Some(&payer.pubkey()),
        );

        transaction.sign(&[&payer, &user], recent_blockhash);

        // Test transaction succeeded
        assert!(banks_client.process_transaction(transaction).await.is_ok());

        // Test that margin account info is what we expect
        let mut margin_account = banks_client
            .get_account(margin_account_pk)
            .await
            .unwrap()
            .unwrap();
        let account_info: AccountInfo = (&margin_account_pk, &mut margin_account).into();

        let margin_account =
            MarginAccount::load_checked(&program_id, &account_info, &mango_group.mango_group_pk)
                .unwrap();
        assert_eq!(margin_account.info, margin_account_info);
    }

    //ideally this is how it operates, rn I'm just adding it and it works fine
    //verify that the guy we just added has no info??
    // add info
    //verify that the info we added was what we expected
}

//test that it overwrites a saved guy

// test that something happens if the input is too long?
