#!/usr/bin/env python3
"""
Evaluate CSE (Compositional Symbolic Execution) for KMIR on SPL Token.

This script measures three stages for each proof target:
1. baseline: prove without summaries
2. cse_gen: prove with --cse while generating/reusing shared summaries
3. cse_reuse: prove using summaries only

The primary acceptance metric is the amortized suite-level relationship:
    reuse_total ~= baseline_total - callee_sum_total

All timing calculations use proof execution_time from proof.json / cse_result.json
when available, with log parsing only as a fallback.
"""

from __future__ import annotations

import sys
sys.setrecursionlimit(50000)  # Large KCFG JSON parsing needs deep recursion

import argparse
import json
import math
import os
import re
import shutil
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Any

EXCLUDED_TESTS: set[str] = set()
DEFAULT_TARGET_RELATIVE_ERROR = 0.15
DEFAULT_PROVE_OPTS = ['--max-iterations', '10000', '--max-depth', '10000']


def get_proof_groups_from_proofs_md(proofs_md_path: Path) -> dict[str, list[str]]:
    """Parse proofs.md and extract grouped proof names."""
    if not proofs_md_path.exists():
        print(f'[ERROR] proofs.md not found at {proofs_md_path}')
        sys.exit(1)

    regular: list[str] = []
    multisig: list[str] = []
    section = 'regular'
    with open(proofs_md_path) as f:
        for line in f:
            if line.startswith('Proofs to run with `run-proofs.sh -m`'):
                section = 'multisig'
                continue
            regular_match = re.match(r'^\| (test_p[a-zA-Z0-9:_]*) *\|', line)
            multisig_match = re.match(r'^\| m \| (test_p[a-zA-Z0-9:_]*) *\|', line)
            if section == 'regular' and regular_match:
                regular.append(regular_match.group(1))
            elif section == 'multisig' and multisig_match:
                multisig.append(multisig_match.group(1))
    return {
        'regular': regular,
        'multisig': multisig,
        'all': regular + multisig,
    }


def extract_time_from_output(output: str) -> int | None:
    """Extract execution time from log output, starting from 'Starting KoreServer'."""
    lines = output.splitlines()
    start_ts = None
    for line in lines:
        if 'Starting KoreServer' in line:
            match = re.search(r'(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})', line)
            if match:
                start_ts = datetime.strptime(match.group(1), '%Y-%m-%d %H:%M:%S')
                break
    end_ts = None
    for line in reversed(lines):
        match = re.search(r'(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})', line)
        if match:
            end_ts = datetime.strptime(match.group(1), '%Y-%m-%d %H:%M:%S')
            break
    if start_ts and end_ts:
        return int((end_ts - start_ts).total_seconds())
    return None


def _git_rev_parse(repo: Path) -> str:
    try:
        proc = subprocess.run(
            ['git', '-C', str(repo), 'rev-parse', '--short', 'HEAD'],
            check=True,
            capture_output=True,
            text=True,
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return 'unknown'
    return proc.stdout.strip() or 'unknown'


def run_kmir(
    args: list[str],
    cwd: Path,
    log_file: Path | None = None,
    env_overrides: dict[str, str] | None = None,
) -> tuple[int, str]:
    """Run a kmir command and return (returncode, output)."""
    cmd = ['uv', '--project', 'mir-semantics/kmir', 'run', '--', 'kmir'] + args
    print(f"  Running: {' '.join(cmd[:8])}...")
    env = os.environ.copy()
    if env_overrides:
        env.update(env_overrides)
    process = subprocess.Popen(
        cmd,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    output_lines = []
    if process.stdout:
        for line in process.stdout:
            output_lines.append(line)
    process.wait()
    output = ''.join(output_lines)

    if log_file is not None:
        log_file.parent.mkdir(parents=True, exist_ok=True)
        log_file.write_text(output)
    return process.returncode, output


def _reset_proof_artifacts(proof_dir: Path, proof_id: str) -> None:
    proof_dir.mkdir(parents=True, exist_ok=True)
    target = proof_dir / proof_id
    if target.exists():
        subprocess.run(['rm', '-rf', str(target)], check=False)


def _find_proof_json(proof_dir: Path) -> Path | None:
    if not proof_dir.exists():
        return None
    proof_files = sorted(
        path
        for path in proof_dir.rglob('proof.json')
        if 'cse-callee-proofs' not in path.parts
    )
    if not proof_files:
        return None
    return min(proof_files, key=lambda path: (len(path.parts), len(str(path))))


def _read_json(path: Path) -> dict[str, Any] | None:
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError:
        return None
    return data if isinstance(data, dict) else None


def _ensure_smir_input(artifacts_dir: Path, artifact_basename: str, script_dir: Path) -> Path:
    smir_file = artifacts_dir / f'{artifact_basename}.smir.json'
    if smir_file.exists():
        return smir_file.resolve()

    default_smir = script_dir / 'artefacts' / f'{artifact_basename}.smir.json'
    if default_smir.exists():
        artifacts_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(default_smir, smir_file)
        return smir_file.resolve()

    raise FileNotFoundError(f'Could not find SMIR input at {smir_file} or fallback {default_smir}')


def _read_exec_time_from_proof_dir(proof_dir: Path) -> float | None:
    proof_json = _find_proof_json(proof_dir)
    if proof_json is None:
        return None
    proof_data = _read_json(proof_json)
    if proof_data is None:
        return None
    exec_time = proof_data.get('execution_time')
    if exec_time is None:
        return None
    return round(float(exec_time), 3)


def _read_cse_result(proof_dir: Path) -> dict[str, Any] | None:
    return _read_json(proof_dir / 'cse_result.json')


def _observed_call_files(summary_dir: Path) -> list[Path]:
    observed_dir = summary_dir / 'observed-calls'
    if not observed_dir.exists():
        return []
    return sorted(path for path in observed_dir.glob('ty-*.json') if not path.name.endswith('.cterm.json'))


def _summary_files(summary_dir: Path) -> list[Path]:
    return sorted(path for path in summary_dir.glob('*.json') if not path.name.endswith('.skip.json'))


def _read_kcfg_summary(proof_dir: Path) -> dict[str, Any]:
    kcfg_files = sorted(proof_dir.rglob('kcfg/kcfg.json'))
    if not kcfg_files:
        return {}
    data = _read_json(kcfg_files[0])
    if data is None:
        return {}
    nodes = data.get('nodes')
    edges = data.get('edges')
    splits = data.get('splits')
    covers = data.get('covers')
    stuck = data.get('stuck')
    vacuous = data.get('vacuous')
    ndbranches = data.get('ndbranches')
    return {
        'kcfg_path': str(kcfg_files[0]),
        'node_count': len(nodes) if isinstance(nodes, list) else None,
        'edge_count': len(edges) if isinstance(edges, list) else None,
        'split_count': len(splits) if isinstance(splits, list) else None,
        'ndbranch_count': len(ndbranches) if isinstance(ndbranches, list) else 0,
        'cover_count': len(covers) if isinstance(covers, list) else None,
        'edge_depth_sum': sum(edge.get('depth', 0) for edge in edges) if isinstance(edges, list) else None,
        'stuck_nodes': stuck if isinstance(stuck, list) else [],
        'vacuous_nodes': vacuous if isinstance(vacuous, list) else [],
    }


def _skeleton_preserved(result: dict[str, Any]) -> bool | None:
    baseline_splits = result.get('baseline_split_count')
    reuse_splits = result.get('cse_reuse_split_count')
    reuse_ndbranches = result.get('cse_reuse_ndbranch_count', 0)
    if baseline_splits is None or reuse_splits is None:
        return None
    # CSE frontier summaries produce NDBranch for callee branches.  The total
    # branch points (splits + ndbranches) should be >= baseline splits because
    # CSE may introduce additional branching from multi-frontier callees while
    # preserving the caller's original splits.
    reuse_total_branches = reuse_splits + reuse_ndbranches
    return reuse_total_branches >= baseline_splits


def _render_paper_table_rows(results: list[dict[str, Any]]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for result in sorted(results, key=lambda item: item['test_name']):
        rows.append(
            {
                'suite': result.get('suite', 'regular'),
                'test_name': result['test_name'],
                'baseline_passed': result.get('baseline_passed'),
                'cse_passed': result.get('cse_passed'),
                'baseline_kore_time': result.get('baseline_kore_time'),
                'cse_reuse_kore_time': result.get('cse_reuse_kore_time'),
                'baseline_depth_sum': result.get('baseline_edge_depth_sum'),
                'reuse_depth_sum': result.get('cse_reuse_edge_depth_sum'),
                'skeleton_preserved': _skeleton_preserved(result),
                'summaries_used': result.get('summaries_used'),
                'failure_class': result.get('failure_class'),
            }
        )
    return rows


def _write_paper_table_artifacts(artifacts_dir: Path, results: list[dict[str, Any]]) -> None:
    rows = _render_paper_table_rows(results)
    csv_path = artifacts_dir / 'cse_paper_table.csv'
    tex_path = artifacts_dir / 'cse_paper_table.tex'

    header = [
        'suite',
        'test_name',
        'baseline_passed',
        'cse_passed',
        'baseline_kore_time',
        'cse_reuse_kore_time',
        'baseline_depth_sum',
        'reuse_depth_sum',
        'skeleton_preserved',
        'summaries_used',
        'failure_class',
    ]
    csv_lines = [','.join(header)]
    for row in rows:
        csv_lines.append(','.join(str(row.get(column, '')) for column in header))
    csv_path.write_text('\n'.join(csv_lines) + '\n')

    tex_lines = [
        r'\begin{table*}[t]',
        r'\centering',
        r'\caption{KMIR CSE results across SPL Token proof targets.}',
        r'\label{tab:kmir-cse-suite}',
        r'\begin{tabular}{llccrrrrcl}',
        r'\toprule',
        r'Suite & Test & Base & CSE & Base(s) & Reuse(s) & BaseDepth & ReuseDepth & Skeleton & Status \\',
        r'\midrule',
    ]
    for row in rows:
        tex_lines.append(
            ' & '.join(
                [
                    str(row['suite']),
                    str(row['test_name']).replace('_', r'\_'),
                    'Y' if row['baseline_passed'] else 'N',
                    'Y' if row['cse_passed'] else 'N',
                    '-' if row['baseline_kore_time'] is None else f"{float(row['baseline_kore_time']):.3f}",
                    '-' if row['cse_reuse_kore_time'] is None else f"{float(row['cse_reuse_kore_time']):.3f}",
                    '-' if row['baseline_depth_sum'] is None else str(row['baseline_depth_sum']),
                    '-' if row['reuse_depth_sum'] is None else str(row['reuse_depth_sum']),
                    '-' if row['skeleton_preserved'] is None else ('Y' if row['skeleton_preserved'] else 'N'),
                    str(row['failure_class']).replace('_', r'\_'),
                ]
            )
            + r' \\'
        )
    tex_lines.extend(
        [
            r'\bottomrule',
            r'\end{tabular}',
            r'\end{table*}',
            '',
        ]
    )
    tex_path.write_text('\n'.join(tex_lines))


def _classify_result(result: dict[str, Any], target_relative_error: float) -> str:
    if not result.get('baseline_passed'):
        return 'baseline correctness failure'
    if not result.get('cse_gen_passed', True):
        return 'reuse correctness failure'
    if not result.get('cse_passed', False):
        return 'reuse correctness failure'
    callee_total = result.get('callee_total_kore_time')
    if callee_total in (None, 0) and result.get('summaries_used', 0) == 0:
        return 'coverage failure'
    if result.get('relative_error') is None:
        return 'accounting mismatch'
    if float(result['relative_error']) > target_relative_error:
        return 'performance overhead too high'
    return 'accepted'


def _render_eval_md(
    *,
    tests: list[str],
    all_results: list[dict[str, Any]],
    stats: dict[str, Any],
    state: dict[str, Any],
) -> str:
    lines = ['# CSE Eval']
    lines.append('')
    lines.append(f"- Updated: {state['updated_at']}")
    lines.append(f"- Scope: {state['scope']['suite']} ({state['scope']['source']})")
    lines.append(f"- Requested tests this run: {len(tests)}")
    lines.append(f"- Latest valid test: {state.get('latest_valid_test')}")
    lines.append(f"- Main hypothesis: {state['main_hypothesis']}")
    lines.append('')
    lines.append('## Verdict')
    lines.append('')
    if 'error' in stats:
        lines.append(f"- Status: blocked")
        lines.append(f"- Reason: {stats['error']}")
    else:
        lines.append(f"- Status: {'pass' if stats['suite_within_target'] else 'fail'}")
        lines.append(
            f"- Suite metric: reuse_total={stats['reuse_total_s']}s vs "
            f"effective_target_total={stats['effective_target_total_s']}s "
            f"(relative_error={stats['suite_relative_error']})"
        )
    lines.append('')
    lines.append('## Findings')
    lines.append('')
    recent = sorted(all_results, key=lambda item: item.get('timestamp', ''))
    for result in recent[-min(len(recent), 8) :]:
        lines.append(
            f"- {result['test_name']}: {result.get('failure_class', 'unknown')}"
            f", baseline_passed={result.get('baseline_passed')}"
            f", cse_passed={result.get('cse_passed')}"
            f", relative_error={result.get('relative_error')}"
        )
    lines.append('')
    lines.append('## Next Action')
    lines.append('')
    lines.append(f"- {state['next_action']}")
    lines.append('')
    return '\n'.join(lines)


def _write_harness_artifacts(
    *,
    harness_dir: Path,
    suite_name: str,
    tests: list[str],
    all_results: list[dict[str, Any]],
    stats: dict[str, Any],
    target_relative_error: float,
    script_dir: Path,
    stop_after_first_valid: bool,
) -> None:
    harness_dir.mkdir(parents=True, exist_ok=True)
    baseline_passers = [result['test_name'] for result in all_results if result.get('baseline_passed')]
    accepted = [result['test_name'] for result in all_results if result.get('failure_class') == 'accepted']
    latest_valid = next(
        (
            result['test_name']
            for result in reversed(sorted(all_results, key=lambda item: item.get('timestamp', '')))
            if result.get('baseline_passed')
        ),
        None,
    )
    main_hypothesis = (
        'Need a baseline-passing regular proof before CSE accounting can be judged.'
        if not baseline_passers
        else 'Baseline correctness is sufficient for at least one regular proof; continue CSE correctness/accounting on passing candidates.'
    )
    next_action = (
        'Continue baseline discovery across the regular suite until a passing candidate is found.'
        if not baseline_passers
        else (
            f'Promote {latest_valid} as the sprint-0 candidate and iterate on CSE correctness/accounting.'
            if not accepted
            else 'Expand from accepted sprint-0 candidates to the wider regular suite.'
        )
    )
    state = {
        'updated_at': datetime.now().isoformat(),
        'scope': {
            'suite': suite_name,
            'source': str(script_dir / 'proofs.md'),
            'target_relative_error': target_relative_error,
            'stop_after_first_valid': stop_after_first_valid,
        },
        'commits': {
            'eval_repo': _git_rev_parse(script_dir.parent.parent),
            'embedded_mir_semantics': _git_rev_parse(script_dir / 'mir-semantics'),
            'canonical_mir_semantics_cse': _git_rev_parse(Path('/home/zhaoji/projs/mir-semantics-cse')),
        },
        'tests_requested': tests,
        'baseline_passing_tests': baseline_passers,
        'accepted_tests': accepted,
        'latest_valid_test': latest_valid,
        'failure_classes': sorted({result.get('failure_class', 'unknown') for result in all_results}),
        'main_hypothesis': main_hypothesis,
        'next_action': next_action,
    }
    (harness_dir / 'state.json').write_text(json.dumps(state, indent=2))
    (harness_dir / 'results.json').write_text(json.dumps(all_results, indent=2))
    (harness_dir / 'eval.md').write_text(
        _render_eval_md(tests=tests, all_results=all_results, stats=stats, state=state)
    )


def run_baseline(
    test_name: str,
    script_dir: Path,
    proof_dir: Path,
    logs_dir: Path,
    smir_file: Path,
    start_prefix: str,
    prove_opts: list[str],
) -> dict[str, Any]:
    start_symbol = f'{start_prefix}{test_name}'
    proof_id = f'{smir_file.stem}.{start_symbol}'
    _reset_proof_artifacts(proof_dir, proof_id)
    t0 = time.time()
    ret, output = run_kmir(
        [
            'prove-rs',
            '--smir',
            str(smir_file),
            '--proof-dir',
            str(proof_dir),
            '--verbose',
            '--start-symbol',
            start_symbol,
            '--reload',
            *prove_opts,
        ],
        cwd=script_dir,
        log_file=logs_dir / f'{test_name}_baseline.log',
        env_overrides={
            'KMIR_CSE_SUMMARY_GENERATION': '0',
            'KMIR_CSE_OBSERVE_ONLY': '0',
            'KMIR_CSE_REUSE_ONLY': '0',
            'KMIR_CSE_ONLINE_AUTOREUSE': '0',
        },
    )
    wall_time = round(time.time() - t0, 1)
    proof_exec_time = _read_exec_time_from_proof_dir(proof_dir)
    kore_time = proof_exec_time if proof_exec_time is not None else extract_time_from_output(output)
    kcfg_summary = _read_kcfg_summary(proof_dir)
    return {
        'returncode': ret,
        'wall_time': wall_time,
        'kore_time': kore_time,
        'passed': ret == 0,
        **kcfg_summary,
    }


def run_with_summaries(
    test_name: str,
    script_dir: Path,
    proof_dir: Path,
    logs_dir: Path,
    smir_file: Path,
    start_prefix: str,
    summary_dir: Path,
    prove_opts: list[str],
) -> dict[str, Any]:
    start_symbol = f'{start_prefix}{test_name}'
    proof_id = f'{smir_file.stem}.{start_symbol}'
    _reset_proof_artifacts(proof_dir, proof_id)

    t0 = time.time()
    ret, output = run_kmir(
        [
            'prove-rs',
            '--smir',
            str(smir_file),
            '--proof-dir',
            str(proof_dir),
            '--verbose',
            '--start-symbol',
            start_symbol,
            '--cse',
            '--summary-dir',
            str(summary_dir),
            *prove_opts,
        ],
        cwd=script_dir,
        log_file=logs_dir / f'{test_name}_cse_reuse.log',
        env_overrides={
            'KMIR_CSE_SUMMARY_GENERATION': '0',
            'KMIR_CSE_OBSERVE_ONLY': '0',
            'KMIR_CSE_REUSE_ONLY': '1',
            'KMIR_CSE_ONLINE_AUTOREUSE': '0',
        },
    )
    wall_time = round(time.time() - t0, 1)
    cse_result = _read_cse_result(proof_dir) or {}
    proof_exec_time = cse_result.get('final_proof_exec_time')
    if proof_exec_time is None:
        proof_exec_time = _read_exec_time_from_proof_dir(proof_dir)
    kore_time = proof_exec_time if proof_exec_time is not None else extract_time_from_output(output)
    kcfg_summary = _read_kcfg_summary(proof_dir)
    return {
        'returncode': ret,
        'wall_time': wall_time,
        'kore_time': kore_time,
        'passed': ret == 0,
        'summaries_used': len(_summary_files(summary_dir)),
        'cse_result_path': str(proof_dir / 'cse_result.json'),
        **kcfg_summary,
    }


def generate_summaries_via_cse(
    test_name: str,
    script_dir: Path,
    proof_dir: Path,
    summary_dir: Path,
    logs_dir: Path,
    smir_file: Path,
    start_prefix: str,
    prove_opts: list[str],
) -> dict[str, Any]:
    start_symbol = f'{start_prefix}{test_name}'
    proof_id = f'{smir_file.stem}.{start_symbol}'
    _reset_proof_artifacts(proof_dir, proof_id)
    observe_result: dict[str, Any] = {
        'observe_wall_time': None,
        'observe_passed': None,
        'observed_runtime_callees': len(_observed_call_files(summary_dir)),
    }

    t0 = time.time()
    ret, output = run_kmir(
        [
            'prove-rs',
            '--smir',
            str(smir_file),
            '--proof-dir',
            str(proof_dir),
            '--verbose',
            '--start-symbol',
            start_symbol,
            '--reload',
            '--cse',
            '--summary-dir',
            str(summary_dir),
            *prove_opts,
        ],
        cwd=script_dir,
        log_file=logs_dir / f'{test_name}_cse_gen.log',
        env_overrides={
            'KMIR_CSE_SUMMARY_GENERATION': os.environ.get('KMIR_CSE_SUMMARY_GENERATION', '1'),
            'KMIR_CSE_OBSERVE_ONLY': '0',
            'KMIR_CSE_REUSE_ONLY': '0',
            'KMIR_CSE_ONLINE_AUTOREUSE': '1',
        },
    )
    wall_time = round(time.time() - t0, 1)
    cse_result = _read_cse_result(proof_dir) or {}
    observe_result['observed_runtime_callees'] = len(_observed_call_files(summary_dir))
    summary_files = _summary_files(summary_dir)
    callee_times = cse_result.get('callee_times') if isinstance(cse_result.get('callee_times'), dict) else {}
    callee_results = cse_result.get('callee_results') if isinstance(cse_result.get('callee_results'), dict) else {}
    summarized_callees = [
        name
        for name, detail in callee_results.items()
        if isinstance(detail, dict) and detail.get('summary_path')
    ]
    return {
        'returncode': ret,
        'wall_time': wall_time,
        'passed': ret == 0,
        **observe_result,
        'total_summaries': len(summary_files),
        'callee_times': callee_times,
        'callee_total_kore_time': cse_result.get('callee_total_kore_time'),
        'callee_total_export_time': cse_result.get('callee_total_export_time'),
        'final_proof_exec_time': cse_result.get('final_proof_exec_time'),
        'final_proof_wall_time': cse_result.get('final_proof_wall_time'),
        'online_generated_summaries': cse_result.get('online_generated_summaries', []),
        'dynamic_summary_hits': cse_result.get('dynamic_summary_hits', {}),
        'skipped': cse_result.get('skipped', {}),
        'summaries_generated': len(summarized_callees),
        'cse_result_path': str(proof_dir / 'cse_result.json'),
        'log_fallback_kore_time': extract_time_from_output(output),
        **_read_kcfg_summary(proof_dir),
    }


def calculate_statistics(results: list[dict[str, Any]], target_relative_error: float) -> dict[str, Any]:
    """Calculate aggregate suite-level statistics from results."""
    valid = [
        result
        for result in results
        if (
            result.get('baseline_passed')
            and result.get('cse_passed')
            and result.get('baseline_kore_time') is not None
            and result.get('callee_total_kore_time') is not None
            and result.get('cse_reuse_kore_time') is not None
        )
    ]
    if not valid:
        return {'error': 'No valid results to calculate statistics'}

    speedups = []
    per_test_relative_errors = []
    within_target = 0
    for result in valid:
        baseline = float(result['baseline_kore_time'])
        reuse = float(result['cse_reuse_kore_time'])
        effective_target = float(result['effective_target_time'])
        relative_error = float(result['relative_error'])
        per_test_relative_errors.append(relative_error)
        if relative_error <= target_relative_error:
            within_target += 1
        if reuse > 0:
            speedups.append(baseline / reuse)
        if effective_target > 0:
            result['speedup_vs_effective_target'] = round(baseline / effective_target, 3)

    baseline_total = sum(float(result['baseline_kore_time']) for result in valid)
    callee_total = sum(float(result['callee_total_kore_time']) for result in valid)
    reuse_total = sum(float(result['cse_reuse_kore_time']) for result in valid)
    effective_target_total = baseline_total - callee_total
    suite_relative_error = abs(reuse_total - effective_target_total) / max(baseline_total, 1.0)

    geomean = None
    median = None
    if speedups:
        log_sum = sum(math.log(speedup) for speedup in speedups)
        geomean = round(math.exp(log_sum / len(speedups)), 3)
        sorted_speedups = sorted(speedups)
        mid = len(sorted_speedups) // 2
        median = (
            round((sorted_speedups[mid - 1] + sorted_speedups[mid]) / 2, 3)
            if len(sorted_speedups) % 2 == 0
            else round(sorted_speedups[mid], 3)
        )

    return {
        'n': len(valid),
        'target_relative_error': target_relative_error,
        'baseline_total_s': round(baseline_total, 3),
        'callee_total_s': round(callee_total, 3),
        'effective_target_total_s': round(effective_target_total, 3),
        'reuse_total_s': round(reuse_total, 3),
        'suite_relative_error': round(suite_relative_error, 3),
        'suite_within_target': suite_relative_error <= target_relative_error,
        'per_test_within_target': within_target,
        'per_test_total': len(valid),
        'per_test_mean_relative_error': round(sum(per_test_relative_errors) / len(per_test_relative_errors), 3),
        'baseline_vs_reuse_geomean_speedup': geomean,
        'baseline_vs_reuse_median_speedup': median,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description='Evaluate CSE for KMIR on SPL Token')
    parser.add_argument('-n', '--name', default='test_process_get_account_data_size')
    parser.add_argument(
        '--suite',
        choices=['regular', 'multisig', 'all'],
        default='regular',
        help='Select which proof group from proofs.md to evaluate',
    )
    parser.add_argument('-a', '--all', action='store_true', help='Alias for --suite all')
    parser.add_argument(
        '--find-first-valid',
        action='store_true',
        help='Scan the regular suite until the first baseline-passing test is found, then run full CSE on it.',
    )
    parser.add_argument('--max-tests', type=int, default=None, help='Optional cap on the number of tests to scan')
    parser.add_argument('-s', '--skip-existing', action='store_true')
    parser.add_argument('--plot-only', action='store_true')
    parser.add_argument('--format', choices=['png', 'pdf', 'svg'], default='png')
    args = parser.parse_args()

    start_prefix = os.environ.get('START_PREFIX', 'pinocchio_token_program::entrypoint::')
    artifact_basename = os.environ.get('ARTIFACT_BASENAME', 'p-token')
    target_relative_error = float(os.environ.get('TARGET_RELATIVE_ERROR', str(DEFAULT_TARGET_RELATIVE_ERROR)))

    script_dir = Path(__file__).parent.resolve()
    os.chdir(script_dir)

    artifacts_dir = Path(os.environ.get('ARTIFACTS_DIR', 'artefacts'))

    baseline_proof_dir = artifacts_dir / 'cse-eval-baseline-proofs'
    cse_proof_dir = artifacts_dir / 'cse-eval-cse-proofs'
    summary_dir = artifacts_dir / 'cse-eval-summaries'
    logs_dir = artifacts_dir / 'cse-eval-logs'
    results_file = artifacts_dir / 'cse_evaluation_results.json'
    stats_file = artifacts_dir / 'cse_evaluation_stats.json'
    harness_dir = artifacts_dir / 'cse-harness'

    for directory in [baseline_proof_dir, cse_proof_dir, summary_dir, logs_dir]:
        directory.mkdir(parents=True, exist_ok=True)

    smir_file = _ensure_smir_input(artifacts_dir, artifact_basename, script_dir)

    prove_opts = DEFAULT_PROVE_OPTS

    if args.plot_only:
        if not results_file.exists():
            print(f'[ERROR] Results file not found: {results_file}')
            sys.exit(1)
        all_results = json.loads(results_file.read_text())
        stats = calculate_statistics(all_results, target_relative_error)
        _write_harness_artifacts(
            harness_dir=harness_dir,
            suite_name='plot-only',
            tests=[],
            all_results=all_results,
            stats=stats,
            target_relative_error=target_relative_error,
            script_dir=script_dir,
            stop_after_first_valid=False,
        )
        _write_paper_table_artifacts(artifacts_dir, all_results)
        if 'error' in stats:
            print(f"[ERROR] {stats['error']}")
        else:
            print(json.dumps(stats, indent=2))
        return

    proof_groups = get_proof_groups_from_proofs_md(script_dir / 'proofs.md')
    selected_suite = 'all' if args.all else args.suite
    selected_tests = [test for test in proof_groups[selected_suite] if test not in EXCLUDED_TESTS]
    stop_after_first_valid = args.find_first_valid and not args.all
    tests = selected_tests if (args.all or args.find_first_valid or args.suite != 'regular') else [args.name]
    if args.max_tests is not None:
        tests = tests[: args.max_tests]

    all_results: list[dict[str, Any]] = []
    if results_file.exists():
        try:
            all_results = json.loads(results_file.read_text())
        except json.JSONDecodeError:
            all_results = []

    completed = {
        result['test_name']
        for result in all_results
        if (
            result.get('baseline_kore_time') is not None
            and result.get('callee_total_kore_time') is not None
            and result.get('cse_reuse_kore_time') is not None
            and 'error' not in result
        )
    }

    print(f'=== CSE Evaluation: {len(tests)} tests ===')
    print(f'Suite: {selected_suite}')
    print(f'Shared summary dir: {summary_dir}')
    print(f"Existing summaries: {len(_summary_files(summary_dir))}")
    print(f'Target suite relative error: {target_relative_error:.2%}')
    print()

    for index, test_name in enumerate(tests, 1):
        if args.skip_existing and test_name in completed:
            print(f'[{index}/{len(tests)}] {test_name}: SKIP (already has structured results)')
            continue

        print(f'[{index}/{len(tests)}] {test_name}:')
        result: dict[str, Any] = {
            'test_name': test_name,
            'suite': selected_suite if selected_suite != 'all' else ('multisig' if test_name in proof_groups['multisig'] else 'regular'),
            'timestamp': datetime.now().isoformat(),
            'target_relative_error': target_relative_error,
        }

        print('  Baseline proving...', flush=True)
        baseline = run_baseline(
            test_name,
            script_dir,
            baseline_proof_dir / test_name,
            logs_dir,
            smir_file,
            start_prefix,
            prove_opts,
        )
        result['baseline_wall_time'] = baseline['wall_time']
        result['baseline_kore_time'] = baseline['kore_time']
        result['baseline_passed'] = baseline['passed']
        print(
            f"  [Baseline] {test_name}: {'PASS' if baseline['passed'] else 'FAIL'} "
            f"wall={baseline['wall_time']}s kore={baseline['kore_time']}s"
        )
        result['baseline_stuck_nodes'] = baseline.get('stuck_nodes', [])
        result['baseline_vacuous_nodes'] = baseline.get('vacuous_nodes', [])
        result['baseline_node_count'] = baseline.get('node_count')
        result['baseline_edge_count'] = baseline.get('edge_count')
        result['baseline_split_count'] = baseline.get('split_count')
        result['baseline_cover_count'] = baseline.get('cover_count')
        result['baseline_edge_depth_sum'] = baseline.get('edge_depth_sum')
        result['baseline_kcfg_path'] = baseline.get('kcfg_path')

        if not baseline['passed']:
            result['error'] = 'baseline failed'
            result['failure_class'] = 'baseline correctness failure'
            all_results = [existing for existing in all_results if existing.get('test_name') != test_name]
            all_results.append(result)
            results_file.write_text(json.dumps(all_results, indent=2))
            continue

        pre_summaries = len(_summary_files(summary_dir))
        print('  CSE proving (with summary generation)...', flush=True)
        cse_gen = generate_summaries_via_cse(
            test_name,
            script_dir,
            cse_proof_dir / test_name,
            summary_dir,
            logs_dir,
            smir_file,
            start_prefix,
            prove_opts,
        )
        post_summaries = len(_summary_files(summary_dir))
        new_summaries = post_summaries - pre_summaries
        result['cse_gen_wall_time'] = cse_gen['wall_time']
        result['cse_gen_passed'] = cse_gen['passed']
        result['cse_observe_wall_time'] = cse_gen['observe_wall_time']
        result['cse_observe_passed'] = cse_gen['observe_passed']
        result['observed_runtime_callees'] = cse_gen['observed_runtime_callees']
        result['new_summaries_generated'] = new_summaries
        result['total_summaries_available'] = post_summaries
        result['callee_times'] = cse_gen['callee_times']
        result['callee_total_kore_time'] = cse_gen['callee_total_kore_time']
        result['callee_total_export_time'] = cse_gen['callee_total_export_time']
        result['cse_gen_main_proof_kore_time'] = cse_gen['final_proof_exec_time']
        result['cse_gen_main_proof_wall_time'] = cse_gen['final_proof_wall_time']
        result['cse_gen_skipped'] = cse_gen['skipped']
        result['cse_result_path'] = cse_gen['cse_result_path']
        result['cse_gen_stuck_nodes'] = cse_gen.get('stuck_nodes', [])
        result['cse_gen_node_count'] = cse_gen.get('node_count')
        result['cse_gen_edge_count'] = cse_gen.get('edge_count')
        result['cse_gen_split_count'] = cse_gen.get('split_count')
        result['cse_gen_cover_count'] = cse_gen.get('cover_count')
        result['cse_gen_edge_depth_sum'] = cse_gen.get('edge_depth_sum')
        cse_result_path = Path(cse_gen['cse_result_path'])
        if cse_result_path.exists():
            shutil.copy2(cse_result_path, cse_result_path.with_name('cse_result_gen.json'))
        print(
            f"  [CSE Observe] {test_name}: "
            f"{'PASS' if cse_gen['observe_passed'] else ('SKIP' if cse_gen['observe_passed'] is None else 'FAIL')} "
            f"wall={cse_gen['observe_wall_time']}s observed={cse_gen['observed_runtime_callees']}"
        )
        print(
            f"  [CSE Gen] {test_name}: {'PASS' if cse_gen['passed'] else 'FAIL'} "
            f"wall={cse_gen['wall_time']}s new_summaries={new_summaries} "
            f"callee_total={cse_gen['callee_total_kore_time']}s total={post_summaries}"
        )

        available_summaries = _summary_files(summary_dir)
        print(f'  CSE reuse proving ({len(available_summaries)} summaries)...', flush=True)
        cse_reuse = run_with_summaries(
            test_name,
            script_dir,
            cse_proof_dir / test_name,
            logs_dir,
            smir_file,
            start_prefix,
            summary_dir,
            prove_opts,
        )
        result['cse_reuse_wall_time'] = cse_reuse['wall_time']
        result['cse_reuse_kore_time'] = cse_reuse['kore_time']
        result['cse_passed'] = cse_reuse['passed']
        result['summaries_used'] = cse_reuse['summaries_used']
        result['cse_reuse_stuck_nodes'] = cse_reuse.get('stuck_nodes', [])
        result['cse_reuse_node_count'] = cse_reuse.get('node_count')
        result['cse_reuse_edge_count'] = cse_reuse.get('edge_count')
        result['cse_reuse_split_count'] = cse_reuse.get('split_count')
        result['cse_reuse_ndbranch_count'] = cse_reuse.get('ndbranch_count', 0)
        result['cse_reuse_cover_count'] = cse_reuse.get('cover_count')
        result['cse_reuse_edge_depth_sum'] = cse_reuse.get('edge_depth_sum')

        if baseline['kore_time'] is not None and cse_gen['callee_total_kore_time'] is not None:
            effective_target_time = round(float(baseline['kore_time']) - float(cse_gen['callee_total_kore_time']), 3)
            result['effective_target_time'] = effective_target_time
        else:
            result['effective_target_time'] = None

        if result['effective_target_time'] is not None and cse_reuse['kore_time'] is not None:
            delta = round(float(cse_reuse['kore_time']) - float(result['effective_target_time']), 3)
            result['delta_vs_effective_target'] = delta
            result['relative_error'] = round(
                abs(delta) / max(float(baseline['kore_time'] or 0), 1.0),
                3,
            )
            result['within_target'] = result['relative_error'] <= target_relative_error
        else:
            result['delta_vs_effective_target'] = None
            result['relative_error'] = None
            result['within_target'] = False

        if baseline['kore_time'] and cse_reuse['kore_time'] and cse_reuse['kore_time'] > 0:
            result['speedup_vs_baseline'] = round(float(baseline['kore_time']) / float(cse_reuse['kore_time']), 3)
        else:
            result['speedup_vs_baseline'] = None

        if baseline['kore_time'] and result['effective_target_time'] and result['effective_target_time'] > 0:
            result['speedup_vs_effective_target'] = round(
                float(baseline['kore_time']) / float(result['effective_target_time']),
                3,
            )
        else:
            result['speedup_vs_effective_target'] = None

        status = 'PASS' if cse_reuse['passed'] else 'FAIL'
        rel_error = result['relative_error']
        rel_error_str = f'{rel_error:.3f}' if rel_error is not None else 'N/A'
        print(
            f"  [CSE Reuse] {test_name}: {status} "
            f"wall={cse_reuse['wall_time']}s kore={cse_reuse['kore_time']}s "
            f"target={result['effective_target_time']}s rel_err={rel_error_str}"
        )
        print()

        result['failure_class'] = _classify_result(result, target_relative_error)
        all_results = [existing for existing in all_results if existing.get('test_name') != test_name]
        all_results.append(result)
        results_file.write_text(json.dumps(all_results, indent=2))
        if stop_after_first_valid:
            print(f'  Stopping after first baseline-passing candidate: {test_name}')
            break

    print('\n' + '=' * 70)
    print('CSE EVALUATION COMPLETE')
    print('=' * 70)

    stats = calculate_statistics(all_results, target_relative_error)
    if 'error' in stats:
        print(f"[WARNING] {stats['error']}")
    else:
        print(
            f"\nN={stats['n']}, baseline_total={stats['baseline_total_s']}s, "
            f"callee_total={stats['callee_total_s']}s, reuse_total={stats['reuse_total_s']}s"
        )
        print(
            f"effective_target_total={stats['effective_target_total_s']}s, "
            f"suite_relative_error={stats['suite_relative_error']}, within_target={stats['suite_within_target']}"
        )
        print(
            f"per_test_within_target={stats['per_test_within_target']}/{stats['per_test_total']}, "
            f"mean_relative_error={stats['per_test_mean_relative_error']}"
        )
        if stats['baseline_vs_reuse_geomean_speedup'] is not None:
            print(
                f"baseline_vs_reuse_speedup geomean={stats['baseline_vs_reuse_geomean_speedup']}x "
                f"median={stats['baseline_vs_reuse_median_speedup']}x"
            )
        summary_files = _summary_files(summary_dir)
        print(f'\nShared summaries: {len(summary_files)} function-level summaries available')
        for summary_file in sorted(summary_files):
            print(f'  {summary_file.stem}')
        stats_file.write_text(json.dumps(stats, indent=2))

    _write_harness_artifacts(
        harness_dir=harness_dir,
        suite_name=selected_suite,
        tests=tests,
        all_results=all_results,
        stats=stats,
        target_relative_error=target_relative_error,
        script_dir=script_dir,
        stop_after_first_valid=stop_after_first_valid,
    )
    _write_paper_table_artifacts(artifacts_dir, all_results)

    print(f'\nResults: {results_file}')
    print(f'Stats: {stats_file}')
    print(f'Paper table CSV: {artifacts_dir / "cse_paper_table.csv"}')
    print(f'Paper table TeX: {artifacts_dir / "cse_paper_table.tex"}')
    print(f'Logs: {logs_dir}')


if __name__ == '__main__':
    main()
