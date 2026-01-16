#!/usr/bin/env python3
"""
Evaluate PR #907: --to-module, --minimize-proof, --add-module

Usage: python evaluate.py [-n TEST_NAME] [-a] [-p] [-s] [--plot-only] [--format FORMAT]
  -n NAME      : single test name (default: test_process_get_account_data_size)
  -a           : run all tests from proofs.md (excluding certain tests)
  -p           : skip proof generation (use existing proof)
  -s           : skip tests that already have results in JSON
  --plot-only  : only generate plot and stats from existing results
  --format FMT : output format for plot: png (default), pdf, or svg
"""
import argparse
import json
import math
import os
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Any

# Tests to exclude when using -a
EXCLUDED_TESTS = {
}


def get_all_tests_from_proofs_md(proofs_md_path: Path) -> list[str]:
    """Parse proofs.md and extract all test names from the first table."""
    if not proofs_md_path.exists():
        print(f"[ERROR] proofs.md not found at {proofs_md_path}")
        sys.exit(1)

    tests = []
    with open(proofs_md_path) as f:
        for line in f:
            match = re.match(r'^\| (test_p[a-zA-Z0-9:_]*) *\|', line)
            if match:
                tests.append(match.group(1))
    return tests


def get_git_commit(path: Path | None = None) -> str:
    """Get short git commit hash."""
    try:
        cmd = ["git", "rev-parse", "--short", "HEAD"]
        if path:
            cmd = ["git", "-C", str(path), "rev-parse", "--short", "HEAD"]
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return "unknown"


def extract_time_from_output(output: str) -> int | None:
    """Extract execution time from log output, starting from 'Starting KoreServer'."""
    lines = output.splitlines()

    # Find "Starting KoreServer" timestamp
    start_ts = None
    for line in lines:
        if "Starting KoreServer" in line:
            m = re.search(r'(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})', line)
            if m:
                start_ts = datetime.strptime(m.group(1), "%Y-%m-%d %H:%M:%S")
                break

    # Find last timestamp
    end_ts = None
    for line in reversed(lines):
        m = re.search(r'(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})', line)
        if m:
            end_ts = datetime.strptime(m.group(1), "%Y-%m-%d %H:%M:%S")
            break

    if start_ts and end_ts:
        return int((end_ts - start_ts).total_seconds())
    return None


def run_kmir(args: list[str], cwd: Path, log_file: Path | None = None) -> tuple[int, str]:
    """Run kmir command and return (returncode, output)."""
    cmd = ["uv", "--project", "mir-semantics/kmir", "run", "--", "kmir"] + args
    print(f"Running: {' '.join(cmd)}")

    process = subprocess.Popen(
        cmd,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )

    output_lines = []
    if process.stdout:
        for line in process.stdout:
            print(line, end="")
            output_lines.append(line)

    process.wait()
    output = "".join(output_lines)

    # Save output to log file if specified
    if log_file:
        log_file.parent.mkdir(parents=True, exist_ok=True)
        with open(log_file, "w") as f:
            f.write(output)

    return process.returncode, output


def run_single_test(
    test_name: str,
    script_dir: Path,
    proof_dir: Path,
    module_dir: Path,
    augmented_proof_dir: Path,
    logs_dir: Path,
    smir_file: Path,
    start_prefix: str,
    artifact_basename: str,
    prove_opts: list[str],
    skip_proof: bool,
) -> dict[str, Any]:
    """Run a single test and return timing results."""
    start_symbol = f"{start_prefix}{test_name}"
    proof_id = f"{artifact_basename}.smir.{start_symbol}"
    module_file = module_dir / f"{test_name}.json"
    test_augmented_proof_dir = augmented_proof_dir / test_name

    # Create test-specific augmented proof dir
    test_augmented_proof_dir.mkdir(parents=True, exist_ok=True)

    times: dict[str, int | None] = {}
    results: dict[str, Any] = {
        "test_name": test_name,
        "start_symbol": start_symbol,
        "timestamp": datetime.now().isoformat(),
    }

    # Step 1: Generate proof (original semantics)
    if not skip_proof:
        print(f"\n=== Step 1: Generating proof for {test_name} ===")
        log_file = logs_dir / f"{test_name}_step1_prove.log"
        ret, output = run_kmir(
            ["prove-rs", "--smir", str(smir_file), "--proof-dir", str(proof_dir),
             "--verbose", "--start-symbol", start_symbol, "--reload"] + prove_opts,
            cwd=script_dir,
            log_file=log_file,
        )
        if ret != 0:
            print(f"[ERROR] Step 1 failed with return code {ret}")
            results["error"] = f"Step 1 failed with return code {ret}"
            results["step1_time"] = None
            return results
        times["step1"] = extract_time_from_output(output)
        if times["step1"] is not None:
            print(f"Step 1 execution time: {times['step1']}s")

    # Step 2: Export minimized module
    print(f"\n=== Step 2: Exporting module with --to-module --minimize-proof ===")
    log_file = logs_dir / f"{test_name}_step2_export.log"
    ret, output = run_kmir(
        ["show", "--proof-dir", str(proof_dir), proof_id,
         "--to-module", str(module_file), "--minimize-proof"],
        cwd=script_dir,
        log_file=log_file,
    )
    if ret != 0:
        print(f"[ERROR] Step 2 failed with return code {ret}")
        results["error"] = f"Step 2 failed with return code {ret}"
        return results
    times["step2"] = extract_time_from_output(output)
    if times["step2"] is not None:
        print(f"Step 2 execution time: {times['step2']}s")

    if not module_file.exists():
        print(f"[ERROR] Module file was not created: {module_file}")
        results["error"] = f"Module file was not created: {module_file}"
        return results

    # Step 3: Re-run with --add-module (summarized semantics)
    print(f"\n=== Step 3: Re-running proof with --add-module ===")
    log_file = logs_dir / f"{test_name}_step3_augmented.log"
    ret, output = run_kmir(
        ["prove-rs", "--smir", str(smir_file), "--proof-dir", str(test_augmented_proof_dir),
         "--verbose", "--start-symbol", start_symbol, "--reload",
         "--add-module", str(module_file)] + prove_opts,
        cwd=script_dir,
        log_file=log_file,
    )
    if ret != 0:
        print(f"[ERROR] Step 3 failed with return code {ret}")
        results["error"] = f"Step 3 failed with return code {ret}"
        return results
    times["step3"] = extract_time_from_output(output)
    if times["step3"] is not None:
        print(f"Step 3 execution time: {times['step3']}s")

    # Populate results
    results["step1_time"] = times.get("step1")  # Original semantics time
    results["step2_time"] = times.get("step2")
    results["step3_time"] = times.get("step3")  # Summarized semantics time
    results["module_file"] = str(module_file)
    results["augmented_proof_dir"] = str(test_augmented_proof_dir)

    return results


def calculate_statistics(results: list[dict[str, Any]]) -> dict[str, Any]:
    """Calculate aggregate statistics from results."""
    # Filter valid results with both times
    valid_results = [
        r for r in results
        if r.get("step1_time") is not None and r.get("step3_time") is not None
    ]

    if not valid_results:
        return {"error": "No valid results to calculate statistics"}

    n = len(valid_results)
    speedups = []
    wins = 0
    ties = 0
    losses = 0

    for r in valid_results:
        original = r["step1_time"]
        summarized = r["step3_time"]
        if summarized > 0:
            speedup = original / summarized
            speedups.append(speedup)

            # Wins/Ties/Losses with 1% tolerance for ties
            if speedup > 1.01:
                wins += 1
            elif speedup < 0.99:
                losses += 1
            else:
                ties += 1

    if not speedups:
        return {"error": "No valid speedups calculated"}

    # Geomean
    log_sum = sum(math.log(s) for s in speedups)
    geomean = math.exp(log_sum / len(speedups))

    # Median
    sorted_speedups = sorted(speedups)
    mid = len(sorted_speedups) // 2
    if len(sorted_speedups) % 2 == 0:
        median = (sorted_speedups[mid - 1] + sorted_speedups[mid]) / 2
    else:
        median = sorted_speedups[mid]

    # P90
    p90_idx = int(len(sorted_speedups) * 0.9)
    p90 = sorted_speedups[min(p90_idx, len(sorted_speedups) - 1)]

    return {
        "n": n,
        "geomean": round(geomean, 3),
        "median": round(median, 3),
        "p90": round(p90, 3),
        "wins": wins,
        "ties": ties,
        "losses": losses,
        "speedups": speedups,
    }


def generate_plot(results: list[dict[str, Any]], output_file: Path):
    """Generate scatter plot comparing original vs summarized semantics time."""
    try:
        import matplotlib.pyplot as plt
    except ImportError:
        print("[WARNING] matplotlib not installed, skipping plot generation")
        print("Install with: pip install matplotlib")
        return

    # Filter valid results
    valid_results = [
        r for r in results
        if r.get("step1_time") is not None and r.get("step3_time") is not None
    ]

    if not valid_results:
        print("[WARNING] No valid results to plot")
        return

    original_times = [r["step1_time"] for r in valid_results]
    summarized_times = [r["step3_time"] for r in valid_results]

    plt.figure(figsize=(10, 8))

    # Plot scatter points
    plt.scatter(original_times, summarized_times, color='blue', alpha=0.7, s=50)

    # Plot y=x line (no improvement line)
    max_time = max(max(original_times), max(summarized_times)) * 1.1
    plt.plot([0, max_time], [0, max_time], 'r--', linewidth=2, label='No improvement')

    plt.xlabel('Original Semantics Time (seconds)', fontsize=12)
    plt.ylabel('Summarized Semantics Time (seconds)', fontsize=12)
    plt.legend(loc='upper left')
    plt.xlim(0, max_time)
    plt.ylim(0, max_time)
    plt.grid(True, alpha=0.3)

    plt.tight_layout()
    # Determine format based on file extension
    ext = output_file.suffix.lower()
    if ext == '.pdf':
        plt.savefig(output_file, format='pdf', bbox_inches='tight')
    elif ext == '.svg':
        plt.savefig(output_file, format='svg', bbox_inches='tight')
    else:
        plt.savefig(output_file, dpi=150)
    plt.close()
    print(f"Plot saved to: {output_file}")


def print_summary_table(stats: dict[str, Any]):
    """Print summary table similar to Table 2."""
    print("\n" + "=" * 70)
    print("Table: Aggregate performance improvements")
    print("Wins/Ties/Losses count pairs with speedup >1.0, ~1.0 (+/-1%), and <1.0")
    print("=" * 70)
    print(f"{'Scenario':<12} {'N':>6} {'Geomean':>10} {'Median':>10} {'p90':>10} {'Wins/Ties/Losses':>18}")
    print("-" * 70)
    print(f"{'Symbolic':<12} {stats['n']:>6} {stats['geomean']:>10.3f} {stats['median']:>10.3f} {stats['p90']:>10.3f} {stats['wins']}/{stats['ties']}/{stats['losses']:>12}")
    print("=" * 70)


def main():
    parser = argparse.ArgumentParser(description="Evaluate PR #907")
    parser.add_argument("-n", "--name", default="test_process_get_account_data_size",
                        help="Test name (default: test_process_get_account_data_size)")
    parser.add_argument("-a", "--all", action="store_true",
                        help="Run all tests from proofs.md (excluding certain tests)")
    parser.add_argument("-p", "--skip-proof", action="store_true",
                        help="Skip proof generation (use existing proof)")
    parser.add_argument("-s", "--skip-existing", action="store_true",
                        help="Skip tests that already have results in JSON")
    parser.add_argument("--plot-only", action="store_true",
                        help="Only generate plot and stats from existing results")
    parser.add_argument("--format", choices=["png", "pdf", "svg"], default="png",
                        help="Output format for plot (default: png)")
    args = parser.parse_args()

    # Configuration
    start_prefix = os.environ.get("START_PREFIX", "pinocchio_token_program::entrypoint::")
    artifact_basename = os.environ.get("ARTIFACT_BASENAME", "p-token")

    # Paths
    script_dir = Path(__file__).parent.resolve()
    os.chdir(script_dir)

    repo_commit = get_git_commit()
    mir_commit = get_git_commit(script_dir / "mir-semantics")

    artifacts_dir = Path(os.environ.get("ARTIFACTS_DIR", "artefacts"))
    proof_dir = artifacts_dir / f"proof-{repo_commit}-{mir_commit}"
    module_dir = artifacts_dir / "modules-pr907"
    smir_file = artifacts_dir / f"{artifact_basename}.smir.json"
    augmented_proof_dir = artifacts_dir / "proof-pr907-augmented"
    logs_dir = artifacts_dir / "logs-pr907"
    results_file = artifacts_dir / "evaluation_results.json"
    plot_file = artifacts_dir / f"evaluation_plot.{args.format}"

    # Create directories
    proof_dir.mkdir(parents=True, exist_ok=True)
    module_dir.mkdir(parents=True, exist_ok=True)
    augmented_proof_dir.mkdir(parents=True, exist_ok=True)
    logs_dir.mkdir(parents=True, exist_ok=True)

    prove_opts = ["--max-iterations", "10000", "--max-depth", "10000"]

    # Plot only mode
    if args.plot_only:
        if not results_file.exists():
            print(f"[ERROR] Results file not found: {results_file}")
            sys.exit(1)
        with open(results_file) as f:
            all_results = json.load(f)
        stats = calculate_statistics(all_results)
        if "error" not in stats:
            generate_plot(all_results, plot_file)
            print_summary_table(stats)
        else:
            print(f"[ERROR] {stats['error']}")
        return

    # Determine tests to run
    if args.all:
        proofs_md_path = script_dir / "proofs.md"
        all_tests = get_all_tests_from_proofs_md(proofs_md_path)
        tests = [t for t in all_tests if t not in EXCLUDED_TESTS]
        print(f"Running {len(tests)} tests (excluded: {EXCLUDED_TESTS})")
    else:
        tests = [args.name]

    all_results: list[dict[str, Any]] = []

    # Load existing results if any
    if results_file.exists():
        try:
            with open(results_file) as f:
                all_results = json.load(f)
        except json.JSONDecodeError:
            all_results = []

    # Build set of completed test names for quick lookup
    completed_tests = set()
    if args.skip_existing:
        for r in all_results:
            if (r.get("test_name") and
                r.get("step1_time") is not None and
                r.get("step3_time") is not None and
                "error" not in r):
                completed_tests.add(r["test_name"])
        skipped = [t for t in tests if t in completed_tests]
        if skipped:
            print(f"[INFO] Will skip {len(skipped)} tests with existing results: {skipped}")

    # Run tests
    for i, test_name in enumerate(tests, 1):
        print(f"\n{'#' * 70}")
        print(f"# Test {i}/{len(tests)}: {test_name}")
        print(f"{'#' * 70}")

        # Skip if already has valid results
        if args.skip_existing and test_name in completed_tests:
            print(f"[SKIP] Test already has results in JSON, skipping...")
            continue

        result = run_single_test(
            test_name=test_name,
            script_dir=script_dir,
            proof_dir=proof_dir,
            module_dir=module_dir,
            augmented_proof_dir=augmented_proof_dir,
            logs_dir=logs_dir,
            smir_file=smir_file,
            start_prefix=start_prefix,
            artifact_basename=artifact_basename,
            prove_opts=prove_opts,
            skip_proof=args.skip_proof,
        )

        # Update or add result
        existing_idx = next(
            (i for i, r in enumerate(all_results) if r.get("test_name") == test_name),
            None
        )
        if existing_idx is not None:
            all_results[existing_idx] = result
        else:
            all_results.append(result)

        # Save results after each test
        with open(results_file, "w") as f:
            json.dump(all_results, f, indent=2)
        print(f"Results saved to: {results_file}")

        # Print individual test summary
        print(f"\n--- Summary for {test_name} ---")
        if result.get("step1_time") is not None:
            print(f"Original semantics time (Step 1):    {result['step1_time']}s")
        if result.get("step2_time") is not None:
            print(f"Module export time (Step 2):         {result['step2_time']}s")
        if result.get("step3_time") is not None:
            print(f"Summarized semantics time (Step 3):  {result['step3_time']}s")
        if result.get("step1_time") and result.get("step3_time"):
            speedup = result["step1_time"] / result["step3_time"]
            print(f"Speedup:                             {speedup:.2f}x")

    # Final summary
    print("\n" + "=" * 70)
    print("EVALUATION COMPLETE")
    print("=" * 70)

    stats = calculate_statistics(all_results)
    if "error" not in stats:
        generate_plot(all_results, plot_file)
        print_summary_table(stats)

        # Save stats to JSON
        stats_file = artifacts_dir / "evaluation_stats.json"
        with open(stats_file, "w") as f:
            json.dump(stats, f, indent=2)
        print(f"\nStatistics saved to: {stats_file}")
    else:
        print(f"[WARNING] {stats['error']}")

    print(f"Results saved to: {results_file}")
    print(f"Logs saved to: {logs_dir}")


if __name__ == "__main__":
    main()
