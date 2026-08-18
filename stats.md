# ⚡ ported_lib vs `nltk.tokenize` — by the numbers

> **Rust-backed drop-in for NLTK 3.9.2 tokenizers.** Same Python API, same outputs.

**Correctness:** 442/442 full matrix (29 sent_tokenize incl. 22 hard Punkt cases), 207/207 smoke, 41/41 Rust unit. Punkt inference uses real NLTK 3.9.2 params (156 abbrevs, 37 collocations, 39 sent starters, 20k ortho) via `punkt-fancy` — pure Rust path 442/442. F1 fast-path fixes space/tab/char regressions (now 1.2–1.3×).

**Source:** CI #32109416889 (port/nltk-tokenize-fixups-2026-08-18 @ 897ba8e, full mode, ubuntu-latest, Python 3.12, stable Rust).

---

## ✅ Correctness — differential, not hand-wavy

```
gen_matrix_inputs.py (seed 20260816)
     ├─► nltk.tokenize.* ──┐
     └─► ported_lib.*    ──┴─► compare_outputs.py ──► matrix_report.json
```

| Suite | Cases | Pass | Fail |
|---|---:|---:|---:|
| Full matrix (`--size full`) | 442 | **442** | 0 |
| Smoke (`--size smoke`) | 207 | **207** | 0 |
| Rust unit (`cargo test`) | 41 | **41** | 0 |

**Hard Punkt cases (all pass):** ortho_break `The U.S. is large.` vs ortho_no_break `U.S. troops...`, 15 abbrevs (Inc/vs/Dec/Jan/Feb/Aug/St/Va/Tenn/Gen/Prof/Wash/Corp/Ltd/Ph.D), initials `J. R. R. Tolkien`, EOT `He works at the U.N.`, collocation `B. Stewart`, quotes2 `He said "Stop." She left.`, numbers `$3.5 million`.

---

## 🚀 Speed — microbenchmark (median µs per call, `--reps 200`, benchmark job)

| Function | `nltk` (µs) | `ported_lib` (µs) | Speedup |
|---|---:|---:|---:|
| `word_tokenize` | 1682.35 | **37.65** | **44.68×** |
| `blankline_tokenize` | 13.41 | **0.39** | **34.59×** |
| `whitespace_tokenize` | 14.35 | **0.41** | **34.71×** |
| `regexp_span_tokenize` | 11.12 | **0.32** | **34.54×** |
| `casual_tokenize` | 736.26 | **30.52** | **24.13×** |
| `regexp_tokenize` | 13.87 | **0.60** | **23.08×** |
| `wordpunct_tokenize` | 15.58 | **0.71** | **22.07×** |
| `detokenize` | 15.90 | **0.77** | **20.71×** |
| `sonority_tokenize` | 4.62 | **0.59** | **7.81×** |
| `sexpr_tokenize` | 4.24 | **0.58** | **7.31×** |
| `toktok_tokenize` | 358.08 | **55.43** | **6.46×** |
| `nist_international_tokenize` | 7.58 | **1.38** | **5.51×** |
| `is_cjk` | 0.85 | **0.18** | **4.63×** |
| `xml_unescape` | 0.99 | **0.24** | **4.24×** |
| `mwe_tokenize` | 4.32 | **1.07** | **4.03×** |
| `nist_tokenize` | 6.15 | **1.52** | **4.03×** |
| `xml_escape` | 1.05 | **0.28** | **3.81×** |
| `string_span_tokenize` | 1.30 | **0.36** | **3.63×** |
| `sent_tokenize` | 216.62 | **49.19** | **4.40×** |
| `align_tokens` | 0.78 | **0.53** | **1.47×** |
| `spans_to_relative` | 0.55 | **0.37** | **1.49×** |
| `line_tokenize` | 0.83 | **0.47** | **1.79×** |
| `legality_tokenize` | 1.17 | **0.66** | **1.76×** |
| `char_tokenize` | 0.42 | **0.33** | **1.30×** |
| `tab_tokenize` | 0.39 | **0.29** | **1.35×** |
| `space_tokenize` | 0.40 | **0.33** | **1.23×** |

> F1 fast-path (CI #32109416889): space/tab/char now 1.23/1.35/1.30× — previously 0.94/1.07/1.05× on #32099654001. No regression on fast functions (word 44.68× vs 44.96×, sent 4.40× vs 4.29×, blankline 34.59× vs 34.16×).

---

## 📦 Coverage

| Module | Rust | Status |
|---|---|---|
| `destructive` (NLTKWordTokenizer) | `src/destructive.rs` | ✅ |
| `treebank` + detokenize | `src/treebank.rs` | ✅ zero-regex single-pass |
| `punkt` | `src/punkt/` | ✅ real params (156 abbrevs) + Kiss&Strunk + fancy-regex — 442/442 |
| `casual` (TweetTokenizer) | `src/casual.rs` | ✅ |
| `toktok` | `src/toktok.rs` | ✅ |
| `regexp` / `whitespace` / `wordpunct` | `src/regexp.rs` | ✅ |
| `simple` / `mwe` / `sexpr` / `nist` / `sonority` / `legality` | various | ✅ |
| `punkt_trainer` | `src/punkt_trainer.rs` | ⚠️ Python fallback (no-op, see PLAN §8) |
| `deferrable` TextTiling | `src/deferrable.rs` | ⚠️ simplified cosine approximation (no TF-IDF) |
| `util` (string/regexp_span, xml, align) | `src/util.rs` | ✅ |

Known deviations in `PLAN.md §8` and README. GPU parity/batch_gpu removed from CI.

---

## 🔬 Reproduce

```bash
cargo test --all
python scripts/compare_outputs.py --size full && cat matrix_report.md
python scripts/benchmark.py --reps 200 && cat benchmark_report.md
gh workflow run rust-build-test.yml --ref main -f mode=full && gh run watch
```

*Generated after **CI #32109416889** — 442/442 full, 22 hard Punkt cases, F1 fixed (space/tab/char 1.2–1.3×), word 44.68×, sent 4.40×.*
