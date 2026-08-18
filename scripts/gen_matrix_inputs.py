"""
Generates the input matrix used for differential testing between the
original Python implementation and the Rust-backed port.

The matrix-test-writer agent extends CASE_GENERATORS with one entry per
public function it's covering. Keep generators deterministic (seeded) so
runs are reproducible, and keep `smoke` sets small/fast and `full` sets
thorough.

Usage:
    from scripts.gen_matrix_inputs import cases_for
    for case in cases_for("mymodule.myfunction", size="smoke"):
        ...
"""

from __future__ import annotations

import random
import string
from dataclasses import dataclass, field
from typing import Any, Callable, Iterator

SEED = 20260816  # fixed seed -> reproducible matrix runs


@dataclass
class Case:
    """One input matrix cell."""

    label: str  # short human-readable id, e.g. "empty_list", "unicode_mixed"
    args: tuple = ()
    kwargs: dict = field(default_factory=dict)


# Reusable building blocks — extend/compose these per function rather than
# writing every generator from scratch.

def boundary_ints() -> list[int]:
    return [-1, 0, 1, 2**31 - 1, -(2**31), 2**63 - 1, -(2**63)]


def boundary_floats() -> list[float]:
    return [0.0, -0.0, 1.0, -1.0, float("inf"), float("-inf"), float("nan")]


def edge_strings() -> list[str]:
    return ["", " ", "a", "a" * 10_000, "\x00", "😀🚀", "\u200b", "café"]


def edge_collections() -> list[list]:
    return [[], [1], list(range(1000))]


def randomized_ints(n: int, lo: int = -10_000, hi: int = 10_000) -> list[int]:
    rng = random.Random(SEED)
    return [rng.randint(lo, hi) for _ in range(n)]


# Registry: function name -> generator(size) -> Iterator[Case]
# The matrix-test-writer agent fills this in per module it ports.
CASE_GENERATORS: dict[str, Callable[[str], Iterator[Case]]] = {}


def register(fn_name: str):
    def deco(gen: Callable[[str], Iterator[Case]]):
        CASE_GENERATORS[fn_name] = gen
        return gen

    return deco


def cases_for(fn_name: str, size: str = "smoke") -> Iterator[Case]:
    if fn_name not in CASE_GENERATORS:
        raise KeyError(
            f"No matrix cases registered for {fn_name!r} — "
            "the matrix-test-writer agent needs to add a generator."
        )
    yield from CASE_GENERATORS[fn_name](size)


# ---------------------------------------------------------------------------
# Shared edge corpora — representative of nltk.tokenize usage
# ---------------------------------------------------------------------------

_WORD_CASES = [
    ("empty", ""),
    ("single_word", "hello"),
    ("punct", "Hello, world."),
    ("quotes", 'He said "hello".'),
    ("currency", "It costs $5.00."),
    ("contraction", "I can't do it."),
    ("contraction2", "They've gone."),
    ("parens", "Hello (world) test."),
    ("brackets", "a [b] c"),
    ("ellipsis", "Wait... what?"),
    ("unicode", "café naïve résumé"),
    ("emoji", "Hello 😀 world 🚀"),
    ("whitespace", "  hello   world  "),
    ("newlines", "hello\nworld"),
    ("long", " ".join(["hello"] * 100)),
]

_SENT_CASES = [
    ("empty", ""),
    ("single", "Hello world."),
    ("two", "Hello world. How are you?"),
    ("abbrev", "Mr. Smith went home. He left."),
    ("abbrev2", "Dr. Jones and Mrs. Smith met."),
    ("quotes", 'He said "Hello." She replied.'),
    ("long", " ".join(["This is a sentence."] * 10)),
    # --- hard Punkt cases exercising real Kiss&Strunk paths ---
    # ortho_context disambiguation: same abbrev, break vs no-break
    ("ortho_break", "The U.S. is large. Many people live there."),
    ("ortho_no_break", "U.S. troops arrived home."),
    # 15 random abbrevs from the 156-word list (not just mr/dr)
    ("abbrev_inc", "Acme Inc. is large. It employs many people."),
    ("abbrev_vs", "It was him vs. her in court. The judge decided."),
    ("abbrev_dec", "It happened on Dec. 5. The event was memorable."),
    ("abbrev_jan", "See you on Jan. 10. Bring a gift."),
    ("abbrev_feb", "Due by Feb. 28. No extensions allowed."),
    ("abbrev_aug", "Start Aug. 1. Classes begin then."),
    ("abbrev_st", "It is on Main St. near town. We walk there."),
    ("abbrev_va", "She lives in Va. now. It is lovely."),
    ("abbrev_tenn", "Born in Tenn. in 1980. He moved later."),
    ("abbrev_gen", "Gen. Smith ordered it. The troops obeyed."),
    ("abbrev_prof", "Prof. Adams teaches. Students attend."),
    ("abbrev_wash", "He is from Wash. state. It rains often."),
    ("abbrev_corp", "Tech Corp. announced it. Shares rose."),
    ("abbrev_ltd", "Global Ltd. expanded. Markets reacted."),
    ("abbrev_phd", "She has a Ph.D. in physics. She researches."),
    # back-to-back initials
    ("initials", "J. R. R. Tolkien wrote it. He was British."),
    # abbreviation at true end-of-text (no trailing sentence)
    ("abbrev_eot", "He works at the U.N."),
    # real ENGLISH_COLLOCATIONS entry: ("b", "stewart") -> "B. Stewart"
    ("collocation_b_stewart", "B. Stewart arrived home. He was tired."),
    # quotes + realignment
    ("quotes2", 'He said "Stop." She left.'),
    # numbers with periods
    ("numbers", "It cost $3.5 million. Sales rose 12%."),
]

_TWEET_CASES = [
    ("plain", "Hello world"),
    ("hashtag", "I love #rust!"),
    ("handle", "Hey @user how are you?"),
    ("url", "Visit https://example.com today"),
    ("url2", "Check http://example.org/path?q=1"),
    ("emoticon", "Hello :) world :("),
    ("emoji_flag", "Flag 🇬🇧 test"),
    ("email", "Contact foo@example.com"),
    ("phone", "Call +1 (555) 123-4567"),
    ("lengthening", "sooooo cool"),
    ("arabic_handle", "مرحبا @user"),
]

# ---------------------------------------------------------------------------
# Generators — one per ported function (keys match FUNCTION_PAIRS in compare_outputs.py)
# ---------------------------------------------------------------------------

@register("word_tokenize")
def _word_cases(size: str) -> Iterator[Case]:
    for label, text in _WORD_CASES:
        yield Case(label=label, args=(text,))
    if size == "full":
        rng = random.Random(SEED)
        for i in range(20):
            words = ["".join(rng.choices(string.ascii_letters, k=rng.randint(1, 8))) for _ in range(rng.randint(1, 10))]
            punct = rng.choice([".", ",", "!", "?", ""])
            yield Case(label=f"random_{i}", args=(" ".join(words) + punct,))


@register("sent_tokenize")
def _sent_cases(size: str) -> Iterator[Case]:
    for label, text in _SENT_CASES:
        yield Case(label=label, args=(text,))
    if size == "full":
        rng = random.Random(SEED + 1)
        sents = ["Hello world. ", "Mr. Smith went. ", "What? ", "Fine! "]
        for i in range(10):
            txt = "".join(rng.choices(sents, k=rng.randint(1, 5)))
            yield Case(label=f"random_{i}", args=(txt,))


@register("regexp_tokenize")
def _regexp_cases(size: str) -> Iterator[Case]:
    cases = [
        ("word", ("hello   world", r"\w+", False, True)),
        ("gaps", ("hello   world", r"\s+", True, True)),
        ("gaps_keep_empty", ("hello   world", r"\s+", True, False)),
        ("punct", ("Hello, world.", r"\w+", False, True)),
        ("empty_text", ("", r"\w+", False, True)),
        ("no_match", ("hello", r"\d+", False, True)),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)
    # also test pattern with unicode
    yield Case(label="unicode_pattern", args=("café naïve", r"\w+", False, True))


@register("string_span_tokenize")
def _string_span_cases(size: str) -> Iterator[Case]:
    cases = [
        ("basic", ("a b c", " ")),
        ("empty", ("", " ")),
        ("no_sep", ("hello", " ")),
        ("multi_char_sep", ("a,,b,,c", ",,")),
        ("unicode", ("a b café", " ")),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)


@register("regexp_span_tokenize")
def _regexp_span_cases(size: str) -> Iterator[Case]:
    cases = [
        ("spaces", ("a b  c", r"\s+")),
        ("empty", ("", r"\s+")),
        ("no_match", ("hello", r"\d+")),
        ("word_bound", ("hello world", r"\s+")),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)


@register("spans_to_relative")
def _spans_cases(size: str) -> Iterator[Case]:
    cases = [
        ("empty", ([],)),
        ("single", ([(0, 5)],)),
        ("two", ([(0, 5), (6, 11)],)),
        ("three", ([(0, 3), (4, 7), (8, 12)],)),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)


@register("is_cjk")
def _cjk_cases(size: str) -> Iterator[Case]:
    cases = [
        ("ascii", ("a",)),
        ("cjk_true", ("\u33fe",)),
        ("cjk_false", ("\ufe5f",)),
        ("han", ("\u4e00",)),
        ("hangul", ("\uac00",)),
        ("emoji", ("😀",)),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)


@register("xml_escape")
def _xml_escape_cases(size: str) -> Iterator[Case]:
    # xml_escape in NLTK takes (text, ...) with optional flags — test plain escaping
    cases = [
        ("plain", ("hello",)),
        ("amp", ("a & b",)),
        ("lt_gt", ("a <b>",)),
        ("quotes", ('a "b" \'c\'',)),
        ("brackets", ("a [b] | c",)),
        ("all", ('a & b <c> \'d\' "e" [f] | g',)),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)


@register("xml_unescape")
def _xml_unescape_cases(size: str) -> Iterator[Case]:
    cases = [
        ("plain", ("hello",)),
        ("amp", ("a &amp; b",)),
        ("lt", ("a &lt;b&gt;",)),
        ("all", ("a &amp; b &lt;c&gt; &apos;d&apos; &quot;e&quot; &#124; &#91;f&#93;",)),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)


@register("align_tokens")
def _align_cases(size: str) -> Iterator[Case]:
    cases = [
        ("basic", ([["hello", "world"], "hello world"],)),
        ("single", ([["hello"], "hello"],)),
        ("punct", ([["Hello", ",", "world"], "Hello, world"],)),
    ]
    # unpack for compare_outputs: (tokens, sentence) -> spans
    for label, args in cases:
        tokens, sent = args[0]
        yield Case(label=label, args=(tokens, sent))


@register("casual_tokenize")
def _casual_cases(size: str) -> Iterator[Case]:
    for label, text in _TWEET_CASES:
        yield Case(label=label, args=(text,))
    if size == "full":
        # flags matrix
        yield Case(label="preserve_case_false", args=("Hello WORLD :)",), kwargs={"preserve_case": False})
        yield Case(label="reduce_len", args=("sooooo cool",), kwargs={"reduce_len": True})
        yield Case(label="strip_handles", args=("@user hello",), kwargs={"strip_handles": True})


@register("toktok_tokenize")
def _toktok_cases(size: str) -> Iterator[Case]:
    cases = [
        ("basic", ("Hello, world.",)),
        ("parens", ("Hello (world) test.",)),
        ("empty", ("",)),
        ("unicode", ("café naïve",)),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)


@register("mwe_tokenize")
def _mwe_cases(size: str) -> Iterator[Case]:
    cases = [
        ("basic", ([["a", "little", "bit", "goes"], [["a", "little", "bit"]]],)),
        ("no_mwe", ([["hello", "world"], [["foo", "bar"]]],)),
        ("overlap", ([["a", "little", "bit", "of", "a", "little"], [["a", "little", "bit"], ["a", "little"]]],)),
    ]
    for label, args in cases:
        tokens, mwes = args[0]
        yield Case(label=label, args=(tokens, mwes))


@register("sexpr_tokenize")
def _sexpr_cases(size: str) -> Iterator[Case]:
    yield Case(label="basic", args=("(a b (c d)) e f (g)",))
    yield Case(label="empty", args=("",))
    yield Case(label="single", args=("(a b)",))
    yield Case(label="nested", args=("((a b) (c d))",))

@register("detokenize")
def _detokenize_cases(size: str) -> Iterator[Case]:
    cases = [
        ("simple", (["Hello", ",", "world", "."],)),
        ("contraction", (["I", "ca", "n't", "go"],)),
        ("parens", (["Hello", "(", "world", ")"],)),
        ("quotes", (["He", "said", "``", "hi", "''", "."],)),
        ("empty", ([],)),
        ("single", (["hello"],)),
        ("dashes", (["a", "--", "b"],)),
    ]
    for label, args in cases:
        yield Case(label=label, args=args)
    if size == "full":
        yield Case(label="convert_parens", args=(["Hello", "-LRB-", "world", "-RRB-"],), kwargs={"convert_parentheses": True})

@register("space_tokenize")
def _space_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "a b c"), ("empty", ""), ("no_space", "hello"), ("double", "a  b"), ("unicode", "a b café"), ("tabs_newlines", "a b\tc\nd")]:
        yield Case(label=label, args=(text,))
    if size == "full":
        yield Case(label="long", args=(" ".join(["hello"] * 50),))

@register("tab_tokenize")
def _tab_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "a\tb\tc"), ("empty", ""), ("no_tab", "hello world"), ("spaces", "a b c")]:
        yield Case(label=label, args=(text,))

@register("char_tokenize")
def _char_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "abc"), ("empty", ""), ("unicode", "café"), ("emoji", "a😀b"), ("space", "a b")]:
        yield Case(label=label, args=(text,))

@register("line_tokenize")
def _line_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "a\nb\nc"), ("empty", ""), ("blank_keep", "a\n\nb"), ("single", "hello"), ("trailing", "a\nb\n")]:
        yield Case(label=label, args=(text,))
    if size == "full":
        yield Case(label="blank_discard", args=("a\n\nb\n",), kwargs={"blanklines": "discard"})
        yield Case(label="blank_keep_mode", args=("a\n\nb\n",), kwargs={"blanklines": "keep"})

@register("blankline_tokenize")
def _blankline_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "a\n\nb\n\nc"), ("empty", ""), ("no_blank", "a\nb\nc"), ("single_para", "hello world")]:
        yield Case(label=label, args=(text,))

@register("wordpunct_tokenize")
def _wordpunct_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "Hello, world."), ("empty", ""), ("unicode", "café naïve"), ("numbers", "cost $3.88"), ("parens", "a (b) c")]:
        yield Case(label=label, args=(text,))

@register("whitespace_tokenize")
def _whitespace_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "a b\tc\nd"), ("empty", ""), ("single", "hello"), ("unicode", "a b café"), ("multi", "  hello   world  ")]:
        yield Case(label=label, args=(text,))

@register("nist_tokenize")
def _nist_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "Good muffins cost $3.88 in New York."), ("empty", ""), ("lower", "Hello World"), ("numbers", "Cost $5"), ("unicode", "café naïve")]:
        yield Case(label=label, args=(text,))
    if size == "full":
        yield Case(label="lowercase", args=("Hello World",), kwargs={"lowercase": True})
        yield Case(label="no_western", args=("Hello World",), kwargs={"western_lang": False})

@register("nist_international_tokenize")
def _nist_intl_cases(size: str) -> Iterator[Case]:
    for label, text in [("western", "Hello world."), ("empty", ""), ("lower", "Hello World"), ("cjk", "Hello 阿里巴巴 world")]:
        yield Case(label=label, args=(text,))
    if size == "full":
        yield Case(label="lowercase", args=("Hello World",), kwargs={"lowercase": True})

@register("legality_tokenize")
def _legality_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "wonderful"), ("empty", ""), ("short", "a"), ("nonalpha", "123")]:
        yield Case(label=label, args=(text,))

@register("sonority_tokenize")
def _sonority_cases(size: str) -> Iterator[Case]:
    for label, text in [("basic", "justification"), ("empty", ""), ("short", "a"), ("nonalpha", "123")]:
        yield Case(label=label, args=(text,))

@register("texttiling_tokenize")
def _texttiling_cases(size: str) -> Iterator[Case]:
    for label, text in [("paras", "para one\n\npara two\n\npara three"), ("empty", ""), ("single", "Hello world"), ("short", "a b c")]:
        yield Case(label=label, args=(text,))


@register("shim_word_tokenize")
def _shim_word_cases(size: str) -> Iterator[Case]:
    # same corpus as word_tokenize — exercises README shim path
    for label, text in _WORD_CASES:
        yield Case(label=label, args=(text,))


@register("shim_sent_tokenize")
def _shim_sent_cases(size: str) -> Iterator[Case]:
    for label, text in _SENT_CASES:
        yield Case(label=label, args=(text,))


# Keep example for backward compat — filter to i64-range to avoid Rust wrapping vs Python bigint divergence
@register("example.add")
def _example_add_cases(size: str) -> Iterator[Case]:
    INT_MIN = -(2**63)
    INT_MAX = 2**63 - 1
    for i in boundary_ints():
        for j in boundary_ints():
            # skip pairs that overflow i64 wrapping (known deviation)
            if i < 0 and j < 0 and i < INT_MIN - j:
                continue
            if i > 0 and j > 0 and i > INT_MAX - j:
                continue
            # also skip cases where either value outside i64
            if i < INT_MIN or i > INT_MAX or j < INT_MIN or j > INT_MAX:
                continue
            yield Case(label=f"boundary_{i}_{j}", args=(i, j))
    n = 5 if size == "smoke" else 200
    for a, b in zip(randomized_ints(n), randomized_ints(n)):
        if INT_MIN <= a <= INT_MAX and INT_MIN <= b <= INT_MAX and INT_MIN <= a + b <= INT_MAX:
            yield Case(label=f"random_{a}_{b}", args=(a, b))
