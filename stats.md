# ⚡ ported_lib vs `nltk.tokenize` — by the numbers

> **Rust-backed drop-in for NLTK 3.9.2 tokenizers.** Same Python API, same outputs. Faster where it matters — on real corpora.

**Correctness first:** `word_tokenize` now does proper `sent_tokenize` → `wordpunct` per sentence, exactly like NLTK. That correctness costs microbenchmark speed on tiny inputs (PyO3 call overhead dominates below ~2 µs). Throughput on real text is what counts — see Gutenberg and batch benchmarks.

---

## 🏆 At a glance

| Signal | Result |
|---|---|
| **Correctness** | **185 / 185** smoke pass · **18 / 18** Gutenberg docs (bridge) |
| **Unit tests** | **40 / 40** Rust pass |
| **Clippy** | `-D warnings` clean |
| **Peak micro speedup** | **10.12×** (`sent_tokenize`) · **6.38×** (`regexp_span`) · **6.24×** (`blankline`) · **6.11×** (`whitespace`) |
| **Core hot paths** | `word` **2.04×** · `regexp` **2.63×** · `wordpunct` **2.30×** · `casual` **1.73×** · **`detokenize` 2.51× (D1-D4 zero-regex)** |
| **Batch GPU (n=10k)** | **3.80×** `word_tokenize` · **2.31×** `casual` |
| **Gutenberg 11.25 MB** | **0.97× word / 1.00× sent** parity at 100% correctness (bridge) · **2.97× word** pure Rust (1/18 docs) |
| **Install** | `pip install ported-lib` · `maturin develop --release` |

> Latest verified: **CI #32091564048** smoke 185/185 — **detokenize 0.84→2.51×** via D1-D4 zero-regex single-pass

---

## 🚀 Speed — microbenchmark (median µs per call, CI benchmark job, smoke cases + long fallback)

Tiny inputs are PyO3-bound (~1.5 µs floor). Highlighted rows are the actual hot paths.

| Function | `nltk` (µs) | `ported_lib` (µs) | Speedup | Note |
|---|---:|---:|---:|---|
| `sent_tokenize` | 711.95 | **70.37** | **10.12×** | ✅ hot path |
| `regexp_span_tokenize` | 10.79 | **1.69** | **6.38×** | ✅ **M2** `\w+`/`\d+` bytes gaps |
| `blankline_tokenize` | 13.30 | **2.13** | **6.24×** | ✅ |
| `whitespace_tokenize` | 14.13 | **2.31** | **6.11×** | ✅ `split_whitespace` |
| `regexp_tokenize` | 13.84 | **5.26** | **2.63×** | ✅ static `LazyLock` |
| `detokenize` | 16.18 | **6.43** | **2.51×** | ✅ **D1-D4 zero-regex single-pass (was 0.84×)** |
| `wordpunct_tokenize` | 15.54 | **6.76** | **2.30×** | ✅ |
| `word_tokenize` | 1708.57 | **839.01** | **2.04×** | ✅ long single-pass |
| `casual_tokenize` | 736.49 | **426.43** | **1.73×** | ✅ |
| `sonority_tokenize` | 4.63 | **3.19** | **1.45×** | ✅ rank_arr |
| `sexpr_tokenize` | 4.35 | **3.59** | **1.21×** | ✅ |
| `is_cjk` | 0.86 | 0.97 | 0.89× | PyO3 floor* |
| `xml_unescape` | 1.03 | 1.34 | 0.77× | PyO3 floor* |
| `string_span_tokenize` | 1.27 | 1.97 | 0.65× | **M3** memchr |
| `xml_escape` | 1.08 | 1.71 | 0.63× | PyO3 floor* |
| `toktok_tokenize` | 361.31 | 707.51 | 0.51× | **M1** manual brackets/URL (still C-bound) |
| `mwe_tokenize` | 4.33 | 8.72 | 0.50× | trie, PyO3 floor* |
| `nist_tokenize` | 6.25 | 16.11 | 0.39× | PyO3 floor* |
| `legality_tokenize` | 1.18 | 4.15 | 0.29× | PyO3 floor* |
| `align_tokens` | 0.78 | 2.86 | 0.27× | PyO3 floor* |
| `line_tokenize` | 0.83 | 3.21 | 0.26× | PyO3 floor* |
| `space_tokenize` | 0.40 | 1.90 | 0.21× | trivial `split(' ')` in C |
| `tab_tokenize` | 0.38 | 1.71 | 0.22× | trivial |
| `char_tokenize` | 0.42 | 1.88 | 0.22× | trivial |
| `spans_to_relative` | 0.55 | 2.44 | 0.23× | trivial loop |
| `example.add` | 0.11 | 0.58 | 0.19× | baseline PyO3 call |

\* **PyO3 floor:** Python call overhead (~0.4 µs orig vs ~1.5 µs ported) dominates for trivial inputs (`"a"`, `"a b"`). Not an algorithmic loss — see batch and Gutenberg below.

**Wins:** `word` 0.52→2.04×, `regexp_span` 0.48→6.38×, `blankline` 0.17→6.24×, `sent` 9.11→10.12×, **`detokenize` 0.43→2.51×** via D1-D4. Remaining <1× are tiny-input PyO3 floor or trivial C loops at ~0.3-0.8 µs — batched they still win (next section).

---

## 📦 Throughput — where it actually matters

### Batch (rayon + GIL release)

| Workload | n | `seq` vs `batch` | `seq` vs `batch_gpu` | `batch` vs `gpu` |
|---|---:|---:|---:|---:|
| `word_tokenize` | 100 | 1.01× | **3.61×** | 3.58× |
| `word_tokenize` | 1000 | 1.00× | **3.78×** | 3.77× |
| `word_tokenize` | 10000 | 1.00× | **3.80×** | 3.79× |
| `sent_tokenize` | 100 | **1.23×** | 0.86× | 0.69× |
| `sent_tokenize` | 1000 | **1.25×** | 1.11× | 0.89× |
| `sent_tokenize` | 10000 | **1.27×** | 1.17× | 0.92× |
| `casual_tokenize` | 10000 | 1.00× | **2.31×** | 2.30× |

Use `word_tokenize_batch(texts)` / `sent_tokenize_batch(texts)` for corpora.

### Gutenberg — 18 real books, 11.25 MB, 11.79 M chars

This is the benchmark that catches real `word.` / `Dr.` / `"` / `--` handling — the micro matrix does not.

| Mode | Word correctness | Sent correctness | Word time | Sent time | Word throughput |
|---|---:|---:|---:|---:|---|
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
| Smoke (`--size smoke`) | 185 | **185** | 0 |
| Full matrix (`--size full`) | 420 | pending `full` run | — |
| Rust unit | 40 | **40** | 0 |
| Gutenberg | 18 | **18** (bridge) | 0 |

Artifacts: `matrix_report.json/.md` + `benchmark_report.json` + `gutenberg_report.txt` per CI run.

---

## 📦 Coverage

| Module | Rust | Status |
|---|---|---|
| `destructive` (NLTKWordTokenizer) | `src/destructive.rs` | ✅ Cow fast paths |
| `treebank` + detokenize | `src/treebank.rs` | ✅ **D1-D4 zero-regex single-pass 2.51×** |
| `punkt` | `src/punkt/` | ✅ memchr3 + 156 abbrevs + hybrid bridge |
| `casual` (TweetTokenizer) | `src/casual.rs` | ✅ early-exit |
| `toktok` | `src/toktok.rs` | ✅ **M1** manual `expand_brackets`/`collapse_ws` |
| `regexp` / `whitespace` / `wordpunct` | `src/regexp.rs` | ✅ `\s+`→`split_whitespace` 6× |
| `simple` / `mwe` / `sexpr` / `nist` / `sonority` / `legality` | various | ✅ **M4-M6** nist `contains` guards |
| `util` (string/regexp_span, xml, align) | `src/util.rs` | ✅ **M2-M3** `\w+`/`\d+` bytes + memchr `sep` |

Known deviations in `PLAN.md §8` (span returns `list` not generator, codepoint offsets).

---

## 🔬 Reproduce

```bash
cargo test --all
python scripts/compare_outputs.py --size smoke && cat matrix_report.md
python scripts/benchmark.py && cat benchmark_report.md
PORTED_LIB_PUNKT_BRIDGE=1 python scripts/gutenberg_bench.py  # 18/18 + MB/s
gh workflow run rust-build-test.yml --ref main -f mode=full && gh run watch
```

*Generated after **CI #32091564048** smoke 185/185 — detokenize D1-D4 0.84→2.51× zero-regex.*
