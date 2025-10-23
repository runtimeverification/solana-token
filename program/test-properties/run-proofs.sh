#!/bin/bash
# Minimal wrapper to reuse p-token run-proofs for spl-token
set -euo pipefail

SCRIPT_DIR="$(realpath "$(dirname "$0")")"
PTOKEN_DIR="$(realpath "${SCRIPT_DIR}/../../p-token/test-properties")"

# Use p-token's proofs.md by forwarding all args directly
cd "${PTOKEN_DIR}"
START_PREFIX="spl_token::entrypoint::" \
ARTIFACT_BASENAME="spl-token" \
ARTIFACTS_DIR="${SCRIPT_DIR}/artefacts" \
PROOF_STATUS_DIR="${SCRIPT_DIR}/proof_status" \
exec ./run-proofs.sh "$@"
