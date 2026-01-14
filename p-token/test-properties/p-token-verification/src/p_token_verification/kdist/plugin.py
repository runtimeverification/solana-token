from __future__ import annotations

import shutil
from pathlib import Path
from typing import TYPE_CHECKING

from pyk.kdist.api import Target
from pyk.ktool.kompile import LLVMKompileType, PykBackend, kompile

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping
    from typing import Any, Final


class SourceTarget(Target):
    SRC_DIR: Final = Path(__file__).parent

    def build(self, output_dir: Path, deps: dict[str, Path], args: dict[str, Any], verbose: bool) -> None:
        shutil.copytree(self.SRC_DIR / 'p-token', output_dir / 'p-token')

    def source(self) -> tuple[Path, ...]:
        return (self.SRC_DIR,)

    def deps(self) -> tuple[()]:
        return ()


class KompileTarget(Target):
    _kompile_args: Callable[[Path, Path], Mapping[str, Any]]

    def __init__(self, kompile_args: Callable[[Path], Mapping[str, Any]]):
        self._kompile_args = kompile_args

    def build(self, output_dir: Path, deps: dict[str, Path], args: dict[str, Any], verbose: bool) -> None:
        kompile_args = self._kompile_args(
            deps['mir-semantics.source'],
            deps['p-token-verification.source'],
        )
        kompile(output_dir=output_dir, verbose=verbose, **kompile_args)

    def deps(self) -> tuple[str, ...]:
        return ('mir-semantics.source', 'p-token-verification.source')


def _default_args(include_dirs: list[Path]) -> dict[str, Any]:
    return {
        'include_dirs': include_dirs,
        'warnings_to_errors': True,
    }


__TARGETS__: Final = {
    'source': SourceTarget(),
    'llvm': KompileTarget(
        lambda kmir_src_dir, ptoken_src_dir: {
            'main_file': ptoken_src_dir / 'p-token/verification.md',
            'backend': PykBackend.LLVM,
            'md_selector': 'k & ! symbolic',
            'opt_level': 2,
            **_default_args(include_dirs=[kmir_src_dir, ptoken_src_dir]),
        },
    ),
    'llvm-library': KompileTarget(
        lambda kmir_src_dir, ptoken_src_dir: {
            'main_file': ptoken_src_dir / 'p-token/verification.md',
            'backend': PykBackend.LLVM,
            'llvm_kompile_type': LLVMKompileType.C,
            'md_selector': 'k & ! symbolic',
            'opt_level': 2,
            **_default_args(include_dirs=[kmir_src_dir, ptoken_src_dir]),
        },
    ),
    'haskell': KompileTarget(
        lambda kmir_src_dir, ptoken_src_dir: {
            'main_file': ptoken_src_dir / 'p-token/verification.md',
            'backend': PykBackend.HASKELL,
            'md_selector': 'k & ! concrete',
            **_default_args(include_dirs=[kmir_src_dir, ptoken_src_dir]),
        },
    ),
}
