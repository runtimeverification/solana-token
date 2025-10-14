# SPL Token Program Formal Verification Guide

## Architecture

- **Production code**: `src/entrypoint.rs` - Used for normal builds
- **Runtime verification code (Original Version)**: `src/entrypoint-rvo.rs` - Used when `rvo` feature is enabled. This version is to be tested by the original spl token tests.
- **Runtime verification code**: `src/entrypoint-runtime-verification.rs` - Used when `runtime-verification` feature is enabled. This version is constrained and constructed with formal verification in mind.

## Setup test environment

1. Make sure you can pass the original tests in the root directory:
```sh
pnpm programs:build && pnpm programs:test
```
2. Check if the specs in `program/entrypoint-rvo.rs` are passing with the same tests (important step after adding new specs):
```sh
pnpm programs:build -- --features rvo && pnpm programs:test -- --features rvo
```

## Scripts
- `./scripts/compare-test-functions.py` - Compares matching `test_*` functions between the runtime verification entrypoints and highlights differences or missing tests. Run it directly (`./scripts/compare-test-functions.py`) to diff `p-token/src/entrypoint-runtime-verification.rs` against both `program/src/entrypoint-rvo.rs` and `program/src/entrypoint-runtime-verification.rs`. Pass extra pairs with `--pairs left:path:right:path` if you need custom comparisons.
