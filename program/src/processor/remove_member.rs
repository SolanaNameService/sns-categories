//! Remove category member

use crate::{
    cpi, error::SnsCategoriesError, state::Tag, state::category_metadata::CategoryMetadata,
    utils::get_category_member_key,
};
use {
    bonfida_utils::{
        BorshSize, InstructionsAccount,
        checks::{check_account_key, check_account_owner, check_signer},
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::program_pack::Pack,
    solana_program::{
        account_info::{AccountInfo, next_account_info},
        entrypoint::ProgramResult,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    spl_name_service::state::NameRecordHeader,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize)]
pub struct Params {
    // The category member
    pub category_member: String,
}

#[derive(InstructionsAccount)]
pub struct Accounts<'a, T> {
    /// The system program account
    pub system_program: &'a T,

    /// The SPL name service program
    pub name_service_program: &'a T,

    /// The fee payer account
    #[cons(writable, signer)]
    pub fee_payer: &'a T,

    /// The category metadata account
    #[cons(writable)]
    pub category_metadata: &'a T,

    /// The category metadata account
    #[cons(writable)]
    pub category_member: &'a T,

    pub central_state: &'a T,

    /// The required instruction signer
    #[cons(signer)]
    #[cfg(not(feature = "no-signer"))]
    pub signer: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            system_program: next_account_info(accounts_iter)?,
            name_service_program: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            category_metadata: next_account_info(accounts_iter)?,
            category_member: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            #[cfg(not(feature = "no-signer"))]
            signer: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(
            accounts.system_program,
            &solana_system_interface::program::ID,
        )?;
        check_account_key(accounts.name_service_program, &spl_name_service::ID)?;
        check_account_key(accounts.central_state, &crate::central_state::KEY)?;
        #[cfg(not(feature = "no-signer"))]
        check_account_key(accounts.signer, &crate::state::SIGNER)?;

        // Check owners
        check_account_owner(accounts.category_metadata, &spl_name_service::ID)?;
        check_account_owner(accounts.category_member, &spl_name_service::ID)?;

        // Check signer
        check_signer(accounts.fee_payer)?;
        #[cfg(not(feature = "no-signer"))]
        check_signer(accounts.signer)?;

        Ok(accounts)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], params: Params) -> ProgramResult {
    let Params { category_member } = params;
    let accounts = Accounts::parse(accounts, program_id)?;

    let member_registry = NameRecordHeader::unpack_unchecked(
        &accounts.category_member.data.borrow()[..NameRecordHeader::LEN],
    )?;
    check_account_key(accounts.category_metadata, &member_registry.parent_name)?;

    let mut category_metadata =
        CategoryMetadata::from_buffer(accounts.category_metadata, Tag::CategoryMetadata)?;

    category_metadata.nb_of_registered_domains = category_metadata
        .nb_of_registered_domains
        .checked_sub(1)
        .ok_or(SnsCategoriesError::Overflow)?;

    cpi::name_service_update(
        0,
        borsh::to_vec(&category_metadata)?,
        cpi::NameServiceUpdateAccounts {
            name_account: accounts.category_metadata,
            name_service_program: accounts.name_service_program,
            signer: accounts.central_state,
        },
    )?;

    let key = get_category_member_key(&category_member, accounts.category_metadata.key);
    check_account_key(accounts.category_member, &key)?;

    cpi::name_service_delete(cpi::NameServiceDeleteAccounts {
        name_account: accounts.category_member,
        name_service_program: accounts.name_service_program,
        system_program: accounts.system_program,
        signer: accounts.central_state,
        refund_target: accounts.fee_payer,
    })?;

    Ok(())
}
