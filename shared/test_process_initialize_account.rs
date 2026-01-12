#[inline(never)]
fn test_process_initialize_account(accounts: &[AccountInfo; 4]) -> ProgramResult {
    cheatcode_account!(&accounts[0]);
    cheatcode_mint!(&accounts[1]);
    cheatcode_account!(&accounts[2]);
    cheatcode_rent!(&accounts[3]);

    //-Initial State-----------------------------------------------------------
    let initial_state_new_account = get_account(&accounts[0]).account_state();

    let minimum_balance = get_rent(&accounts[3]).minimum_balance(accounts[0].data_len()); // TODO float problem
    let is_native_mint = key!(accounts[1]) == native_mint_id!();
    let mint_is_initialised = get_mint(&accounts[1]).is_initialized();

    //-Process Instruction-----------------------------------------------------
    let result = call_process_initialize_account!(accounts);

    //-Assert Postconditions---------------------------------------------------
    if accounts.len() < 4 {
        assert_eq!(result, Err(ProgramError::NotEnoughAccountKeys));
    } else if key!(accounts[3]) != rent_id!() {
        assert_eq!(result, Err(ProgramError::InvalidArgument))
    } else if accounts[0].data_len() != Account::LEN {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if initial_state_new_account.is_err() {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if initial_state_new_account.unwrap() != account_state_uninitialized!() {
        assert_eq!(result, Err(ProgramError::Custom(6)))
    } else if accounts[0].lamports() < minimum_balance {
        assert_eq!(result, Err(ProgramError::Custom(0)))
    } else if !is_native_mint && owner!(accounts[1]) != program_id!() {
        assert_eq!(result, Err(ProgramError::IncorrectProgramId))
    } else if !is_native_mint
        && owner!(accounts[1]) == program_id!()
        && mint_is_initialised.is_err()
    {
        assert_eq!(result, Err(ProgramError::InvalidAccountData))
    } else if !is_native_mint
        && owner!(accounts[1]) == program_id!()
        && !mint_is_initialised.unwrap()
    {
        assert_eq!(result, Err(ProgramError::Custom(2)))
    } else {
        let new_account_new = get_account(&accounts[0]);

        assert!(result.is_ok());
        assert_eq!(
            new_account_new.account_state().unwrap(),
            account_state_initialized!()
        );
        assert_eq!(mint!(new_account_new), *key!(accounts[1]));
        assert_eq!(account_owner!(new_account_new), *key!(accounts[2]));

        if is_native_mint {
            assert!(new_account_new.is_native());
            assert_eq!(new_account_new.native_amount().unwrap(), minimum_balance);
            assert_eq!(
                new_account_new.amount(),
                accounts[0].lamports() - minimum_balance
            );
        }
    }

    result
}
