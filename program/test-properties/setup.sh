#!/bin/bash
#
# Setup for running spl-token property tests with kmir
# * checks out submodule mir-semantics (recursively including stable-mir-json)
# * builds stable-mir-json and mir-semantics
# * builds spl-token with stable-mir-json and links SMIR JSON artefacts
#
# Usage:
#   ./setup.sh                     # Normal execution, including refreshing submodules
#   ./setup.sh --skip-submodules   # Skip refreshing submodules
#
# After running it, proofs can be executed with
#   `uv --project mir-semantics/kmir run -- kmir ...`
######################################################################

set -xeuo pipefail

SKIP_SUBMODULES=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-submodules)
            SKIP_SUBMODULES=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-submodules    Skip refreshing git submodules"
            echo "  -h, --help           Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

SCRIPT_DIR="$(realpath "$(dirname "$0")")"
cd "${SCRIPT_DIR}"

if [ "${SKIP_SUBMODULES}" = false ]; then
    echo "Refreshing git submodules..."
    git submodule update --init --recursive
    git submodule status --recursive

    ROOT_DIR="$(git rev-parse --show-toplevel)"
    MIR_BRANCH="$(git config -f "${ROOT_DIR}/.gitmodules" submodule."program/test-properties/mir-semantics".branch || true)"
    MIR_BRANCH="${MIR_BRANCH:-feature/spl-token}"
    echo "Checking out mir-semantics at origin/${MIR_BRANCH}..."
    if git -C mir-semantics status --porcelain | grep -v '^?? ' >/dev/null; then
        echo "Skipping checkout: tracked local changes detected."
    else
        git -C mir-semantics fetch origin "${MIR_BRANCH}"
        git -C mir-semantics checkout --quiet "origin/${MIR_BRANCH}"
    fi
else
    echo "Skipping git submodule refresh..."
fi

if [ -z "$(cd mir-semantics && git status --porcelain)" ]; then
    printf "\n\n# avoid workspace confusion in token repo\n[workspace]\n" \
        >> mir-semantics/deps/stable-mir-json/Cargo.toml
fi

make -C mir-semantics stable-mir-json build CARGO_BUILD_OPTS=--release

export RUSTC="$PWD/mir-semantics/deps/.stable-mir-json/release.sh"
"${RUSTC}" --version

cd .. # program
cargo clean && cargo build --features runtime-verification
cd test-properties

SMIRS=$(ls ../../target/debug/deps/*smir.json | sort)
ls ${SMIRS}

mkdir -p artefacts/
uv --project mir-semantics/kmir run -- kmir link ${SMIRS} -o artefacts/spl-token.smir.json
