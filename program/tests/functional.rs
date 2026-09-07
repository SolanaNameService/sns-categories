use std::str::FromStr;

use crate::common::utils::sign_send_instructions;
use sns_categories::instruction::{add_member, create_category, remove_member};
use sns_categories::state::category_member::CategoryMember;
use sns_categories::state::category_metadata::CategoryMetadata;
use sns_categories::state::{CATEGORY_TLD, Tag};
use sns_categories::utils::get_name_key;
use sns_categories::{entrypoint::process_instruction, utils::get_hashed_name};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use spl_name_service::state::{NameRecordHeader, get_seeds_and_key};
use {
    borsh::BorshDeserialize,
    solana_program_test::{ProgramTest, processor},
    solana_sdk::{
        account::Account,
        signer::{Signer, keypair::Keypair},
    },
};
pub mod common;

#[tokio::test]
async fn test_offer() {
    // Create program and test environment
    let alice = Keypair::new();
    let bob = Keypair::new();
    let category_name = String::from("999-club");
    let category_member = String::from("000");
    let domain_key = get_name_key(&category_member).unwrap();

    let mut program_test = ProgramTest::new(
        "sns_categories",
        sns_categories::ID,
        processor!(process_instruction),
    );
    program_test.add_program(
        "spl_name_service",
        spl_name_service::id(),
        processor!(spl_name_service::processor::Processor::process_instruction),
    );

    program_test.add_account(
        alice.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );
    program_test.add_account(
        bob.pubkey(),
        Account {
            lamports: 100_000_000_000,
            ..Account::default()
        },
    );

    let category_tld_account = NameRecordHeader {
        parent_name: Pubkey::from_str("ZoAhWEqTVqHVqupYmEanDobY7dee5YKbQox9BNASZzU").unwrap(),
        owner: sns_categories::central_state::KEY,
        class: Pubkey::default(),
    };
    program_test.add_account(
        sns_categories::state::CATEGORY_TLD,
        Account {
            lamports: 100_000_000_000,
            data: borsh::to_vec(&category_tld_account).unwrap(),
            ..Account::default()
        },
    );

    ////
    // Create test context
    ////
    let mut prg_test_ctx = program_test.start_with_context().await;

    let hashed = get_hashed_name(&category_name);
    let (category_metadata, _) =
        get_seeds_and_key(&spl_name_service::ID, hashed, None, Some(&CATEGORY_TLD));

    let ix = create_category(
        create_category::Accounts {
            system_program: &solana_system_interface::program::ID,
            name_service_program: &spl_name_service::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            central_state: &sns_categories::central_state::KEY,
            #[cfg(not(feature = "no-signer"))]
            signer: &Pubkey::default(),
            category_metadata: &category_metadata,
            category_tld: &CATEGORY_TLD,
        },
        create_category::Params {
            category_name: category_name.clone(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    let hashed = get_hashed_name(&category_member);
    let (member, _) = get_seeds_and_key(
        &spl_name_service::ID,
        hashed,
        None,
        Some(&category_metadata),
    );

    let ix = add_member(
        add_member::Accounts {
            name_service_program: &spl_name_service::ID,
            system_program: &solana_system_interface::program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            category_metadata: &category_metadata,
            category_member: &member,
            central_state: &sns_categories::central_state::KEY,
            #[cfg(not(feature = "no-signer"))]
            signer: &Pubkey::default(),
        },
        add_member::Params {
            category_member: category_member.clone(),
            category_name: category_name.clone(),
        },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    ///////////////////////////////////
    // Verify some state
    ///////////////////////////////////

    let acc = prg_test_ctx
        .banks_client
        .get_account(category_metadata)
        .await
        .unwrap()
        .unwrap();
    let registry = NameRecordHeader::unpack_unchecked(&acc.data[..NameRecordHeader::LEN]).unwrap();
    assert_eq!(registry.class, Pubkey::default());
    assert_eq!(registry.owner, sns_categories::central_state::KEY);
    assert_eq!(registry.parent_name, CATEGORY_TLD);

    let des: CategoryMetadata =
        CategoryMetadata::deserialize(&mut &acc.data[NameRecordHeader::LEN..]).unwrap();
    assert_eq!(des.name, category_name);
    assert_eq!(des.nb_of_registered_domains, 1);
    matches!(des.tag, Tag::CategoryMetadata);

    let acc = prg_test_ctx
        .banks_client
        .get_account(member)
        .await
        .unwrap()
        .unwrap();
    let registry = NameRecordHeader::unpack_unchecked(&acc.data[..NameRecordHeader::LEN]).unwrap();
    assert_eq!(registry.class, Pubkey::default());
    assert_eq!(registry.owner, sns_categories::central_state::KEY);
    assert_eq!(registry.parent_name, category_metadata);

    let des: CategoryMember =
        CategoryMember::deserialize(&mut &acc.data[NameRecordHeader::LEN..]).unwrap();
    assert_eq!(des.name, category_member);
    matches!(des.tag, Tag::CategoryMember);
    assert_eq!(des.domain_key, domain_key);

    let ix = remove_member(
        remove_member::Accounts {
            name_service_program: &spl_name_service::ID,
            system_program: &solana_system_interface::program::ID,
            fee_payer: &prg_test_ctx.payer.pubkey(),
            category_metadata: &category_metadata,
            category_member: &member,
            central_state: &sns_categories::central_state::KEY,
            #[cfg(not(feature = "no-signer"))]
            signer: &Pubkey::default(),
        },
        remove_member::Params { category_member },
    );
    sign_send_instructions(&mut prg_test_ctx, vec![ix], vec![])
        .await
        .unwrap();

    ///////////////////////////////////
    // Verify some state
    ///////////////////////////////////

    let acc = prg_test_ctx
        .banks_client
        .get_account(category_metadata)
        .await
        .unwrap()
        .unwrap();
    let des: CategoryMetadata =
        CategoryMetadata::deserialize(&mut &acc.data[NameRecordHeader::LEN..]).unwrap();
    assert_eq!(des.name, category_name);
    assert_eq!(des.nb_of_registered_domains, 0);
    matches!(des.tag, Tag::CategoryMetadata);
}
