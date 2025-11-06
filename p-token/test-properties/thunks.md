# Thunks

This file documents known thunks in the proofs and ties them to where they appear in proofs.

Here's the summary table. To get the full thunk go to the references

| Details | Thunks                                                                   | Problematic |
|---------|--------------------------------------------------------------------------|-------------|
| [1](#1) | `#cast ( Range ( ListItem (Integer ( ARG_UINT214:Int , 8 , false ))...`  | ?           |
| [2](#2) | `operandConstant ( constOperand ( ... span: span ( 600186 ) , ...`       | ?           |
| [3](#3) | `#cast ( Integer ( ?STATE:Int , 8 , false ) , castKindTransmute , ... `  | No          |
| [4](#4) | `#cast ( Float ( 0.20000000000000000e1 , 64 ) , castKindTransmute , ...` | ?           |

This table keeps track of the remaining thuk-types in the proofs.

| Checked Proofs                                | Remaining Thunks (kind-wise) |
|-----------------------------------------------|------------------------------|
| test_ptoken_domain_data                       | 0                            |
| test_process_approve                          | 3                            |
| test_process_approve_checked                  | 3                            |
| test_process_withdraw_excess_lamports_account | 3                            |

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

### Appeareances

Appears on:
- `test_process_approve`
- `test_process_approve_checked`

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

### Appeareances

Appears on:
- `test_process_approve`
- `test_process_approve_checked`
- `test_process_withdraw_excess_lamports_account`

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

### Appeareances

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

### Appeareances

- `test_process_withdraw_excess_lamports_account`
