# Solana SPL Token and P-Token Program Equivalence Proof

In this project, our goal is to demonstrate that the SPL Token and P-Token programs are _equivalent_.
More formally, we will verify that the P-Token program can _simulate_ the SPL-Token program, which, roughly speaking, states that for each SPL Token program state and instruction, there is an equivalent P-Token program state and instruction that does the exact same thing.

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
- only the owner program of some account _A_ can:
  1. decrement _A_'s `lamports` field
  2. mutate any field of account _A_ that is not `lamports`
- only native programs can mutate an account's `executable` or `rent_epoch` fields

<!--
  -- * only built-in programs can change the `executable` or `rent_epoch` fields because:
  --   * the BPF interpreter deserialization logic does not commit updates to other fields
  --   * an `InvokeContext` handle is required to commit account info changes, but this context is not passed to user-defined programs
  -- * the runtime checks that account modifications performed by user-defined programs are permissible are performed once BPF interpreter execution completes; these checks are performed in two places:
  --   1. in `program_runtime::serialization::deserialize_parameters`, the `BorrowedInstructionAccount` setters `set_lamports`, `set_owner`, `set_data`
  --   2. in `svm::transaction_processor::TransactionBatchProcessor::execute_loaded_transaction`, the `transaction_accounts_lamports_sum` check
  -->

### Program Equivalence Primer

Abstractly, a program _P_ can be defined as by a set of _P-states_, an _initial P-state_, a set of operation/input labels $\Sigma$, and a set of _P-transitions_[^trans-system] where:

[^trans-system]: In general, a tuple containing a set of _states_, a set of _labels_, and a set of _labeled transitions_, is called a _labeled transition system_ in the computer science literature.

1.  An individual _P-state_ is a snapshot of the program's world in a particular moment in time; it fully describes what the program is currently doing as well as what it will do next;
2.  The _initial P-state_, denoted $initial(P)$, is the designated state from which program execution begins;
3.  Given two P-states _a_ and _b_ and a label $\sigma$, we define a _P-transition_ (denoted $a \overset{\sigma}{\rightarrow} b$) as a rule which permits program _P_, upon receiving an input or performing an operation $\sigma\in\Sigma$, to transition from _P_-state _a_ to _b_. In case such a transition exists, we say that _a_ is a predcessor of _b_ or _b_ is a successor of _a_.

It is sometimes useful to talk about properties that describe a particular set of _P_-states; formally, we call such a property a _predicate_.
Graphically, we denote membership of a state _a_ in a predicate $\Phi$ by writing $a\in\Phi$.
Given two predicates $\Phi$ and $\Psi$, we write $\Phi \subseteq \Psi$ to mean every state in $\Phi$ is also contained in $\Psi$.

One important predicate is $reach(P)$, the set of _reachable_ _P_-states, defined as the smallest set of _P_-states that:

   1. contains P's initial state and;
   2. contains all successor P-states of any reachable P-state.

Given some predicate $\Iota$, whenever $reach(P) \subseteq \Iota$, we say that $\Iota$ is an _invariant_.
Furthermore, if we have a predicate $\Iota$ that:

1. holds for the intial P-state;
2. for each pair of reachable P-states _a_ and _b_ and transition $a \overset{\sigma}{\rightarrow} b$, assuming $a\in\Iota$, we can prove that $b\in\Iota$;

then we say that $\Iota$ is an _inductive invariant_.

With this framework, we can now discuss what we mean when we say that some program _P_ _simulates_ another program _Q_.
To begin, since programs _P_ and _Q_ are different, they will have different kinds of states[^programlabels].
So we need a notion of _state equivalence_, or a way to associate _P_-states to _Q_-states that mean the same thing.
Graphically, we denote equivalences with a triple-bar symbol $(\equiv)$.
We can now present our key definition:

A program _P_ _simulates_ a program _Q_ if and only if,

1. $initial(P) \equiv initial(Q)$
2. for each pair of states $a,a'\in reach(Q)$ and _Q_-transition $a\overset{\sigma}{\rightarrow} a'$, there exists _P_-states _b_ and _b'_ such that:
   - $a \equiv b$
   - $a' \equiv b'$
   - $b\overset{\sigma}{\rightarrow} b'$ is a valid _P_-transition

[^programlabels]: Of course, programs _P_ and _Q_ will also have different kinds of operations, but since we assume that _P_ is designed as a re-implementation of _Q_, we require that if _Q_ has some operation _op_, then _P_ also has operation _op_ (though _P_ may have unique operations not contained in _Q_).

Sometimes, we may not want to consider the entire set of possible operations/inputs for a program $P$, but only a subset of them.
Given a program _P_ with operations/inputs $\Sigma$, define the _restriction_ of _P_ to $\Sigma_0\subseteq \Sigma$ (denoted $P|_{\Sigma_0}$) as a program with the same states and initial state as $P$, the set of operations/inputs $\Sigma_0$, where $P|_{\Sigma_0}$ contains the $P$-transition $a\overset{\sigma}{\rightarrow}b$ whenever $\sigma\in\Sigma_0$.

### Formal Verification Methodology

To verify that the P-Token program simulates the SPL Token program, we follow a particular formal verification methodology which breaks down the equivalence into a few parts.
We survey our overall methodology in Figure 1 below:

```
┏━━━━━━━━━━━━━━━━━━━━━┓               ┏━━━━━━━━━━━━━━━━━━━━━┓
┃ SPL Token Rust Code ┃               ┃  P-Token Rust Code  ┃
┗━━━━━━━━━━━━━━━━━━━━━┛               ┗━━━━━━━━━━━━━━━━━━━━━┛     Rust-to-K-MIR Compilation
-----------🡻-------------------------------------🡻---------------------------------------
┏━━━━━━━━━━━━━━━━━━━━━┓               ┏━━━━━━━━━━━━━━━━━━━━━┓
┃ SPL Token MIR Code  ┃               ┃   P-Token MIR Code  ┃       MIR Program Interpreter
┗━━━━━━━━━━━━━━━━━━━━━┛               ┗━━━━━━━━━━━━━━━━━━━━━┛    Generation via K Framework
-----------🡻-------------------------------------🡻---------------------------------------
┏━━━━━━━━━━━━━━━━━━━━━━┓              ┏━━━━━━━━━━━━━━━━━━━━━┓
┃  SPL Token Concrete  ┃              ┃  P-Token Concrete   ┃            State Abstraction/
┃    MIR Semantics     ┃              ┃    MIR Semantics    ┃               Reachable State 
┗━━━━━━━━━━━━━━━━━━━━━━┛              ┗━━━━━━━━━━━━━━━━━━━━━┛            Over-Approximation 
-----------🡻-------------------------------------🡻---------------------------------------
┏━━━━━━━━━━━━━━━━━━━━━━┓              ┏━━━━━━━━━━━━━━━━━━━━━┓
┃  SPL Token Symbolic  ┃              ┃  P-Token Symbolic   ┃   
┃    MIR Semantics     ┃              ┃    MIR Semantics    ┃  Instruction Variant Specific         
┗━━━━━━━━━━━━━━━━━━━━━━┛              ┗━━━━━━━━━━━━━━━━━━━━━┛            State Partitioning
-----------🡻-------------------------------------🡻---------------------------------------
┏━━━━━━━━━━━━━━━━━━━━━━┓              ┏━━━━━━━━━━━━━━━━━━━━━┓
┃      SPL Token       ┃              ┃      P-Token        ┃
|  Instruction Variant ┃              ┃ Instruction Variant ┃
┃  Symb. MIR Semantics ┃              ┃ Symb. MIR Semantics ┃          
┗━━━━━━━━━━━━━━━━━━━━━━┛              ┗━━━━━━━━━━━━━━━━━━━━━┛          Big-Step Abstraction
-----------🡻-------------------------------------🡻---------------------------------------
┏━━━━━━━━━━━━━━━━━━━━━━┓              ┏━━━━━━━━━━━━━━━━━━━━━┓
┃      SPL Token       ┃              ┃      P-Token        ┃
┃       Labeled        ┃  =========>  ┃      Labeled        ┃
|  Transition System   ┃              ┃  Transition System  ┃
┗━━━━━━━━━━━━━━━━━━━━━━┛              ┗━━━━━━━━━━━━━━━━━━━━━┛              Check Simulation
-------------------------------------------------------------------------------------------

                           Figure 1.
                 Formal Verification Methodology
```

There are many details to discuss, but, to summarize the entire process in a single sentence:

We perform a multi-phase abstraction of the SPL Token and P-Token programs that converts their Rust source into labeled transitions systems with identical states and identical labels where we can apply standard program simulation checks using standard equality as our state equivalence relation.

Let us now break down the formal equivalence check methodology step-by-step:

1. Compile Rust to MIR via KMIR Rust Compiler

   We use a modified version of the Rust compiler that emits a simplified version of Rust code referred to as the Rust compiler's Mid-level Intermediate Representation (MIR) for use with the K symbolic proof engine; compared to standard Rust code, in MIR code:

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

2. Generate Program-Specific Concrete Semantics via $\mathbb{K}$ Formal Verification Framework + MIR Semantics

   The $\mathbb{K}$ framework is a toolkit for designing and modeling any programming language (PL) and also for formally reasoning about programs in a particular language.
   The $\mathbb{K}$ framework takes a formal PL semantics and generates a suite of correct-by-construction tools for that specific PL including:

   - a parser and unparser
   - a concrete interpreter
   - a symbolic execution engine and theorem prover

   In this step, we use our MIR semantics as an input to the $\mathbb{K}$ framework in order to a MIR interpreter.
   For those unfamiliar with the idea, a program language semantics is a complete set of rules that defines precisely how well-formed MIR programs behave.
   We then pass our MIR source programs as an input to the MIR interpreter and obtain a semantics for SPL Token and P-Token programs, i.e., program-specific interpreters which consume SPL Token/P-Token formatted byte strings as inputs and return a status code as an output.

   The goal of this step is to precisely capture of the SPL Token and P-Token programs so that we can analyze them.
   The correct-by-construction nature of these generated tools ensures _soundness_, i.e., if our semantics witness a particular state, then that state is reachable when we execute the actual program.

   Note that, in order conservatively over-approximate real implementations, a semantics will get stuck on undefined behaviors (while in practice, the Rust compiler will choose some arbitrary behavior).

3. Generate Program-Specific Symbolic Semantics via State Abstraction/Reachable State Over-approximation

   While examining the behavior of the program-specific formal semantics from step (2) might seem sufficient, we cannot be certain, when we execute them, that we will cover _all_ possible program states.
   So in this step, we over-approximate the entire set of reachable program states using a constrained term and update our semantics to now perform execution symbolically beginning from this abstract state.
   The goal of making this move is to ensure _completeness_, i.e., if the source programs can perform some behavior, than when we execute our symbolic semantics, we will be able to reproduce.
   Note that, as part of this shift, we also rely on $\mathbb{K}$'s built-in theorem proving capabilities that let us use matching logic, co-induction, and SMT solvers to reason about (a potentially infinite number of) program states and transitions in a finite amount of time[^limitations].

4. Generate Program+Instruction-Variant-Specific Symbolic Semantics via State Partitioning

   While state abstraction alone ensures completeness of reasoning, it makes for very large and complex proofs.
   To counteract this tendency, we perform a per-instruction-variant partitioning of the abstract state space.
   In essense, after performing this partitioning, we obtain a per-instruction-variant-specific symbolic semantics for the SPL Token and P-Token programs that can only accept a that instruction variant (e.g., one variant only accepts `Transfer`s, one variant only accepts `InitializeAccount`, etc...).
   As mentioned above, we don't gain any additional reasoning capabilities by making this move; this is purely _proof engineering_.
   In essence, this is a form of _modularization_, and the payoffs exactly mirror those that we in broader software development world.

5. Perform a Big-Step Semantics Style Abstraction

   In this step, we apply a big-step semantics style abstraction to our semantics.
   For those familiar with big-step style semantics, they evaluate a program, from its initial state to its final state, in one-shot.
   By that we mean, the notion of intermediate states is entirely dropped.
   Here, we view each execution of the SPL Token or P-Token program as just a quadruple that contains:

   1. an input state (a set of program-owned accounts with certain keys, lamport balances, and data payloads)
   2. a string of input instruction bytes
   3. an unsigned 64-bit output status code
   4. an output state (also a set of program-owned accounts with certain keys, lamport balances, and data payloads)

   By concatenating items (2)-(3) together into a single item, we now have just what we need to construct a labelled transition system where:

   - states are partial functions from account keys to pairs of lamport balances and data payloads;
   - transitions are triples containing an input state, a label (which is a pair of an instruction format plus the return code obtained from executing that instruction format), and an output state.

6. Check Simulation; Show When Provided Identical Pre-states/Inputs, Identical Post-States Reached
   
   Finally, to check that the P-Token program actually simulates the SPL Token program, we execute each instruction-specific semantics for SPL Token and P-Token from the same starting symbolic state with the same symbolic input.
   We then verify that each exeuction reaches an identical set of symbolic output states.
   This is equivalent to _batching_ the required P-Token-can-copy-SPL-Token-moves checks instead of checking that for each individual SPL-Token transition, we can find a corresponding individual P-Token transition.
   In fact, without this batching, performing an exhaustive simulation check would be extremely difficult (if not impossible) on modern hardware.

[^limitations]: Note that there is no free lunch and symbolic execution is not a panacea that can be trivially used to solve all verification problems. In certain cases, we must provide the execution engine
with lemmas (in other words, hints) that help the execution engine make progress when it gets stuck.

**TODO:** Cleanup details below.

Now that we have surveyed the entire equivalence check process, there are a few details that we need to fill in.
We needed to prove the following lemmas by hand in order for the verification to work:

1. We manually prove that any valid SPL Token instruction format is also valid for P-Token.
   This means that any P-Token instruction format is byte-for-byte compatible with SPL Token.

2. We manually derive and prove an inductive invariant $\Iota$ for _both_ SPL Token and P-Token and use this as our reachable state over-approximation.
   This invariant is the union of the following properties:

   1. The account formats for all SPL Token (resp. P-Token) owned accounts are either:

      - fully zeroed out (when the account is uninitialized) or;
      - correspond to an encoded and properly initialized Mint, Token Account, or Multisig.

      An easy corollary of (1) is that the total set of states (ignoring reachability) that the SPL Token or P-Token program can take on are identical.

   2. The `lamports` field for all SPL Token (resp. P-Token) owned accounts is sufficient for rent exemption

   3. For a given Mint, the sum of all Token Account balances for that Mint equals the Mint's supply

   4. The `lamports` field for a native Token Account is greater than or equal to the Token Account's balance plus its rent exemption price.

Additionally, we now describe our state paritioning process in more detail:

1. We partition the complete set of possible instruction formats into a finite (and small!) set $\{instr_i\}_i$ using symbolic constraints.

2. For each instruction format instance $instr_i$, we partition the complete set of potential account format sequences that it can accept $\alpha_{(i,j)}=\left\{acct_{(i,j,0)},\cdots,acct_{(i,j,arity(i,j))}\right\}_{(i,j)}$ --- also using symbolic constraints where $arity(i,j)$ denotes the arity of the account format sequenced indexed by $(i,j)$.

3. For each combination of an instruction format $instr_i$ and corresponding account sequence $\alpha_{(i,j)}$, we symbolically execute the SPL Token program and record its return code $ret_{(i,j)}$ and updated account set $\alpha'_{(i,j)}$.

## Introduction 

The Solana Programming Library (SPL) is a set of Solana-specific programs and libraries, written in the Rust programming language, that can be used to create custom dApps for Solana. An important component of the SPL toolkit is the SPL Token Program, a program that enables the creation and usage of custom fungible and non-fungible tokens on the Solana blockchain. 

As the name indicates, the SPL Token Program contains the source code for a deployable program on Solana.
Like other kinds of Solana programs, the SPL Token program has:

1. a handful of data format variant that can be stored in any account owned by the Token program;
2. a handful of instruction format variants that instruct the program to perform various operations.

We will describe these formats more fully below.

As usage of the SPL Token Program increased, eventually, a compute-optimized version of the SPL Token Program was proposed in Solana Improvement Document (SIMD-0266) by febo and Jon Cinque.
The idea: a program with byte-for-byte equivalent account and instruction formats that operates more efficiently, using less validator time, and thus permitting more instructions to be validated in a single Solana slot.

### SPL Token Account Format Variants

As discussed in the [Solana Programming Model](#solana-programming-model) section, the accounts that are owned by a particular program are used to store that program's state.
However, recall that Solana does not have runtime-enforced type checking of values stored in an account's `data` field.
Thus, determining what kinds of values a program permits saving as its owned account's `data` requires a careful reading of the program source code.
To aid with this task, the SPL provides a few packages to aid in account manipulation:

- solana-account-info - a data structure `AccountInfo` that represents the Solana account structure
- solana-program-entrypoint - an entrypoint wrapper macro that parses all encoded Solana accounts and deserializes them into a slice of `AccountInfo`s and also extracts the instruction format
- solana-program-pack - provides a `Pack` helper trait that can be used to de/serialize encoded values in the `data` field of `AccountInfo` that always checks length of the slice used as the de/serialization source/target.

By a quick scan of the source code, we see that:

- a slice containing all `AccountInfo`s passed to the entrypoint are created by the solana-program-entrypoint package;
- all `AccountInfo`s are accessed via the `next_account_info` iterator helper function;
- all writes to `AccountInfo.data` occurs via a call to the `Pack` trait's `pack` function.

Essentially, this means that, assuming that the solana-program-entrypoint package correctly initializes `AccountInfo` and the called `pack` implementations are consistent, then we know that only `pack`able data will be stored in accounts owned by the SPL Token program.

From the above argumentation, we can determine that the account data format has the following variants (note that the data is densely packed, i.e., stored without any padding):

1. Account (165 bytes)
   - key of mint account (32 bytes)
   - key of owner account (32 bytes)
   - `u64` account balance (8 bytes) 
   - optional key of delegate account (36 bytes - 4 bytes for option tag + 32 bytes for key)
   - fieldless account state enumeration (1 byte)
   - optional native rent exempt reserve amount (12 bytes - 4 bytes for option tag + 8 bytes for `u64` amount)
   - `u64` amount authorized for delegate to spend (8 bytes)

2. Mint (82 bytes)
   - optional key of mint authority (36 bytes - 4 bytes for option tag + 32 bytes for key)
   - `u64` total supply of all non-native minted tokens - (8 bytes)
   - `u8` number of decimals for minted token units - (1 byte)
   - `bool` initialized indicator - (1 byte)
   - optional key for freeze authority (36 bytes - 4 bytes for option tag + 32 bytes for key)

3. Multisig (355 bytes)
   - `u8` number of signers required - (1 byte)
   - `u8` number of valid signers - (1 byte)
   - `bool` initialized indicator - (1 byte)
   - `[Pubkey; 11]` fixed size valid signers array - (11 * 32 = 352 bytes)

### SPL Token Instruction Formats

In a similar fashion to our previous analysis, we can the byte layout of instruction format variants.
To do this, we examine the SPL Token program entrypoint and check how the instruction format is parsed.
There is a single parser function that is called `TokenInstruction::unpack`.
This parser function determines that the various instruction format variants have the following layouts:

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

Note that the P-Token program has two additional instruction format variants that SPL Token does not have.
However, we will examine these further since they are not relevant for the equivalence proof.

| Enum Tag | Enum Variant             | Associated Data                                           | Comment                                                   | Success Case |
| ---      | ---                      | ---                                                       | ---                                                       | ---          |
|  38      | WithdrawExcessLamports   |                                                           |                                                           | 1            |
| 255      | Batch                    | (accts:u8,len:u8,len)*                                    |                                                           | N            |

| Enum Tag | AuthorityType Variant |
| ---      | ---                   |
| 0        | MintTokens            |
| 1        | FreezeAccount         |
| 2        | AccountOwner          |
| 3        | CloseAccount          |

| Universal Failure Cases |
| ---                     |
| Input Too Short         |
| Not Enough Accounts     |

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
