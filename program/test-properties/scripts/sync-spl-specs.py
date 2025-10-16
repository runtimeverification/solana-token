#!/usr/bin/env python3
"""Regenerate the Solana SPL entrypoint harnesses from the pinocchio source.

The flow intentionally mirrors the high-level steps documented in `main()`:

1. Parse the pinocchio entrypoint and extract every `test_process_*` harness.
2. Load a transformation config that spells out how each harness and match arm
   must diverge from the original pinocchio version. The config focuses on
   portable operations (comment out, remove, replace) so that differences are
   obvious at a glance.
3. Apply the configured transforms, splice the generated snippets into the
   runtime-entrypoint template, and overwrite the SPL copy.

By keeping the transformation requirements in `sync_spl_specs_config.json`,
the script remains easy to read: every edit to the SPL spec is described as
data, and the Python code stays focused on orchestrating the pipeline.
"""

from __future__ import annotations

import json
import re
import textwrap
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, Iterable, List

REPO_ROOT = Path(__file__).resolve().parents[3]
SCRIPTS_DIR = REPO_ROOT / "program/test-properties/scripts"
CONFIG_PATH = SCRIPTS_DIR / "sync_spl_specs_config.json"

MATCH_ARM_TEMPLATE = """        // {discriminator} - {title}
        {discriminator} => {{
            {function_name}(
                program_id,
                {account_line},{account_line_comment}
                instruction_data.first_chunk().ok_or(TokenError::InvalidInstruction)?,
            )
        }}"""


# Data models -----------------------------------------------------------------
@dataclass
class SectionRule:
    separator: str
    trailing_newline: bool


@dataclass
class Replacement:
    raw_from: str
    replacement: str
    is_regex: bool = False
    pattern: re.Pattern[str] | None = None
    literal_from: str | None = None

    @classmethod
    def from_dict(cls, data: Dict[str, str]) -> "Replacement":
        raw = data["from"]
        replacement = data["to"]
        if raw.startswith("regex:"):
            pattern_text = raw[len("regex:") :]
            pattern = re.compile(pattern_text)
            return cls(
                raw_from=raw,
                replacement=replacement,
                is_regex=True,
                pattern=pattern,
            )
        return cls(
            raw_from=raw,
            replacement=replacement,
            literal_from=raw,
        )

    def clone(self) -> "Replacement":
        if self.is_regex and self.pattern is not None:
            return Replacement(
                raw_from=self.raw_from,
                replacement=self.replacement,
                is_regex=True,
                pattern=re.compile(self.pattern.pattern),
            )
        return Replacement(
            raw_from=self.raw_from,
            replacement=self.replacement,
            literal_from=self.literal_from,
        )


@dataclass
class HarnessConfig:
    comment_out: List[str] = field(default_factory=list)
    replacements: List[Replacement] = field(default_factory=list)
    presets: List[str] = field(default_factory=list)

    @classmethod
    def from_dict(cls, data: Dict) -> "HarnessConfig":
        return cls(
            comment_out=list(data.get("comment_out", [])),
            replacements=[Replacement.from_dict(item) for item in data.get("replacements", [])],
            presets=list(data.get("presets", [])),
        )

    @staticmethod
    def empty() -> "HarnessConfig":
        return HarnessConfig(
            comment_out=[],
            replacements=[],
            presets=[],
        )

    def clone(self) -> "HarnessConfig":
        return HarnessConfig(
            comment_out=list(self.comment_out),
            replacements=[repl.clone() for repl in self.replacements],
            presets=list(self.presets),
        )

    def expand_presets(
        self,
        preset_map: Dict[str, "HarnessConfig"],
        *,
        cache: Dict[str, "HarnessConfig"] | None = None,
        stack: tuple[str, ...] = (),
    ) -> "HarnessConfig":
        if cache is None:
            cache = {}

        combined = HarnessConfig.empty()
        for preset_name in self.presets:
            if preset_name in cache:
                preset_resolved = cache[preset_name]
            else:
                # Recurse into preset graph while tracking the stack to spot cycles early.
                if preset_name in stack:
                    cycle = " -> ".join(stack + (preset_name,))
                    raise ValueError(f"Circular preset dependency detected: {cycle}")
                if preset_name not in preset_map:
                    raise KeyError(f"Unknown preset `{preset_name}` referenced in harness config")
                preset_resolved = preset_map[preset_name].expand_presets(
                    preset_map,
                    cache=cache,
                    stack=stack + (preset_name,),
                )
                cache[preset_name] = preset_resolved
            combined = merge_harness_configs(combined, preset_resolved)

        own = self.clone()
        own.presets = []
        # Overlay preset content with the harness' own directives, preserving precedence.
        combined = merge_harness_configs(combined, own)
        combined.comment_out = dedupe_preserve_order(combined.comment_out)
        combined.replacements = dedupe_preserve_order(
            combined.replacements,
            key=lambda repl: (repl.raw_from, repl.replacement, repl.is_regex),
        )
        if stack:
            cache[stack[-1]] = combined
        return combined


def dedupe_preserve_order(
    items: Iterable,
    *,
    key=lambda x: x,
) -> List:
    """Behaves like dict.fromkeys but keeps caller-provided key projector."""
    seen: set = set()
    output: List = []
    for item in items:
        marker = key(item)
        if marker in seen:
            continue
        seen.add(marker)
        output.append(item)
    return output


def merge_harness_configs(base: HarnessConfig, extra: HarnessConfig) -> HarnessConfig:
    return HarnessConfig(
        comment_out=base.comment_out + extra.comment_out,
        replacements=base.replacements + extra.replacements,
        presets=[],
    )


@dataclass
class FunctionConfig:
    name: str
    discriminator: int
    harness: HarnessConfig

    @classmethod
    def from_dict(cls, data: Dict) -> "FunctionConfig":
        harness = HarnessConfig.from_dict(data["harness"])
        return cls(
            name=data["name"],
            discriminator=data["discriminator"],
            harness=harness,
        )


@dataclass
class SyncConfig:
    source: Path
    template: Path
    target: Path
    placeholders: Dict[str, str]
    section_rules: Dict[str, SectionRule]
    presets: Dict[str, HarnessConfig]
    functions: List[FunctionConfig]

    @classmethod
    def load(cls, path: Path) -> "SyncConfig":
        data = json.loads(path.read_text())
        section_rules = {
            name: SectionRule(**rule) for name, rule in data["sections"].items()
        }
        preset_map = {
            name: HarnessConfig.from_dict(item)
            for name, item in data.get("presets", {}).items()
        }
        preset_cache: Dict[str, HarnessConfig] = {}
        for preset_name in preset_map:
            if preset_name not in preset_cache:
                preset_cache[preset_name] = preset_map[preset_name].expand_presets(
                    preset_map,
                    cache=preset_cache,
                    stack=(preset_name,),
                )

        functions = [FunctionConfig.from_dict(item) for item in data.get("functions", [])]
        for idx, func in enumerate(functions):
            functions[idx] = FunctionConfig(
                name=func.name,
                discriminator=func.discriminator,
                harness=func.harness.expand_presets(preset_map, cache=preset_cache),
            )
        return cls(
            source=REPO_ROOT / data["source"],
            template=REPO_ROOT / data["template"],
            target=REPO_ROOT / data["target"],
            placeholders=data["placeholders"],
            section_rules=section_rules,
            presets=preset_cache,
            functions=functions,
        )


# Parsing helpers --------------------------------------------------------------
def find_matching_brace(text: str, start: int) -> int:
    depth = 0
    i = start
    length = len(text)
    while i < length:
        ch = text[i]
        # Ignore braces that appear inside comments.
        if text.startswith("//", i):
            newline = text.find("\n", i)
            if newline == -1:
                return length - 1
            i = newline + 1
            continue
        if text.startswith("/*", i):
            close = text.find("*/", i + 2)
            if close == -1:
                raise ValueError("Unterminated block comment")
            i = close + 2
            continue
        # Treat string and char literals as opaque so delimiter characters inside them are skipped.
        if ch == '"':
            i += 1
            while i < length:
                if text[i] == "\\":
                    i += 2
                elif text[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            continue
        if ch == "'":
            i += 1
            while i < length:
                if text[i] == "\\":
                    i += 2
                elif text[i] == "'":
                    i += 1
                    break
                else:
                    i += 1
            continue
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise ValueError(f"Unmatched braces starting at index {start}")


def extract_test_functions(text: str) -> Dict[str, str]:
    pattern = re.compile(r"(?m)^[ \t]*(?:pub[ \t]+)?fn[ \t]+(?P<name>test_process_[A-Za-z0-9_]+)\s*\(")
    functions: Dict[str, str] = {}
    for match in pattern.finditer(text):
        name = match.group("name")
        fn_start = match.start()
        # Capture any leading doc comments or attributes that belong to the harness.
        start = fn_start
        while start > 0:
            prev_newline = text.rfind("\n", 0, start - 1)
            if prev_newline == -1:
                start = 0
                break
            line_start = prev_newline + 1
            line = text[line_start:start]
            stripped = line.strip()
            if stripped.startswith("///") or stripped.startswith("#["):
                start = line_start
                continue
            if stripped == "":
                start = line_start
                continue
            break
        brace_start = text.find("{", match.end())
        if brace_start == -1:
            raise ValueError(f"Missing body for {name}")
        brace_end = find_matching_brace(text, brace_start)
        snippet = text[start : brace_end + 1]
        functions[name] = snippet.strip("\n")
    return functions


# Text transforms --------------------------------------------------------------
def comment_out_lines(text: str, patterns: Iterable[str]) -> str:
    if not patterns:
        return text

    compiled: List[tuple[str, str | re.Pattern[str]]] = []
    # Allow the config to opt into regex-powered comment markers via the `regex:` prefix.
    for raw in patterns:
        if raw.startswith("regex:"):
            pattern = raw[len("regex:") :].strip()
            compiled.append(("regex", re.compile(pattern)))
        else:
            compiled.append(("literal", raw))

    lines = text.splitlines()
    for idx, line in enumerate(lines):
        stripped = line.lstrip()
        indent_len = len(line) - len(stripped)
        indent = line[:indent_len]

        for mode, matcher in compiled:
            matched = False
            if mode == "literal":
                matched = stripped == matcher
            else:
                assert isinstance(matcher, re.Pattern)
                matched = bool(matcher.search(line))

            if matched:
                if stripped.startswith("//"):
                    break
                # Rebuild the line with the original indentation to avoid diff churn.
                lines[idx] = f"{indent}// {stripped}"
                break

    return "\n".join(lines)


def apply_replacements(text: str, replacements: Iterable[Replacement]) -> str:
    for repl in replacements:
        if repl.is_regex and repl.pattern is not None:
            text = repl.pattern.sub(repl.replacement, text)
        elif repl.literal_from is not None:
            text = text.replace(repl.literal_from, repl.replacement)
    return text


def infer_instruction_types(snippet: str) -> tuple[str | None, str | None]:
    if "{" not in snippet:
        return None, None
    header, _body = snippet.split("{", 1)
    match = re.search(r"instruction_data\s*:\s*([^,\)]+)", header, flags=re.DOTALL)
    if match is None:
        # No instruction data argument in the original harness, treat payload as empty.
        return "[u8; 0]", "&[u8; 1]"

    raw_type = match.group(1).strip()
    compact = re.sub(r"\s+", "", raw_type)

    array_match = re.fullmatch(r"&?\[u8;(\d+)\]", compact)
    if array_match:
        payload_len = int(array_match.group(1))
        payload_type = f"[u8; {payload_len}]"
        instruction_type = f"&[u8; {payload_len + 1}]"
        return payload_type, instruction_type

    return None, None


def resolve_instruction_types(snippet: str, func_name: str) -> tuple[str, str]:
    payload, instruction_type = infer_instruction_types(snippet)
    if payload is None or instruction_type is None:
        raise ValueError(
            f"Unable to infer instruction data types for `{func_name}`. "
            "Update the script to handle this signature."
        )
    return payload, instruction_type


def transform_harness(snippet: str, func_cfg: FunctionConfig) -> tuple[str, str, str]:
    cfg = func_cfg.harness
    payload_type, instruction_data_type = resolve_instruction_types(snippet, func_cfg.name)
    brace_start = snippet.find("{")
    if brace_start == -1:
        raise ValueError(f"Malformed function snippet for {func_cfg.name}")
    body_block = snippet[brace_start + 1 :]
    if "}" not in body_block:
        raise ValueError(f"Function body missing closing brace for {func_cfg.name}")
    body_block, _closing = body_block.rsplit("}", 1)

    header_block = snippet[:brace_start].rstrip()
    header_lines = header_block.splitlines()
    doc_lines: List[str] = []
    attr_lines: List[str] = []
    original_account_line: str | None = None
    # Track metadata lines that should be preserved on the generated harness.
    for line in header_lines:
        stripped = line.strip()
        if stripped.startswith("///"):
            doc_lines.append(stripped)
        elif stripped.startswith("#["):
            attr_lines.append(stripped)
        elif original_account_line is None and stripped.startswith("accounts:"):
            original_account_line = stripped

    if original_account_line is None:
        account_match = re.search(r"(accounts\s*:\s*[^,\)]+)(,|\))", header_block)
        if account_match:
            account_text = account_match.group(1).strip()
            if account_match.group(2) == ",":
                account_text += ","
            original_account_line = account_text

    signature_lines = []
    signature_lines.append(f"fn {func_cfg.name}(")
    signature_lines.append("    program_id: &Pubkey,")

    if original_account_line is None:
        raise ValueError(
            f"Unable to infer accounts parameter for `{func_cfg.name}`; ensure the source harness contains it or add a replacement."
        )

    account_line_candidate = original_account_line
    for repl in cfg.replacements:
        if repl.is_regex or repl.literal_from is None:
            continue
        if repl.literal_from in account_line_candidate:
            account_line_candidate = account_line_candidate.replace(
                repl.literal_from, repl.replacement
            )

    account_line, account_expr, account_comment = prepare_account_metadata(
        original_account_line,
        account_line_candidate,
    )

    signature_lines.append(f"    {account_line}")
    signature_lines.append(f"    instruction_data: {instruction_data_type},")
    signature_lines.append(") -> ProgramResult {")
    signature = "\n".join(signature_lines)

    body = textwrap.dedent(body_block.rstrip())
    body = comment_out_lines(body, cfg.comment_out)
    body = apply_replacements(body, cfg.replacements)

    body_lines = [line.rstrip() for line in body.splitlines()]
    while body_lines and not body_lines[-1].strip():
        body_lines.pop()
    if not body_lines or body_lines[-1].strip() != "result":
        raise ValueError(f"Expected `{func_cfg.name}` body to end with `result`")
    body_lines.pop()  # drop the trailing `result`
    while body_lines and not body_lines[-1].strip():
        body_lines.pop()

    while body_lines and not body_lines[0].strip():
        body_lines.pop(0)

    leading_uses: List[str] = []
    while body_lines and body_lines[0].strip().startswith("use "):
        leading_uses.append(body_lines.pop(0).strip())
    if body_lines and body_lines[0].strip() == "":
        body_lines.pop(0)

    while body_lines and not body_lines[0].strip():
        body_lines.pop(0)

    prologue = [
        "// Set descriminator and program id to concrete value",
        f"cheatcode_set_descriminator({func_cfg.discriminator}, instruction_data);",
        "cheatcode_set_program_id(program_id);",
        "",
        "// Strip discriminator so instruction data is equivalent p-token harness",
        "let instruction_data_with_discriminator = &instruction_data.clone();",
        f"let instruction_data: &{payload_type} = instruction_data.last_chunk().unwrap();",
        "",
    ]

    epilogue = [
        "// Ensure instruction_data was not mutated",
        "assert_eq!(*instruction_data, instruction_data_with_discriminator[1..]);",
        "",
    ]

    output_lines: List[str] = []
    if doc_lines:
        output_lines.append("\n".join(doc_lines))
    if attr_lines:
        output_lines.append("\n".join(attr_lines))
    output_lines.append(signature)

    body_output: List[str] = []
    if leading_uses:
        body_output.extend(leading_uses)
        body_output.append("")
    # Every harness begins by pinning the discriminator and slicing the payload.
    body_output.extend(prologue)
    if body_lines:
        body_output.extend(body_lines)
        body_output.append("")
    body_output.extend(epilogue)
    body_output.append("result")

    indented_body = "\n".join("    " + line if line else "" for line in body_output)
    output_lines.append(indented_body)
    output_lines.append("}")
    return "\n".join(output_lines), account_expr, account_comment


def to_title(label: str) -> str:
    base = label
    if base.startswith("test_process_"):
        base = base[len("test_process_") :]
    if base.startswith("test_"):
        base = base[len("test_") :]
    parts = [part for part in base.split("_") if part]
    return " ".join(word.capitalize() for word in parts) or label


def prepare_account_metadata(
    original_account_line: str | None,
    candidate_line: str,
) -> tuple[str, str, str]:
    # Returns the rewritten parameter line plus the expression/comment used in the match arm.
    stripped_candidate = candidate_line.strip()
    if not stripped_candidate:
        raise ValueError("Accounts parameter line is empty after replacements")

    code_part, _, comment_part = stripped_candidate.partition("//")
    code_part = code_part.strip()
    inline_comment = comment_part.strip().rstrip(", ")
    code_without_comma = code_part.rstrip(",")

    type_part = ""
    if ":" in code_without_comma:
        _, type_part = code_without_comma.split(":", 1)
        type_part = type_part.strip()

    if "[AccountInfo;" in type_part:
        account_expr = "accounts.first_chunk().ok_or(TokenError::InvalidInstruction)?"
    else:
        account_expr = "accounts"

    comment_segments: List[str] = []
    orig_code_part = None
    if original_account_line is not None:
        orig_code_part_raw, _, _ = original_account_line.strip().partition("//")
        orig_code_part = orig_code_part_raw.strip().rstrip(",")

    if orig_code_part and orig_code_part != code_part:
        change_note = f"CHANGE P-Token: {orig_code_part}"
        if not (inline_comment and orig_code_part in inline_comment):
            comment_segments.append(change_note)

    if inline_comment:
        comment_segments.append(inline_comment)

    seen: set[str] = set()
    unique_comments: List[str] = []
    for segment in comment_segments:
        if segment and segment not in seen:
            seen.add(segment)
            unique_comments.append(segment)

    comment_suffix = ""
    if unique_comments:
        comment_suffix = " // " + "; ".join(unique_comments)

    rebuilt_line = code_without_comma
    if not rebuilt_line.endswith(","):
        rebuilt_line += ","

    if inline_comment:
        rebuilt_line = f"{rebuilt_line} // {inline_comment}"

    return rebuilt_line, account_expr, comment_suffix


def render_default_match_arm(
    func_cfg: FunctionConfig,
    account_expr: str,
    comment_block: str,
) -> str:

    rendered = MATCH_ARM_TEMPLATE.format(
        discriminator=func_cfg.discriminator,
        title=to_title(func_cfg.name),
        function_name=func_cfg.name,
        account_line=account_expr,
        account_line_comment=comment_block,
    )

    return rendered


def replace_placeholder(text: str, placeholder: str, replacement: str) -> str:
    if placeholder not in text:
        raise RuntimeError(f"Placeholder `{placeholder}` not found in template")
    return text.replace(placeholder, replacement if replacement else placeholder, 1)


def render_template(template_text: str, sections: Dict[str, List[str]], cfg: SyncConfig) -> str:
    rendered = template_text
    for name, placeholder in cfg.placeholders.items():
        items = sections.get(name, [])
        rule = cfg.section_rules[name]
        if not items:
            replacement = placeholder
        else:
            chunk = rule.separator.join(items)
            if rule.trailing_newline:
                chunk += "\n"
            replacement = chunk
        rendered = replace_placeholder(rendered, placeholder, replacement)
    return rendered.rstrip("\n") + "\n"


# Pipeline --------------------------------------------------------------------
def assemble_sections(config: SyncConfig, functions: Dict[str, str]) -> Dict[str, List[str]]:
    harnesses: List[str] = []
    match_arms: List[str] = []

    for func_cfg in config.functions:
        # Look up the raw harness snippet and push it through the configured transforms.
        source_snippet = functions.get(func_cfg.name)
        if source_snippet is None:
            raise KeyError(f"Missing function `{func_cfg.name}` in source file")

        harness, account_expr, account_comment = transform_harness(source_snippet, func_cfg)
        match_arms.append(render_default_match_arm(func_cfg, account_expr, account_comment))
        harnesses.append(harness)
    return {
        "match_arms": match_arms,
        "harnesses": harnesses,
    }


def summarize_differences(config: SyncConfig) -> None:
    print("Configured transforms:")
    for func_cfg in config.functions:
        harness = func_cfg.harness
        print(
            f" - {func_cfg.name}: "
            f"{len(harness.comment_out)} comment-out, "
            f"{len(harness.replacements)} replacements"
        )


def main() -> None:
    config = SyncConfig.load(CONFIG_PATH)
    source_text = config.source.read_text()
    template_text = config.template.read_text()

    functions = extract_test_functions(source_text)
    sections = assemble_sections(config, functions)
    rendered = render_template(template_text, sections, config)
    config.target.write_text(rendered)

    summarize_differences(config)
    print(
        "Wrote "
        f"{config.target.relative_to(REPO_ROOT)} "
        "from pinocchio transformations."
    )


if __name__ == "__main__":
    main()
