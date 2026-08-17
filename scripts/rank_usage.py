"""
Ranks a Python package's public symbols by how often they're actually
called/imported -- inside the package itself, its own tests, and
optionally an external directory that depends on it -- so the porting plan
can tackle the highest-impact code first instead of an arbitrary order.

Deliberately simple: name-based call-site counting via the `ast` module,
not full cross-module resolution (it doesn't track which `foo` a call to
`foo()` actually resolves to if multiple modules define a `foo`). It's a
prioritization *signal* for the planner agent, not a precise call graph.
Treat close scores as a tie and break it by risk/size instead.

Usage:
    python scripts/rank_usage.py path/to/package [extra search dir ...]

Prints ranked JSON on stdout (most-used first) and a human-readable table
on stderr.
"""
from __future__ import annotations

import ast
import json
import sys
from collections import Counter
from pathlib import Path


def public_symbols(pkg_path: Path) -> dict[str, str]:
    """name -> defining file, for every top-level function/class and
    method not prefixed with an underscore (leading-underscore = treated
    as private/internal, excluded from ranking)."""
    symbols: dict[str, str] = {}
    for py_file in pkg_path.rglob("*.py"):
        if "__pycache__" in py_file.parts:
            continue
        try:
            tree = ast.parse(py_file.read_text(errors="replace"), filename=str(py_file))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                if not node.name.startswith("_"):
                    symbols.setdefault(node.name, str(py_file))
    return symbols


def count_usage(search_paths: list[Path], names: set[str]) -> Counter:
    counts: Counter = Counter()
    for root in search_paths:
        if not root.exists():
            continue
        for py_file in root.rglob("*.py"):
            if "__pycache__" in py_file.parts:
                continue
            try:
                tree = ast.parse(py_file.read_text(errors="replace"), filename=str(py_file))
            except SyntaxError:
                continue
            for node in ast.walk(tree):
                if isinstance(node, ast.Call):
                    fn = node.func
                    if isinstance(fn, ast.Name) and fn.id in names:
                        counts[fn.id] += 1
                    elif isinstance(fn, ast.Attribute) and fn.attr in names:
                        counts[fn.attr] += 1
                elif isinstance(node, ast.ImportFrom):
                    for alias in node.names:
                        if alias.name in names:
                            counts[alias.name] += 1
    return counts


def main() -> int:
    if len(sys.argv) < 2:
        print("Usage: rank_usage.py path/to/package [extra search dir ...]", file=sys.stderr)
        return 1

    pkg_path = Path(sys.argv[1]).resolve()
    search_paths = [pkg_path] + [Path(p).resolve() for p in sys.argv[2:]]

    symbols = public_symbols(pkg_path)
    if not symbols:
        print("[]")
        return 0

    counts = count_usage(search_paths, set(symbols))

    ranked = sorted(
        (
            {"name": name, "defined_in": file, "usage_count": counts.get(name, 0)}
            for name, file in symbols.items()
        ),
        key=lambda r: r["usage_count"],
        reverse=True,
    )

    print(json.dumps(ranked, indent=2))

    print(f"\n{'Symbol':<30} {'Uses':<8} Defined in", file=sys.stderr)
    print("-" * 70, file=sys.stderr)
    for r in ranked[:40]:
        print(f"{r['name']:<30} {r['usage_count']:<8} {r['defined_in']}", file=sys.stderr)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
