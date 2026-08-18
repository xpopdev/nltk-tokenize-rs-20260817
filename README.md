# nltk-tokenize-rs

[![CI](https://github.com/xpopdev/nltk-tokenize-rs-20260817/actions/workflows/rust-build-test.yml/badge.svg)](https://github.com/xpopdev/nltk-tokenize-rs-20260817/actions)
[![PyPI](https://img.shields.io/badge/pip-ported--lib-blue)](https://github.com/xpopdev/nltk-tokenize-rs-20260817/releases)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](LICENSE)
[![Python 3.9–3.13](https://img.shields.io/badge/python-3.9--3.13-blue)](pyproject.toml)
[![Open In Colab](https://colab.research.google.com/assets/colab-badge.svg)](https://colab.research.google.com/github/xpopdev/nltk-tokenize-rs-20260817/blob/main/benchmark_speed_test.ipynb)

**Rust-backed drop-in for `nltk.tokenize` (NLTK 3.9.2/3.10.x) — 44× faster `word_tokenize`, 486/486 differential-test parity.**

Built with PyO3 + maturin. Powers `word_tokenize`/`sent_tokenize` plus the full tokenizer family. Swapping the wheel is the only change — same imports, same outputs.

## Install

```bash
pip install ported-lib          # pre-built wheel (recommended)
# — or from source (requires Rust ≥1.80) —
pip install maturin && maturin develop --release
```

## Quickstart

```python
import ported_lib as tok

tok.word_tokenize("Hello, world. It costs $5.00.")
# ['Hello', ',', 'world', '.', 'It', 'costs', '$', '5.00', '.']

tok.sent_tokenize("Mr. Smith went home. He left.", language="english")
# ['Mr. Smith went home.', 'He left.']

tok.regexp_tokenize("hello   world", r"\s+", gaps=True)

# spans, CJK, xml, align
tok.string_span_tokenize_py("a b c", " ")
tok.is_cjk_py("漢")
tok.xml_escape_py("a & b <c>")
```

**Zero-code-change shim** (patches `nltk` at import time):

```python
try:
    from ported_lib import word_tokenize, sent_tokenize
    import nltk.tokenize
    nltk.tokenize.word_tokenize = word_tokenize
    nltk.tokenize.sent_tokenize = sent_tokenize
except ImportError:
    pass  # fallback to pure-Python NLTK

# now every downstream call uses Rust
import nltk
nltk.word_tokenize("Hello, world.")
nltk.sent_tokenize("Mr. Smith went home. He left.")
```

See [`examples/`](examples/) for `basic.py`, `shim.py`, and `batch.py`.

## Benchmarks

Median µs/call (`--reps 200`, `benchmark_report.md`, CI #32133315068, release build):

| Function | `nltk` | `ported_lib` | Speedup |
|---|---:|---:|---:|
| `word_tokenize` | 1694.67 | **37.70** | **44.95×** |
| `blankline_tokenize` | 13.25 | **0.37** | **35.89×** |
| `whitespace_tokenize` | 14.18 | **0.40** | **35.61×** |
| `regexp_span_tokenize` | 10.91 | **0.32** | **34.68×** |
| `casual_tokenize` | 736.03 | **30.00** | **24.54×** |
| `detokenize` | 16.35 | **0.78** | **21.02×** |
| `sent_tokenize` | 218.19 | **49.57** | **4.40×** |
| `space/tab/char` | 0.40 | **0.33** | **1.2×** |

Full table → [`stats.md`](stats.md) · raw → [`benchmark_report.md`](benchmark_report.md)

> All 486/486 differential cases pass (incl. 44 shim cases). `space/tab/char` were sub-µs FFI-bound; F1 fast-path now wins at 1.2× — no function below 1×.

## Project structure

```
src/                  # Rust crate (ported_lib)
  api.rs              # TokenizerI trait
  destructive.rs      # NLTKWordTokenizer
  punkt/              # Punkt sentence segmenter (real NLTK params)
  treebank.rs         # Treebank detokenizer (zero-regex single-pass)
  casual.rs           # TweetTokenizer
  regexp.rs / simple.rs / toktok.rs / mwe.rs / sexpr.rs / nist.rs
  util.rs             # CJK, xml, align, spans
  deferrable.rs       # Legality / Sonority / TextTiling
  gpu/                # batch = rayon CPU (wgpu removed in 1.0.0)
matrix_tests/         # hypothesis + shim integration tests
scripts/              # gen_matrix_inputs, compare_outputs, benchmark
examples/             # basic.py, shim.py, batch.py
.github/workflows/    # rust-build-test, wheels, release-wheels
```

## What's ported

| Module | Rust | Notes |
|---|---|---|
| `destructive.py` | `src/destructive.rs` | `(?#X)` stripped, `(?=--)` rewritten |
| `treebank.py` | `src/treebank.rs` | |
| `punkt` | `src/punkt/` | real NLTK 3.9.2 params via `punkt-fancy` (fancy-regex) — 486/486 |
| `util` | `src/util.rs` | CJK, xml_escape, spans |
| `regexp` / `simple` / `toktok` / `mwe` / `sexpr` / `nist` | various | |
| `deferrable` | `src/deferrable.rs` | Legality/Sonority full; TextTiling = cosine approx (no TF-IDF) |
| `punkt_trainer` | `src/punkt_trainer.rs` | **Python fallback** — no-op |
| `punkt` training, NIST/Stanford/REPP still Python shims — see `PLAN.md §8` |

Known deviations: `span_tokenize` returns `list` not generator; codepoint offsets; TextTiling is approximation. Full list → `PLAN.md §8` + `CHANGELOG.md`.

## Testing

Differential testing against NLTK on seeded matrices — same inputs, same outputs:

```bash
make check              # cargo check + clippy
make test-smoke         # cargo test + small matrix
python scripts/compare_outputs.py --size full   # → matrix_report.md (486/486)
pytest matrix_tests/     # hypothesis + shim
make bench              # → benchmark_report.md
```

Heavy builds on low-RAM (Termux) are offloaded to CI:

```bash
make offload MODE=full  # push + gh workflow run rust-build-test.yml
```

## Release

`1.0.0` — see [`CHANGELOG.md`](CHANGELOG.md). Tag `v1.0.0` is the release (all R1–R6 green). Wheels for `cp39`–`cp313` on Linux x86_64/aarch64 via `cibuildwheel`; macOS/Windows via `release-wheels`.

## License

Apache-2.0 — see [`LICENSE`](LICENSE). Same as NLTK.

## Acknowledgements

Ported via [rust-port-kit](https://github.com/xpopdev/nltk-tokenize-rs-20260817) — see `PLAN.md`/`ANALYSIS.md` for per-regex decisions and risk notes.
