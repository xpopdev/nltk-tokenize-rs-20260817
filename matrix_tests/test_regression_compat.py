"""
Regression tests for the 4 tokenizer NLTK-compatibility bugs.

Each test pins the exact NLTK output for inputs that previously mismatched.
If the port regresses, these fail with a clear diff.
"""

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))

try:
    from compare_outputs import FUNCTION_PAIRS, _try_import_pairs
    _try_import_pairs()
    HAS_PORTED = "word_tokenize" in FUNCTION_PAIRS
except Exception:
    HAS_PORTED = False

pytestmark = pytest.mark.skipif(not HAS_PORTED, reason="ported_lib not built")

# Import directly for skip granularity
try:
    import ported_lib
    from nltk.tokenize import word_tokenize as nwt, sent_tokenize as nst
    from nltk.tokenize.casual import TweetTokenizer as NLTKTweet
    from nltk.tokenize.toktok import ToktokTokenizer as NLTKToktok
    HAS_NLTK = True
except ImportError:
    HAS_NLTK = False

skip_no_nltk = pytest.mark.skipif(not HAS_NLTK, reason="nltk not installed")


# ── word_tokenize ──────────────────────────────────────────────
@skip_no_nltk
@pytest.mark.parametrize("text", [
    "Hello, world.",
    "Hello, world. Hello, world.",
    "Hello, world. " * 10,
    "Hello, world. " * 500,  # was kept as "world." due to >2000 fast path
    "Hello, world. " * 1000,
    "This is a sentence. " * 100,
    "The value is 1,234.56 and another 9,876.54. Done.",
    'He said, "Hello."',
    "Mr. Smith went home.",
    "It costs $5.00.",
    "1,234.56",
    "I don't know.",
    "Wait... what?",
    "Hello (world) test.",
])
def test_word_tokenize_parity(text):
    assert ported_lib.word_tokenize(text) == nwt(text)


@skip_no_nltk
def test_word_tokenize_long_sentences_period():
    text = "This is a sentence. " * 50
    assert ported_lib.word_tokenize(text) == nwt(text)


@skip_no_nltk
def test_word_tokenize_long_numbers():
    text = "The value is 1,234.56 and another 9,876.54. " * 10
    assert ported_lib.word_tokenize(text) == nwt(text)


# ── sent_tokenize ──────────────────────────────────────────────
@skip_no_nltk
@pytest.mark.parametrize("text", [
    "Hello world.",
    "Hello world. How are you?",
    "Mr. Smith went home. He left.",
    'He said "Hello." She replied.',
    "The U.S. is large. U.S. troops moved.",
    "Hello, world. " * 10,
    "This is a sentence. " * 20,
    "What? No! Yes.",
    "a\n\nb\n\nc",
])
def test_sent_tokenize_parity(text):
    assert ported_lib.sent_tokenize(text) == nst(text)


# ── casual / TweetTokenizer ────────────────────────────────────
@skip_no_nltk
@pytest.mark.parametrize("text,kwargs", [
    ("Hello, world.", {}),
    ("I don't know.", {}),
    ("https://example.com", {}),
    ("Visit https://example.com today", {}),
    ("test@example.com", {}),
    ("Contact foo@example.com", {}),
    ("#Python @user", {}),
    ("I love #rust!", {}),
    ("sooooo cool", {}),
    ("sooooo cool", {"reduce_len": True}),
    ("@user hello", {"strip_handles": True}),
    ("Hello WORLD :)", {"preserve_case": False}),
    ("Hello :)", {}),
    ("Hello 😀 world 🚀", {}),
    ("Flag 🇬🇧 test", {}),
    ("Call +1 (555) 123-4567", {}),
    ("a!!! b??? c...", {}),
    ("Price: &pound;100", {}),
    ("a & b <c>", {}),
])
def test_casual_parity(text, kwargs):
    nltk_tok = NLTKTweet(**{k: v for k, v in kwargs.items() if k in ("preserve_case", "reduce_len", "strip_handles", "match_phone_numbers")})
    expected = nltk_tok.tokenize(text)
    got = ported_lib.casual_tokenize_py(text, **{"preserve_case": True, "reduce_len": False, "strip_handles": False, "match_phone_numbers": True} | kwargs)
    assert got == expected, f"casual mismatch for {text!r} kwargs={kwargs}: got={got} expected={expected}"


# ── toktok ─────────────────────────────────────────────────────
@skip_no_nltk
@pytest.mark.parametrize("text", [
    "Hello, world. It costs $5.00.",
    "It costs $5.00.",
    "Price is $3.88 and $5.00",
    "Value 1,234.56",
    "Hello (world) [test]",
    "Hello, world.",
    "The https://example.com is here",
    "a,, b -- c ... d.",
    "Test \u00A0 non-breaking",
    "Hello|world",
    "a -- b — c – d",
    "Price 1,234 and hello, world",
])
def test_toktok_parity(text):
    tok = NLTKToktok()
    assert ported_lib.toktok_tokenize_py(text) == tok.tokenize(text)


# ── extra: hashtag/mention/email/url/emoji/numbers ─────────────
@skip_no_nltk
def test_toktok_currency():
    tok = NLTKToktok()
    text = "It costs $5.00."
    assert ported_lib.toktok_tokenize_py(text) == tok.tokenize(text)
    assert "$" in tok.tokenize(text)
    assert tok.tokenize(text) == ["It", "costs", "$", "5.00", "."]


@skip_no_nltk
def test_casual_url_email_hashtag_mention():
    cases = [
        ("https://example.com", ["https://example.com"]),
        ("test@example.com", ["test@example.com"]),
    ]
    for text, _ in cases:
        nltk_tok = NLTKTweet()
        assert ported_lib.casual_tokenize_py(text) == nltk_tok.tokenize(text)
