# ANALYSIS — nltk.tokenize (NLTK 3.9.2)

## Summary

`nltk.tokenize` is the tokenization subsystem of NLTK. It splits raw text into sentences and sentences into word/subword tokens. It ships ~19 modules (320–1826 LOC each, 5796 total) covering a spectrum from trivial whitespace splits to a trained statistical sentence-boundary detector (Punkt), a destructive regex-cascade word tokenizer (Treebank/NLTK), a Twitter-aware tokenizer (Tweet/Casual), and several niche segmenters. No I/O beyond reading pre-trained Punkt pickle data; otherwise pure string→list[string] / string→spans transforms. Public call sites are dominated by two convenience wrappers `word_tokenize` / `sent_tokenize` (in `__init__.py`) that compose `PunktSentenceTokenizer` + `NLTKWordTokenizer`.

## Public API — file by file

### `nltk/tokenize/__init__.py` (145 lines)
Re-export facade + two top-level functions that are the de-facto public API.

| Symbol | Kind | Signature | Description |
|---|---|---|---|
| `sent_tokenize` | function | `(text: str, language="english") -> List[str]` | LRU-cached `PunktTokenizer(language).tokenize(text)` |
| `word_tokenize` | function | `(text: str, language="english", preserve_line=False) -> List[str]` | `sent_tokenize` then `NLTKWordTokenizer` per sentence |
| *(re-exports)* | — | — | `TweetTokenizer`, `NLTKWordTokenizer`, `MWETokenizer`, `PunktTokenizer`, `RegexpTokenizer`, `ToktokTokenizer`, `TreebankWordTokenizer`, etc. |

Side effects: `_get_punkt_tokenizer` is `@lru_cache` — global mutable cache keyed by language string.
Tests: `nltk/test/test_tokenize.py` exercises both wrappers with english corpora.

### `nltk/tokenize/api.py` (83 lines)
Abstract base.

| Symbol | Kind | Description |
|---|---|---|
| `TokenizerI` | abstract class | Defines `tokenize(s) -> List[str]` (abstract), `span_tokenize(s) -> Iterator[(int,int)]` (NotImplemented by default), `tokenize_sents`/`span_tokenize_sents` batch helpers |
| `StringTokenizer` | abstract class | `tokenize` via `s.split(self._string)`, `span_tokenize` via `string_span_tokenize` |

Typing: fully annotated. No exception contract beyond `NotImplementedError` if subclass omits `span_tokenize`.

### `nltk/tokenize/destructive.py` (234 lines)

| Symbol | Kind | Description |
|---|---|---|
| `MacIntyreContractions` | class | Holds `CONTRACTIONS2/3/4` regex strings (contraction splitters) |
| `NLTKWordTokenizer` | class (`TokenizerI`) | Regex-cascade word tokenizer; destructive (cannot round-trip). Implements `tokenize(text, convert_parentheses=False, return_str=False)` and `span_tokenize` via `align_tokens` |

Key behavior: ~15 ordered `re.compile`+`sub` passes (STARTING_QUOTES → PUNCTUATION → PARENS → DOUBLE_DASHES → ENDING_QUOTES → CONTRACTIONS). `return_str` is deprecated. `span_tokenize` re-aligns tokens via `align_tokens` and has special quote-restoration logic.
Deps: `re`, `warnings`, `nltk.tokenize.util.align_tokens`.

### `nltk/tokenize/treebank.py` (402 lines)

| Symbol | Kind | Description |
|---|---|---|
| `TreebankWordTokenizer` | class (`TokenizerI`) | Older Penn-Treebank regex-cascade; same cascade shape as `NLTKWordTokenizer` but fewer quote/unicode rules |
| `TreebankWordDetokenizer` | class | Reverse cascade — joins tokens back with inverse regexes. `tokenize(tokens, convert_parentheses=False) -> str`, alias `detokenize` |

Deps: `re`, `warnings`.

### `nltk/tokenize/punkt.py` (1826 lines) — largest, highest risk

| Symbol | Kind | Description |
|---|---|---|
| `PunktLanguageVars` | class | Language-dependent regexes (`_re_period_context`, `_re_word_tokenizer`, `re_boundary_realignment`, etc.) |
| `PunktParameters` | class | Learned params: `abbrev_types`, `collocations`, `sent_starters`, `ortho_context` (dicts/sets/counters) |
| `PunktBaseClass` | class | Common init for trainer/tokenizer; holds `lang_vars` + `params` |
| `PunktToken` | class | Wrapper around a word+context window, carries `period_final`, `sentbreak` flags |
| `PunktTrainer` | class | Unsupervised learner: `train(text)`, `finalize_training()`, helpers `find_abbrev_types`, `find_collocations`, etc. |
| `PunktTokenizer` / `PunktSentenceTokenizer` | class (`TokenizerI`) | Inference: `tokenize(text, realign_boundaries=True)`, `sentences_from_text`, `text_contains_sentbreak`, `span_tokenize`; uses learned `PunktParameters` |
| `word_tokenize` | function | Thin wrapper (legacy) |
| `load_punkt_params` / `save_punkt_params` / `dump` / `load_lang` | functions | Pickle I/O for params |

Deps: `re`, `math`, `collections`, `nltk.probability.FreqDist`, `nltk.data.load` for pickle models.
Numeric: `FreqDist` counts, log-likelihood thresholds; no float-precision edge cases beyond Python float.
State: trained `PunktParameters` are pickled blobs shipped in `nltk_data/tokenizers/punkt/*.pickle`.

### `nltk/tokenize/casual.py` (458 lines)

| Symbol | Kind | Description |
|---|---|---|
| `TweetTokenizer` | class (`TokenizerI`) | Twitter tokenizer; lazy-compiled `WORD_RE` / `PHONE_WORD_RE` via `regex` crate (not `re`) |
| `casual_tokenize` | function | Convenience wrapper for `TweetTokenizer` |
| `reduce_lengthening` | function | `(.)\1{2,}` → `\1\1\1` |
| `remove_handles` | function | `HANDLES_RE.sub(" ", text)` |
| `REGEXPS` / `REGEXPS_PHONE` / `URLS` / `EMOTICONS` / `FLAGS` / `PHONE_REGEX` / `HANG_RE` / `EMOTICON_RE` / `ENT_RE` | constants | Ordered regex components (URL, emoticon, HTML tag, arrow, @handle, hashtag, email, ZWJ emoji, flags, ellipsis, `\S`) |

Deps: `regex` (third-party, supports `\U`, verbose, unicode properties beyond stdlib `re`), `html`.
Order-dependent: `PHONE_REGEX` must be first; `EMOTICONS` before `tags`; last element is catch-all `\S`.

### `nltk/tokenize/regexp.py` (220 lines)

| Symbol | Kind | Description |
|---|---|---|
| `RegexpTokenizer` | class (`TokenizerI`) | Generic tokenizer over a single regex; `gaps`+`discard_empty`+`flags` mode in ctor |
| `WhitespaceTokenizer` | subclass | `r"\s+"` gaps |
| `BlanklineTokenizer` | subclass | `r"\s*\n\s*\n\s*"` gaps |
| `WordPunctTokenizer` | subclass | `r"\w+|[^\w\s]+"` tokens |
| `regexp_tokenize` / `blankline_tokenize` / `wordpunct_tokenize` | functions | Functional wrappers |

Python trick: `__init__(pattern, ..., flags=...)` accepts both string and compiled pattern via `getattr(pattern, "pattern", pattern)`.

### `nltk/tokenize/simple.py` (139 lines)

| Symbol | Kind | Description |
|---|---|---|
| `SpaceTokenizer` | class (`StringTokenizer`) | `_string = " "` |
| `TabTokenizer` | class (`StringTokenizer`) | `_string = "\t"` |
| `LineTokenizer` | class | `blanklines_are_separators` flag, delegates to `string_span_tokenize` |
| `CharTokenizer` | class (`StringTokenizer`) | `_string = ""` (edge: empty sep) |
| `line_tokenize` | function | Wrapper |

### `nltk/tokenize/toktok.py` (180 lines)

| Symbol | Kind | Description |
|---|---|---|
| `ToktokTokenizer` | class (`TokenizerI`) | Rule-based tokenizer with CJK-aware `is_cjk`; holds ~10 compiled regexes + a detokenizer pass |

### `nltk/tokenize/mwe.py` (124 lines)

| Symbol | Kind | Description |
|---|---|---|
| `MWETokenizer` | class | Merges multi-word expressions via a trie; `add_mwe`, `tokenize` mutates via join semantics |
| `add_mwe` | method | alias |

### `nltk/tokenize/util.py` (295 lines)

| Symbol | Kind | Description |
|---|---|---|
| `string_span_tokenize` | function | Yield `(start,end)` by `str.index(sep, ...)`; raises `ValueError` if `len(sep)==0` |
| `regexp_span_tokenize` | function | Yield gaps between `finditer(regexp)` matches |
| `spans_to_relative` | function | Convert absolute spans to `(offset, length)` |
| `align_tokens` | function | Find each token via `sentence.index(token, point)`; raises `ValueError` if token not found |
| `CJKChars` / `is_cjk` | class+function | CJK codepoint range checks (int comparisons) |
| `xml_escape` / `xml_unescape` | functions | `xml.sax.saxutils` with extra entities (`|`, `[`, `]`, `&apos;`, `&quot;`) |

### Other modules (lower priority — niche/externally-coupled)

| Symbol | File | Notes |
|---|---|---|
| `SExprTokenizer` / `sexpr_tokenize` | `sexpr.py` (140) | Parentheses-depth tokenizer for S-expressions |
| `LegalitySyllableTokenizer` / `find_legal_onsets` / `onset` | `legality_principle.py` (147) | Syllabification via sonority; language-specific |
| `SyllableTokenizer` / `assign_values` / `validate_syllables` | `sonority_sequencing.py` (194) | Same family — assigns sonority scores |
| `NISTTokenizer` / `international_tokenize` / `lang_independent_sub` | `nist.py` (179) | NIST MT evaluation tokenization; unicode ranges |
| `ReppTokenizer` / `generate_repp_command` / `parse_repp_outputs` / `find_repptokenizer` | `repp.py` (149) | External binary `repp` — shells out, no pure logic |
| `StanfordTokenizer` | `stanford.py` (115) | Shells out to Java `Stanford CoreNLP`; no logic to port |
| `StanfordSegmenter` / `segment_sents` / `segment_file` / `segment` / `default_config` | `stanford_segmenter.py` (292) | Same — Java segmenter wrapper |
| `TextTilingTokenizer` / `TokenSequence` / `TokenTableField` / `blk_frq` / `smooth` | `texttiling.py` (474) | Topic-boundary detection; math-heavy (cosine, smoothing) |
| `TreebankWordDetokenizer.detokenize` | `treebank.py` | Alias; covered with Treebank |

---

## Usage ranking (`scripts/rank_usage.py` on `nltk/tokenize`)

Counts are **intra-package** (calls/imports inside `nltk/tokenize` itself). External downstream usage (the real signal for NLTK as a library) is not captured — so `word_tokenize`/`sent_tokenize` undercount massively.

| Rank | Symbol | Uses | Defined in | External-usage note |
|---|---|---|---|---|
| 1 | `TokenizerI` | 16 | `api.py` | Internal base class — high count because every tokenizer subclasses it. Not a user call site. |
| 2 | `tokenize` | 11 | `api.py` | Abstract method name, not a distinct API. |
| 3 | `string_span_tokenize` | 5 | `util.py` | Internal helper. |
| 3 | `regexp_span_tokenize` | 5 | `util.py` | Internal helper. |
| 5 | `MacIntyreContractions` | 4 | `destructive.py` | Holder for contraction regexes, not called by users. |
| — | **`word_tokenize`** | **1** | `punkt.py` / re-exported in `__init__.py` | **Intra-package undercount — by far the #1 external call site.** Treat as rank 1 for planning. See user guidance. |
| — | **`sent_tokenize`** | **1** | `__init__.py` | **Same — #2 externally. Direct wrapper over Punkt.** |
| — | `NLTKWordTokenizer` | 2 | `destructive.py` | **The implementation behind `word_tokenize`; actual workhorse. Plan ranks this highest per guidance.** |
| — | `PunktSentenceTokenizer` / `PunktTokenizer` | 1–2 | `punkt.py` | Behind `sent_tokenize`; next highest external impact. |
| 8 | `TweetTokenizer` | 2 | `casual.py` | Moderate external use (social-text pipelines). |
| 8 | `RegexpTokenizer` | 2 | `regexp.py` | Used externally for custom patterns. |
| 10+ | Many others | 0–1 | various | Long tail; see full JSON below. |

**Surprising results:**
- `TreebankWordTokenizer` counts 1 but `NLTKWordTokenizer` counts 2 — the older class is now largely superseded; `NLTKWordTokenizer` is what `word_tokenize` actually calls (the plan reflects this).
- `nist.py`, `stanford.py`, `stanford_segmenter.py`, `repp.py` all score 0–1. Consistent with user guidance to skip them unless external ranking shows real use — they shell out to external tools, little pure logic.
- 30+ symbols at 0 — many are `PunktParameters` internals (`type_no_period`, `is_ellipsis`, etc.) that are only invoked reflectively/via attribute lookup, invisible to AST call-counting; `punkt.py` is larger than its rank suggests.

Full ranking JSON (abbreviated to top 40, full file at `rank_usage.json` if generated):
`TokenizerI:16, tokenize:11, string_span_tokenize:5, regexp_span_tokenize:5, MacIntyreContractions:4, blk_frq:4, align_tokens:4, ...` — see script output in session log.

## Porting risk (highest first)

1. **Regex lookaround — `re` → Rust `regex` crate incompatibility** (affects `treebank.py`, `destructive.py`, `punkt.py`)
   - `destructive.py:31` `r"(?i)\b(wan)(?#X)(na)(?=\s)"` — contains `(?=...)` positive lookahead (`(?#X)` is a comment but the lookahead is real).
   - `punkt.py` contains both `(?=--)` and lookahead blocks in `period_context` pattern — e.g. `re_boundary_realignment = re.compile(r'["\')\]}]+?(?:\s+|(?=--)|$)', re.MULTILINE)` and the multi-line `(?= # Sequences...` in `_re_word_tokenizer`, plus `text_contains_sentbreak` lookahead.
   - Rust `regex` does **not** support lookahead/lookbehind. Requires either `fancy-regex` (backtracking, slower, supports lookaround) or a manual rewrite. Decision must be per-regex and recorded in PLAN.md — not left to `rust-porter` to pick silently.

2. **Punkt is a trained statistical model** (`punkt.py`, 1826 lines)
   - Inference and training are interleaved in `PunktTrainer`/`PunktTokenizer`. Training mutates `FreqDist` counters, computes log-likelihood thresholds, and discovers abbrevs/collocations. Reproducing bit-identical training from Python in Rust is high-risk (floating thresholds, `FreqDist` semantics, random tie-breaks if any). The pickle'd parameters shipped in `nltk_data` are the artifact actually exercised at call time. Must split: milestone for **inference** (scoring text against existing params — port this) vs **training** (fitting new params from corpus — either port with loose tolerance or keep as documented Python fallback under "Known deviations").

3. **Casual/TweetTokenizer uses `regex` (not `re`) + order-dependent cascade**
   - Depends on `regex` crate semantics: `regex.VERBOSE|I|UNICODE`, `\U` escapes, `\u200d` ZWJ sequences, emoji flag pairs, verbose `(?x)` blocks. Pure `re` in Rust is insufficient; the Rust `regex` crate covers most but ZWJ/emoji segmentation needs explicit Unicode handling (`unicode-segmentation` or `regex` with `unicode` feature). Order matters: `PHONE_REGEX` must be tried first, `EMOTICONS` before `tags`, last pattern `\S` is catch-all. A wrong order silently changes tokenization. Also `HANG_RE` (`([^a-zA-Z0-9])\1{3,}`) and `EMOTICON_RE` must be ported exactly.

4. **Destructive tokenizers are not round-trippable**
   - `NLTKWordTokenizer` / `TreebankWordTokenizer` munge input via sequential `re.sub` then `split()` — whitespace is lost. `span_tokenize` reconstructs via `align_tokens` (which uses `str.index(token, point)`). Rust must preserve both `tokenize` and `span_tokenize` semantics, including the quote-restoration branch (`matched = [m.group() for m in re.finditer(r"``|'{2}|\"", text)]`). `align_tokens` raises `ValueError` if a token not found — error surfaces must match Python.

5. **Unicode / CJK edge cases**
   - `ToktokTokenizer`, `CJKChars`, `is_cjk`, `punkt.py` thresholds, and `texttiling.py` handle CJK ranges, curly quotes `«»“”‘’`, ellipsis `...` / `\.{2,}`, and currency/number punctuation `([:,])([^\d])`. Must test with non-ASCII; Rust string is UTF-8, but byte-vs-char offsets for `span_tokenize` must be byte offsets matching Python's character indices semantics (careful: Rust `.len()` is bytes, Python `len(s)` is codepoints).

6. **Mutable/default-arg & GIL**
   - `PunktTrainer.train(text, verbose=False)` accumulates state; `WordTokenizer` caches compiled regexes lazily on the class (`_WORD_RE`, `_PHONE_WORD_RE` singletons). No threading concerns in original, but singleton compilation needs `OnceLock` in Rust. No mutable default-arg traps here (Python side uses `None` then constructs).

7. **External-process modules** — `repp.py`, `stanford.py`, `stanford_segmenter.py` just wrap Java/binary — no logic to port; kept as Python shims.

## Dependencies

| Python dep | Purpose | Rust suggestion |
|---|---|---|
| `re` (stdlib) | All regex cascades (`treebank`, `destructive`, `punkt`, `regexp`, `simple`) | `regex` crate (RE2, no lookaround) + `fancy-regex` where lookaround is kept (see PLAN.md per-regex table) |
| `regex` (pypi) | `casual.py` — PCRE-ish, unicode, verbose | `regex` crate with `unicode` feature; verify ZWJ/emoji with `unic-emoji-char` or keep as `regex` (already handles `\p{Emoji}`) |
| `html.entities` | Entity decoding in casual | `htmlescape` / manual map, or keep in Python shim |
| `xml.sax.saxutils` | `xml_escape`/`xml_unescape` in `util.py` | `quick-xml` or hand-rolled map (tiny surface) |
| `nltk.probability.FreqDist` | Punkt training counters | `std::collections::HashMap<String,i32>` + helper; `FreqDist` is `Counter` with `N()`/`freq()` |
| `nltk.data.load` | Loads pickle'd Punkt params | See Punkt params handling — either `serde_pickle`/`serde` or embed params as Rust structs via build-time conversion script; do not ship pickles to Rust |
| `math`, `collections`, `string`, `html` | Helpers | stdlib equivalents (`libm` not needed) |

## Numeric / equivalence testing notes

- No NaN/inf/overflow semantics — text-only.
- Important for matrix testing: empty string, whitespace-only, unicode (CJK, emoji, ZWJ, regional flags `🇬🇧`), URLs, @handles, hashtags, emails, phone numbers, numbers with separators (`$3.88`, `3,36`), ellipsis `...`, contractions (`can't`→`ca n't` vs `can't` depending on tokenizer), quote styles (`"`, `` ``, `''`, `«»“”`), and period-ambiguity sentences (`Mr. Smith went...`).
- `align_tokens` / `span_tokenize` error path: `ValueError('substring "X" not found')` must match exactly including message text.
- Float tolerance: only in `texttiling.py` (`blk_frq`, cosine) — otherwise exact string equality.

## Existing tests

- `nltk/test/test_tokenize.py`, doctests in each file. Cover basic word/sentence tokenization; light on unicode edge cases and on `span_tokenize` invariants. Matrix tests should extend beyond them.
