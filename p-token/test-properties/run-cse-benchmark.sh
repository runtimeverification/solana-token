#!/usr/bin/env bash
#
# Compare proof time for:
#   1. CSE with an empty per-test summary store (cold)
#   2. CSE with the same summary store populated by cold-cse (warm)
#   3. The same proof without CSE
#
# CSE target functions are inferred from the SMIR call graph for each test.
# The automatic selector only considers reachable p-token processor functions,
# keeps the largest reachable process_* target by default, and adds additional
# processor functions only when their MIR instruction count reaches the
# configured threshold. Trace mode can also include the proof start symbol as
# a wrapper so warm runs replay large harness-side branch structures.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

START_PREFIX="${START_PREFIX:-pinocchio_token_program::entrypoint::}"
ARTIFACT_BASENAME="${ARTIFACT_BASENAME:-p-token}"
ARTIFACTS_DIR="${ARTIFACTS_DIR:-artefacts}"
KMIR_PROJECT="${KMIR_PROJECT:-mir-semantics/kmir}"
INFER_SCRIPT="${SCRIPT_DIR}/scripts/infer-cse-functions.py"

TIMEOUT=28800
SHOW_TIMEOUT="${SHOW_TIMEOUT:-300}"
TIMEOUT_KILL_AFTER="${TIMEOUT_KILL_AFTER:-60s}"
MAX_WORKERS=1
BASELINE_MAX_WORKERS=1
MIN_INSTRUCTIONS=20
MAX_CSE_FUNCTIONS=0
PROVE_OPTS=(--fail-fast)
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)"
KEEP=false
RESUME_PROOFS=false
PLAN_ONLY=false
RUN_ALL=false
RUN_MULTISIG=false
FORCE_LARGEST_PROCESS=true
MIRROR_CSE_BREAKPOINTS="${MIRROR_CSE_BREAKPOINTS:-true}"
INCLUDE_START_FUNCTION="${INCLUDE_START_FUNCTION:-false}"

declare -a TESTS=()
declare -a EXTRA_CSE_FUNCTIONS=()
declare -a CSE_FUNCTIONS=()

usage() {
    cat <<'EOF'
Usage: ./run-cse-benchmark.sh [OPTIONS] [TEST_NAME...]

Runs cold CSE, warm CSE, and no-CSE proofs, then writes timing data under
artefacts/cse-bench/. CSE functions are inferred per test from the SMIR call
graph and filtered by MIR instruction count.

Options:
  -a              Run all start symbols from the first proofs.md table.
  -m              Run all start symbols from the multisig proofs.md table.
  -f FUNC         Extra CSE function. Repeatable; appended after auto targets.
  -i N            Minimum MIR instruction count for non-forced targets. Default: 20
  -x N            Maximum automatic CSE targets per test, or 0 for unlimited. Default: 0
  -p              Plan only: print inferred CSE targets without running proofs.
  -t SEC          Timeout per proof run. Default: 28800
  -w N            max-workers for CSE runs. Must be 1 on current kmir CSE. Default: 1
  -b N            max-workers for no-CSE baseline. Default: 1
  -o OPTS         Extra prove-rs options as one shell string. Default: --fail-fast
  -r ID           Run id / output directory suffix. Default: current UTC timestamp
  -k              Keep an existing run directory instead of removing it first.
  --resume-proofs Keep existing proof directories and omit --reload, allowing a
                  timed-out proof to continue from proof-dir data.
  --no-force-largest-process
                  Do not keep the largest reachable process_* below threshold.
  --include-start-function
                  Add each proof start symbol as a CSE trace wrapper target.
                  This is useful for large tests whose branch explosion happens
                  before the selected processor function is reached.
  -h, --help      Show this help.

Environment:
  START_PREFIX       Start symbol prefix.
  ARTIFACT_BASENAME  SMIR basename. Default: p-token
  ARTIFACTS_DIR      Artefact directory. Default: artefacts
  KMIR_PROJECT       uv project for kmir. Default: mir-semantics/kmir
  SHOW_TIMEOUT       Timeout for optional kmir show after each proof. Default: 300
  TIMEOUT_KILL_AFTER Duration after TERM before timeout sends KILL. Default: 60s
  MIRROR_CSE_BREAKPOINTS
                     Add --break-on-function for selected CSE targets to no-CSE
                     runs so proof structure is compared with the same cuts.
                     Default: true
  INCLUDE_START_FUNCTION
                     Same as --include-start-function when set to true.
                     Default: false

Examples:
  ./run-cse-benchmark.sh -p test_process_get_account_data_size
  ./run-cse-benchmark.sh -i 40 test_process_transfer test_process_burn
  ./run-cse-benchmark.sh -a -t 28800
EOF
}

die() {
    echo "[ERROR] $*" >&2
    exit 2
}

while [[ "$#" -gt 0 ]]; do
    case "$1" in
        -a)
            RUN_ALL=true
            shift
            ;;
        -m)
            RUN_MULTISIG=true
            shift
            ;;
        -f)
            [[ "$#" -ge 2 ]] || die "-f requires a function name"
            EXTRA_CSE_FUNCTIONS+=("$2")
            shift 2
            ;;
        -i)
            [[ "$#" -ge 2 ]] || die "-i requires an instruction count"
            MIN_INSTRUCTIONS="$2"
            shift 2
            ;;
        -x)
            [[ "$#" -ge 2 ]] || die "-x requires a function count"
            MAX_CSE_FUNCTIONS="$2"
            shift 2
            ;;
        -p)
            PLAN_ONLY=true
            shift
            ;;
        -t)
            [[ "$#" -ge 2 ]] || die "-t requires a timeout"
            TIMEOUT="$2"
            shift 2
            ;;
        -w)
            [[ "$#" -ge 2 ]] || die "-w requires a worker count"
            MAX_WORKERS="$2"
            shift 2
            ;;
        -b)
            [[ "$#" -ge 2 ]] || die "-b requires a worker count"
            BASELINE_MAX_WORKERS="$2"
            shift 2
            ;;
        -o)
            [[ "$#" -ge 2 ]] || die "-o requires prove-rs options"
            # shellcheck disable=SC2206
            PROVE_OPTS=($2)
            shift 2
            ;;
        -r)
            [[ "$#" -ge 2 ]] || die "-r requires a run id"
            RUN_ID="$2"
            shift 2
            ;;
        -k)
            KEEP=true
            shift
            ;;
        --resume-proofs)
            RESUME_PROOFS=true
            shift
            ;;
        --no-force-largest-process)
            FORCE_LARGEST_PROCESS=false
            shift
            ;;
        --include-start-function)
            INCLUDE_START_FUNCTION=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        --)
            shift
            TESTS+=("$@")
            break
            ;;
        -*)
            die "Invalid option: $1"
            ;;
        *)
            TESTS+=("$1")
            shift
            ;;
    esac
done

[[ "${MIN_INSTRUCTIONS}" =~ ^[0-9]+$ ]] || die "-i must be a non-negative integer"
[[ "${MAX_CSE_FUNCTIONS}" =~ ^[0-9]+$ ]] || die "-x must be a non-negative integer"
[[ "${MAX_WORKERS}" =~ ^[0-9]+$ ]] || die "-w must be a non-negative integer"
[[ "${BASELINE_MAX_WORKERS}" =~ ^[0-9]+$ ]] || die "-b must be a non-negative integer"

if [[ "${MAX_WORKERS}" != "1" ]]; then
    die "kmir CSE currently requires CSE max-workers to be 1"
fi

mapfile -t ALL_NAMES < <(sed -n -e 's/^| \(test_p[a-zA-Z0-9:_]*\) *|.*/\1/p' proofs.md)
mapfile -t MULTISIG_NAMES < <(sed -n -e 's/^| m | \(test_p[a-zA-Z0-9:_]*\) *|.*/\1/p' proofs.md)

if [[ "${RUN_ALL}" == true ]]; then
    TESTS+=("${ALL_NAMES[@]}")
fi
if [[ "${RUN_MULTISIG}" == true ]]; then
    TESTS+=("${MULTISIG_NAMES[@]}")
fi

declare -A SEEN_TESTS=()
declare -a UNIQUE_TESTS=()
for test_name in "${TESTS[@]}"; do
    [[ -n "${test_name}" ]] || continue
    if [[ -z "${SEEN_TESTS[${test_name}]:-}" ]]; then
        UNIQUE_TESTS+=("${test_name}")
        SEEN_TESTS["${test_name}"]=1
    fi
done
TESTS=("${UNIQUE_TESTS[@]}")

if [[ "${#TESTS[@]}" -eq 0 ]]; then
    die "No test function names given. Use -a/-m or provide at least one name."
fi

SMIR_FILE="${ARTIFACTS_DIR}/${ARTIFACT_BASENAME}.smir.json"
if [[ ! -f "${SMIR_FILE}" ]]; then
    echo "[ERROR] Missing ${SMIR_FILE}; run ./setup.sh --skip-submodules first." >&2
    exit 1
fi
if [[ ! -f "${INFER_SCRIPT}" ]]; then
    echo "[ERROR] Missing ${INFER_SCRIPT}" >&2
    exit 1
fi

REPO_COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
MIR_COMMIT="$(git -C mir-semantics rev-parse --short HEAD 2>/dev/null || echo unknown)"
BENCH_ROOT="${ARTIFACTS_DIR}/cse-bench/${RUN_ID}-${REPO_COMMIT}-${MIR_COMMIT}"
RESULTS_CSV="${BENCH_ROOT}/results.csv"
RESULTS_MD="${BENCH_ROOT}/results.md"

if [[ "${KEEP}" == false ]]; then
    rm -rf "${BENCH_ROOT}"
fi
mkdir -p "${BENCH_ROOT}"

printf 'test,case,exit_code,duration_seconds,proof_status,nodes,pending,failing,stuck,terminal,summary_count,trace_count,subsumption_count,store_bytes,trace_hits,trace_misses,subsume_hits,subsume_misses,execute_requests,implies_requests,cse_functions,proof_dir,log\n' > "${RESULTS_CSV}"

join_cse_functions() {
    if [[ "${#CSE_FUNCTIONS[@]}" -eq 0 ]]; then
        return
    fi
    local IFS=';'
    printf '%s' "${CSE_FUNCTIONS[*]}"
}

summary_count() {
    local summary_store="$1"
    if [[ -d "${summary_store}/summaries" ]]; then
        find "${summary_store}/summaries" -type f -name '*.json' | wc -l
    else
        echo 0
    fi
}

trace_count() {
    local summary_store="$1"
    if [[ -d "${summary_store}/traces" ]]; then
        find "${summary_store}/traces" -type f -name '*.json' | wc -l
    else
        echo 0
    fi
}

subsumption_count() {
    local summary_store="$1"
    if [[ -d "${summary_store}/subsumptions" ]]; then
        find "${summary_store}/subsumptions" -type f -name '*.json' | wc -l
    else
        echo 0
    fi
}

store_bytes() {
    local summary_store="$1"
    if [[ -d "${summary_store}" ]]; then
        du -sb "${summary_store}" | awk '{print $1}'
    else
        echo 0
    fi
}

log_count() {
    local log_file="$1"
    local pattern="$2"
    grep -c "${pattern}" "${log_file}" 2>/dev/null || true
}

proof_status_value() {
    local log_file="$1"
    local status
    status="$(grep -m1 'status: ProofStatus' "${log_file}" 2>/dev/null | sed -E 's/.*ProofStatus\.([A-Z_]+).*/\1/' || true)"
    if [[ -n "${status}" ]]; then
        echo "${status}"
    else
        echo "UNKNOWN"
    fi
}

log_metric() {
    local log_file="$1"
    local key="$2"
    local value
    value="$(grep -m1 "^[[:space:]]*${key}:" "${log_file}" 2>/dev/null | awk -F': *' '{print $2}' || true)"
    if [[ -n "${value}" ]]; then
        echo "${value}"
    else
        echo ""
    fi
}

mark_summary_store_state() {
    local summary_store="$1"
    local test_name="$2"
    local case_name="$3"
    local exit_code="$4"
    local proof_dir="$5"
    local log_file="$6"
    local complete="$7"

    python3 - "$summary_store" "$test_name" "$case_name" "$exit_code" "$proof_dir" "$log_file" "$complete" <<'PY'
import json
import sys
import time
from pathlib import Path

summary_store, test_name, case_name, exit_code, proof_dir, log_file, complete = sys.argv[1:]
path = Path(summary_store)
path.mkdir(parents=True, exist_ok=True)
status = 'UNKNOWN'
log_path = Path(log_file)
if log_path.is_file():
    for line in log_path.read_text(errors='replace').splitlines():
        if 'status: ProofStatus.' in line:
            status = line.rsplit('ProofStatus.', 1)[1].split()[0]
            break

manifest = {
    'schema': 1,
    'test': test_name,
    'case': case_name,
    'complete': complete == 'true',
    'exit_code': exit_code,
    'proof_status': status,
    'proof_dir': proof_dir,
    'log': log_file,
    'updated_at_utc': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
}
(path / 'benchmark-store-state.json').write_text(json.dumps(manifest, indent=2, sort_keys=True))
PY
}

write_case_result() {
    local test_name="$1"
    local case_name="$2"
    local exit_code="$3"
    local duration="$4"
    local summary_store="$5"
    local proof_dir="$6"
    local log_file="$7"
    local status nodes pending failing stuck terminal summaries traces subsumptions bytes hits misses subsume_hits subsume_misses executes implies functions

    status="$(proof_status_value "${log_file}")"
    nodes="$(log_metric "${log_file}" nodes)"
    pending="$(log_metric "${log_file}" pending)"
    failing="$(log_metric "${log_file}" failing)"
    stuck="$(log_metric "${log_file}" stuck)"
    terminal="$(log_metric "${log_file}" terminal)"
    summaries="$(summary_count "${summary_store}" | tr -d ' ')"
    traces="$(trace_count "${summary_store}" | tr -d ' ')"
    subsumptions="$(subsumption_count "${summary_store}" | tr -d ' ')"
    bytes="$(store_bytes "${summary_store}" | tr -d ' ')"
    hits="$(log_count "${log_file}" 'CSE trace cache hit' | tr -d ' ')"
    misses="$(log_count "${log_file}" 'CSE trace cache miss' | tr -d ' ')"
    subsume_hits="$(log_count "${log_file}" 'CSE trace subsumption cache hit' | tr -d ' ')"
    subsume_misses="$(log_count "${log_file}" 'CSE trace subsumption cache miss' | tr -d ' ')"
    executes="$(log_count "${log_file}" 'Sending request.* - execute' | tr -d ' ')"
    implies="$(log_count "${log_file}" 'Sending request.* - implies' | tr -d ' ')"
    functions="$(join_cse_functions)"

    printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
        "${test_name}" "${case_name}" "${exit_code}" "${duration}" "${status}" \
        "${nodes}" "${pending}" "${failing}" "${stuck}" "${terminal}" \
        "${summaries}" "${traces}" "${subsumptions}" "${bytes}" "${hits}" "${misses}" \
        "${subsume_hits}" "${subsume_misses}" "${executes}" "${implies}" \
        "${functions}" "${proof_dir}" "${log_file}" >> "${RESULTS_CSV}"
}

write_skipped_result() {
    local test_name="$1"
    local case_name="$2"
    local summary_store="$3"
    local summaries traces subsumptions bytes functions

    summaries="$(summary_count "${summary_store}" | tr -d ' ')"
    traces="$(trace_count "${summary_store}" | tr -d ' ')"
    subsumptions="$(subsumption_count "${summary_store}" | tr -d ' ')"
    bytes="$(store_bytes "${summary_store}" | tr -d ' ')"
    functions="$(join_cse_functions)"
    printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
        "${test_name}" "${case_name}" "SKIPPED" "0" "SKIPPED" \
        "" "" "" "" "" "${summaries}" "${traces}" "${subsumptions}" "${bytes}" \
        "" "" "" "" "" "" "${functions}" "" "" >> "${RESULTS_CSV}"
}

infer_cse_plan() {
    local test_name="$1"
    local plan_tsv="$2"
    local start_symbol="${START_PREFIX}${test_name}"
    local infer_args=(
        --smir "${SMIR_FILE}"
        --start-symbol "${start_symbol}"
        --min-instructions "${MIN_INSTRUCTIONS}"
        --max-functions "${MAX_CSE_FUNCTIONS}"
        --format tsv
    )

    if [[ "${FORCE_LARGEST_PROCESS}" == false ]]; then
        infer_args+=(--no-force-largest-process)
    fi
    if [[ "${INCLUDE_START_FUNCTION}" == true ]]; then
        infer_args+=(--include-start-symbol)
    fi
    for func in "${EXTRA_CSE_FUNCTIONS[@]}"; do
        infer_args+=(--extra-function "${func}")
    done

    uv --project "${KMIR_PROJECT}" run -- python "${INFER_SCRIPT}" "${infer_args[@]}" > "${plan_tsv}"
}

load_cse_functions() {
    local plan_tsv="$1"
    CSE_FUNCTIONS=()
    while IFS=$'\t' read -r func _role _instructions _reason; do
        [[ -n "${func}" ]] || continue
        CSE_FUNCTIONS+=("${func}")
    done < "${plan_tsv}"
}

append_plan_markdown() {
    local test_name="$1"
    local plan_tsv="$2"

    {
        echo "### ${test_name}"
        echo
        if [[ ! -s "${plan_tsv}" ]]; then
            echo "No CSE targets selected."
            echo
            return
        fi
        echo "| function | role | MIR instructions | reason |"
        echo "| --- | --- | ---: | --- |"
        while IFS=$'\t' read -r func role instructions reason; do
            printf '| `%s` | %s | %s | %s |\n' "${func}" "${role}" "${instructions}" "${reason}"
        done < "${plan_tsv}"
        echo
    } >> "${RESULTS_MD}"
}

run_case() {
    local test_name="$1"
    local case_name="$2"
    local proof_dir="$3"
    local log_file="$4"
    local workers="$5"
    local use_cse="$6"
    local summary_store="$7"
    local start_symbol="${START_PREFIX}${test_name}"
    local proof_id="${ARTIFACT_BASENAME}.smir.${start_symbol}"

    if [[ "${RESUME_PROOFS}" == false ]]; then
        rm -rf "${proof_dir}"
    fi
    mkdir -p "${proof_dir}"

    local cse_args=()
    if [[ "${use_cse}" == true ]]; then
        for func in "${CSE_FUNCTIONS[@]}"; do
            cse_args+=(--cse-function "${func}")
        done
        cse_args+=(--cse-summary-store "${summary_store}")
    fi

    local reload_args=()
    if [[ "${RESUME_PROOFS}" == false ]]; then
        reload_args+=(--reload)
    fi

    local mirror_break_args=()
    if [[ "${use_cse}" == false && "${MIRROR_CSE_BREAKPOINTS}" == true ]]; then
        for func in "${CSE_FUNCTIONS[@]}"; do
            mirror_break_args+=(--break-on-function "${func}")
        done
    fi

    echo "[INFO] Running ${test_name} / ${case_name}"
    echo "[INFO] Proof dir: ${proof_dir}"
    echo "[INFO] Log: ${log_file}"

    local start end rc
    start="$(date +%s)"
    set +e
    timeout --kill-after="${TIMEOUT_KILL_AFTER}" --preserve-status -v "${TIMEOUT}" \
        uv --project "${KMIR_PROJECT}" run -- \
        kmir prove-rs --smir "${SMIR_FILE}" \
        --proof-dir "${proof_dir}" --verbose --start-symbol "${start_symbol}" \
        "${reload_args[@]}" --max-workers "${workers}" \
        "${PROVE_OPTS[@]}" "${mirror_break_args[@]}" "${cse_args[@]}" \
        2>&1 | tee "${log_file}"
    rc="${PIPESTATUS[0]}"
    set -e
    end="$(date +%s)"

    write_case_result "${test_name}" "${case_name}" "${rc}" "$((end - start))" "${summary_store}" "${proof_dir}" "${log_file}"

    timeout --kill-after="${TIMEOUT_KILL_AFTER}" --preserve-status -v "${SHOW_TIMEOUT}" \
        uv --project "${KMIR_PROJECT}" run -- \
        kmir show --proof-dir "${proof_dir}" "${proof_id}" --statistics --leaves \
        > "${proof_dir}/${test_name}.show.txt" 2>&1 || true

    return "${rc}"
}

is_timeout_rc() {
    local rc="$1"
    [[ "${rc}" == "124" || "${rc}" == "137" || "${rc}" == "143" ]]
}

{
    echo "# CSE benchmark"
    echo
    echo "- run_id: \`${RUN_ID}\`"
    echo "- tests: \`${TESTS[*]}\`"
    echo "- repo_commit: \`${REPO_COMMIT}\`"
    echo "- mir_commit: \`${MIR_COMMIT}\`"
    echo "- smir: \`${SMIR_FILE}\`"
    echo "- timeout_seconds: \`${TIMEOUT}\`"
    echo "- show_timeout_seconds: \`${SHOW_TIMEOUT}\`"
    echo "- timeout_kill_after: \`${TIMEOUT_KILL_AFTER}\`"
    echo "- cse_workers: \`${MAX_WORKERS}\`"
    echo "- baseline_workers: \`${BASELINE_MAX_WORKERS}\`"
    echo "- prove_opts: \`${PROVE_OPTS[*]}\`"
    echo "- min_mir_instructions: \`${MIN_INSTRUCTIONS}\`"
    echo "- max_auto_cse_functions: \`${MAX_CSE_FUNCTIONS}\`"
    echo "- force_largest_process: \`${FORCE_LARGEST_PROCESS}\`"
    echo "- include_start_function: \`${INCLUDE_START_FUNCTION}\`"
    echo "- mirror_cse_breakpoints: \`${MIRROR_CSE_BREAKPOINTS}\`"
    echo "- resume_proofs: \`${RESUME_PROOFS}\`"
    echo "- extra_cse_functions: \`${EXTRA_CSE_FUNCTIONS[*]:-}\`"
    echo
    echo "## CSE plans"
    echo
} > "${RESULTS_MD}"

overall_rc=0

for test_name in "${TESTS[@]}"; do
    test_root="${BENCH_ROOT}/${test_name}"
    plan_tsv="${test_root}/cse-plan.tsv"
    summary_store="${test_root}/summary-store"
    mkdir -p "${test_root}" "${summary_store}"

    infer_cse_plan "${test_name}" "${plan_tsv}"
    load_cse_functions "${plan_tsv}"
    append_plan_markdown "${test_name}" "${plan_tsv}"

    if [[ "${PLAN_ONLY}" == true ]]; then
        continue
    fi

    if [[ "${#CSE_FUNCTIONS[@]}" -eq 0 ]]; then
        echo "[INFO] No CSE targets selected for ${test_name}; skipping CSE runs."
        write_skipped_result "${test_name}" "cold-cse" "${summary_store}"
        write_skipped_result "${test_name}" "warm-cse" "${summary_store}"
    else
        cold_rc=0
        run_case "${test_name}" "cold-cse" "${test_root}/proof-cold-cse" "${test_root}/cold-cse.log" "${MAX_WORKERS}" true "${summary_store}" || cold_rc=$?
        if is_timeout_rc "${cold_rc}"; then
            mark_summary_store_state "${summary_store}" "${test_name}" "cold-cse" "${cold_rc}" "${test_root}/proof-cold-cse" "${test_root}/cold-cse.log" false
        else
            mark_summary_store_state "${summary_store}" "${test_name}" "cold-cse" "${cold_rc}" "${test_root}/proof-cold-cse" "${test_root}/cold-cse.log" true
        fi
        if [[ "${cold_rc}" != "0" ]]; then
            overall_rc="${cold_rc}"
        fi
        if is_timeout_rc "${cold_rc}"; then
            echo "[INFO] Skipping ${test_name} / warm-cse because cold-cse timed out or was signaled (${cold_rc}); summary store may be incomplete."
            write_skipped_result "${test_name}" "warm-cse" "${summary_store}"
        else
            run_case "${test_name}" "warm-cse" "${test_root}/proof-warm-cse" "${test_root}/warm-cse.log" "${MAX_WORKERS}" true "${summary_store}" || overall_rc=$?
        fi
    fi

    run_case "${test_name}" "no-cse" "${test_root}/proof-no-cse" "${test_root}/no-cse.log" "${BASELINE_MAX_WORKERS}" false "${summary_store}" || overall_rc=$?
done

if [[ "${PLAN_ONLY}" == false ]]; then
    {
        echo "## Results"
        echo
        echo "| test | case | exit | seconds | status | nodes | pending | failing | stuck | terminal | summaries | traces | subsumptions | hits | misses | subsume hits | subsume misses | execute | implies |"
        echo "| --- | --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
        tail -n +2 "${RESULTS_CSV}" | while IFS=, read -r test_name case_name exit_code duration status nodes pending failing stuck terminal summaries traces subsumptions _bytes hits misses subsume_hits subsume_misses executes implies _functions _proof_dir _log_file; do
            printf '| %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s | %s |\n' \
                "${test_name}" "${case_name}" "${exit_code}" "${duration}" "${status}" \
                "${nodes:- }" "${pending:- }" "${failing:- }" "${stuck:- }" "${terminal:- }" \
                "${summaries:-0}" "${traces:-0}" "${subsumptions:-0}" "${hits:-0}" "${misses:-0}" \
                "${subsume_hits:-0}" "${subsume_misses:-0}" "${executes:-0}" "${implies:-0}"
        done
        echo
        echo "CSV: \`${RESULTS_CSV}\`"
        echo
        echo "Root: \`${BENCH_ROOT}\`"
    } >> "${RESULTS_MD}"
else
    {
        echo "Plan only; proofs were not run."
        echo
        echo "Root: \`${BENCH_ROOT}\`"
    } >> "${RESULTS_MD}"
fi

cat "${RESULTS_MD}"
exit "${overall_rc}"
