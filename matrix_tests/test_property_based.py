"""
Property-based tests complementing the fixed differential matrix in
scripts/compare_outputs.py. Where the matrix proves equivalence on a
curated set of inputs, hypothesis proves it holds across a whole generated
input space — good for catching edge cases the matrix's authors didn't
think to include.

The matrix-test-writer agent adds one of these per function where a clean
property exists (equivalence to the original is always a valid property;
add algebraic ones like commutativity/idempotence where they genuinely
apply — don't force one that isn't true of the function).

Run with: pytest matrix_tests/test_property_based.py
"""
from __future__ import annotations

import math

from hypothesis import given, strategies as st

# Populated by matrix-test-writer, same pattern as scripts/compare_outputs.py:
#   from mylib_original import add as add_original
#   from mylib import add as add_ported
try:
    from scripts.compare_outputs import FUNCTION_PAIRS
except ImportError:
    FUNCTION_PAIRS = {}


def _values_equal(a, b) -> bool:
    if isinstance(a, float) and isinstance(b, float):
        if math.isnan(a) and math.isnan(b):
            return True
        return math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-12)
    return a == b


# --- Example (the matrix-test-writer agent replaces this with real,
# per-function property tests once FUNCTION_PAIRS is populated) -----------

@given(a=st.integers(), b=st.integers())
def test_example_add_equivalence(a: int, b: int) -> None:
    """Original and ported implementations agree across generated inputs,
    not just the fixed matrix cases."""
    if "example.add" not in FUNCTION_PAIRS:
        return  # skip until the real library is wired in
    original, ported = FUNCTION_PAIRS["example.add"]

    orig_exc = port_exc = None
    try:
        orig_result = original(a, b)
    except Exception as e:  # noqa: BLE001
        orig_exc = e
    try:
        port_result = ported(a, b)
    except Exception as e:  # noqa: BLE001
        port_exc = e

    if orig_exc is not None or port_exc is not None:
        assert type(orig_exc) is type(port_exc), (orig_exc, port_exc)
        return

    assert _values_equal(orig_result, port_result), (orig_result, port_result)
