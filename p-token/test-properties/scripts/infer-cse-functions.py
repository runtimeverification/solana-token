#!/usr/bin/env python3
"""Infer useful CSE summary targets from an SMIR call graph.

The selection is intentionally conservative:
- only functions reachable from the requested start symbol are considered;
- only p-token processor process_* functions are considered automatic CSE
  targets;
- the largest reachable process_* function is kept by default;
- other process_* functions are kept only when their MIR instruction count
  reaches the configured threshold.
- helpers and entrypoint wrappers can be added explicitly when a benchmark
  needs to test them.
- the proof start symbol can be included explicitly as a trace wrapper.

The MIR instruction count used here is:
    sum(len(block.statements) + 1 terminator for each block)
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from kmir.smir import SMIRInfo, compute_closure


PROCESSOR_PREFIX = 'pinocchio_token_program::processor::'
ENTRYPOINT_PREFIX = 'pinocchio_token_program::entrypoint::'
EXCLUDED_PROCESS_NAMES = {
    'process_batch',
}


@dataclass(frozen=True)
class SelectedFunction:
    name: str
    role: str
    instructions: int
    reason: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--smir', type=Path, required=True, help='SMIR JSON file')
    parser.add_argument('--start-symbol', required=True, help='Proof start symbol')
    parser.add_argument(
        '--min-instructions',
        type=int,
        default=20,
        help='Minimum MIR instruction count for non-forced automatic targets',
    )
    parser.add_argument(
        '--max-functions',
        type=int,
        default=0,
        help='Maximum selected automatic targets, or 0 for no cap',
    )
    parser.add_argument(
        '--extra-function',
        action='append',
        default=[],
        help='Additional CSE function to append after automatic selection',
    )
    parser.add_argument(
        '--no-force-largest-process',
        action='store_true',
        help='Do not keep the largest reachable process_* when it is below threshold',
    )
    parser.add_argument(
        '--include-start-symbol',
        action='store_true',
        help='Include the proof start symbol as a CSE trace wrapper target',
    )
    parser.add_argument(
        '--format',
        choices=('names', 'tsv', 'json'),
        default='names',
        help='Output format',
    )
    return parser.parse_args()


def mono_item_fn(item: dict[str, Any]) -> dict[str, Any] | None:
    kind = item.get('mono_item_kind', {})
    fn = kind.get('MonoItemFn')
    return fn if isinstance(fn, dict) else None


def function_name_for_ty(info: SMIRInfo, ty: int) -> str | None:
    symbol = info.function_symbols.get(int(ty))
    if symbol is None:
        return None

    normal_symbol = symbol.get('NormalSym')
    if normal_symbol in info.items:
        fn = mono_item_fn(info.items[normal_symbol])
        if fn is not None:
            return fn.get('name')

    return normal_symbol or symbol.get('IntrinsicSym')


def item_by_function_name(info: SMIRInfo) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for item in info.items.values():
        fn = mono_item_fn(item)
        if fn is None:
            continue
        name = fn.get('name')
        if isinstance(name, str):
            result.setdefault(name, item)
    return result


def mir_instruction_count(item: dict[str, Any] | None) -> int:
    if item is None:
        return 0

    fn = mono_item_fn(item)
    if fn is None:
        return 0

    body = fn.get('body')
    if not isinstance(body, dict):
        return 0

    blocks = body.get('blocks', [])
    if not isinstance(blocks, list):
        return 0

    count = 0
    for block in blocks:
        statements = block.get('statements', [])
        if isinstance(statements, list):
            count += len(statements)
        if 'terminator' in block:
            count += 1
    return count


def function_basename(name: str) -> str:
    return name.rsplit('::', 1)[-1]


def is_auto_candidate(name: str, start_symbol: str) -> bool:
    if '::{closure#' in name:
        return False
    if name == start_symbol:
        return False
    basename = function_basename(name)
    if basename.startswith('test_'):
        return False
    if name.startswith(PROCESSOR_PREFIX):
        return basename.startswith('process_') and basename not in EXCLUDED_PROCESS_NAMES
    return False


def is_process_function(name: str) -> bool:
    return function_basename(name).startswith('process_')


def function_role(name: str) -> str:
    if is_process_function(name):
        return 'process'
    if name.startswith(ENTRYPOINT_PREFIX):
        return 'wrapper'
    return 'helper'


def reachable_function_names(info: SMIRInfo, start_symbol: str) -> set[str]:
    try:
        start_ty = info.function_tys[start_symbol]
    except KeyError as err:
        raise SystemExit(f'[ERROR] start symbol not found in SMIR: {start_symbol}') from err

    reached_tys = compute_closure([start_ty], info.call_edges)
    result: set[str] = set()
    for ty in reached_tys:
        name = function_name_for_ty(info, int(ty))
        if name is not None:
            result.add(name)
    return result


def add_selected(selected: list[SelectedFunction], candidate: SelectedFunction) -> None:
    if any(existing.name == candidate.name for existing in selected):
        return
    selected.append(candidate)


def infer_targets(
    info: SMIRInfo,
    start_symbol: str,
    min_instructions: int,
    max_functions: int,
    extra_functions: list[str],
    force_largest_process: bool,
    include_start_symbol: bool,
) -> list[SelectedFunction]:
    items = item_by_function_name(info)
    reachable = reachable_function_names(info, start_symbol)

    candidates: list[SelectedFunction] = []
    for name in sorted(reachable):
        if not is_auto_candidate(name, start_symbol):
            continue
        instructions = mir_instruction_count(items.get(name))
        if instructions <= 0:
            continue
        role = function_role(name)
        candidates.append(SelectedFunction(name, role, instructions, 'candidate'))

    process_candidates = [candidate for candidate in candidates if candidate.role == 'process']
    selected: list[SelectedFunction] = []

    if include_start_symbol:
        add_selected(
            selected,
            SelectedFunction(
                start_symbol,
                'start',
                mir_instruction_count(items.get(start_symbol)),
                'trace-start-wrapper',
            ),
        )

    if process_candidates:
        largest_process = max(process_candidates, key=lambda candidate: (candidate.instructions, candidate.name))
        if force_largest_process or largest_process.instructions >= min_instructions:
            add_selected(
                selected,
                SelectedFunction(
                    largest_process.name,
                    largest_process.role,
                    largest_process.instructions,
                    'largest-reachable-process',
                ),
            )
    elif candidates:
        largest_processor = max(candidates, key=lambda candidate: (candidate.instructions, candidate.name))
        if force_largest_process or largest_processor.instructions >= min_instructions:
            add_selected(
                selected,
                SelectedFunction(
                    largest_processor.name,
                    largest_processor.role,
                    largest_processor.instructions,
                    'largest-reachable-processor',
                ),
            )

    threshold_candidates = sorted(candidates, key=lambda candidate: (-candidate.instructions, candidate.name))
    for candidate in threshold_candidates:
        if candidate.instructions < min_instructions:
            continue
        if max_functions > 0 and len(selected) >= max_functions:
            break
        add_selected(
            selected,
            SelectedFunction(
                candidate.name,
                candidate.role,
                candidate.instructions,
                f'mir-instructions>={min_instructions}',
            ),
        )

    for name in extra_functions:
        add_selected(
            selected,
            SelectedFunction(
                name,
                'manual',
                mir_instruction_count(items.get(name)),
                'manual-extra',
            ),
        )

    return selected


def emit(selected: list[SelectedFunction], output_format: str) -> None:
    if output_format == 'names':
        for target in selected:
            print(target.name)
    elif output_format == 'tsv':
        for target in selected:
            print(f'{target.name}\t{target.role}\t{target.instructions}\t{target.reason}')
    else:
        print(json.dumps([target.__dict__ for target in selected], indent=2, sort_keys=True))


def main() -> int:
    args = parse_args()
    if args.min_instructions < 0:
        print('[ERROR] --min-instructions must be non-negative', file=sys.stderr)
        return 2
    if args.max_functions < 0:
        print('[ERROR] --max-functions must be non-negative', file=sys.stderr)
        return 2

    info = SMIRInfo.from_file(args.smir)
    selected = infer_targets(
        info,
        args.start_symbol,
        args.min_instructions,
        args.max_functions,
        args.extra_function,
        not args.no_force_largest_process,
        args.include_start_symbol,
    )
    emit(selected, args.format)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
