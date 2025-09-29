Proofs timings and steps with default settings from run-proofs.sh:
max-depth 300, max-iterations 100, timeout 45 min (2700)


| Start symbol name                       | Sec  | Status  | max steps |                                                                               |
|-----------------------------------------|------|---------|-----------|-------------------------------------------------------------------------------|
| test_process_approve                    | 590  | Stuck   | 715       | branch: account state in valid range. Both stuck on #traverseProjection of alloc `core::result::Result<(), pinocchio::program_error::ProgramError>`, `pinocchio_token_interface::state::account_state::AccountState` (both unable to decode) |
| test_process_approve_checked            | 640  | Stuck   | 794       | branch: account state in valid range. One branch stuck on #traverseProjection of `pinocchio_token_interface::state::account_state::AccountState` which we can't decode, other branches again on account being initialised. Both those branches stuck on `thunk( #applyBinOp ( binOpOffset , PtrLocal ... ))` |
| test_process_withdraw_excess_lamports   | 1300 | Stuck   | 1201      | reads alloc(AcctState), **projection error on cheat code data**, traverse projection stuck on large stack data maybe (?) `#traverseProjection ( toStack ( 2, local ( 2 ) ), Aggregate (variantIdx ( 0 ), ListItem (Intger (ARG_UINT1:Int,, *, false)) &times; 64 ), PAccountMint, ... )`               |
| test_process_initialize_mint_freeze     | 2450 | Stuck   | 1064      | 3x Overflow (Rent), 4x stuck on (trivial) ptr offset                          |
| test_process_initialize_mint_no_freeze  | 2450 | Stuck   | 1064      | 3x Overflow (Rent), 4x stuck on (trivial) ptr offset                          |
| test_process_initialize_account         | 2700 | Timeout | 2182      | 3x Overflow (Rent), Float + key comparison branches, pending                  |
| test_process_initialize_account2        | 2700 | Timeout | 2282      | 3x Overflow (Rent), Float + key comparison branches, pending                  |
| test_process_transfer                   | 2700 | Timeout | 2294      | 2x reads alloc(AcctState), **non-det branch on DELEGATE key**, pending        |
| test_process_mint_to                    | 660  | Stuck   | 845       | I think all decoding AccountState and other allocs                            |
| test_process_burn                       | 2700 | Timeout | 2510      | reads alloc(AcctState), **non-det branch on DELEGATE key**, pending           |
| test_process_close_account              | 2650 | Stuck   | 2409      | reads alloc(AcctState)                                                        |
| test_process_transfer_checked           | 2700 | Timeout | 2483      | 2x reads alloc (AcctState), **non-det branch on DELEGATE key**, pending       |
| test_process_burn_checked               | 2700 | Timeout | 2510      | 2x reads alloc (AcctState), **non-det branch on DELEGATE key**, pending       |
| test_process_initialize_account3        | 2070 | Stuck   | 628       | branching on thunked ptr cast, "ExposeAddress", `assert_inhab`, vacuous       |
| test_process_initialize_mint2_freeze    | 2120 | Stuck   | 275       | branching on thunked ptr cast, "ExposeAddress", `assert_inhab`, vacuous       |
| test_process_initialize_mint2_no_freeze | 2150 | Stuck   | 275       | branching on thunked ptr cast, "ExposeAddress", `assert_inhab`, vacuous       |
| test_process_revoke                     | 970  | Stuck   | 1124      | call to `unwrap_failed`, reads alloc                                          |
| test_process_freeze_account             | 2480 | Stuck   | 1888      | reads alloc(AcctState)                                                        |
| test_process_thaw_account               | 2460 | Stuck   | 1888      | reads alloc(AcctState)                                                        |
| test_process_mint_to_checked            | 680  | Timeout | 845       | thunking pointer offsets. Decoding account state enums                        |
| test_process_sync_native                | 2480 | Stuck   | 2776      | reads allocs (AcctState, 2x PrgErr)                                           |
| test_process_get_account_data_size      | 1270 | Stuck   | 1928      | reads allocs (2x PrgResult), _terminates on 1 branch_                         |
| test_process_initialize_immutable_owner | 840  | Stuck   | 1425      | reads alloc(AcctState)                                                        |
| test_process_amount_to_ui_amount        | 2580 | Stuck   | 2430      | reads allocs(2x PrgResult), stuck on (trivial) ptr offset                     |
| test_process_ui_amount_to_amount        | 580  | Stuck   | 90        | call to `core::str::convert::from_utf8` (stdlib)                              |

Cheat codes are missing or a problem for these proofs, therefore not recommended to execute them
(keep the empty first column so `run-proofs.sh` won't pick these up):

|   | test_process_initialize_multisig  |
|   | test_process_initialize_multisig2 |
|   | test_process_set_authority        |
