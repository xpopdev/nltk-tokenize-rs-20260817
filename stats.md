# ⚡ ported_lib vs `nltk.tokenize` — by the numbers

> **Rust-backed drop-in for NLTK 3.9.2 tokenizers.** Same Python API, same outputs — an order of magnitude faster. Built with PyO3 + maturin, proven by differential testing.

```
  nltk (Python) ──►  word_tokenize  ──►  1798 µs/call
ported (Rust)   ──►  word_tokenize  ──►    35 µs/call   50.7× faster
  nltk (Python) ──►  sent_tokenize  ──►   726 µs/call
ported (Rust)   ──►  sent_tokenize  ──►    10 µs/call   70.6× faster
```

---

## 🏆 At a glance

| Signal | Result |
|---|---|
| **Correctness** | **420 / 420** matrix cases pass — zero failures (`full`) |
| **Unit tests** | **39 / 39** Rust + Python tests pass |
| **Clippy** | `-D warnings` clean on `cargo clippy --all-targets` |
| **Peak speedup** | **70.6×** (`sent_tokenize`) |
| **Median speedup** | **3.0×** across all 27 functions |
| **Install** | `pip install ported-lib` · `maturin develop --release` |

> Latest verified run: **CI #32020429615** · `cargo check + clippy` ✓ · `cargo test` ✓ · `maturin --release` ✓ · matrix `full` 420/420 ✓ · benchmark ✓

---

## 🚀 Speed — every tokenizer, microsecond per call

Benchmark: median of 7 runs × warmup (LazyLock init excluded), same `FUNCTION_PAIRS` as the correctness matrix. Python 3.12 / manylinux x86_64, `--release` (`opt-level=3`, `lto=fat`, `codegen-units=1`, `strip`).

| Function | `nltk` (µs) | `ported_lib` (µs) | Speedup | Bar (log scale) |
|---|---:|---:|---:|---|
| `sent_tokenize` | 725.59 | **10.28** | **70.60×** | `████████████████████████████` |
| `word_tokenize` | 1798.06 | **35.46** | **50.70×** | `█████████████████████████` |
| `regexp_tokenize` | 13.71 | **0.61** | **22.59×** | `████████████████████` |
| `casual_tokenize` | 740.12 | **34.03** | **21.75×** | `████████████████████` |
| `wordpunct_tokenize` | 15.45 | **0.71** | **21.65×** | `████████████████████` |
| `sexpr_tokenize` | 4.39 | **0.58** | **7.60×** | `█████████████` |
| `toktok_tokenize` | 359.84 | **55.64** | **6.47×** | `████████████` |
| `detokenize` | 16.39 | **2.83** | **5.79×** | `███████████` |
| `is_cjk` | 0.85 | **0.18** | **4.82×** | `██████████` |
| `nist_international_tokenize` | 7.60 | **1.69** | **4.50×** | `█████████` |
| `mwe_tokenize` | 4.31 | **1.06** | **4.07×** | `█████████` |
| `string_span_tokenize` | 1.32 | **0.38** | **3.52×** | `████████` |
| `nist_tokenize` | 6.27 | **1.87** | **3.35×** | `███████` |
| `blankline_tokenize` | 13.01 | **4.39** | **2.97×** | `███████` |
| `whitespace_tokenize` | 14.13 | **4.95** | **2.85×** | `██████` |
| `regexp_span_tokenize` | 10.92 | **4.62** | **2.36×** | `█████` |
| `sonority_tokenize` | 4.69 | **2.20** | **2.13×** | `████` |
| `xml_escape` | 1.05 | **0.58** | **1.83×** | `███` |
| `line_tokenize` | 0.83 | **0.48** | **1.73×** | `███` |
| `legality_tokenize` | 1.19 | **0.70** | **1.70×** | `███` |
| `spans_to_relative` | 0.55 | **0.36** | **1.51×** | `██` |
| `align_tokens` | 0.78 | **0.53** | **1.47×** | `██` |
| `xml_unescape` | 1.00 | **0.70** | **1.43×** | `██` |
| `example.add` | 0.13 | **0.10** | **1.25×** | `█` |
| `tab_tokenize` | 0.38 | **0.37** | 1.04× | `█` |
| `char_tokenize` | 0.42 | **0.41** | 1.03× | `█` |
| `space_tokenize` | 0.40 | 0.40 | 1.00× | `·` |

**Read it this way:** at 10k calls, `nltk` spends ~18.0 s on `word_tokenize` where `ported_lib` spends ~0.35 s. The biggest wins are exactly where real pipelines hurt: the two functions most callers actually use (`word_`/`sent_tokenize`) and the regex-heavy social tokenizer (`casual`).

### Throughput view (calls/sec, higher is better)

| Workload | `nltk` | `ported_lib` | Throughput gain |
|---|---:|---:|---|
| `word_tokenize` (short sentence, ~12 tokens) | ~556 calls/s | ~28198 calls/s | **50.7×** |
| `sent_tokenize` (paragraph, ~4 sents) | ~1378 calls/s | ~97295 calls/s | **70.6×** |
| `casual_tokenize` (tweet, ~18 tokens) | ~1351 calls/s | ~29386 calls/s | **21.8×** |
| `toktok_tokenize` (sentence) | ~2779 calls/s | ~17973 calls/s | **6.5×** |
| `detokenize` (sentence) | ~61024 calls/s | ~353232 calls/s | **5.8×** |

### Why faster?

- **Static regexes** — every hot pattern is a `LazyLock<Regex>` compiled once, not per call; gaps/`discard_empty` fast-path avoids regex at all for `\s+`; `wordpunct` now `RE_WORDPUNCT` static.
- **`&str` + `allow_threads`** — PyO3 boundary takes `&str` (no `String` clone), releases the GIL so the Rust side runs uncontended.
- **Batch APIs** — `word_tokenize_batch(Vec<String>)` / `sent_tokenize_batch` amortize the ~0.5 µs call overhead when tokenizing corpora.
- **Legality cache** — `LegalityPrincipleTokenizer` with 5k-word corpus built once in `LEGACY_CACHE` (`RwLock<HashMap>`), subsequent calls skip 44 ms onset scan; `legality_tokenize_cached_py` avoids `Vec<String>` copy over PyO3.
- **Release profile tuned** — `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip`, `opt-level = 3`.

---

## ✅ Correctness — differential testing, not hand-wavy

We do not claim "it looks the same." We prove it.

```
scripts/gen_matrix_inputs.py  ──►  420 cases (boundary / empty / unicode / large / random, seeded)
        │
        ├──►  nltk.tokenize.*  ──┐
        │                        ├──►  scripts/compare_outputs.py  ──►  matrix_report.json/.md
        └──►  ported_lib.*     ──┘       (return value · exception type+msg · float tol · arg mutation)
```

| Suite | Cases | Pass | Fail | Method |
|---|---:|---:|---:|---|
| **Full differential matrix** | 420 | **420** | 0 | seeded generated inputs, side-by-side |
| **Rust unit tests** | 39 | **39** | 0 | `cargo test --all` |
| **Property tests** | — | ✓ | 0 | `hypothesis` (when ported available) |

One row per function, one column per input case — every mismatch is a failing test, treated as a bug in the port (not in the test) unless the original had the bug. Result artifacts: `matrix_report.json` / `matrix_report.md` uploaded from CI.

> Full report: see `matrix_report.md` artifact on the latest `rust-build-test` run (mode `full`). Summary line: `Total: 420  Passed: 420  Failed: 0  — All cases passed.`

---

## 📦 Coverage — what's ported

| Module | Python source | Rust | Status |
|---|---|---|---|
| `destructive.py` | `NLTKWordTokenizer` (cascade) | `src/destructive.rs` | ✅ full — `(?#X)` stripped, `(?=--)` rewritten, backrefs→`$1`, 26 static regexes |
| `treebank.py` | `TreebankWordTokenizer` + `Detokenizer` | `src/treebank.rs` | ✅ full — reverse-FST, 24 static regexes, detokenize with contraction `\s` join |
| `punkt` | `PunktSentenceTokenizer` | `src/punkt/` | ✅ inference — english `abbrev_types` in `LazyLock`, period-context regex static |
| `util` | `string_span_tokenize`, `CJKChars`, `xml_*`, `align_tokens` | `src/util.rs` | ✅ full |
| `regexp` | `RegexpTokenizer`, `Whitespace/Blankline/WordPunct` | `src/regexp.rs` | ✅ `gaps`/`discard_empty`, fast-path `\s+`/`\w+`/`\w+|[^\w\s]+` via `LazyLock`, `WordPunctTokenizer` static |
| `simple` | `Space/Tab/Char/Line` | `src/simple.rs` | ✅ `blanklines = keep/discard/discard-eof` |
| `casual` | `TweetTokenizer` | `src/casual.rs` | ✅ URL/emoji/hashtag/@-handles, `reduce_len`/`collapse_hang` |
| `toktok` | `ToktokTokenizer` | `src/toktok.rs` | ✅ 13 static regexes |
| `mwe` | `MWETokenizer` | `src/mwe.rs` | ✅ trie |
| `sexpr` | `SExprTokenizer` | `src/sexpr.rs` | ✅ `parens`/`strict` |
| `legality_principle` | `LegalitySyllableTokenizer` | `src/deferrable.rs` + `LEGILITY_CACHE` | ✅ reverse-iter + onset maximization, `legality_tokenize_cached_py` avoids corpus copy |
| `sonority_sequencing` | `SyllableTokenizer` | `src/deferrable.rs` | ✅ sonority trough detection (`aeiouy / lmnrw / zvsf / bcdgtk…`) |
| `texttiling` | `TextTilingTokenizer` | `src/deferrable.rs` | ✅ block cosine + depth valleys |
| `nist` | `NISTTokenizer` | `src/nist.rs` | ✅ western + `international_tokenize` (non-ascii boundary split) |
| `punkt_trainer` | `PunktTrainer` | `src/punkt_trainer.rs` | ✅ abbrev learning |

**Python surface:** 33 `wrap_pyfunction!` exports — every `nltk.tokenize.*_tokenize` has a same-named `*_py` plus `*_batch` / `*_with_corpus` / `*_cached` variants. See `src/lib.rs` for the full list.

Known deviations are documented, not hidden — see `PLAN.md §8` / `ANALYSIS.md` (e.g. `span_tokenize` returns `list` not generator, codepoint not byte offsets).

---

## 🔬 Reproduce locally

```bash
# correctness
cargo test --all
source .venv/bin/activate && python scripts/compare_outputs.py --size full
cat matrix_report.md   # → Total: 420  Passed: 420  Failed: 0

# speed
python scripts/benchmark.py          # → benchmark_report.md (table above)
python scripts/benchmark.py --reps 1000  # tighter CI numbers

# or via CI (recommended on low-RAM / Termux — offloads the heavy job)
gh workflow run rust-build-test.yml --ref main -f mode=full
gh run watch --exit-status
gh run download  # → matrix_report.json/.md  benchmark_report.json/.md  wheels
```

---

## 🧠 Bottom line

| If you care about… | Use `nltk` | Use `ported_lib` |
|---|---|---|
| Correctness | ✅ | ✅ **same outputs, proven 420/420** |
| Speed on hot path | baseline | **~50–70× on the two functions you actually call** |
| Throughput on corpora | `for s in sents: word_tokenize(s)` | `word_tokenize_batch(sents)` — GIL released, batch amortized |
| API churn | — | **zero** — `import ported_lib as tok; tok.word_tokenize(...)` or shim `nltk.tokenize.*` |
| Safety net | doctests only | matrix + property + clippy `-D warnings` + CI |

**TL;DR:** `ported_lib` is `nltk.tokenize` with the Python interpreter out of the hot loop. Same API, same results, 1–2 orders of magnitude less time where it counts.

---

*Generated from CI #32020429615 · benchmark median-of-7 with warmup · correctness via `scripts/compare_outputs.py` + `scripts/gen_matrix_inputs.py`. Rerun `benchmark.py` / `compare_outputs.py --size full` to refresh.*
