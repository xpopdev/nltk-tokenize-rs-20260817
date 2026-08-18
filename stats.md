# ⚡ ported_lib vs `nltk.tokenize` — by the numbers

> **Rust-backed drop-in for NLTK 3.9.2 tokenizers.** Same Python API, same outputs.

**Correctness:** 442/442 full matrix (29 sent_tokenize incl. 22 hard Punkt cases), 207/207 smoke, 41/41 Rust unit. Punkt inference uses real NLTK 3.9.2 params (156 abbrevs, 37 collocations, 39 sent starters, 20k ortho) via `punkt-fancy` — pure Rust path now 442/442 without bridge.

**Source:** CI #32099654001 (main @ bcb4991, full mode, ubuntu-latest, Python 3.12, stable Rust).

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
| `word_tokenize` | 1680.50 | **37.38** | **44.96×** |
| `blankline_tokenize` | 13.24 | **0.39** | **34.16×** |
| `whitespace_tokenize` | 14.15 | **0.40** | **35.11×** |
| `regexp_span_tokenize` | 10.78 | **0.32** | **34.22×** |
| `casual_tokenize` | 741.30 | **30.23** | **24.53×** |
| `regexp_tokenize` | 13.68 | **0.62** | **22.04×** |
| `wordpunct_tokenize` | 15.36 | **0.74** | **20.83×** |
| `detokenize` | 16.39 | **0.77** | **21.36×** |
| `sonority_tokenize` | 4.69 | **0.60** | **7.83×** |
| `sexpr_tokenize` | 4.40 | **0.59** | **7.47×** |
| `toktok_tokenize` | 362.03 | **53.46** | **6.77×** |
| `nist_international_tokenize` | 7.60 | **1.36** | **5.60×** |
| `is_cjk` | 0.85 | **0.19** | **4.57×** |
| `xml_unescape` | 1.01 | **0.24** | **4.28×** |
| `mwe_tokenize` | 4.29 | **1.05** | **4.09×** |
| `nist_tokenize` | 6.20 | **1.52** | **4.09×** |
| `xml_escape` | 1.05 | **0.28** | **3.79×** |
| `string_span_tokenize` | 1.31 | **0.36** | **3.66×** |
| `sent_tokenize` | 215.21 | **50.22** | **4.29×** |
| `align_tokens` | 0.79 | **0.51** | **1.56×** |
| `spans_to_relative` | 0.56 | **0.36** | **1.55×** |
| `line_tokenize` | 0.83 | **0.47** | **1.76×** |
| `legality_tokenize` | 1.21 | **0.67** | **1.79×** |
| `char_tokenize` | 0.42 | 0.40 | 1.05× |
| `tab_tokenize` | 0.39 | 0.36 | 1.07× |
| `space_tokenize` | 0.40 | 0.42 | 0.94× |

> Real Punkt is slightly **faster** than the prior stub (44.96× vs 40.79× word, 4.29× vs 3.74× sent, 34× vs 38× blankline) despite extra collocation/ortho lookups and fancy-regex — the realign fix and reduced PyO3 overhead dominate. Not hidden as a regression; reported as measured on CI.

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

*Generated after **CI #32099654001** — 442/442 full, 22 hard Punkt cases, real params + realign fix. Full matrix_report.json rows for sent_tokenize all `pass` (see CI artifact).*
