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
from pathlib import Path
from typing import Any, Callable

# Ensure `scripts/` is importable when run as `python scripts/compare_outputs.py`
sys.path.insert(0, str(Path(__file__).resolve().parent))

from gen_matrix_inputs import cases_for, CASE_GENERATORS

# fn matrix-key -> (original_callable, ported_callable)
# The matrix-test-writer agent populates this, e.g.:
#   from mylib_original import add as add_original
#   from mylib import add as add_ported
#   FUNCTION_PAIRS["example.add"] = (add_original, add_ported)
FUNCTION_PAIRS: dict[str, tuple[Callable, Callable]] = {}

# --- helpers to lazily wire pairs so missing deps don't break import ---

def _try_import_pairs():
    """Populate FUNCTION_PAIRS. Called at runtime, after maturin develop on CI."""
    # example
    try:
        from ported_lib import add as add_ported
        def add_original(a, b): return a + b
        FUNCTION_PAIRS["example.add"] = (add_original, add_ported)
    except ImportError:
        pass

    # word_tokenize — NLTKWordTokenizer path
    try:
        import ported_lib
        from nltk.tokenize import word_tokenize as wt_orig
        # ported word_tokenize takes (text, convert_parentheses=None); original takes (text, language, preserve_line)
        # Wrap both to single-arg (text) for matrix
        def orig_wt(text, **kw):
            return wt_orig(text)
        def ported_wt(text, **kw):
            return ported_lib.word_tokenize(text)
        FUNCTION_PAIRS["word_tokenize"] = (orig_wt, ported_wt)
    except ImportError:
        pass

    # sent_tokenize
    try:
        import ported_lib
        from nltk.tokenize import sent_tokenize as st_orig
        def orig_st(text, **kw):
            return st_orig(text, language=kw.get("language", "english"))
        def ported_st(text, **kw):
            return ported_lib.sent_tokenize(text, language=kw.get("language", "english"))
        FUNCTION_PAIRS["sent_tokenize"] = (orig_st, ported_st)
    except ImportError:
        pass

    # regexp_tokenize
    try:
        import ported_lib
        from nltk.tokenize import regexp_tokenize as rt_orig
        def orig_rt(text, pattern, gaps=False, discard_empty=True, **kw):
            return rt_orig(text, pattern, gaps=gaps, discard_empty=discard_empty)
        def ported_rt(text, pattern, gaps=False, discard_empty=True, **kw):
            return ported_lib.regexp_tokenize(text, pattern, gaps=gaps, discard_empty=discard_empty)
        FUNCTION_PAIRS["regexp_tokenize"] = (orig_rt, ported_rt)
    except ImportError:
        pass

    # util: string_span_tokenize
    try:
        import ported_lib
        from nltk.tokenize.util import string_span_tokenize as sst_orig
        def orig_sst(s, sep, **kw): return list(sst_orig(s, sep))
        def ported_sst(s, sep, **kw): return ported_lib.string_span_tokenize_py(s, sep)
        FUNCTION_PAIRS["string_span_tokenize"] = (orig_sst, ported_sst)
    except ImportError:
        pass

    # util: regexp_span_tokenize
    try:
        import ported_lib
        from nltk.tokenize.util import regexp_span_tokenize as rst_orig
        def orig_rst(s, pattern, **kw): return list(rst_orig(s, pattern))
        def ported_rst(s, pattern, **kw): return ported_lib.regexp_span_tokenize_py(s, pattern)
        FUNCTION_PAIRS["regexp_span_tokenize"] = (orig_rst, ported_rst)
    except ImportError:
        pass

    # util: spans_to_relative
    try:
        import ported_lib
        from nltk.tokenize.util import spans_to_relative as str_orig
        def orig_str(spans, **kw): return list(str_orig(spans))
        def ported_str(spans, **kw): return ported_lib.spans_to_relative_py(spans)
        FUNCTION_PAIRS["spans_to_relative"] = (orig_str, ported_str)
    except ImportError:
        pass

    # util: is_cjk
    try:
        import ported_lib
        from nltk.tokenize.util import is_cjk as cjk_orig
        # ported is_cjk_py takes String char
        def orig_cjk(ch, **kw): return cjk_orig(ch)
        def ported_cjk(ch, **kw): return ported_lib.is_cjk_py(ch)
        FUNCTION_PAIRS["is_cjk"] = (orig_cjk, ported_cjk)
    except ImportError:
        pass

    # util: xml_escape / unescape
    try:
        import ported_lib
        from nltk.tokenize.util import xml_escape as xe_orig, xml_unescape as xu_orig
        def orig_xe(text, **kw): return xe_orig(text)
        def ported_xe(text, **kw): return ported_lib.xml_escape_py(text)
        def orig_xu(text, **kw): return xu_orig(text)
        def ported_xu(text, **kw): return ported_lib.xml_unescape_py(text)
        FUNCTION_PAIRS["xml_escape"] = (orig_xe, ported_xe)
        FUNCTION_PAIRS["xml_unescape"] = (orig_xu, ported_xu)
    except ImportError:
        pass

    # util: align_tokens
    try:
        import ported_lib
        from nltk.tokenize.util import align_tokens as at_orig
        def orig_at(tokens, sentence, **kw): return list(at_orig(tokens, sentence))
        def ported_at(tokens, sentence, **kw): return ported_lib.align_tokens_py(tokens, sentence)
        FUNCTION_PAIRS["align_tokens"] = (orig_at, ported_at)
    except ImportError:
        pass

    # casual / TweetTokenizer
    try:
        import ported_lib
        from nltk.tokenize.casual import TweetTokenizer
        def orig_ct(text, **kw):
            tok = TweetTokenizer(
                preserve_case=kw.get("preserve_case", True),
                reduce_len=kw.get("reduce_len", False),
                strip_handles=kw.get("strip_handles", False),
            )
            return tok.tokenize(text)
        def ported_ct(text, **kw): return ported_lib.casual_tokenize_py(text, preserve_case=kw.get("preserve_case", True), reduce_len=kw.get("reduce_len", False), strip_handles=kw.get("strip_handles", False), match_phone_numbers=kw.get("match_phone_numbers", True))
        FUNCTION_PAIRS["casual_tokenize"] = (orig_ct, ported_ct)
    except ImportError:
        pass

    # toktok
    try:
        import ported_lib
        from nltk.tokenize.toktok import ToktokTokenizer
        _orig_tok = ToktokTokenizer()
        def orig_toktok(text, **kw): return _orig_tok.tokenize(text)
        def ported_toktok(text, **kw): return ported_lib.toktok_tokenize_py(text)
        FUNCTION_PAIRS["toktok_tokenize"] = (orig_toktok, ported_toktok)
    except ImportError:
        pass

    # mwe
    try:
        import ported_lib
        from nltk.tokenize.mwe import MWETokenizer
        def orig_mwe(tokens, mwes, **kw):
            tok = MWETokenizer(mwes, separator=kw.get("separator", "_"))
            return tok.tokenize(tokens)
        def ported_mwe(tokens, mwes, **kw):
            return ported_lib.mwe_tokenize_py(tokens, mwes, kw.get("separator", "_"))
        FUNCTION_PAIRS["mwe_tokenize"] = (orig_mwe, ported_mwe)
    except ImportError:
        pass

    # sexpr
    try:
        import ported_lib
        from nltk.tokenize.sexpr import SExprTokenizer
        def orig_sexpr(text, **kw):
            return SExprTokenizer(parens=kw.get("parens", "()"), strict=kw.get("strict", True)).tokenize(text)
        def ported_sexpr(text, **kw):
            return ported_lib.sexpr_tokenize_py(text, parens=kw.get("parens", "()"), strict=kw.get("strict", True))
        FUNCTION_PAIRS["sexpr_tokenize"] = (orig_sexpr, ported_sexpr)
    except ImportError:
        pass


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

    _try_import_pairs()

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
        if fn_key not in CASE_GENERATORS:
            print(f"WARN: no CASE_GENERATORS for {fn_key!r} — skipping", file=sys.stderr)
            continue
        for case in cases_for(fn_key, size=args.size):
            rows.append(run_case(fn_key, original, ported, case))

    # also cover example.add if present
    if "example.add" in FUNCTION_PAIRS and "example.add" in CASE_GENERATORS:
        # already handled above; dedup guard
        pass

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
        else:
            f.write("All cases passed.\n")

    print(f"{len(rows) - len(failed)}/{len(rows)} matrix cases passed.")
    if failed:
        print(f"{len(failed)} failures — see matrix_report.md", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
