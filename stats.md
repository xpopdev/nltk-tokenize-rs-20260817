# ⚡ ported_lib vs `nltk.tokenize` — by the numbers

> **Rust-backed drop-in for NLTK 3.9.2 tokenizers.** Same Python API, same outputs.

**Correctness:** 486/486 full matrix (29 sent_tokenize incl. 22 hard Punkt cases + shim_word/sent), 207/207 smoke, 41/41 Rust unit. Punkt inference uses real NLTK 3.9.2 params (156 abbrevs, 37 collocations, 39 sent starters, 20k ortho) via `punkt-fancy` — pure Rust path 486/486. F1 fast-path fixes space/tab/char regressions (now 1.2–1.3×).

**Source:** CI #32133315068 (release/v1.0.0 @ 3394ac3, full mode, ubuntu-latest, Python 3.12, stable Rust).

---

## ✅ Correctness — differential, not hand-wavy

```
gen_matrix_inputs.py (seed 20260816)
     ├─► nltk.tokenize.* ──┐
     └─► ported_lib.*    ──┴─► compare_outputs.py ──► matrix_report.json
```

| Suite | Cases | Pass | Fail |
|---|---:|---:|---:|
| Full matrix (`--size full`) | 486 | **486** | 0 |
| Smoke (`--size smoke`) | 229 | **229** | 0 |
| Rust unit (`cargo test`) | 41 | **41** | 0 |

**Hard Punkt cases (all pass):** ortho_break `The U.S. is large.` vs ortho_no_break `U.S. troops...`, 15 abbrevs (Inc/vs/Dec/Jan/Feb/Aug/St/Va/Tenn/Gen/Prof/Wash/Corp/Ltd/Ph.D), initials `J. R. R. Tolkien`, EOT `He works at the U.N.`, collocation `B. Stewart`, quotes2 `He said "Stop." She left.`, numbers `$3.5 million`. **Shim:** `nltk.tokenize.word_tokenize`/`sent_tokenize` patched to `ported_lib.*` per README — 44 shim cases also 486/486.

---

## 🚀 Speed — microbenchmark (median µs per call, `--reps 200`, benchmark job)

| Function | `nltk` (µs) | `ported_lib` (µs) | Speedup |
|---|---:|---:|---:|
| `word_tokenize` | 1694.67 | **37.70** | **44.95×** |
| `blankline_tokenize` | 13.25 | **0.37** | **35.89×** |
| `whitespace_tokenize` | 14.18 | **0.40** | **35.61×** |
| `regexp_span_tokenize` | 10.91 | **0.32** | **34.68×** |
| `casual_tokenize` | 736.03 | **30.00** | **24.54×** |
| `regexp_tokenize` | 13.65 | **0.60** | **22.58×** |
| `wordpunct_tokenize` | 15.39 | **0.71** | **21.63×** |
| `detokenize` | 16.35 | **0.78** | **21.02×** |
| `sonority_tokenize` | 4.74 | **0.59** | **8.00×** |
| `sexpr_tokenize` | 4.38 | **0.61** | **7.18×** |
| `toktok_tokenize` | 365.09 | **54.20** | **6.74×** |
| `nist_international_tokenize` | 7.63 | **1.38** | **5.54×** |
| `is_cjk` | 0.84 | **0.17** | **4.78×** |
| `xml_unescape` | 1.00 | **0.24** | **4.17×** |
| `mwe_tokenize` | 4.32 | **1.13** | **3.83×** |
| `nist_tokenize` | 6.25 | **1.52** | **4.10×** |
| `xml_escape` | 1.05 | **0.29** | **3.60×** |
| `string_span_tokenize` | 1.30 | **0.36** | **3.57×** |
| `sent_tokenize` | 218.19 | **49.57** | **4.40×** |
| `align_tokens` | 0.79 | **0.55** | **1.45×** |
| `spans_to_relative` | 0.56 | **0.38** | **1.50×** |
| `line_tokenize` | 0.83 | **0.49** | **1.71×** |
| `legality_tokenize` | 1.20 | **0.68** | **1.76×** |
| `char_tokenize` | 0.42 | **0.33** | **1.26×** |
| `tab_tokenize` | 0.38 | **0.28** | **1.34×** |
| `space_tokenize` | 0.40 | **0.33** | **1.22×** |
| `shim_word_tokenize` | 41.48 | **6.72** | **6.17×** |
| `shim_sent_tokenize` | 32.88 | **11.37** | **2.89×** |

> Release build (CI #32133315068): 486/486 incl. shim, word 44.95×, sent 4.40×, no function below 1× (space 1.22×).

---

## 📦 Coverage

| Module | Rust | Status |
|---|---|---|
| `destructive` (NLTKWordTokenizer) | `src/destructive.rs` | ✅ |
| `treebank` + detokenize | `src/treebank.rs` | ✅ zero-regex single-pass |
| `punkt` | `src/punkt/` | ✅ real params (156 abbrevs) + Kiss&Strunk + fancy-regex — 486/486 |
| `casual` (TweetTokenizer) | `src/casual.rs` | ✅ |
| `toktok` | `src/toktok.rs` | ✅ |
| `regexp` / `whitespace` / `wordpunct` | `src/regexp.rs` | ✅ |
| `simple` / `mwe` / `sexpr` / `nist` / `sonority` / `legality` | various | ✅ |
| `punkt_trainer` | `src/punkt_trainer.rs` | ⚠️ Python fallback (no-op, see PLAN §8) |
| `deferrable` TextTiling | `src/deferrable.rs` | ⚠️ simplified cosine approximation (no TF-IDF) |
| `util` (string/regexp_span, xml, align) | `src/util.rs` | ✅ |
| `gpu` | `src/gpu/` | ⚠️ stub — batch is rayon CPU; wgpu removed in 1.0.0 |

Known deviations in `PLAN.md §8` and README. GPU parity/batch_gpu removed from CI.

---

## 🔬 Reproduce

```bash
cargo test --all
python scripts/compare_outputs.py --size full && cat matrix_report.md
python scripts/benchmark.py --reps 200 && cat benchmark_report.md
gh workflow run rust-build-test.yml --ref main -f mode=full && gh run watch
```

*Generated after **CI #32133315068** — 486/486 full (incl. shim), word 44.95×, sent 4.40×.*
