"""Drop-in shim — patch nltk to use Rust under the hood."""
try:
    import ported_lib
    import nltk.tokenize
    nltk.tokenize.word_tokenize = ported_lib.word_tokenize
    nltk.tokenize.sent_tokenize = ported_lib.sent_tokenize
    print("shim active: nltk now uses Rust")
    print(nltk.tokenize.word_tokenize("Hello, world. It costs $5.00."))
    print(nltk.tokenize.sent_tokenize("Mr. Smith went home. He left."))
except ImportError as e:
    print(f"shim fallback: {e}")

# also works via nltk. top-level
import nltk
print(nltk.word_tokenize("Hello world"))
