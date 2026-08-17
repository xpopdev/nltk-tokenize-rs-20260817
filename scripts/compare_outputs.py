"""
Runs every registered matrix case against both the original Python
implementation and the new Rust-backed package, and writes a pass/fail
report.

The matrix-test-writer agent registers one FUNCTION_PAIRS entry per ported
function: (matrix key, original callable, ported callable). This script
stays generic — it doesn't know anything about the specific library.

Usage:
    python scripts/compare_outputs.py --size smoke
    python scripts/compare_outputs.py --size full
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from dataclasses import dataclass, field
from typing import Any, Callable

from gen_matrix_inputs import cases_for, CASE_GENERATORS

# fn matrix-key -> (original_callable, ported_callable)
# The matrix-test-writer agent populates this, e.g.:
#   from mylib_original import add as add_original
#   from mylib import add as add_ported
#   FUNCTION_PAIRS["example.add"] = (add_original, add_ported)
FUNCTION_PAIRS: dict[str, tuple[Callable, Callable]] = {}


@dataclass
class Row:
    fn: str
    label: str
    status: str  # "pass" | "fail" | "error"
    detail: str = ""


def values_equal(a: Any, b: Any) -> bool:
    if isinstance(a, float) and isinstance(b, float):
        if math.isnan(a) and math.isnan(b):
            return True
        return math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-12)
    return a == b


def run_case(fn_key: str, original: Callable, ported: Callable, case) -> Row:
    orig_result = orig_exc = None
    port_result = port_exc = None

    try:
        orig_result = original(*case.args, **case.kwargs)
    except Exception as e:  # noqa: BLE001 - we want to compare exception shape too
        orig_exc = e

    try:
        port_result = ported(*case.args, **case.kwargs)
    except Exception as e:  # noqa: BLE001
        port_exc = e

    if orig_exc is not None or port_exc is not None:
        if type(orig_exc) is not type(port_exc):
            return Row(
                fn_key, case.label, "fail",
                f"exception mismatch: original={orig_exc!r} ported={port_exc!r}",
            )
        if orig_exc is not None and str(orig_exc) != str(port_exc):
            return Row(
                fn_key, case.label, "fail",
                f"exception message mismatch: {str(orig_exc)!r} vs {str(port_exc)!r}",
            )
        return Row(fn_key, case.label, "pass", "matching exception")

    if not values_equal(orig_result, port_result):
        return Row(
            fn_key, case.label, "fail",
            f"value mismatch: original={orig_result!r} ported={port_result!r}",
        )
    return Row(fn_key, case.label, "pass")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--size", choices=["smoke", "full"], default="smoke")
    args = parser.parse_args()

    if not FUNCTION_PAIRS:
        print(
            "No FUNCTION_PAIRS registered yet — emitting empty report (M1 scaffold).",
            file=sys.stderr,
        )
        with open("matrix_report.json", "w") as f:
            json.dump([], f, indent=2)
        with open("matrix_report.md", "w") as f:
            f.write(f"# Matrix report ({args.size}) — empty (no pairs yet, M1 scaffold)\n\nNo pairs registered.\n")
        return 0

    rows: list[Row] = []
    for fn_key, (original, ported) in FUNCTION_PAIRS.items():
        for case in cases_for(fn_key, size=args.size):
            rows.append(run_case(fn_key, original, ported, case))

    failed = [r for r in rows if r.status != "pass"]

    with open("matrix_report.json", "w") as f:
        json.dump([r.__dict__ for r in rows], f, indent=2)

    with open("matrix_report.md", "w") as f:
        f.write(f"# Matrix report ({args.size})\n\n")
        f.write(f"Total: {len(rows)}  Passed: {len(rows) - len(failed)}  Failed: {len(failed)}\n\n")
        if failed:
            f.write("## Failures\n\n| Function | Case | Detail |\n|---|---|---|\n")
            for r in failed:
                f.write(f"| {r.fn} | {r.label} | {r.detail} |\n")

    print(f"{len(rows) - len(failed)}/{len(rows)} matrix cases passed.")
    if failed:
        print(f"{len(failed)} failures — see matrix_report.md", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
