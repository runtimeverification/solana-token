#[inline(never)]
fn inner_test_validate_owner(
    expected_owner: &Pubkey,
    owner_account_info: &AccountInfo,
    tx_signers: &[AccountInfo],
    maybe_multisig_is_initialised: Option<Result<bool, ProgramError>>,
    result: Result<(), ProgramError>,
) -> Result<(), ProgramError> {
    if expected_owner != key!(owner_account_info) {
        assert_eq!(result, Err(ProgramError::Custom(4)));
        return result;
    }
    // We add the `maybe_multisig_is_initialised.is_some()` to not branch vacuously in the
    // non-multisig cases
    else if maybe_multisig_is_initialised.is_some()
        && owner_account_info.data_len() == Multisig::LEN
        && is_owned_by_program!(owner_account_info)
    {
        let multisig_is_initialised = maybe_multisig_is_initialised.unwrap();
        if multisig_is_initialised.is_err() {
            assert_eq!(result, Err(ProgramError::InvalidAccountData));
            return result;
        } else if !multisig_is_initialised.unwrap() {
            assert_eq!(result, Err(ProgramError::UninitializedAccount));
            return result;
        } else {
            let multisig = get_multisig(owner_account_info);
            let unsigned_exists = tx_signers.iter().any(|potential_signer| {
                multisig_signers!(multisig).iter().any(|registered_key| {
                    registered_key == key!(potential_signer) && !is_signer!(potential_signer)
                })
            });
            if unsigned_exists {
                assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
                return result;
            }

            let signers_count = multisig_signers!(multisig)
                .iter()
                .filter_map(|registered_key| {
                    tx_signers.iter().find(|potential_signer| {
                        key!(potential_signer) == registered_key && is_signer!(potential_signer)
                    })
                })
                .count();
            if signers_count < multisig_m!(multisig) as usize {
                assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
                return result;
            }

            return Ok(());
        }
    }
    // Non-multisig case - check if owner_account_info.is_signer
    else if !is_signer!(owner_account_info) {
        assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
        return result;
    }

    Ok(())
}
