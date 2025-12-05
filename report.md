# Solana SPL Token and P-Token Program Equivalence Proof

In this project, our goal is to demonstrate that the SPL Token and P-Token programs are _equivalent_.
More formally, we will verify that the P-Token program can _inductively simulate_ the SPL-Token program, which, roughly speaking, states that for each SPL Token program state and instruction, there is an equivalent P-Token program state and instruction that does the exact same thing.

In the next section, we cover some technical prelminaries on the Solana blockchain itself and our proof methodology for the interested reader; everyone else should skip to the section (**TODO:** fill section name).

## Technical Preliminaries

### Solana Programming Model

The Solana blockchain supports on-chain programmability via Solana _programs_, _accounts_, _messages_, and _instructions_.
Roughly speaking:

1.  A Solana _account_ is an addressable unit of mutable, on-chain data with the following fields:

    | Field Name | Description                                                    |
    | ---        | ---                                                            |
    | owner      | ID of program that owns this account                           |
    | lamports   | monetary balance of the account denominated in lamports        |
    | data       | bytes stored in account                                        |
    | exectuable | boolean flag that indicates whether this account is executable |
    | rent_epoch | the epoch when this account will next owe rent (deprecated)    |

    whose address is called its _key_.
    We also use the term _format_ when describing the particular data payloads that a program might store in one of its owned accounts.

2. A Solana _program_ is an account whose associated data is immutable and contains _executable code_[^program] and whose key is referred to as a _program ID_;
3. A Solana _message_ is a list of accounts (which are marked as either mutable, a signer, both, or neither) and a list of instructions;
4. A Solana _instruction_ is a program ID, a list of indices of message accounts, and an arbitrary, length-prefixed data payload that we refer to as the instruction's _format_.

Note that there are some special Solana programs called _native programs_ which have extra priviliges above and beyond those of standard (non-native) programs; we will have more remarks to make about that later.

[^program]: Technically, in order to support upgradability, the associated data of user-defined program accounts is actually just the address of some account that contains its executable data

The executable code of a Solana program must satisfy a few requirements:

1.  It must be encoded an [ELF](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format) binary file that contains a Solana-specific variant of the [extended Berkeley Packet Filter (eBPF)](https://en.wikipedia.org/wiki/EBPF) virtual machine bytecode called _sBPF_ or sometimes just _SBF_.
2.  The ELF file must contain a function named `entrypoint`.
3.  The `entrypoint` function must have a single input argument which is a pointer to a byte array and return an unsigned 64-bit number as a status code (where 0 indicates success and any other number indicates failure).
3.  The input byte array must encode:

    1. the fields of all accounts passed to the program as well as their mutable/signer status and;
    2. the instruction format (i.e., an arbitrary length byte string);
    3. the ID of the program being invoked.

    using the encoding documented [here](https://github.com/solana-foundation/solana-com/blob/a8cee23bfc4d96326b8172a60539c8e7dea47ab3/content%2Fdocs%2Fen%2Fprograms%2Ffaq.mdx).

    Note that any passed in account may be modified by directly overwriting its serialized value in the input byte array; once execution completes, if the result was successful, the Solana validator runtime will commit the updated account state back to the account database.

Furthermore, the execution of each program instruction must satisfy a few invariants:

- the lamport balance of all accounts referenced in an instruction must be invariant
- changing the `owner` field of an account requires that the account's `data` is zeroed out
- accounts whose `executable` field is set to `true` cannot have their `data` or `owner` field mutated
- only owner program of some account _A_ can:
  1. mutate _A_'s `data` field
  2. mutate _A_'s `owner` field
  3. decrement _A_'s `lamports` field
  4. set _A_'s `executable` flag
- only native programs can set an account's `executable` or `rent_epoch` fields

<!--
  -- * only built-in programs can change the `executable` or `rent_epoch` fields because:
  --   * the BPF interpreter deserialization logic does not commit updates to other fields
  --   * an `InvokeContext` handle is required to commit account info changes, but this context is not passed to user-defined programs
  -- * the runtime checks that account modifications performed by user-defined programs are permissible are performed once BPF interpreter execution completes; these checks are performed in two places:
  --   1. in `program_runtime::serialization::deserialize_parameters`, the `BorrowedInstructionAccount` setters `set_lamports`, `set_owner`, `set_data`
  --   2. in `svm::transaction_processor::TransactionBatchProcessor::execute_loaded_transaction`, the `transaction_accounts_lamports_sum` check
  -->

### Program Equivalence Primer

Abstractly, a program _P_ can be defined as by a set of _P-states_, an _initial state_, and a set of _P-transitions_ where:

1.  An individual _P-state_ is a snapshot of the program's world in a particular moment in time; it fully describes what the program is currently doing as well as what it will do next;
2.  P's _initial state_ is the first possible _P_-state or, in other words, the state from which _P_ execution begins;
2.  An individual _P-transition_ is a link between two program states; transitions describe the _effect_ of some particular operation on a _P-state_.

Formally, we call this pair of _states_ and _transitions_ (linked to operations) a _labeled transition system_ where each transition's _label_ is just the operation $op$ that effected the transition. Given states _a_ and _b_ and a label $o$, we denote a transition graphically by writing $a \overset{op}{\rightarrow} b$ and we say that:

1. _a_ is a predcessor of _b_;
2. _b_ is a successor of _a_.

We also define the set of _reachable_ P-states as the smallest set of P-states that:

   1. contains P's initial state and;
   2. contains all successor P-states of any reachable P-state.

Within this framework, we can discuss what we mean when we say that some program _P_ _inductively simulates_ another program _Q_. To begin, since programs _P_ and _Q_ are different, they will have different kinds of states[^programlabels]. So we need a notion of _state equivalence_, or a way to associate _P_-states to _Q_-states that mean the same thing. Graphically, we denote equivalences with a triple-bar symbol $(\equiv)$.  We can now present our key definition:

A program _P_ _inductively simulates_ a program _Q_ if and only if,

1. _P_ and _Q_ have equivalent initial states;
2. for each reachable _Q_-state _a_ and _Q_-transition $a\overset{op}{\rightarrow} a'$, there exists reachable _P_-states _b_ and _b'_ such that $a \equiv b$, $a' \equiv b'$, and $b\overset{op}{\rightarrow} b'$ is a valid _P_-transition.

[^programlabels]: Of course, programs _P_ and _Q_ will also have different kinds of operations, but since we assume that _P_ is designed as a re-implementation of _Q_, we require that if _Q_ has some operation _op_, then _P_ also has operation _op_ (though _P_ may have unique operations not contained in _Q_).

### Formal Verification Methodology

To verify that the P-Token program inductively simulates the SPL Token program, we follow a particular formal verification methodology which roughly breaks down into two parts:

1. we apply our semantics-based symbolic execution engine to obtain a complete set of transitions over reachable states, and
2. we show that if the SPL Token program can perform some transition, then the P-Token program can perform an equivalent transition.

The entire process is depicted in the Figure 1 below:

```
┏━━━━━━━━━━━━━━━━━━━━━┓                 ┏━━━━━━━━━━━━━━━━━━━┓                
┃ SPL Token Rust Code ┃                 ┃ P-Token Rust Code ┃
┗━━━━━━━━━━━━━━━━━━━━━┛                 ┗━━━━━━━━━━━━━━━━━━━┛
           🡻                                     🡻
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃                     KMIR Rust Compiler                    ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
           🡻                                     🡻
┏━━━━━━━━━━━━━━━━━━━━━┓                 ┏━━━━━━━━━━━━━━━━━━━┓                
┃ SPL Token MIR Code  ┃                 ┃ P-Token MIR Code  ┃
┗━━━━━━━━━━━━━━━━━━━━━┛                 ┗━━━━━━━━━━━━━━━━━━━┛
           🡻                                     🡻
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓ 
┃                       MIR Semantics                       ┃ 
┃      ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓      ┃ 
┃      ┃                  K Prover                   ┃      ┃
┃      ┃       Symbolic Execution Proof Engine       ┃      ┃ 
┃      ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛      ┃ 
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ 
           🡻                                     🡻
┏━━━━━━━━━━━━━━━━━━━━━┓                 ┏━━━━━━━━━━━━━━━━━━━┓                
┃   SPL Token State   ┃       ??        ┃   P-Token State   ┃
┃     Transition      ┃   ━━━━━━━━━🡺   ┃    Transition     ┃
┗━━━━━━━━━━━━━━━━━━━━━┛                 ┗━━━━━━━━━━━━━━━━━━━┛

                           Figure 1.
                 Formal Verification Methodology
```

We describe each of the key components below:

1. KMIR Rust Compiler - a modified version of the Rust compiler that emits a simplified version of Rust code referred to as the Rust compiler's Mid-level Intermediate Representation (MIR) for use with the K symbolic proof engine; compared to standard Rust code, in MIR code:

   - all references to module items are fully qualified;
   - all generic items are fully [monomorphized](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html);
   - static and constant module items are assigned to literal byte strings;
   - all variable `drop`s at end-of-scope are made explicit;
   - all local variable names are replaced with numeric identifiers;
   - implicit borrows are inserted;
   - implicit closure structures are generated
   - etc...

   The reason why we use MIR instead of Rust source code is its _simplicity_ mentioned above;
   using MIR lets us fully capture the behavior of Rust programs without worrying about compleixities like:

   - trait instance validation
   - borrow checking
   - type layout and alignment

2. MIR Semantics - a set of rules that defines how well-formed MIR programs behave.
3. K Prover - a symbolic execution-based prover that uses symbolic states and co-induction to reason about (a potentially infinite number of) program states and transitions in a finite amount of time[^limitations].

Finally, we compare state transitions as follows:

1. We partition the complete set of possible instruction formats into a finite (and small!) set $\{instr_i\}_i$ using symbolic constraints.
3. For each instruction format instance $instr_i$, we partition the complete set of potential account format sequences that it can accept $\alpha_{(i,j)}=\{acct_{(i,j,0)},\cdots,acct_{(i,j,\#(i,j))}\}_{(i,j)}$ --- also using symbolic constraints where $\#(i,j)$ denotes the arity of the account format sequenced indexed by $(i,j)$.
3. For each combination of an instruction format $instr_i$ and corresponding account sequence $\alpha_{(i,j)}$, we symbolically execute the SPL Token program and record its return code $ret_{(i,j)}$ and updated account set $\alpha'_{(i,j)}$.
<!-- 3.(b) We check that, if $ret_{(i,j)}$ is 0, then $\alpha'_{(i,j)}$ is a well-formed account format sequence. -->
4. Finally, we repeat step (3) but with P-Token instead of SPL Token and verify that the returned status code and updated account set are equal to what the SPL Token produced.

Combined together, steps (1)-(4) prove that the P-Token program inductively simulates the SPL Token program. <!-- Note that step (4) is essential to prove that the simulation holds inductively, since otherwise, our partitioning of the complete set of account format sequences would be incomplete (i.e., we would need more cases to describe the complete set of account format sequences) -->

[^limitations]: Note that there is no free lunch and symbolic execution is not a panacea that can be trivially used to solve all verification problems. In certain cases, we must provide the execution engine
with lemmas (in other words, hints) that help the execution engine make progress when it gets stuck.

## Introduction 

The Solana Programming Library (SPL) is a set of Solana-specific programs and libraries, written in the Rust programming language, that can be used to create custom dApps for Solana. An important component of the SPL toolkit is the SPL Token Program, a program that enables the creation and usage of custom fungible and non-fungible tokens on the Solana blockchain. 

As the name indicates, the SPL Token Program contains the source code for a deployable program on Solana.
Like other kinds of Solana programs, the SPL Token program has:

1. a handful of potential data formats that can be stored in any account owned by the Token program;
2. a handful of potential instruction formats.

We will describe these formats more fully below.

As usage of the SPL Token Program increased, eventually, a compute-optimized version of the SPL Token Program was proposed in Solana Improvement Document (SIMD-0266) by febo and Jon Cinque. The idea: a program with byte-for-byte equivalent account and instruction formats that operates more efficiently, using less validator time, and thus permitting more instructions to be validated in a single slot.

### SPL Token Account Format

1. Mint
2. Account
3. Multisig

In the two versions of the program, we have different ways of parsing these forms of account data.
In the original spl token implementation, there are explicit parsing functions.
In the p-token implementation, byte strings are transmuted into Rust structs.
In order to make this work, the transmutable p-token Rust structs:

1. use values that always have 1 byte alignment;
2. must ensure that borrows against the transmuted Rust structs are unique.

### SPL Token Instruction Formats

| Universal Failure Cases |
| ---                     |
| Input Too Short         |
| Not Enough Accounts     |

| Enum Tag | Enum Variant             | Associated Data                                           | Comment                                                   | Success Case |
| ---      | ---                      | ---                                                       | ---                                                       | ---          |
|   0      | InitializeMint           | u8, Pubkey:MintAuthority, COption<Pubkey>:FreezeAuthority |                                                           | 2            |
|   1      | InitializeAccount        |                                                           |                                                           | 1            |
|   2      | InitializeMultisig       | u8                                                        |                                                           | 1            |
|   3      | Transfer                 | u64                                                       |                                                           | 1            |
|   4      | Approve                  | u64                                                       |                                                           | 1            |
|   5      | Revoke                   |                                                           |                                                           | 1            |
|   6      | SetAuthority             | AuthorityType, COption<Pubkey>:NewAuthority               | AuthorityType == AccountOwner => is_defined(NewAuthority) | 7            |
|   7      | MintTo                   | u64                                                       |                                                           | 1            |
|   8      | Burn                     | u64                                                       |                                                           | 1            |
|   9      | CloseAccount             |                                                           |                                                           | 1            |
|  10      | FreezeAccount            |                                                           |                                                           | 1            |
|  11      | ThawAccount              |                                                           |                                                           | 1            |
|  12      | TransferChecked          | u64, u8                                                   |                                                           | 1            |
|  13      | ApproveChecked           | u64, u8                                                   |                                                           | 1            |
|  14      | MintToChecked            | u64, u8                                                   |                                                           | 1            |
|  15      | BurnChecked              | u64, u8                                                   |                                                           | 1            |
|  16      | InitializeAccount2       | Pubkey:Owner                                              |                                                           | 1            |
|  17      | SyncNative               |                                                           |                                                           | 1            |
|  18      | InitializeAccount3       | Pubkey:Owner                                              |                                                           | 1            |
|  19      | InitializeMultisig2      | u8                                                        |                                                           | 1            |
|  20      | InitializeMint2          | u8, Pubkey:MintAuthority, COption<Pubkey>:FreezeAuthority |                                                           | 2            |
|  21      | GetAccountDataSize       |                                                           |                                                           | 1            |
|  22      | InitializeImmutableOwner |                                                           |                                                           | 1            |
|  23      | AmountToUiAmount         | u64                                                       |                                                           | 1            |
|  24      | UiAmountToAmount         | &str                                                      |                                                           | 1            |
|  38      | WithdrawExcessLamports   |                                                           |                                                           | 1            |
| 255      | Batch                    | (accts:u8,len:u8,len)*                                    |                                                           | N            |

| Enum Tag | AuthorityType Variant |
| ---      | ---                   |
| 0        | MintTokens            |
| 1        | FreezeAccount         |
| 2        | AccountOwner          |
| 3        | CloseAccount          |

## Proof Body

Here, we provide an overview of the proof process in English by enumerating all of the instruction and account format sequences and their associated return codes and updated account sequences.

- Transfer
  - Account1, Account2, Account3, ...

    Result is X.

    Updated accounts are ...

  - Account1, Account2, Account3, ...

    Result is Y.

    Updated accounts are ...

(**TODO:** List all proof cases.)

(**TODO:** Improve presentation of instruction and account format data.
           A current proposal is each kind of symbolic input constraint and assign it a short English name.
           Then input cases can be classed by the set of constraints that hold.
           This will basically re-iterate what the proof specs say but in English.
)

<!--

The cases below are important cases to focus on:

- Transfer
- TransferChecked
- Burn
- BurnChecked
- MintTo
- MintToChecked

-->

## Conclusion

**TODO:** Write conclusion.