/// This function encapsulates the specification of validating the signature
/// requirements In particular, code from mod.rs::validate_owner is checked
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
        && (owner!(owner_account_info) == &PROGRAM_ID)
    {
        // Guaranteed to succeed by `cheatcode_is_multisig`
        let multisig_is_initialised = maybe_multisig_is_initialised.unwrap();
        if multisig_is_initialised.is_err() {
            assert_eq!(result, Err(ProgramError::InvalidAccountData));
            return result;
        } else if !multisig_is_initialised.unwrap() {
            assert_eq!(result, Err(ProgramError::UninitializedAccount));
            return result;
        } else {
            let multisig = get_multisig(owner_account_info);

            // Single loop matching the implementation's validate_owner:
            // outer over tx_signers, inner over registered keys, with
            // matched[position] to prevent double-counting and to skip
            // unsigned detection when a position was already claimed.
            let mut num_signers: usize = 0;
            let mut matched = [false; MAX_SIGNERS as usize];

            for potential_signer in tx_signers.iter() {
                for (position, registered_key) in
                    multisig.signers[0..multisig.n as usize].iter().enumerate()
                {
                    if registered_key == key!(potential_signer) && !matched[position] {
                        if !is_signer!(potential_signer) {
                            assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
                            return result;
                        }
                        matched[position] = true;
                        num_signers += 1;
                    }
                }
            }

            // Check if we have enough signers
            if num_signers < multisig.m as usize {
                assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
                return result;
            }

            return Ok(());
        }
    }
    // Non-multisig case - check if owner_account_info.is_signer()
    else if !is_signer!(owner_account_info) {
        assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
        return result;
    }

    Ok(())
}
