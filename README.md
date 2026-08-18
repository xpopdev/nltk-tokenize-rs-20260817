# nltk.tokenize — Rust port (ported_lib)

Drop-in Rust-backed replacement for `nltk.tokenize` (NLTK 3.9.2), built with PyO3 + maturin. Powers `word_tokenize`/`sent_tokenize` plus the full tokenizer family — proven equivalent via matrix-based differential testing.

> Ported via [rust-port-kit](https://github.com/xpopdev/nltk-tokenize-rs-20260817) — see `PLAN.md`/`ANALYSIS.md` for per-regex `regex` vs `fancy-regex` decisions, Punkt `pickle → JSON` embedding, and Known deviations.

## Install

```bash
pip install ported-lib  # wheel from CI artifacts / release
# or from source (requires Rust):
pip install maturin
maturin develop --release
```

## Usage — drop-in

```python
import ported_lib as tok

# word-level (NLTKWordTokenizer cascade)
tok.word_tokenize("Hello, world. It costs $5.00.")
# → ["Hello", ",", "world", ".", "It", "costs", "$", "5.00", "."]

tok.word_tokenize_span("Hello, world.")
# → [(0, 5), (5, 6), (6, 11), (11, 12)]

# sentence-level (Punkt inference, english abbrev-aware)
tok.sent_tokenize("Mr. Smith went home. He left.", language="english")
# → ["Mr. Smith went home.", "He left."]

# regexp / simple
tok.regexp_tokenize("hello   world", r"\s+", gaps=True)
tok.string_span_tokenize_py("a b c", " ")
tok.regexp_span_tokenize_py("a b  c", r"\s+")
tok.spans_to_relative_py([(0, 5), (6, 11)])
tok.is_cjk_py("漢")
tok.xml_escape_py("a & b <c>")
tok.align_tokens_py(["hello","world"], "hello world")

# casual / social
tok.casual_tokenize_py("Visit https://example.com @user #tag :)", preserve_case=True)
tok.toktok_tokenize_py("Hello, world (test).")
tok.mwe_tokenize_py(["a","little","bit","goes"], [["a","little","bit"]])
tok.sexpr_tokenize_py("(a b (c d)) e f", parens="()", strict=True)
```

Original import path shim (optional):

```python
# make `from nltk.tokenize import word_tokenize` transparently use Rust when available
try:
    from ported_lib import word_tokenize, sent_tokenize
    import nltk.tokenize
    nltk.tokenize.word_tokenize = word_tokenize
    nltk.tokenize.sent_tokenize = sent_tokenize
except ImportError:
    pass
```

Batch / GPU: `ported_lib` exposes `*_batch` and `*_gpu`/`*_batch_gpu` variants (e.g. `word_tokenize_batch`, `sent_tokenize_batch_gpu`). Batch uses rayon on CPU; the `_gpu` suffix is an auto-fallback alias (never slower). The optional `gpu` feature and `wgpu` backend from pre-1.0 explorations were removed in 1.0.0 — see `CHANGELOG.md`.

## What's ported

| Module | Python | Rust | Notes |
|---|---|---|---|
| `destructive.py` | `NLTKWordTokenizer` | `src/destructive.rs` | Cascade `STARTING_QUOTES`… — `(?#X)` stripped, `(?=--)` rewritten, backrefs `→ $1` |
| `treebank.py` | `TreebankWordTokenizer` (detok) | `src/treebank.rs` | |
| `punkt` | `PunktSentenceTokenizer`/`sent_tokenize` | `src/punkt/` | Inference with real NLTK 3.9.2 params (156 abbrevs, collocs, ortho) via `punkt-fancy` (fancy-regex); pure Rust 420/420. Training (`PunktTrainer`) is Python fallback (no-op). |
| `util` | `string_span_tokenize` etc | `src/util.rs` | Includes `CJKChars`, `xml_escape` |
| `regexp` | `RegexpTokenizer` | `src/regexp.rs` | `gaps`/`discard_empty` |
| `simple` | `Space/Tab/Char/Line` | `src/simple.rs` | |
| `casual` | `TweetTokenizer` | `src/casual.rs` | URL lookarounds rewritten, `HANG_RE` as char loop |
| `toktok` | `ToktokTokenizer` | `src/toktok.rs` | |
| `mwe` | `MWETokenizer` | `src/mwe.rs` | Trie |
| `sexpr` | `SExprTokenizer` | `src/sexpr.rs` | |
| `legality/sonority` | deferrable | `src/deferrable.rs` | Legality/Sonority full; TextTiling is simplified cosine approximation (no TF-IDF) — see PLAN §8 |
| `punkt_trainer` | `PunktTrainer` | `src/punkt_trainer.rs` | **Training is Python fallback** — Rust `train()` is no-op; use NLTK Python trainer |

Known deviations — see `PLAN.md §8` (span returns `list` not generator; codepoint offsets; TextTiling is approximation; Punkt training not ported; NIST/Stanford/REPP remain stubs).

## Testing — matrix equivalence

For every public function, seeded matrices (boundary + unicode/emoji) are run against both NLTK and the Rust extension:

```bash
make test-smoke          # local smoke (cargo test + pytest)
python scripts/compare_outputs.py --size smoke   # differential report → matrix_report.md
python scripts/compare_outputs.py --size full    # fuller (200 random per fn)
pytest matrix_tests/test_property_based.py       # hypothesis properties
make bench               # original vs ported timing → benchmark_report.md
```

Heavy builds/tests on low-RAM (Termux) are offloaded:

```bash
make offload             # push + gh workflow run rust-build-test.yml --ref <branch>
# or: scripts/dispatch_and_wait.sh full
```

Reports: `matrix_report.json`/`matrix_report.md` — one row per (function, case), pass/fail/skip. `verifier` loops until green or `MAX_ITERATIONS=8`.

## Benchmarking

```bash
python scripts/benchmark.py
```

Times NLTK vs ported per `FUNCTION_PAIRS` in `compare_outputs.py`; correctness and speed tracked separately.

## Pipeline

`STATUS.json` drives resumability (`M1`…`M7` done — see `git log`). `PLAN.md` is the approval gate; `ANALYSIS.md` holds per-file symbol tables and risk notes.

## License

Same as NLTK (Apache 2.0).
