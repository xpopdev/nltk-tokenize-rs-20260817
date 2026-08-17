"""
Times the original Python implementation against the Rust-backed port for
each registered function, using the same FUNCTION_PAIRS the correctness
matrix uses (scripts/compare_outputs.py) so there's one source of truth
for "what am I comparing against what".

This is a secondary signal, separate from correctness — a port that's
correct but not faster than the original is a design problem worth
surfacing, not just a passing test.

Usage:
    python scripts/benchmark.py [--reps 5000]
"""
from __future__ import annotations

import argparse
import json
import statistics
import time
from typing import Callable

from compare_outputs import FUNCTION_PAIRS, _try_import_pairs
from gen_matrix_inputs import cases_for


def time_calls(fn: Callable, args_list: list[tuple], reps: int) -> float:
    """Median wall-clock seconds per call, over `reps` repetitions of the
    given case list (so short-running functions still produce a stable
    number)."""
    start = time.perf_counter()
    for _ in range(reps):
        for args, kwargs in args_list:
            try:
                fn(*args, **kwargs)
            except Exception:
                pass  # timing only cares about wall clock, not correctness here
    elapsed = time.perf_counter() - start
    total_calls = reps * len(args_list)
    return elapsed / total_calls if total_calls else float("nan")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reps", type=int, default=50)
    args = parser.parse_args()

    _try_import_pairs()
    if not FUNCTION_PAIRS:
        print("No FUNCTION_PAIRS registered yet — nothing to benchmark.")
        return 1

    rows = []
    for fn_key, (original, ported) in FUNCTION_PAIRS.items():
        cases = [(c.args, c.kwargs) for c in cases_for(fn_key, size="smoke")]
        if not cases:
            continue
        orig_time = time_calls(original, cases, args.reps)
        port_time = time_calls(ported, cases, args.reps)
        speedup = orig_time / port_time if port_time else float("inf")
        rows.append({
            "fn": fn_key,
            "original_us": round(orig_time * 1e6, 3),
            "ported_us": round(port_time * 1e6, 3),
            "speedup": round(speedup, 2),
        })

    with open("benchmark_report.json", "w") as f:
        json.dump(rows, f, indent=2)

    with open("benchmark_report.md", "w") as f:
        f.write("# Benchmark report\n\n")
        f.write("| Function | Original (µs/call) | Ported (µs/call) | Speedup |\n")
        f.write("|---|---|---|---|\n")
        for r in rows:
            f.write(f"| {r['fn']} | {r['original_us']} | {r['ported_us']} | {r['speedup']}x |\n")

    for r in rows:
        print(f"{r['fn']}: {r['speedup']}x ({r['original_us']}µs -> {r['ported_us']}µs)")

    slower = [r for r in rows if r["speedup"] < 1]
    if slower:
        print(
            f"\n{len(slower)} function(s) are SLOWER after porting — worth "
            "investigating (allocation patterns, PyO3 conversion overhead "
            "on small inputs, etc.) before calling the port 'done'.",
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
