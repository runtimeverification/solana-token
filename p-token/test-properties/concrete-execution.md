# Concrete Execution Guide

## Overview

The p-token project includes a runtime verification feature flag that enables compilation of p-token with Runtime Verification Inc. specifications by pathing the `entrypoint` module to `p-token/src/entrypoint-runtime-verification`. This feature uses conditional compilation to separate original Anza production code from Runtime Verification Inc. verification code.

The codebase conditionally compiles one of the following files depending on the flag:

- **Anza Production**: `p-token/src/entrypoint.rs` - Standard Solana pinocchio token program entrypoint
- **Runtime Verification**: `p-token/src/entrypoint-runtime-verification.rs` - Enhanced entrypoint with specifications and cheatcode functions for formal verification

## Feature Flag

The `runtime-verification` feature flag enables building for verification, but can also be run concretely against p-token and program test suites:

```toml
[features]
runtime-verification = []
```

## Commands

### Build Commands

```bash
# Standard p-token build
pnpm p-token:build

# Runtime verification p-token build
pnpm p-token:build:rv
```

### P-Token Test Commands

```bash
# Standard p-token tests
pnpm p-token:test

# Runtime verification p-token tests
pnpm p-token:test:rv
```

### Program Test Commands

Run the p-token binary against the program test suite. See [Running P-Token On Programs Tests](#running-p-token-on-programs-tests)

```bash
# Run program tests against standard p-token build
pnpm programs:test:pinocchio

# Run program tests against runtime verification p-token build
pnpm programs:test:pinocchio:rv
```

### Complete Workflow For Running RV Specs Concretely

```bash
# Install dependencies
pnpm install

# Build with runtime verification
pnpm p-token:build:rv

# Run p-token tests with runtime verification specification build
pnpm p-token:test:rv

# Run program tests against runtime verification p-token specification build
pnpm programs:test:pinocchio:rv
```

## Implementation Details

The runtime verification mode includes:

1. **Enhanced Entrypoint**: Modified instruction processing that contains the specifications of correctness embedded as executable Rust
2. **Cheatcode Functions**: Type hints for formal verification tools, however these are empty blocks for concrete execution

## Direct Usage

Instead of using the `pnpm` scripts one can enable the feature flag when building to build with the Runtime Verification specifications:

```bash
cargo build --features runtime-verification
cargo test --features runtime-verification
```

## Running P-Token On Programs Tests

The program test commands allow you to run the original SPL Token program test suite against the p-token implementation. This ensures that p-token behaves identically to the original Solana program (for these tests), providing confidence that the Runtime Verification specifications are correct and that p-token maintains full compatibility.

The workflow for the command is as such:

1. **Build p-token**: Compiles the p-token program (standard or with runtime-verification feature)
2. **Copy binary**: Copies the p-token binary (`.so`) to the location expected by the original program tests
3. **Run tests**: Executes the original program test suite against the p-token implementation
4. **Verify compatibility**: If tests pass, then p-token behaves identically to the original SPL Token program (for these tests)
