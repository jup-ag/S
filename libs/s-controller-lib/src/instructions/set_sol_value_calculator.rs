use s_controller_interface::{
    set_sol_value_calculator_ix, set_sol_value_calculator_ix_with_program_id, SControllerError,
    SetSolValueCalculatorIxArgs, SetSolValueCalculatorKeys,
};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_readonly_account::{
    pubkey::{ReadonlyAccountOwner, ReadonlyAccountPubkey},
    ReadonlyAccountData,
};

use crate::{
    index_to_u32, ix_extend_with_sol_value_calculator_accounts, SetSolValueCalculatorByMintFreeArgs,
};

pub fn set_sol_value_calculator_ix_full(
    accounts: SetSolValueCalculatorKeys,
    lst_index: usize,
    sol_value_calculator_accounts: &[AccountMeta],
    sol_value_calculator_program_id: Pubkey,
) -> Result<Instruction, ProgramError> {
    let lst_index = index_to_u32(lst_index)?;
    let mut ix = set_sol_value_calculator_ix(accounts, SetSolValueCalculatorIxArgs { lst_index })?;
    ix_extend_with_sol_value_calculator_accounts(
        &mut ix,
        sol_value_calculator_accounts,
        sol_value_calculator_program_id,
    )
    .map_err(|_e| SControllerError::MathError)?;
    Ok(ix)
}

pub fn set_sol_value_calculator_ix_by_mint_full<
    S: ReadonlyAccountData,
    L: ReadonlyAccountData,
    M: ReadonlyAccountOwner + ReadonlyAccountPubkey,
>(
    free_args: &SetSolValueCalculatorByMintFreeArgs<S, L, M>,
    sol_value_calculator_accounts: &[AccountMeta],
    sol_value_calculator_program_id: Pubkey,
) -> Result<Instruction, ProgramError> {
    let (keys, lst_index) = free_args.resolve()?;
    let ix = set_sol_value_calculator_ix_full(
        keys,
        lst_index,
        sol_value_calculator_accounts,
        sol_value_calculator_program_id,
    )?;
    Ok(ix)
}

pub fn set_sol_value_calculator_ix_by_mint_full_with_program_id<
    S: ReadonlyAccountData,
    L: ReadonlyAccountData,
    M: ReadonlyAccountOwner + ReadonlyAccountPubkey,
>(
    program_id: Pubkey,
    free_args: &SetSolValueCalculatorByMintFreeArgs<S, L, M>,
    sol_value_calculator_accounts: &[AccountMeta],
    sol_value_calculator_program_id: Pubkey,
) -> Result<Instruction, ProgramError> {
    let (keys, lst_index) = free_args.resolve_for_prog(program_id)?;
    let lst_index = index_to_u32(lst_index)?;
    let mut ix = set_sol_value_calculator_ix_with_program_id(
        program_id,
        keys,
        SetSolValueCalculatorIxArgs { lst_index },
    )?;
    ix_extend_with_sol_value_calculator_accounts(
        &mut ix,
        sol_value_calculator_accounts,
        sol_value_calculator_program_id,
    )
    .map_err(|_e| SControllerError::MathError)?;

    Ok(ix)
}
