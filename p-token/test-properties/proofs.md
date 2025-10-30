Proofs to run with `run-proofs.sh -a`:

| Start symbol name                                      |
|--------------------------------------------------------|
| test_ptoken_domain_data                                |
| test_process_approve_account                           |
| test_process_approve_checked_account                   |
| test_process_withdraw_excess_lamports_account_account  |
| test_process_withdraw_excess_lamports_mint_account     |
| test_process_withdraw_excess_lamports_multisig_account |
| test_process_initialize_mint_freeze                    |
| test_process_initialize_mint_no_freeze                 |
| test_process_initialize_account                        |
| test_process_initialize_account2                       |
| test_process_transfer_account                          |
| test_process_mint_to_account                           |
| test_process_burn_account                              |
| test_process_close_account_account                     |
| test_process_transfer_checked_account                  |
| test_process_burn_checked_account                      |
| test_process_initialize_account3                       |
| test_process_initialize_mint2_freeze                   |
| test_process_initialize_mint2_no_freeze                |
| test_process_revoke_account                            |
| test_process_freeze_account_account                    |
| test_process_thaw_account_account                      |
| test_process_mint_to_checked_account                   |
| test_process_sync_native                               |
| test_process_get_account_data_size                     |
| test_process_initialize_immutable_owner                |
| test_process_amount_to_ui_amount                       |
| test_process_ui_amount_to_amount                       |
| test_process_set_authority_account_account             |
| test_process_set_authority_mint_account                |

Cheat codes are missing or a problem for these proofs, therefore not recommended to execute them
(keep the empty first column so `run-proofs.sh` won't pick these up):

|   | test_process_initialize_multisig  |
|   | test_process_initialize_multisig2 |
