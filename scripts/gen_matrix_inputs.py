"""
Generates the input matrix used for differential testing between the
original Python implementation and the Rust-backed port.

The matrix-test-writer agent extends CASE_GENERATORS with one entry per
public function it's covering. Keep generators deterministic (seeded) so
runs are reproducible, and keep `smoke` sets small/fast and `full` sets
thorough.

Usage:
    from scripts.gen_matrix_inputs import cases_for
    for case in cases_for("mymodule.myfunction", size="smoke"):
        ...
"""

from __future__ import annotations

import random
from dataclasses import dataclass, field
from typing import Any, Callable, Iterator

SEED = 20260816  # fixed seed -> reproducible matrix runs


@dataclass
class Case:
    """One input matrix cell."""

    label: str  # short human-readable id, e.g. "empty_list", "unicode_mixed"
    args: tuple = ()
    kwargs: dict = field(default_factory=dict)


# Reusable building blocks — extend/compose these per function rather than
# writing every generator from scratch.

def boundary_ints() -> list[int]:
    return [-1, 0, 1, 2**31 - 1, -(2**31), 2**63 - 1, -(2**63)]


def boundary_floats() -> list[float]:
    return [0.0, -0.0, 1.0, -1.0, float("inf"), float("-inf"), float("nan")]


def edge_strings() -> list[str]:
    return ["", " ", "a", "a" * 10_000, "\x00", "😀🚀", "\u200b", "café"]


def edge_collections() -> list[list]:
    return [[], [1], list(range(1000))]


def randomized_ints(n: int, lo: int = -10_000, hi: int = 10_000) -> list[int]:
    rng = random.Random(SEED)
    return [rng.randint(lo, hi) for _ in range(n)]


# Registry: function name -> generator(size) -> Iterator[Case]
# The matrix-test-writer agent fills this in per module it ports.
CASE_GENERATORS: dict[str, Callable[[str], Iterator[Case]]] = {}


def register(fn_name: str):
    def deco(gen: Callable[[str], Iterator[Case]]):
        CASE_GENERATORS[fn_name] = gen
        return gen

    return deco


def cases_for(fn_name: str, size: str = "smoke") -> Iterator[Case]:
    if fn_name not in CASE_GENERATORS:
        raise KeyError(
            f"No matrix cases registered for {fn_name!r} — "
            "the matrix-test-writer agent needs to add a generator."
        )
    yield from CASE_GENERATORS[fn_name](size)


# --- Example (delete once real generators are registered) -----------------
@register("example.add")
def _example_add_cases(size: str) -> Iterator[Case]:
    for i in boundary_ints():
        for j in boundary_ints():
            yield Case(label=f"boundary_{i}_{j}", args=(i, j))
    n = 5 if size == "smoke" else 200
    for a, b in zip(randomized_ints(n), randomized_ints(n)):
        yield Case(label=f"random_{a}_{b}", args=(a, b))
