import time
import statistics
import nltk
import ported_lib as tok


# ============================================================
# NLTK SETUP
# ============================================================

def setup_nltk():
    resources = [
        ("tokenizers/punkt_tab/english/", "punkt_tab"),
        ("tokenizers/punkt/english.pickle", "punkt"),
        ("corpora/gutenberg", "gutenberg"),
    ]
    for path, package in resources:
        try:
            nltk.data.find(path)
        except LookupError:
            print(f"Downloading NLTK resource: {package}")
            nltk.download(package, quiet=True)


setup_nltk()

from nltk.tokenize import word_tokenize, sent_tokenize
from nltk.corpus import gutenberg


# ============================================================
# DATASET
# ============================================================

print("\nLoading Gutenberg corpus...")

TEXTS = [gutenberg.raw(fileid) for fileid in gutenberg.fileids()]
TEXTS = [text for text in TEXTS if text.strip()]
LARGE_TEXT = "\n".join(TEXTS)
SIZE_MB = len(LARGE_TEXT.encode("utf-8")) / 1024 / 1024

print(f"Documents: {len(TEXTS):,}")
print(f"Characters: {len(LARGE_TEXT):,}")
print(f"Dataset size: {SIZE_MB:.2f} MB")


# ============================================================
# CORRECTNESS
# ============================================================

def correctness_test(name, nltk_func, ported_func, texts):
    print("\n" + "=" * 78)
    print(f"CORRECTNESS: {name}")
    print("=" * 78)
    total = passed = failed = 0
    first_mismatch = None
    for index, text in enumerate(texts):
        total += 1
        try:
            expected = nltk_func(text)
            actual = ported_func(text)
        except Exception as exc:
            failed += 1
            if first_mismatch is None:
                first_mismatch = {"document": index, "text": text, "expected": "SUCCESS", "actual": repr(exc)}
            continue
        if expected == actual:
            passed += 1
        else:
            failed += 1
            if first_mismatch is None:
                first_mismatch = {"document": index, "text": text, "expected": expected, "actual": actual}
    accuracy = passed / total * 100 if total else 0
    print(f"Documents: {total:,}")
    print(f"Passed:    {passed:,}")
    print(f"Failed:    {failed:,}")
    print(f"Accuracy:  {accuracy:.4f}%")
    if first_mismatch:
        print("\nFirst mismatch")
        print("-" * 78)
        print(f"Document: {first_mismatch['document']}")
        text = first_mismatch["text"]
        print(f"Text preview:\n{text[:500]!r}")
        expected = first_mismatch["expected"]
        actual = first_mismatch["actual"]
        if isinstance(expected, list):
            print("\nNLTK:"); print(expected[:100])
        else:
            print("\nNLTK:"); print(expected)
        if isinstance(actual, list):
            print("\nPorted:"); print(actual[:100])
        else:
            print("\nPorted:"); print(actual)
    return {"total": total, "passed": passed, "failed": failed, "accuracy": accuracy}


# ============================================================
# TOKEN COUNTS
# ============================================================

def count_outputs():
    print("\n" + "=" * 78)
    print("OUTPUT COMPARISON")
    print("=" * 78)
    print("\nRunning NLTK...")
    nltk_words = word_tokenize(LARGE_TEXT)
    nltk_sentences = sent_tokenize(LARGE_TEXT, language="english")
    print("Running ported library...")
    ported_words = tok.word_tokenize(LARGE_TEXT)
    ported_sentences = tok.sent_tokenize(LARGE_TEXT, language="english")
    print("\nWord tokens")
    print(f"NLTK:   {len(nltk_words):,}")
    print(f"Ported: {len(ported_words):,}")
    print(f"Difference: {len(nltk_words) - len(ported_words):+,}")
    print("\nSentences")
    print(f"NLTK:   {len(nltk_sentences):,}")
    print(f"Ported: {len(ported_sentences):,}")
    print(f"Difference: {len(nltk_sentences) - len(ported_sentences):+,}")
    return (nltk_words, ported_words, nltk_sentences, ported_sentences)


# ============================================================
# PERFORMANCE
# ============================================================

def benchmark(name, nltk_func, ported_func, text, iterations=5):
    print("\n" + "=" * 78)
    print(f"BENCHMARK: {name}")
    print("=" * 78)
    print(f"Input size: {SIZE_MB:.2f} MB")
    print(f"Iterations: {iterations}")
    nltk_func(text)
    ported_func(text)
    nltk_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        nltk_func(text)
        elapsed = time.perf_counter() - start
        nltk_times.append(elapsed)
    ported_times = []
    for _ in range(iterations):
        start = time.perf_counter()
        ported_func(text)
        elapsed = time.perf_counter() - start
        ported_times.append(elapsed)
    nltk_mean = statistics.mean(nltk_times)
    ported_mean = statistics.mean(ported_times)
    nltk_median = statistics.median(nltk_times)
    ported_median = statistics.median(ported_times)
    speedup = nltk_mean / ported_mean
    nltk_throughput = SIZE_MB / nltk_mean
    ported_throughput = SIZE_MB / ported_mean
    print("\nNLTK")
    print(f"  Mean:       {nltk_mean:.4f} s")
    print(f"  Median:     {nltk_median:.4f} s")
    print(f"  Min:        {min(nltk_times):.4f} s")
    print(f"  Max:        {max(nltk_times):.4f} s")
    print(f"  Throughput: {nltk_throughput:.2f} MB/s")
    print("\nPorted")
    print(f"  Mean:       {ported_mean:.4f} s")
    print(f"  Median:     {ported_median:.4f} s")
    print(f"  Min:        {min(ported_times):.4f} s")
    print(f"  Max:        {max(ported_times):.4f} s")
    print(f"  Throughput: {ported_throughput:.2f} MB/s")
    print("\nResult")
    print(f"  Speedup: {speedup:.2f}x")
    return {"name": name, "nltk_mean": nltk_mean, "ported_mean": ported_mean, "speedup": speedup, "nltk_throughput": nltk_throughput, "ported_throughput": ported_throughput}


# ============================================================
# MAIN
# ============================================================

def main():
    print("\n" + "=" * 78)
    print("PORTED LIBRARY vs REAL NLTK")
    print("LARGE CORPUS DIFFERENTIAL BENCHMARK")
    print("=" * 78)
    cw = correctness_test("word_tokenize", word_tokenize, tok.word_tokenize, TEXTS)
    cs = correctness_test("sent_tokenize", lambda t: sent_tokenize(t, language="english"), lambda t: tok.sent_tokenize(t, language="english"), TEXTS)
    count_outputs()
    bw = benchmark("word_tokenize (Gutenberg {:.1f} MB)".format(SIZE_MB), word_tokenize, tok.word_tokenize, LARGE_TEXT, iterations=5)
    bs = benchmark("sent_tokenize (Gutenberg {:.1f} MB)".format(SIZE_MB), lambda t: sent_tokenize(t, language="english"), lambda t: tok.sent_tokenize(t, language="english"), LARGE_TEXT, iterations=5)
    print("\n" + "=" * 78)
    print("SUMMARY")
    print("=" * 78)
    print(f"word_tokenize accuracy: {cw['accuracy']:.4f}% ({cw['passed']}/{cw['total']}) speedup {bw['speedup']:.2f}x")
    print(f"sent_tokenize accuracy: {cs['accuracy']:.4f}% ({cs['passed']}/{cs['total']}) speedup {bs['speedup']:.2f}x")
    if cw['failed'] or cs['failed']:
        print("RESULT: FAIL — mismatches found")
        raise SystemExit(1)
    print("RESULT: PASS")


if __name__ == "__main__":
    main()
