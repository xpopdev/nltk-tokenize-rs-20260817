"""Batch API — rayon-parallel for corpora."""
import ported_lib

texts = ["Hello world. How are you?"] * 1000
# parallel batch (rayon) — 4-8× on large corpora
print(len(ported_lib.word_tokenize_batch(texts)))
print(len(ported_lib.sent_tokenize_batch(texts)))
