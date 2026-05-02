#!/usr/bin/env python3
"""Compare CSE benchmark proofs against their no-CSE baseline."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Case:
    test: str
    name: str
    status: str
    proof_dir: Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('benchmark_root', type=Path, help='Root produced by run-cse-benchmark.sh')
    return parser.parse_args()


def digest(obj: Any) -> str:
    return hashlib.sha256(json.dumps(obj, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def proof_root(proof_dir: Path) -> Path | None:
    if not proof_dir.is_dir():
        return None
    candidates = sorted(path.parent for path in proof_dir.glob('*/proof.json'))
    if len(candidates) != 1:
        return None
    return candidates[0]


def load_proof(proof_dir: Path) -> dict[str, Any] | None:
    root = proof_root(proof_dir)
    if root is None:
        return None

    proof_path = root / 'proof.json'
    kcfg_path = root / 'kcfg/kcfg.json'
    if not proof_path.is_file() or not kcfg_path.is_file():
        return None

    proof = json.loads(proof_path.read_text())
    kcfg = json.loads(kcfg_path.read_text())

    def node_hash(node_id: int) -> str:
        node = json.loads((root / f'kcfg/nodes/{node_id}.json').read_text())
        return digest(node['cterm'])

    split_shape = []
    split_constraints = []
    split_constraint_multiset = []
    for split in kcfg.get('splits', []):
        split_shape.append((split['source'], tuple(target['target'] for target in split['targets'])))
        constraints = tuple(digest(target.get('csubst', {}).get('constraints', [])) for target in split['targets'])
        split_constraints.append(constraints)
        split_constraint_multiset.extend(constraints)

    return {
        'nodes': len(kcfg.get('nodes', [])),
        'edges': tuple((edge['source'], edge['target'], edge.get('depth')) for edge in kcfg.get('edges', [])),
        'split_shape': tuple(split_shape),
        'split_constraints': tuple(split_constraints),
        'split_constraint_multiset': tuple(sorted(split_constraint_multiset)),
        'covers': tuple(sorted((cover['source'], cover['target']) for cover in kcfg.get('covers', []))),
        'terminal_count': len(proof.get('terminal', [])),
        'terminal_hashes': tuple(sorted(node_hash(node_id) for node_id in proof.get('terminal', []))),
        'target_hash': node_hash(proof['target']),
        'pending': len(proof.get('pending', [])),
        'failing': len(proof.get('failing', [])),
        'stuck': len(kcfg.get('stuck', [])),
    }


def read_cases(root: Path) -> dict[tuple[str, str], Case]:
    results = root / 'results.csv'
    if not results.is_file():
        raise SystemExit(f'Missing results.csv under {root}')

    cases: dict[tuple[str, str], Case] = {}
    with results.open(newline='') as handle:
        for row in csv.DictReader(handle):
            proof_dir = row.get('proof_dir', '')
            case = Case(
                test=row['test'],
                name=row['case'],
                status=row.get('proof_status', ''),
                proof_dir=Path(proof_dir) if proof_dir else Path(),
            )
            cases[(case.test, case.name)] = case
    return cases


def compare(base: dict[str, Any] | None, other: dict[str, Any] | None) -> dict[str, Any]:
    if base is None or other is None:
        return {
            'checked': False,
            'same_edges': False,
            'same_splits': False,
            'same_split_constraints': False,
            'same_covers': False,
            'same_terminal_hashes': False,
            'same_target_hash': False,
            'base_nodes': '',
            'case_nodes': '',
            'base_terminal': '',
            'case_terminal': '',
        }

    same_terminal_hashes = base['terminal_hashes'] == other['terminal_hashes']
    same_target_hash = base['target_hash'] == other['target_hash']
    same_terminal_count = base['terminal_count'] == other['terminal_count']

    return {
        'checked': True,
        'same_edges': base['edges'] == other['edges'],
        'same_splits': base['split_shape'] == other['split_shape'],
        'same_split_constraints': base['split_constraints'] == other['split_constraints'],
        'same_split_constraint_multiset': base['split_constraint_multiset'] == other['split_constraint_multiset'],
        'same_covers': base['covers'] == other['covers'],
        'same_terminal_hashes': same_terminal_hashes,
        'same_target_hash': same_target_hash,
        'same_terminal_count': same_terminal_count,
        'state_equivalent': same_terminal_hashes and same_target_hash and same_terminal_count,
        'base_nodes': base['nodes'],
        'case_nodes': other['nodes'],
        'base_terminal': base['terminal_count'],
        'case_terminal': other['terminal_count'],
    }


def main() -> None:
    ns = parse_args()
    root = ns.benchmark_root.resolve()
    cases = read_cases(root)
    tests = sorted({test for test, _case in cases})

    rows: list[dict[str, Any]] = []
    for test in tests:
        base_case = cases.get((test, 'no-cse'))
        base = load_proof(base_case.proof_dir) if base_case is not None and base_case.status == 'PASSED' else None
        for case_name in ('cold-cse', 'warm-cse'):
            cse_case = cases.get((test, case_name))
            if cse_case is not None and cse_case.status == 'SKIPPED':
                continue
            other = load_proof(cse_case.proof_dir) if cse_case is not None and cse_case.status == 'PASSED' else None
            row = {
                'test': test,
                'case': case_name,
                'base_status': base_case.status if base_case is not None else 'MISSING',
                'case_status': cse_case.status if cse_case is not None else 'MISSING',
            }
            row.update(compare(base, other))
            rows.append(row)

    csv_path = root / 'proof-equivalence.csv'
    fieldnames = [
        'test',
        'case',
        'base_status',
        'case_status',
        'checked',
        'same_edges',
        'same_splits',
        'same_split_constraints',
        'same_split_constraint_multiset',
        'same_covers',
        'same_terminal_hashes',
        'same_target_hash',
        'same_terminal_count',
        'state_equivalent',
        'base_nodes',
        'case_nodes',
        'base_terminal',
        'case_terminal',
    ]
    with csv_path.open('w', newline='') as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    md_path = root / 'proof-equivalence.md'
    with md_path.open('w') as handle:
        handle.write('# Proof equivalence\n\n')
        handle.write(
            '| test | case | base status | case status | checked | state equivalent | edges | splits | split constraints | split constraint multiset | covers | terminals | target | terminal count same | nodes | terminal count |\n'
        )
        handle.write('| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | ---: | ---: |\n')
        for row in rows:
            handle.write(
                f"| {row['test']} | {row['case']} | {row['base_status']} | {row['case_status']} | {row['checked']} | "
                f"{row.get('state_equivalent', '')} | "
                f"{row['same_edges']} | {row['same_splits']} | "
                f"{row['same_split_constraints']} | {row.get('same_split_constraint_multiset', '')} | "
                f"{row['same_covers']} | {row['same_terminal_hashes']} | "
                f"{row['same_target_hash']} | {row.get('same_terminal_count', '')} | "
                f"{row['case_nodes']} | {row['case_terminal']} |\n"
            )
        handle.write(f'\nCSV: `{csv_path}`\n')

    failed = [row for row in rows if row['checked'] and not row['state_equivalent']]
    print(md_path)
    if failed:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
