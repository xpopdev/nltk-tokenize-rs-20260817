# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-08-18

### Added
- Full tokenizer family ported from `nltk.tokenize` (NLTK 3.9.2/3.10.x): `NLTKWordTokenizer` (destructive.py), `PunktSentenceTokenizer`/`sent_tokenize` with real NLTK params (156 abbrevs, 37 collocations, 39 sent starters, 20k ortho entries via `punkt-fancy` + `fancy-regex`), `RegexpTokenizer`/`Whitespace`/`Blankline`/`WordPunct`, `Space`/`Tab`/`Line`/`Char`, `TweetTokenizer`/`casual_tokenize`, `ToktokTokenizer`, `MWETokenizer`, `SExprTokenizer`, `NIST`, `Legality`/`Sonority`, `TextTiling` (simplified), `util` helpers (`string_span_tokenize`, `regexp_span_tokenize`, `align_tokens`, `is_cjk`, `xml_escape`).
- Batch APIs (`*_batch`, `*_batch_gpu`) via rayon `par_map` for corpus-scale throughput.
- Differential matrix suite — 442/442 full, 207/207 smoke, 41/41 Rust unit — with 22 hard Punkt cases (ortho/colloc/quotes/numbers).

### Performance
- Benchmark results in [`benchmark_report.md`](benchmark_report.md) — representative: `word_tokenize` 44.68×, `regexp_span` 34.54×, `sent_tokenize` 4.40×, `casual` 24.13×. Shipped report is median of 7 runs (`--reps 200`) from CI.

### Known deviations (from `PLAN.md §8`)
- **Punkt training** stays Python fallback (`src/punkt_trainer.rs` is a no-op); inference covers the shipped `english` pickle which is embedded as Rust data. Use NLTK's Python `PunktTrainer` to train new models.
- **NIST/Stanford/REPP wrappers** remain importable stubs (they shell out to external Java/binary; no pure logic to port).
- **TextTiling** is a simplified cosine approximation without TF-IDF weighting (see `PLAN.md §8`).
- **`span_tokenize`** returns `list` not generator; **span indices** are codepoint offsets; **LRU cache** semantics match Python's `@lru_cache` (see `PLAN.md §8`).

### Packaging
- `rust-version = "1.80"` — `std::sync::LazyLock` requires ≥1.80; older toolchains now fail with a clear error.
- Optional `gpu` feature (wgpu/bytemuck/pollster) removed from 1.0.0 surface — no tokenizer calls it; batch parallelism is CPU (rayon) and not gated on GPU.
