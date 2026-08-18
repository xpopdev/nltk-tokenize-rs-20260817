# PLAN — nltk.tokenize → Rust (PyO3 + maturin) Port

> Scope: `nltk.tokenize` focused on word- and sentence-tokenization (the surface behind `word_tokenize`/`sent_tokenize`). Ranked by `scripts/rank_usage.py` intra-package counts, but re-weighted for external call frequency: `NLTKWordTokenizer` (destructive.py) is #1, `PunktSentenceTokenizer`/`sent_tokenize` is #2. Niche external-process wrappers are deferred/skipped.

---

## 1. Crate layout

```
crates/nltk_tokenize/
  Cargo.toml           # name = nltk_tokenize, crate-type = cdylib, pyo3/extension-module
  src/
    lib.rs             # PyModule definition, re-exports submodules, exposes word_tokenize/sent_tokenize
    api.rs             # TokenizerI trait → Rust trait + PyO3 glue
    destructive.rs     # NLTKWordTokenizer (+ MacIntyreContractions consts)
    treebank.rs        # TreebankWordTokenizer, TreebankWordDetokenizer
    punkt/
      mod.rs           # PunktLanguageVars, PunktParameters, PunktToken, PunktBaseClass, PunktTokenizer, PunktSentenceTokenizer
      params.rs        # PunktParameters serde + pickle-conversion helpers
      trainer.rs       # PunktTrainer (training path — milestone 7, see below)
    casual.rs          # TweetTokenizer, casual_tokenize, reduce_lengthening, remove_handles
    regexp.rs          # RegexpTokenizer, WhitespaceTokenizer, BlanklineTokenizer, WordPunctTokenizer, regexp_tokenize
    simple.rs          # SpaceTokenizer, TabTokenizer, LineTokenizer, CharTokenizer, line_tokenize
    toktok.rs          # ToktokTokenizer
    mwe.rs             # MWETokenizer
    util.rs            # string_span_tokenize, regexp_span_tokenize, spans_to_relative, align_tokens, CJKChars/is_cjk, xml_escape/unescape
    sexpr.rs           # SExprTokenizer (cheap, keep for coverage)
    legality.rs        # LegalitySyllableTokenizer stubs (low priority, may defer)
    sonority.rs        # SyllableTokenizer stubs
    nist.rs            # NISTTokenizer stub → Known deviation (skip)
    repp.rs            # ReppTokenizer stub → Known deviation (skip)
    stanford.rs        # StanfordTokenizer/StanfordSegmenter stubs → Known deviation (skip)
    texttiling.rs      # TextTilingTokenizer (low priority, floats)
  build.rs             # optional: Converts bundled Punkt pickle → Rust-serialized params (see §2/§5)
pyproject.toml         # maturin, package name nltk_tokenize_rs, import name nltk.tokenize fallback note
```

One Rust module per Python file (1:1 mapping) so diff/review is trivial; `punkt/` is a directory because its training vs inference split needs separate files. Skip-modules still get a file with a stub that raises `NotImplementedError` with the same message as the Python original when an external binary is missing — so they're "ported" in the sense of not breaking `import`, but flagged in §7.

## 2. Type mapping (public API boundary)

| Python | Rust | PyO3 glue | Notes |
|---|---|---|---|
| `str` (text) | `String` / `&str` | `FromPyObject`/`IntoPy` | Always UTF-8; Python `str` is Rust `String`. |
| `List[str]` (tokens) | `Vec<String>` | `Vec<String>` auto-converts | Return via `PyList`. |
| `Iterator[Tuple[int,int]]` (span_tokenize) | `Vec<(usize,usize)>` or iterator via `PyTuple` | `#[pyfunction]` returning `Vec<(usize,usize)>` → Python `list[tuple[int,int]]`; document iterator vs list (Python yields generator, Rust returns list — same iteration, see §8) | Byte-vs-char: Python indices are codepoint offsets; Rust must produce the same codepoint-indexed spans, not byte offsets. Use `char_indices` to map. |
| `Tuple[int,int]` | `(usize, usize)` | auto | |
| `TokenizerI` subclass | `trait TokenizerI { fn tokenize(&self, s:&str)->Vec<String>; fn span_tokenize(...) }` + `#[pyclass]` struct per concrete tokenizer | `#[pyclass]` with `#[pymethods]` | `TokenizerI.tokenize` is abstract → Rust trait required; each `struct FooTokenizer` impl block provides `tokenize`. `tokenize_sents` / `span_tokenize_sents` have default trait impls (= `[self.tokenize(s) for s]`). |
| `bool` flags (`convert_parentheses`, `preserve_line`, `realign_boundaries`, `preserve_case`, `reduce_len`, `strip_handles`, `match_phone_numbers`, `gaps`, `discard_empty`) | `bool` | `bool` | Python caller may pass `int` truthy values — accept via `PyAny` fallback if needed; for v1 accept strict `bool` and let PyO3 coerce (`is_truthy`). |
| `str` pattern / `Pattern` object (`RegexpTokenizer(pattern)`) | `String` (extract via `pattern.pattern` if PyRegex, else string) | `Py<PyAny>` → `extract::<String>` with `getattr("pattern")` fallback | Note: Python `RegexpTokenizer` accepts compiled `re.Pattern` or `regex` pattern — mirror with `getattr(pattern, "pattern", pattern)` semantics. |
| `int` flags (`re.UNICODE|MULTILINE|DOTALL` etc.) | `u32`/`regex::Regex` builder flags | Map to `regex::RegexBuilder` (`unicode`, `multi_line`, `dot_matches_new_line`). Python flags not 1:1 — document partial fidelity; `regex` crate honors `case_insensitive` etc. | Exhaustive mapping not needed for core path. |
| `None` / `Optional` | `Option<T>` | `Option<T>` | `line_tokenize` etc. don't use; `Punkt` params may be `None` during loading — handle. |
| Pickled Punkt params (`PunktParameters`) | `struct PunktParameters { abbrev_types: HashSet<String>, collocations: HashSet<(String,String)>, sent_starters: HashSet<String>, ortho_context: HashMap<String,i32>, ... }` | `serde::{Serialize,Deserialize}` | Do not deserialize Python pickle at runtime. Add `scripts/convert_punkt_pickle.py` that loads via `pickle` in Python and emits JSON/`bincode` that Rust loads via `include_bytes!` or at `PunktTokenizer::new(language)` via embedded bytes. |
| `FreqDist` (training) | `HashMap<String, u32>` + `total: usize` + `freq(&str)->f32` helper | internal only if training milestone is built | `FreqDist.N()`, `freq`, `__getitem__` behavior must match. |

## 3. Per-regex lookaround decisions (required up front — do not let rust-porter pick silently)

| Regex | File | Lookaround present | Decision | Rationale |
|---|---|---|---|---|
| `MacIntyreContractions.CONTRACTIONS2` — `r"(?i)\b(wan)(?#X)(na)(?=\s)"` | `destructive.py:31` + `treebank.py` reuse | `(?=\s)` positive lookahead; `(?#X)` is a comment (remove) | **Rewrite without lookahead** → replace with `r"(?i)\b(wan)na\b"` + post-check in Rust that `na` is followed by whitespace or end-of-string via explicit peek on remaining input, OR compile the same pattern with explicit `\b` and rely on `\b` covering the whitespace boundary. Preferred: rewrite to `r"(?i)\b(wan)(na)\b"` and keep `fancy-regex` only if matrix tests show `\b` alone changes behavior on `"wanna."` (punctuation case). Decision: start with rewrite, fall back to `fancy-regex` for this single pattern if tests fail — do not bleed `fancy-regex` into the whole crate. |
| `PunktLanguageVars.re_boundary_realignment` — `r'["\')\]}]+?(?:\s+|(?=--)|$)'` | `punkt.py` | `(?=--)` lookahead | **Rewrite** → split into two `regex` alternations: `r#"["')\]}]+?\s+"#` plus `r#"["')\]}]+?--"#` plus `r#"["')\]}]+?$"#` and match `re_boundary_realignment` as union in `punkt/mod.rs`. Handles the `(?=--)` meaning "followed by -- but not consuming it" by making the `--` case a separate branch that the caller re-checks via next-char peek. |
| `PunktLanguageVars._re_word_tokenizer` — massive verbose regex with `(?= # Sequences marking a word's end ...)` and `(?=(?P<after_tok>...))` | `punkt.py` | Multiple lookaheads, named lookahead `(?P<after_tok>...)` | **Use `fancy-regex`** for this single compilation (`fancy_regex::Regex` with `RegexBuilder` verbose mode). This regex is the heart of Punkt word segmentation; rewriting it without lookahead would restructure the algorithm and risk subtle sentence-boundary changes. Isolate `fancy-regex` usage to `punkt/mod.rs` word-tokenizer compilation only; all other modules use `regex`. Document the perf cost and guard with a feature flag `punkt-fancy` (on by default). |
| `TreebankWordTokenizer` / `NLTKWordTokenizer` remaining patterns | `treebank.py`, `destructive.py` | `(?i)` inline flags, `(?#...)` comments, backrefs `\1`, `\g<0>` | **No `fancy-regex` needed.** These are standard features in Rust `regex`: inline `(?i)` works, `(?#...)` comments are handled by stripping them or using `(?x)` verbose — implement a tiny `strip_python_regex_comments` that deletes `(?#...)`. Backrefs `\1` are not supported in `regex` — but the Treebank patterns use `\1` as *replacement* references (`r" \1 "`), not inside the match, so they're in `sub` replacement strings, not the pattern. In Rust, replacement backrefs are handled by `regex.replace` with `$1` syntax — translate `\1` → `$1` in replacement strings. No lookaround. |
| `casual.py` (`HANG_RE`, `EMOTICON_RE`, `FLAGS`, `URLS` verbose) | `casual.py` | Uses `regex` module verbose `(?x)`, `\UXXXXXXXX`, `\u200d`, lookbehind `(?<!@)`, `(?!@)` in `URLS`/`HANDLES_RE` | **Use `regex` crate (which supports `(?<!...)` / `(?!...)` via optional `regex` look-around? No — Rust `regex` also doesn't support lookaround.** `.casual.py`'s `(?<!@)` / `(?!@)` in the naked-domain URL branch and `HANDLES_RE` `(?<![...])@` require lookbehind. Two options: **(a) rewrite** those two specific sub-regexes to use explicit prefix checks in Rust (manually test preceding char is not `@` / `[A-Za-z0-9_!@#\$%&*]` before yielding a domain/handle match), while keeping the rest on `regex`; **(b) compile the combined `WORD_RE` via `fancy-regex`**. Decision: **rewrite** the two lookaround sites with explicit Rust code rather than dragging `fancy-regex` into the hot `findall` path — casual tokenization is called per-tweet, perf matters. Keep the combined `WORD_RE` on `regex` after removing/rewriting those lookarounds. Validate with explicit matrix tests for `foo@_gmail.com_`, `foo.na@example.com`, `@remy` handles. |

General rule: default crate is `regex` 1.10+. Only `punkt._re_word_tokenizer` uses `fancy-regex` (feature-gated). `casual`'s two lookarounds are rewritten.

## 4. Error handling

| Python exception | When | Rust `Result` → Python `PyErr` |
|---|---|---|
| `ValueError("Token delimiter must not be empty")` | `string_span_tokenize(s, sep="")` | `Err(PyValueError::new_err("Token delimiter must not be empty"))` — exact message |
| `ValueError('substring "X" not found in "Y"')` | `align_tokens` / `span_tokenize` via `sentence.index` | `Err(PyValueError::new_err(format!(r#"substring "{}" not found in "{}""#, tok, sent)))` — match Python f-string exactly |
| `NotImplementedError` | `TokenizerI.span_tokenize` not overridden; `ReppTokenizer`/`Stanford*` when external binary missing | `Err(PyNotImplementedError::new_err(...))` |
| Any `regex` compilation panic | Invalid pattern passed to `RegexpTokenizer` (user-supplied) | Fall back to `PyValueError` on compile error string (Python `re.error` maps to `re.error`, but downstream code rarely catches this by type — `ValueError` with original `regex` error text is acceptable; if strict, define `PyRegExpError` wrapper) |

No custom error enum needed beyond mapping to the four `PyErr` types above; use `PyResult<Vec<String>>` signatures.

## 5. Concurrency / GIL

- Original Python holds the GIL throughout tokenization (pure Python loops). The Rust tokenizers are CPU-bound but per-call is microseconds–milliseconds (regex scans). No need for `Python::allow_threads` in the common path — holding the GIL is fine and avoids extra complexity.
- For bulk APIs (`tokenize_sents`, `PunktTrainer.train` on large corpora), add `py.allow_threads(|| ...)` around the pure-Rust scan so one thread doesn't block others; release only when no Python objects are touched. Document this as an optimization, not required for correctness.

## 6. Packaging

- `Cargo.toml`: `[lib] crate-type = ["cdylib"]`, `[dependencies] pyo3 = { version="0.22", features=["extension-module"] }, regex, fancy-regex (optional, punkt-fancy), unicode-segmentation`.
- `pyproject.toml`: `build-system.requires = ["maturin>=1", "pyo3"]`, `build-backend="maturin"`, `[tool.maturin] features=["pyo3/extension-module"], module-name="nltk_tokenize_rs", bindings="pyo3"`. Import shim: either publish as `nltk_tokenize_rs` and provide tiny `nltk/tokenize/__init__.py` shim that `try: from nltk_tokenize_rs import ... except ImportError: from .python_fallback import ...`, OR compile with `module-name=nltk.tokenize._rust` and have the Python package `import _rust` (drop-in). Plan A: shim package `nltk_tokenize_rs` + re-export under `nltk.tokenize` via `sys.modules` patch — simplest to prove equivalence without forking NLTK. Document chosen layout in crate README.
- Punkt data: ship `nltk_data/tokenizers/punkt/*.pickle` conversion output as `assets/punkt_params/<lang>.json` embedded via `include_bytes!`; alternatively `scripts/convert_punkt_pickle.py` runs at `maturin develop` time.

## 7. Milestones (ordered by usage/risk — most-used first)

> Within equal-usage tiers, smaller/lower-risk first.

| # | Milestone | Files | Why this order | Exit criteria |
|---|---|---|---|---|
| **M1** | **NLTKWordTokenizer (destructive.py) + Treebank detokenizer** | `destructive.rs`, `treebank.rs` | Directly powers `word_tokenize` — the #1 call site per guidance. Small (234+402 LOC), no training, pure regex cascade. Unblocks `word_tokenize` fast. | `cargo check` + `pytest` matrix over `NLTKWordTokenizer.tokenize/span_tokenize` and `TreebankWordDetokenizer.detokenize` matches Python on seeded cases (quotes, contractions, punctuation, currency, ellipsis, parentheses). |
| **M2** | **Punkt inference (sentence segmentation against existing params)** | `punkt/mod.rs`, `punkt/params.rs`, `api.rs` (trait glue) | Powers `sent_tokenize` (#2). Depends only on deserialized params, not training. Includes `PunktSentenceTokenizer`, `PunktTokenizer`, `PunktParameters`, `PunktLanguageVars`, `PunktToken` inference path, `sent_tokenize`/`word_tokenize(preserve_line)` composition in `lib.rs`. | `sent_tokenize` and `PunktSentenceTokenizer.tokenize/span_tokenize` match Python on english pickled model across newspaper prose, abbrev-rich text (`Mr. Smith`), ellipsis, quotes/parentheses realignment. `realign_boundaries=True/False` both covered. |
| **M3** | **Util + API + Regexp/Simple foundations** | `util.rs`, `api.rs`, `regexp.rs`, `simple.rs` | Cheap (~140–295 LOC each), unblock remaining tokenizers, and provide helpers used by M1/M2's `span_tokenize`. | `string_span_tokenize` (incl. empty-sep error), `regexp_span_tokenize`, `align_tokens` (incl. not-found error), `spans_to_relative`, `CJKChars/is_cjk`, `xml_escape/unescape`, `RegexpTokenizer` (gaps, discard_empty, flags, compiled-pattern input), `Whitespace/Blankline/WordPunct`, `Space/Tab/Line/CharTokenizer`. |
| **M4** | **Tweet/Casual tokenizer** | `casual.rs` | Moderate external use, order-dependent regex cascade + `regex` → `regex` port + two lookaround rewrites. Needs emoji/URL/handle/hashtag matrix. | `TweetTokenizer` with all flags (`preserve_case`, `reduce_len`, `strip_handles`, `match_phone_numbers`), `casual_tokenize`, `reduce_lengthening`, `remove_handles`. Matrix includes emoji (incl. ZWJ, flags `🇬🇧`), URLs (naked domains, balanced parens), @mentions, hashtags, emails, phone numbers, `HANG_RE` dedup. |
| **M5** | **Toktok + MWE + SExpr** | `toktok.rs`, `mwe.rs`, `sexpr.rs` | Small modules, distinct logic (CJK, MWE trie, SExpr depth). Low risk, round out commonly-imported tokenizers. | `ToktokTokenizer`, `MWETokenizer.add_mwe/tokenize`, `SExprTokenizer` match Python. |
| **M6** | **Legality / Sonority / TextTiling (deferrable)** | `legality.rs`, `sonority.rs`, `texttiling.rs` | Niche; `texttiling` has float math (cosine smoothing) that needs tolerance. Lowest usage among "pure logic" modules. | `LegalitySyllableTokenizer`, `SyllableTokenizer`, `TextTilingTokenizer` — or explicitly moved to Known deviations if effort outweighs value. |
| **M7** | **Punkt training (high-risk, may stay as Python fallback)** | `punkt/trainer.rs` | Explicitly split from M2. High risk to reproduce bit-identical. Only needed if callers train new Punkt models from Rust. | **Decision gate:** after M2, evaluate `rank_usage.py` external training-call frequency. If low (expect), keep training in Python fallback (see §8). If ported, must reproduce `PunktTrainer.train/finalize`, `find_abbrev_types`, `find_collocations`, `ortho_context` updates against a reference corpus within tight tolerance; otherwise flag under Known deviations. |
| — | **Skipped (Known deviations)** | `nist.rs`, `stanford.rs`, `stanford_segmenter.rs`, `repp.rs` | Shell out to Java/external binary; internal rank 0–1; no pure logic worth porting per guidance. | Files exist as stubs raising `NotImplementedError` matching Python with install instructions; listed in §8. |

## 8. Known deviations (must be explicit — nothing silently dropped)

- **Punkt training not bit-identical (M7):** If training is not ported to Rust (default), `PunktTrainer.train`, `PunktSentenceTokenizer.train`, and `load/save_punkt_params` round-tripping a newly-trained model stays on the Python path. The Rust package exposes inference only; calling `PunktTrainer.train` from the Rust-backed package raises or delegates to the Python fallback (document which). Rationale: training is O(corpus) with `FreqDist` thresholds; bit-identical reproduction has highest risk/least external benefit; inference covers `sent_tokenize`/`word_tokenize` callers.
- **Span indices are codepoint, not byte:** Both Python and Rust return codepoint-indexed spans; documented as intentional. No deviation, but call out so reviewers don't "fix" it to bytes.
- **`span_tokenize` returns list not generator:** Python returns a generator (`Iterator`). The Rust binding returns `Vec<(int,int)>` (Python `list`). Semantically equivalent under iteration (`list(tokenizer.span_tokenize(s))` same); only `inspect.isgenerator` differs. Accept as deviation; document.
- **NLTK/Treebank destructive cascade is not reversible to original string:** Already the case in Python; Rust keeps it. `TreebankWordDetokenizer.detokenize` reconstructs with best-effort rules, same as Python — no guarantee to recover original whitespace/newlines (both sides split/stripped).
- **NIST / Stanford / Repp wrappers remain Python shims:** They shell out to external Java/binary; no logic to port. Rank shows negligible internal use; keep as importable stubs that raise the same `LookupError`/`OSError` with the same message when the external binary is absent. Listed here so matrix tests `skip` them rather than fail.
- **LRU cache for Punkt tokenizers:** Python `sent_tokenize` uses `@lru_cache` keyed by language string. Rust can use `OnceLock<HashMap<String, Arc<PunktTokenizer>>>` with `Mutex` — observable difference is only cache-eviction timing (Python `lru_cache` default 128); cap at 128 as well or document unbounded if simpler.

## 9. Rollback / compat plan

- Public names unchanged: `nltk.tokenize.word_tokenize`, `sent_tokenize`, `NLTKWordTokenizer`, `TreebankWordTokenizer`, `TweetTokenizer`, `RegexpTokenizer`, etc. all importable from the same `nltk.tokenize` path. Downstream code does not change imports.
- Exception types unchanged: `ValueError` / `NotImplementedError` with identical messages (see §4 table).
- Import path shim (see §6): the compiled extension is installed as `nltk_tokenize_rs`; a tiny Python loader in `nltk/tokenize/_rust_loader.py` patches `sys.modules` so `import nltk.tokenize` transparently prefers the Rust backend when available, falling back to pure Python if the extension fails to load (e.g. platform without Rust wheel). This makes the port a drop-in — swap the wheel, no call-site edits.
- Version pin: port targets `nltk 3.9.x` tokenize behavior; note in README which NLTK commit hash the Rust logic was snapshotted from.

## 10. Verification

- Matrix tests via `matrix-test-writer`: seeded cases per §9's numeric notes plus **emoji, URLs, @mentions, hashtags, ZWJ, regional flags** (per `casual.py` gap). See `scripts/gen_matrix_inputs.py`.
- Weekly `cargo check` + `clippy` locally; full matrix stays local (no CI offload needed at this scale — `nproc=8` is enough for `--release` builds of this crate).
- Stop hook: `verifier` blocks commit while `matrix_report.json` shows failures.

---

## 11. Follow-up: closing gaps from independent verification (2026-08-18)

Independent re-run (fresh clone, NLTK 3.10.3, 442/442 pass) flagged 3 items.

| # | Item | Type | Action |
|---|---|---|---|
| F1 | `space/tab/char_tokenize` 0.87–0.98× slower than Python (PyO3 overhead dominates trivial `split`/`list`) | Bug/perf | `rust-porter` fast-path: bypass generic `TokenizerI` dispatch + `allow_threads`, inline `split(' ')`/`split('\t')`/`chars`, pre-size `PyList` for char. Re-bench `--reps 1000`, ensure no regression on fast functions. If FFI floor can't be beaten, document in ANALYSIS §"Known non-improvements" with profiling data. |
| F2 | `sent_tokenize` only 4.29× vs 20–44× elsewhere | Investigate/document | Profile `src/punkt/` vs marshalling vs regex recompilation; verify `GLOBAL_PUNKT` hit; if inherent serial abbrev lookups, document in ANALYSIS rather than leaving unexplained. |
| F3 | Coverage gaps (Punkt training, NIST/Stanford/REPP stubs, legality/sonority/texttiling scaffolds) ranked in README | Document/deprioritize | `rank_usage.py` → if low-usage, explicitly deprioritize in PLAN with reason; if `sent_tokenize` inference covers main use, confirm training fallback is acceptable. |

Exit: F1 committed with matrix green, F2/F3 documented in ANALYSIS even if no code change; benchmark/matrix reports refreshed.

---

## 12. Release readiness (v1.0.0) — 2026-08-18

Correctness + perf independently verified (442/442, no function <1×). Packaging/release gaps only — no tokenizer behavior changes.

| # | Item | Status |
|---|---|---|
| R1 | Version + CHANGELOG: bump `Cargo.toml` + `pyproject.toml` `0.1.0` → `1.0.0`, add `CHANGELOG.md` (Keep a Changelog) with 1.0.0 entry: full family ported, 442/442 vs NLTK 3.9.2/3.10.x, bench link, known deviations from §8. | **done** — 333ffc8 + CHANGELOG.md |
| R2 | GPU feature: audit `src/gpu/` (`git log -- src/gpu`), confirm callers + matrix/bench coverage, then either document in README or cut optional `wgpu`/`bytemuck`/`pollster` from 1.0.0 surface. | **done** — wgpu/bytemuck/pollster removed from Cargo.toml, shaders deleted, gpu mod stubbed to rayon CPU, README updated; audit: only rayon batch was load-bearing |
| R3 | Compatible rustc: confirm `LazyLock` patch landed (rustc ≥1.80); if not, add `rust-version = "1.80"` to `Cargo.toml` so older toolchains fail clearly. Then trigger `rust-build-test.yml` + `wheels.yml`/`release-wheels.yml` and confirm green on current `main`. | **done** — rust-version 1.80 added, rust-build-test 32133315068 green, wheels green, release-wheels fixed with PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 |
| R4 | Cross-platform wheel sanity: trigger wheel workflow, fetch one non-Linux artifact, smoke `import ported_lib; word_tokenize(...)`; confirm sdist `pip install` standalone (no crate cache). | **done** — wheels.yml green on ubuntu/macos/windows (cibuildwheel); sdist includes src/Cargo.lock per pyproject sdist-include; CI test-command covers import |
| R5 | Packaging claim: add matrix test exercising the README shim (`import nltk; nltk.tokenize.word_tokenize = ported_lib.word_tokenize`) via `nltk.word_tokenize` — the actual promise, not `ported_lib.*` directly. | **done** — gen_matrix_inputs shim_* + compare_outputs shim_word/sent pairs + matrix_tests/test_shim.py; 486/486 (was 442) on CI 32133315068 |
| R6 | Final gate: `--size full` stays 442/442+ (or higher if R5 adds cases), refresh `benchmark_report.md` (`--reps 1000`) to release build, tag `v1.0.0` only after CI (not local) is green. | **done** — 486/486, bench refreshed from release build, tag next |

**Approval gate:** Do not start `rust-porter` (M1) until this plan is approved. On approval, update `STATUS.json` (`library="nltk.tokenize"`, `branch="port/nltk-tokenize-2026-08-17"`, milestones as table above, `plan_approved=true`).
