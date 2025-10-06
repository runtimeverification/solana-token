Proofs timings and steps with default settings from run-proofs.sh:
max-depth 500, max-iterations 100, timeout 2h min (7200), iterated on timeout

| Start symbol name                       | Min  | Status           | Comments                                                          |
|-----------------------------------------|------|------------------|-------------------------------------------------------------------|
| test_process_approve                    | 41   | 2x stuck         | 2x stuck decoding `Result<(), ProgramError>`                      |
| test_process_approve_checked            | 50   | 4x stuck         | 4x stuck on binOpOffset (non-trivial, for slice)                  |
| test_process_withdraw_excess_lamports   | 140  | 4x stuck 2x vac  | non-det branches (stuck) on `Rent::get()`/`ptrExposeAddress`      |
| test_process_initialize_mint_freeze     | 70   | 7x stuck         | 3x overflow, 4x `binOpOffset` (non-trivial)                       |
| test_process_initialize_mint_no_freeze  | 77   | 7x stuck         | 3x overflow, 4x `binOpOffset` (non-trivial) + `*[u8] -> [u8;32]`  |
| test_process_initialize_account         | 600+ | CONTINUE         | non-det branches                                                  |
| test_process_initialize_account2        | 360+ | CONTINUE         |                                                                   |
| test_process_transfer                   | 67   | 8x stuck         | 8x stuck decoding `Result<(), ProgramError>`                      |
| test_process_mint_to                    | 80   | 4x stuck         | 4x stuck decoding `Result<(), ProgramError>`                      |
| test_process_burn                       | 320  | 16x stuck        | 16x stuck decoding `Result<(), ProgramError>`                     |
| test_process_close_account              | 80   | CRASH            | Server out of memory                                              |
| test_process_transfer_checked           | 175  | 16x stuck        | 16x stuck on binOpOffset (non-trivial) for raw ptr                |
| test_process_burn_checked               | 175  | 16x stuck        | 16x stuck on binOpOffset (non-trivial) for raw ptr                |
| test_process_initialize_account3        | 82   | 4x stuck 2x vac  | non-det branches (stuck) on Rent::get()/ptrExposeAddress          |
| test_process_initialize_mint2_freeze    | 15   | CRASH            | Server out of memory                                              |
| test_process_initialize_mint2_no_freeze | 15   | CRASH            | Server out of memory                                              |
| test_process_revoke                     | 36   | 2x stuck         | 2x stuck decoding `Result<(), ProgramError>`                      |
| test_process_freeze_account             | 150  | 8x stuck         | 8x stuck decoding `Result<(), ProgramError>`                      |
| test_process_thaw_account               | 118  | 8x stuck         | 8x stuck decoding `Result<(), ProgramError>`                      |
| test_process_mint_to_checked            | 62   | 4x stuck         | 4x stuck on `binOpOffset` (non-trivial) for raw ptr               |
| test_process_sync_native                | 112  | 8x stuck         | 4x stuck decoding `Result<(), ProgramError>`                      |
| test_process_get_account_data_size      | 51   | 3x stuck         | 3x stuck decoding `Result<(), ProgramError>`                      |
| test_process_initialize_immutable_owner | 25   | 1x stuck 1x term | stuck decoding `Result<(), ProgramError>`                         |
| test_process_amount_to_ui_amount        | 100  | 6x stuck         | 3x decoding `Result`, `assert_inhabited`, `binOpOffset`, ptr cast |
| test_process_ui_amount_to_amount        | 7    | stuck            | calls `core::str::convert::from_utf8` (stdlib)                    |

Cheat codes are missing or a problem for these proofs, therefore not recommended to execute them
(keep the empty first column so `run-proofs.sh` won't pick these up):

|   | test_process_initialize_multisig  |
|   | test_process_initialize_multisig2 |
|   | test_process_set_authority        |
