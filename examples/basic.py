"""Basic usage — mirrors NLTK's API."""
import ported_lib as tok

print(tok.word_tokenize("Hello, world. It costs $5.00."))
# → ['Hello', ',', 'world', '.', 'It', 'costs', '$', '5.00', '.']

print(tok.sent_tokenize("Mr. Smith went home. He left."))
# → ['Mr. Smith went home.', 'He left.']

print(tok.word_tokenize_span("Hello, world."))
# → [(0, 5), (5, 6), (6, 11), (11, 12)]
