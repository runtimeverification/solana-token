#!/bin/bash
#
# Setup for running property tests with kmir (defaults for p-token).
# - sets up a virtual environment with kompass installed
# - checks out and builds submodule stable-mir-json
# - builds the selected crate with STABLE MIR and links SMIR JSON into artefacts
# - builds the K definitions
#
# Usage:
#   ./setup.sh [OPTIONS]
#
# Options (p-token defaults):
#   --skip-submodules           Skip refreshing git submodules (default)
#   --with-submodules           Refresh git submodules
#   -h, --help                  Show help
#
# Overridable via environment (kept minimal):
#   CRATE_DIR         Crate to build (default: ../ for p-token)
#   ARTIFACT_BASENAME Artefact base name (default: p-token)
#
# After running it, one can use:
#   source deps/.venv/bin/activate
#   kmir ...
######################################################################

set -xeuo pipefail

SCRIPT_DIR="$(realpath "$(dirname "$0")")"

SKIP_SUBMODULES=true
CRATE_DIR="${CRATE_DIR:-$(realpath "${SCRIPT_DIR}/..")}"
ARTIFACT_BASENAME="${ARTIFACT_BASENAME:-p-token}"


while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-submodules)
            SKIP_SUBMODULES=true; shift ;;
        --with-submodules)
            SKIP_SUBMODULES=false; shift ;;
        -h|--help)
            echo "Usage: $0 [--skip-submodules]"; exit 0 ;;
        *)
            echo "Unknown option: $1"; echo "Use --help for usage."; exit 1 ;;
    esac
done

cd "${SCRIPT_DIR}"


#############################
# Step 1: Set up kompass venv
#############################

VENV_DIR='deps/.venv'
KOMPASS_URL='https://github.com/runtimeverification/kompass'
KOMPASS_VERSION=$(cat deps/kompass_release)

if [[ -d "$VENV_DIR" ]]; then
    echo "Virtual environment already exists at ${VENV_DIR}"
    source "$VENV_DIR/bin/activate"
else
    echo "Creating virtual environment at ${VENV_DIR}"
    python3 -m venv "${VENV_DIR}"

    echo "Installing kompass"
    source "$VENV_DIR/bin/activate"
    pip install --upgrade pip
    pip install "git+${KOMPASS_URL}@${KOMPASS_VERSION}"
fi


###############################
# Step 2: Build stable-mir-json
###############################

# Refresh/check out submodules (unless skipped)
if [ "${SKIP_SUBMODULES}" = false ]; then
    echo "Refreshing git submodules..."
    git submodule update --init --recursive
    git submodule status --recursive
else
    echo "Skipping git submodule refresh..."
fi

# Ensure stable-mir-json acts as its own workspace to avoid root workspace capture.
# Add an empty [workspace] if missing (idempotent, independent of git status).
SMJ_CARGO_TOML="deps/stable-mir-json/Cargo.toml"
if ! grep -q '^\[workspace\]' "$SMJ_CARGO_TOML"; then
    printf "\n\n# avoid workspace confusion in token repo\n[workspace]\n" >> "$SMJ_CARGO_TOML"
fi

# Build stable-mir-json
make stable-mir-json CARGO_BUILD_OPTS=--release

export RUSTC=$PWD/deps/.stable-mir-json/release.sh
${RUSTC} --version


##############################
# Step 3: Build and link tests
##############################

# Build selected crate with stable-mir-json (clean first)
pushd "${CRATE_DIR}" >/dev/null
# Force cargo to emit artifacts under the crate's own target directory
export CARGO_TARGET_DIR="${CRATE_DIR}/target"
cargo clean && cargo build --features runtime-verification,assumptions
popd >/dev/null

# Collect SMIR JSONs from the crate's target dir
if ! SMIRS=$(ls "${CRATE_DIR}/target/debug/deps"/*.smir.json 2>/dev/null | sort); then
    echo "[ERROR] No SMIR JSON files found under ${CRATE_DIR}/target/debug/deps."
    echo "        Ensure the crate built with RUSTC wrapper and produced .smir.json outputs."
    exit 3
fi
ls ${SMIRS}

# Link all SMIR JSON and store in artefacts directory
mkdir -p "${ARTEFACTS_DIR:-artefacts}/"
kmir link ${SMIRS} -o "${ARTEFACTS_DIR:-artefacts}/${ARTIFACT_BASENAME}.smir.json"


#############################
# Step 4: Build K definitions
#############################

kdist --verbose build -j4 kompass.\*
