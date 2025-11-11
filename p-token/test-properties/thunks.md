# Thunks

This file documents known thunks in the proofs and ties them to where they appear in proofs.

Here's the summary table. To get the full thunk go to the references

| Details | Thunks                                                                             | Problematic | Summary                   |
|---------|------------------------------------------------------------------------------------|-------------|---------------------------|
| [1](#1) | `#cast ( Range ( ListItem (Integer ( ARG_UINT214:Int , 8 , false ))...`            | **Solved**  | `[u8;8]` -> `U64`         |
| [2](#2) | `operandConstant ( constOperand ( ... span: span ( 600186 ) , ...`                 | ?           | (?) Closure type          |
| [3](#3) | `#cast ( Integer ( ?STATE:Int , 8 , false ) , castKindTransmute , ... `            | No          | `u8` -> `AccountState`    |
| [4](#4) | `#cast ( Float ( 0.20000000000000000e1 , 64 ) , castKindTransmute , ...`           | ?           | `F64` -> `U64`            |
| [5](#5) | `operandMove ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ) ` | ?           | ??                        |
| [6](#6) | `#cast ( Reference ( 0 , place ( ... local: local ( 2 ) , ... ) ...`               | ?           | `castKindPointerCoercion` |
| [7](#7) | `UnableToDecode ( b"\x00" , typeInfoUnionType ( "core::mem::MaybeUninit<u8>" ...`  | ?           | ??                        |

This table keeps track of the remaining thuk-types in the proofs.

| Checked Proofs                                | Remaining Thunks (kind-wise) | Present Thunks                          |
|-----------------------------------------------|------------------------------|-----------------------------------------|
| test_ptoken_domain_data                       | 0                            |                                         |
| test_process_initialize_account               | 3                            | [2](#2) [3](#3) [4](#4)                 |
| test_process_initialize_account2              | 3                            | [2](#2) [3](#3) [4](#4)                 |
| test_process_get_account_data_size            | 1                            | [2](#2)                                 |
| test_process_initialize_immutable_owner       | 2                            | [2](#2) [3](#3)                         |
| test_process_sync_native                      | 2                            | [2](#2) [3](#3)                         |
| test_process_approve_checked                  | 4                            | [2](#2) [3](#3) [5](#5) [6](#6)         |
| test_process_approve                          | 4                            | [2](#2) [3](#3) [5](#5) [6](#6)         |
| test_process_freeze_account                   | 4                            | [2](#2) [3](#3) [5](#5) [6](#6)         |
| test_process_initialize_account3              | 3                            | [2](#2) [3](#3) [4](#4) [6](#6)         |
| test_process_initialize_mint_freeze           | 2                            | [4](#4) [6](#6)                         |
| test_process_initialize_mint2_freeze          | 2                            | [4](#4) [6](#6)                         |
| test_process_initialize_mint_no_freeze        | 2                            | [4](#4) [6](#6)                         |
| test_process_initialize_mint2_no_freeze       | 2                            | [4](#4) [6](#6)                         |
| test_process_mint_to                          | 4                            | [2](#2) [3](#3) [5](#5) [6](#6)         |
| test_process_mint_to_checked                  | 4                            | [2](#2) [3](#3) [5](#5) [6](#6)         |
| test_process_revoke                           | 4                            | [2](#2) [3](#3) [5](#5) [6](#6)         |
| test_process_thaw_account                     | 4                            | [2](#2) [3](#3) [5](#5) [6](#6)         |
| test_process_withdraw_excess_lamports_account | 5                            | [2](#2) [3](#3) [4](#4) [5](#5) [6](#6) |
| test_process_withdraw_excess_lamports_mint    | 4                            | [2](#2) [4](#4) [5](#5) [6](#6)         |
| test_process_burn                             | 2                            | [2](#2) [3](#3)                         |
| test_process_burn_checked                     | 2                            | [2](#2) [3](#3)                         |
| test_process_close_account                    | 3                            | [2](#2) [3](#3) [6](#6)                 |
| test_process_set_authority_account            | 3                            | [2](#2) [3](#3) [6](#6)                 |
| test_process_set_authority_mint               | 4                            | [2](#2) [3](#3) [5](#5)                 |
| test_process_transfer_checked                 | 2                            | [2](#2) [3](#3)                         |
| test_process_transfer                         | 2                            | [2](#2) [3](#3)                         |
| test_process_amount_to_ui_amount              | 2                            | [2](#2) [7](#7)                         |
| test_process_ui_amount_to_amount              | stuck                        | stuck                                   |



## 1

### Full thunk:

```
thunk ( #cast ( Range ( ListItem (Integer ( ARG_UINT214:Int , 8 , false ))
                        ListItem (Integer ( ARG_UINT215:Int , 8 , false ))
                        ListItem (Integer ( ARG_UINT216:Int , 8 , false ))
                        ListItem (Integer ( ARG_UINT217:Int , 8 , false ))
                        ListItem (Integer ( ARG_UINT218:Int , 8 , false ))
                        ListItem (Integer ( ARG_UINT219:Int , 8 , false ))
                        ListItem (Integer ( ARG_UINT220:Int , 8 , false ))
                        ListItem (Integer ( ARG_UINT221:Int , 8 , false ))
                        ) , castKindTransmute , ty ( 600721 ) , ty ( 600142 ) ) )
```

### Explanation:

Cast between an array of 8 `u8`s (`ty ( 600721 )`) and a `U64` (`ty ( 600142 )`).

### Safety

??

### Appearances

Appears on:
- `test_process_approve_checked`
- `test_process_transfer`

## 2

### Full thunk

```
 thunk ( operandConstant ( constOperand ( ... span: span ( 600186 ) , userTy: noUserTypeAnnotationIndex ,
    const: mirConst ( ... kind: constantKindZeroSized , ty: ty ( 600113 ) , id: mirConstId ( 375 ) ) ) ) )
```

### Explanation:

??

### Safety

??

### Appearances

Appears on:
- `test_process_approve`
- `test_process_approve_checked`
- `test_process_withdraw_excess_lamports_account`
- `test_process_transfer`

## 3

### Full thunk

```
thunk ( #cast ( Integer ( ?STATE:Int , 8 , false ) , castKindTransmute , ty ( 600077 ) , ty ( 600110 ) ) )
```

### Explanation:

The thunked cast is a transmute between `u8` (`ty ( 600077 )`) and the enum `AccountState` (`ty ( 600110 )`),
which has three variants: `Uninitialized`, `Initialized` and `Frozen`.

### Safety:

The rule that seems to make this work is the following:

```
  rule <k> #discriminant(
              thunk(#cast (Integer(DATA, _, false), castKindTransmute, _, TY)),
              TY
            ) => Integer(DATA, 0, false) // HACK: bit width 0 means "flexible"
          ...
        </k>
    requires #isEnumWithoutFields(lookupTy(TY))
```

The rule is not taking into account the width of the transmuted integer.

### Appearances

Appears on:
- `test_process_approve`
- `test_process_approve_checked`
- `test_process_withdraw_excess_lamports_account`

## 4

### Full Thunk

```
thunk ( #cast ( Float ( 0.20000000000000000e1 , 64 ) , castKindTransmute , ty ( 601188 ) , ty ( 600142 ) ) )
```

### Explanation

Transmute cast between an `F64` (`ty ( 601188 )`) and a `U64` (`ty ( 600142 )`).

### Safety

???

### Appearances

- `test_process_withdraw_excess_lamports_account`

## 5

### Full Thunk

```
thunk ( operandMove ( place ( ... local: local ( 4 ) , projection: .ProjectionElems ) ) )
```
Where `<locals>` is
```
<locals>
    ListItem (newLocal ( ty ( 600094 ) , mutabilityMut ))

    ListItem (typedValue ( Reference ( 1 , place ( ... local: local ( 27 ) , projection: .ProjectionElems ) , mutabilityNot , metadata ( noMetadataSize , 0 , noMetadataSize ) ) , ty ( 600019 ) , mutabilityNot ))

    ListItem (typedValue ( Moved , ty ( 600054 ) , mutabilityMut ))

    ListItem (typedValue ( Moved , ty ( 600165 ) , mutabilityMut ))

    ListItem (newLocal ( ty ( 600028 ) , mutabilityMut ))

    ListItem (newLocal ( ty ( 600107 ) , mutabilityNot ))

    ListItem (newLocal ( ty ( 600088 ) , mutabilityMut ))
</locals>
```

### Explanation

Source:
```
function: <core::result::Result<(), pinocchio::program_error::ProgramError> as core::clone::Clone>::clone
span: ust/library/core/src/result.rs:1727
```

### Safety

???

### Appearances

- `test_process_approve`

## 6

### Full Thunk

```
thunk ( #cast ( Reference ( 0 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , metadata ( noMetadataSize , 0 , noMetadataSize ) ) , castKindPointerCoercion ( pointerCoercionUnsize ) , ty ( 600020 ) , ty ( 600001 ) ) )
```
Where `<locals>` is
```
<locals>
    ListItem (newLocal ( ty ( 600002 ) , mutabilityMut ))

    ListItem (typedValue ( Aggregate ( variantIdx ( 0 ) , .List ) , ty ( 600003 ) , mutabilityNot ))

    ListItem (typedValue ( Reference ( 1 , place ( ... local: local ( 5 ) , projection: .ProjectionElems ) , mutabilityNot , metadata ( noMetadataSize , 0 , noMetadataSize ) ) , ty ( 600019 ) , mutabilityNot ))

    ListItem (typedValue ( AllocRef ( allocId ( 600101 ) , .ProjectionElems , metadata ( noMetadataSize , 0 , noMetadataSize ) ) , ty ( 600019 ) , mutabilityNot ))

    ListItem (typedValue ( Aggregate ( variantIdx ( 0 ) , .List ) , ty ( 600005 ) , mutabilityNot ))

    ListItem (newLocal ( ty ( 600001 ) , mutabilityMut ))

    ListItem (typedValue ( Reference ( 0 , place ( ... local: local ( 2 ) , projection: .ProjectionElems ) , mutabilityNot , metadata ( noMetadataSize , 0 , noMetadataSize ) ) , ty ( 600020 ) , mutabilityNot ))

    ListItem (newLocal ( ty ( 600001 ) , mutabilityMut ))

    ListItem (newLocal ( ty ( 600020 ) , mutabilityNot ))
</locals>
```

### Explanation

Source:
```
function: core::panicking::assert_failed::<core::result::Result<(), pinocchio::program_error::ProgramError>, core::result::Result<(), pinocchio::program_error::ProgramError>>
span: /library/core/src/panicking.rs:373
```

### Severity

???

## 7

### Full Thunk

```
thunk ( UnableToDecode ( b"\x00" , typeInfoUnionType ( "core::mem::MaybeUninit<u8>" , adtDef ( 600149 ) ) ) )
```

### Explanation

???

### Severity

???

