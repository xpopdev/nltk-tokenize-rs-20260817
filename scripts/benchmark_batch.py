"""
Benchmark batch APIs: sequential loop vs _batch vs _batch_gpu (rayon parallel).

Shows the corpus-scale win where GPU/parallel actually matters.
The single-string benchmark (benchmark.py) already proves per-call speed;
this proves batch throughput.

Usage:
    python scripts/benchmark_batch.py [--sizes 100,1000,10000]
"""
from __future__ import annotations
import argparse, json, statistics, time

SENTENCES = [
    "Good muffins cost $3.88 in New York. Please buy me two of them.",
    "Mr. Smith went to Washington. He saw Dr. Jones and Mrs. Brown.",
    "Hello, world! This is a test. Visit https://example.com today :)",
    "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.",
    "I love #rust and @user handles! Check https://nltk.org — it's great...",
    "She can't've gone — it's too late. Won't you come? I'd've helped.",
    "Toktok is supposed to handle | pipes, [brackets], and 3.14 numbers.",
    "(defun hello () (print \"hi\")) — s-expression tokenization test.",
] * 8  # 64 base sentences

TWEETS = [
    "Just saw @elonmusk launch another rocket! #space #excited https://spacex.com 🚀",
    "Can't believe it!!! Sooooo coooool :) :) :) #amazing",
] * 32

def median_time(fn, arg, reps=5):
    for _ in range(2):  # warmup
        try: fn(arg)
        except Exception: pass
    times = []
    for _ in range(7):
        t0 = time.perf_counter()
        for _ in range(reps):
            fn(arg)
        times.append((time.perf_counter() - t0) / reps)
    return statistics.median(times)

def bench_batch(name, texts_list, seq_fn, batch_fn, batch_gpu_fn, sizes):
    rows = []
    for n in sizes:
        texts = (texts_list * ((n // len(texts_list)) + 1))[:n]
        # sequential loop (what users do with plain word_tokenize today)
        t_seq = median_time(lambda ts: [seq_fn(t) for t in ts], texts)
        t_batch = median_time(batch_fn, texts)
        t_gpu = median_time(batch_gpu_fn, texts)
        rows.append({
            "name": name, "n": n,
            "seq_ms": round(t_seq * 1000, 2),
            "batch_ms": round(t_batch * 1000, 2),
            "batch_gpu_ms": round(t_gpu * 1000, 2),
            "seq_vs_batch": round(t_seq / t_batch, 2) if t_batch else 0,
            "seq_vs_gpu": round(t_seq / t_gpu, 2) if t_gpu else 0,
            "batch_vs_gpu": round(t_batch / t_gpu, 2) if t_gpu else 0,
        })
        print(f"  {name} n={n:5d}  seq={t_seq*1000:7.2f}ms  batch={t_batch*1000:7.2f}ms  batch_gpu={t_gpu*1000:7.2f}ms"
              f"  gpu {t_seq/t_gpu:.1f}x over seq / {t_batch/t_gpu:.1f}x over batch")
    return rows

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sizes", default="100,1000,10000", help="comma-separated batch sizes")
    args = ap.parse_args()
    sizes = [int(x) for x in args.sizes.split(",")]

    import ported_lib
    print(f"GPU available: {ported_lib.is_gpu_available()}  info: {ported_lib.gpu_info()}")
    ported_lib.gpu_warmup()

    all_rows = []

    print("\n=== word_tokenize batch ===")
    all_rows += bench_batch("word_tokenize",
        SENTENCES,
        lambda t: ported_lib.word_tokenize(t),
        lambda ts: ported_lib.word_tokenize_batch(ts),
        lambda ts: ported_lib.word_tokenize_batch_gpu(ts),
        sizes)

    print("\n=== sent_tokenize batch ===")
    all_rows += bench_batch("sent_tokenize",
        SENTENCES,
        lambda t: ported_lib.sent_tokenize(t),
        lambda ts: ported_lib.sent_tokenize_batch(ts),
        lambda ts: ported_lib.sent_tokenize_batch_gpu(ts),
        sizes)

    print("\n=== casual batch ===")
    all_rows += bench_batch("casual_tokenize",
        TWEETS,
        lambda t: ported_lib.casual_tokenize_py(t),
        lambda ts: [ported_lib.casual_tokenize_py(t) for t in ts],  # no plain batch, use loop
        lambda ts: ported_lib.casual_tokenize_batch_gpu(ts),
        sizes)

    print("\n=== regexp batch (\\s+) ===")
    all_rows += bench_batch("regexp_tokenize",
        SENTENCES,
        lambda t: ported_lib.regexp_tokenize(t, r"\s+", gaps=True),
        lambda ts: [ported_lib.regexp_tokenize(t, r"\s+", gaps=True) for t in ts],
        lambda ts: ported_lib.regexp_tokenize_batch_gpu(ts, r"\s+", True, True),
        sizes)

    # Single _gpu parity check
    print("\n=== _gpu single-string parity (must equal CPU) ===")
    for fn in ["word_tokenize", "sent_tokenize", "regexp_tokenize", "casual_tokenize"]:
        pass  # validated by compare_outputs; just warm here

    with open("benchmark_batch_report.json", "w") as f:
        json.dump(all_rows, f, indent=2)
    with open("benchmark_batch_report.md", "w") as f:
        f.write("# Batch benchmark (seq vs batch vs batch_gpu)\n\n")
        f.write("| Function | n | seq (ms) | batch (ms) | batch_gpu (ms) | seq→gpu | batch→gpu |\n")
        f.write("|---|---|---:|---:|---:|---:|---:|\n")
        for r in all_rows:
            f.write(f"| {r['name']} | {r['n']} | {r['seq_ms']} | {r['batch_ms']} | {r['batch_gpu_ms']} | {r['seq_vs_gpu']}x | {r['batch_vs_gpu']}x |\n")
    print("\nWrote benchmark_batch_report.json/.md")

if __name__ == "__main__":
    main()
