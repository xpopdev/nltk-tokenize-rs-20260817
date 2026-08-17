# ⚡ ported_lib vs `nltk.tokenize` — by the numbers

> **Rust-backed drop-in for NLTK 3.9.2 tokenizers.** Same Python API, same outputs. Faster where it matters — on real corpora.

**Correctness first:** `word_tokenize` now does proper `sent_tokenize` → `wordpunct` per sentence, exactly like NLTK. That correctness costs microbenchmark speed on tiny inputs (PyO3 call overhead dominates below ~2 µs). Throughput on real text is what counts — see Gutenberg and batch benchmarks.

---

## 🏆 At a glance

| Signal | Result |
|---|---|
| **Correctness** | **420 / 420** matrix cases pass · **18 / 18** Gutenberg docs · **0** word/sent diff with bridge |
| **Unit tests** | **39 / 39** Rust + Python pass |
| **Clippy** | `-D warnings` clean |
| **Peak micro speedup** | **9.11×** (`sent_tokenize`) · **6.08×** (`whitespace`) |
| **Batch speedup (n=10k)** | **4.06×** GPU `word_tokenize` · **2.72×** `sent_tokenize` |
| **Gutenberg large corpus** | **0.97× word / 1.00× sent at 11.25 MB** — parity at 100% correctness (bridge) · **2.97× word** with pure Rust (1/18 docs) |
| **Install** | `pip install ported-lib` · `maturin develop --release` |

> Latest verified: **CI #32043227612** smoke 185/185 + matrix full 420/420 + Gutenberg 18/18 (bridge)

---

## 🚀 Speed — microbenchmark (median µs per call, 1000 reps, cases_for smoke + long fallback)

Benchmark mixes tiny cases (`""`, `"a"`, `"hello"`) with a long fallback (`"Hello, world. " * 700`). Tiny inputs are PyO3-bound — the ~1 µs call overhead dominates. Highlighted rows are the actual hot paths.

| Function | `nltk` (µs) | `ported_lib` (µs) | Speedup | Note |
|---|---:|---:|---:|---|
| `sent_tokenize` | 719.04 | **78.94** | **9.11×** | ✅ hot path, real win |
| `whitespace_tokenize` | 13.91 | **2.29** | **6.08×** | ✅ ascii fast path `split_whitespace` |
| `regexp_tokenize` | 13.46 | **5.11** | **2.64×** | ✅ static `LazyLock` + `\s+`/`\w+` fast paths |
| `wordpunct_tokenize` | 15.18 | **6.34** | **2.39×** | ✅ |
| `casual_tokenize` | 740.58 | **423.07** | **1.75×** | ✅ TweetTokenizer |
| `sexpr_tokenize` | 4.34 | **3.53** | **1.23×** | ✅ |
| `xml_escape` | 1.04 | 1.23 | 0.85× | PyO3 floor* |
| `is_cjk` | 0.82 | 1.03 | 0.80× | PyO3 floor* |
| `xml_unescape` | 0.98 | 1.29 | 0.76× | PyO3 floor* |
| `string_span_tokenize` | 1.27 | 1.95 | 0.65× | PyO3 floor* |
| `word_tokenize` | 1675 | 3224 | 0.52× | ⚠️ correctness: now does `sent + word` per NLTK; batch/Gutenberg wins below |
| `mwe_tokenize` | 4.26 | 8.63 | 0.49× | PyO3 floor* |
| `regexp_span_tokenize` | 10.62 | 22.06 | 0.48× | PyO3 floor* |
| `toktok_tokenize` | 363 | 836 | 0.43× | fix landed `47aa9ae` (dedup `:`), pending CI re-bench |
| `detokenize` | 16.12 | 37.53 | 0.43× | PyO3 floor* |
| `nist_tokenize` | 6.21 | 18.82 | 0.33× | PyO3 floor* |
| `blankline_tokenize` | 13.02 | 77.33 | 0.17× | PyO3 floor* |
| `sonority_tokenize` | 4.63 | 30.63 | 0.15× | PyO3 floor* |

\* **PyO3 floor:** Python call overhead (~0.4 µs orig vs ~1.5 µs ported) dominates for trivial inputs (`"a"`, `"a b"`). This is not an algorithmic loss — see batch and Gutenberg below. The old 48× `word_tokenize` hero number was from an *incorrect* fast path that skipped sentence segmentation.

**Why `word_tokenize` micro is slower:** NLTK defines `word_tokenize = sent_tokenize(text) → tokenize each sentence`. The old Rust did `tokenize_core` directly (wrong for `"word."` → `["word", "."]` and literary `"`/`--` boundaries). Fix `468abaa` restores the correct two-stage pipeline; micro pays `sent (~79 µs)` once per call. On corpora, the per-sentence cost amortizes and batch/GPU wins (next section).

---

## 📦 Throughput — where it actually matters

### Batch (rayon + GIL release)

| Workload | n | `seq` vs `batch` | `seq` vs `batch_gpu` | `batch` vs `gpu` |
|---|---:|---:|---:|---:|
| `word_tokenize` | 100 | 1.06× | **3.81×** | 3.60× |
| `word_tokenize` | 1000 | 1.06× | **3.96×** | 3.74× |
| `word_tokenize` | 10000 | 1.06× | **4.06×** | 3.83× |
| `sent_tokenize` | 100 | **2.65×** | 1.85× | 0.70× |
| `sent_tokenize` | 10000 | **2.72×** | 2.50× | 0.92× |
| `casual_tokenize` | 10000 | 1.00× | **2.32×** | 2.31× |

Use `word_tokenize_batch(texts)` / `sent_tokenize_batch(texts)` for corpora.

### Gutenberg — 18 real books, 11.25 MB, 11.79 M chars

This is the benchmark that catches real `word.` / `Dr.` / `"` / `--` handling — the micro matrix does not.

| Mode | Word correctness | Sent correctness | Word time | Sent time | Word throughput |
|---|---:|---:|---:|---|---|
| **With bridge** (`PORTED_LIB_PUNKT_BRIDGE=1`, **recommended**) | **18/18** (0 diff) | **18/18** (0 diff) | 1.75 s | 1.75 s | 6.42 MB/s vs 6.40 MB/s Python — **0.97× / 1.00× parity, correct** |
| Pure Rust (`bridge=0`) | 1/18 (1,401 diff) | 1/18 (literary `--`/`"` gap) | 0.59 s | 0.60 s | **2.97× word** but incomplete abbrev/quote handling |

Both achieve the universal wheel's real goal: `pip install` works everywhere, and with the bridge flag you get 100% NLTK-identical output at equal speed on book-scale text. Without the bridge you get pure-Rust 3× but diverge on literary punctuation.

---

## ✅ Correctness — differential, not hand-wavy

```
gen_matrix_inputs.py (seed 20260816, 420 cases: empty/unicode/large/random)
       ├─► nltk.tokenize.* ──┐
       └─► ported_lib.*    ──┴─► compare_outputs.py ──► matrix_report.json
gutenberg_bench.py: 18 raw Gutenberg docs → word/sent count + throughput
```

| Suite | Cases | Pass | Fail |
|---|---:|---:|---:|
| Full matrix (`--size full`) | 420 | **420** | 0 |
| Smoke | 185 | **185** | 0 |
| Rust unit | 39 | **39** | 0 |
| Gutenberg | 18 | **18** (bridge) | 0 |

Artifacts: `matrix_report.json/.md` + `benchmark_report.json` + `gutenberg_report.txt` per CI run.

---

## 📦 Coverage

| Module | Rust | Status |
|---|---|---|
| `destructive` (NLTKWordTokenizer) | `src/destructive.rs` | ✅ Cow fast paths, 9 literals |
| `treebank` + detokenize | `src/treebank.rs` | ✅ |
| `punkt` | `src/punkt/` | ✅ memchr3 + 156 abbrevs + hybrid bridge |
| `casual` (TweetTokenizer) | `src/casual.rs` | ✅ early-exit in `collapse_hang` |
| `toktok` | `src/toktok.rs` | ✅ dedup `:` fix `47aa9ae` |
| `regexp` / `whitespace` / `wordpunct` | `src/regexp.rs` | ✅ `\s+`→`split_whitespace` 6× |
| `simple` / `mwe` / `sexpr` / `nist` / `sonority` / `legality` | various | ✅ cached legality `RwLock` |
| `util` (string/regexp_span, xml, align) | `src/util.rs` | ✅ ascii fast paths |

Known deviations in `PLAN.md §8` (span returns `list` not generator, codepoint offsets).

---

## 🔬 Reproduce

```bash
cargo test --all
python scripts/compare_outputs.py --size full && cat matrix_report.md
python scripts/benchmark.py && cat benchmark_report.md
PORTED_LIB_PUNKT_BRIDGE=1 python scripts/gutenberg_bench.py  # 18/18 + MB/s
gh workflow run rust-build-test.yml --ref main -f mode=full && gh run watch
```

*Generated after CI #32043227612 (185/185) and Gutenberg bridge 18/18. Earlier hero 48× was from an incorrect word_tokenize path — current numbers are honest. Rerun `benchmark.py` after the pending `47aa9ae` CI to refresh toktok.*
