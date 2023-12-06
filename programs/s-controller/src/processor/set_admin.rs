use s_controller_interface::{
    set_admin_verify_account_keys, set_admin_verify_account_privileges, SetAdminAccounts,
};
use s_controller_lib::{try_pool_state_mut, SetAdminFreeArgs};
use sanctum_onchain_utils::utils::{
    load_accounts, log_and_return_acc_privilege_err, log_and_return_wrong_acc_err,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

pub fn process_set_admin(accounts: &[AccountInfo]) -> ProgramResult {
    let checked = verify_set_admin(accounts)?;
    process_set_admin_unchecked(checked)
}

fn process_set_admin_unchecked(
    SetAdminAccounts {
        current_admin: _,
        new_admin,
        pool_state,
    }: SetAdminAccounts,
) -> ProgramResult {
    let mut pool_state_data = pool_state.try_borrow_mut_data()?;
    let pool_state = try_pool_state_mut(&mut pool_state_data)?;
    pool_state.admin = *new_admin.key;

    Ok(())
}

fn verify_set_admin<'a, 'info>(
    accounts: &'a [AccountInfo<'info>],
) -> Result<SetAdminAccounts<'a, 'info>, ProgramError> {
    let actual: SetAdminAccounts = load_accounts(accounts)?;

    let expected = SetAdminFreeArgs {
        new_admin: *actual.new_admin.key,
    }
    .resolve();

    set_admin_verify_account_keys(&actual, &expected).map_err(log_and_return_wrong_acc_err)?;
    set_admin_verify_account_privileges(&actual).map_err(log_and_return_acc_privilege_err)?;

    Ok(actual)
}
