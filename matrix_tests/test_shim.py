"""Shim test: exercises the README drop-in patch."""
import ported_lib
import nltk.tokenize


def test_shim_word_tokenize():
    orig_wt = nltk.tokenize.word_tokenize
    try:
        nltk.tokenize.word_tokenize = ported_lib.word_tokenize
        nltk.tokenize.sent_tokenize = ported_lib.sent_tokenize
        # go through nltk.* shimmed path, not ported_lib directly
        assert nltk.tokenize.word_tokenize("Hello, world. It costs $5.00.") == \
               ported_lib.word_tokenize("Hello, world. It costs $5.00.")
        assert nltk.word_tokenize("Mr. Smith went home. He left.") == \
               ported_lib.word_tokenize("Mr. Smith went home. He left.")
        assert nltk.tokenize.sent_tokenize("Mr. Smith went home. He left.") == \
               ported_lib.sent_tokenize("Mr. Smith went home. He left.")
        assert nltk.sent_tokenize("Hello world. How are you?") == \
               ported_lib.sent_tokenize("Hello world. How are you?")
    finally:
        nltk.tokenize.word_tokenize = orig_wt
        # restore sent_tokenize too if it existed
        try:
            import importlib
            importlib.reload(nltk.tokenize)
        except Exception:
            pass
