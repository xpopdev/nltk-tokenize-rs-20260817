"""
Verify that every _gpu / _batch_gpu variant returns identical results to its CPU counterpart.
Runs the same matrix cases against gpu suffixes, reports divergence.
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from compare_outputs import FUNCTION_PAIRS, _try_import_pairs
from gen_matrix_inputs import cases_for

def main():
    _try_import_pairs()
    import ported_lib
    print(f"GPU available: {ported_lib.is_gpu_available()}  info: {ported_lib.gpu_info()}")
    ported_lib.gpu_warmup()

    # Map each FUNCTION_PAIRS key to its _gpu counterpart
    gpu_map = {
        "word_tokenize": ("word_tokenize_gpu", lambda text, **kw: ported_lib.word_tokenize_gpu(text)),
        "sent_tokenize": ("sent_tokenize_gpu", lambda text, **kw: ported_lib.sent_tokenize_gpu(text, language=kw.get("language","english"), realign_boundaries=True)),
        "regexp_tokenize": ("regexp_tokenize_gpu", lambda text, pattern="\\s+", gaps=False, discard_empty=True, **kw: ported_lib.regexp_tokenize_gpu(text, pattern, gaps, discard_empty)),
        "casual_tokenize": ("casual_tokenize_gpu", lambda text, **kw: ported_lib.casual_tokenize_gpu(text, preserve_case=kw.get("preserve_case",True), reduce_len=kw.get("reduce_len",False), strip_handles=kw.get("strip_handles",False), match_phone_numbers=kw.get("match_phone_numbers",True))),
        "toktok_tokenize": ("toktok_tokenize_gpu", lambda text, **kw: ported_lib.toktok_tokenize_gpu(text)),
        "wordpunct_tokenize": ("wordpunct_tokenize_gpu", lambda text, **kw: ported_lib.wordpunct_tokenize_gpu(text)),
        "whitespace_tokenize": ("whitespace_tokenize_gpu", lambda text, **kw: ported_lib.whitespace_tokenize_gpu(text)),
        "mwe_tokenize": ("mwe_tokenize_gpu", lambda tokens, mwes, **kw: ported_lib.mwe_tokenize_gpu(tokens, mwes, separator=kw.get("separator","_"))),
        "sexpr_tokenize": ("sexpr_tokenize_gpu", lambda text, **kw: ported_lib.sexpr_tokenize_gpu(text, parens=kw.get("parens","()"), strict=kw.get("strict",True))),
        "detokenize": ("detokenize_gpu", lambda tokens, **kw: ported_lib.detokenize_gpu(tokens, convert_parentheses=kw.get("convert_parentheses",False))),
    }

    # Batch variants
    batch_map = {
        "word_tokenize": lambda texts: ported_lib.word_tokenize_batch_gpu(texts),
        "sent_tokenize": lambda texts: ported_lib.sent_tokenize_batch_gpu(texts),
    }

    total = passed = failed = 0
    for fn_key, (gpu_name, gpu_fn) in gpu_map.items():
        if fn_key not in FUNCTION_PAIRS:
            continue
        _, ported_cpu = FUNCTION_PAIRS[fn_key]
        for case in cases_for(fn_key, size="full"):
            total += 1
            try:
                cpu_res = ported_cpu(*case.args, **case.kwargs)
            except Exception as e:
                cpu_res = e
            try:
                gpu_res = gpu_fn(*case.args, **case.kwargs)
            except Exception as e:
                gpu_res = e
            if cpu_res == gpu_res or (type(cpu_res)==type(gpu_res) and str(cpu_res)==str(gpu_res)):
                passed += 1
            else:
                failed += 1
                if failed <= 5:
                    print(f"FAIL {fn_key} {case.label}: cpu={cpu_res!r:.120} gpu={gpu_res!r:.120}")

    # Batch parity: batch_gpu(texts) == [cpu(t) for t in texts]
    for fn_key, batch_gpu_fn in batch_map.items():
        if fn_key not in FUNCTION_PAIRS:
            continue
        _, ported_cpu = FUNCTION_PAIRS[fn_key]
        texts = [c.args[0] for c in cases_for(fn_key, size="smoke")[:20] if c.args and isinstance(c.args[0], str)]
        if not texts:
            continue
        total += 1
        try:
            cpu_batch = [ported_cpu(t) for t in texts]
            gpu_batch = batch_gpu_fn(texts)
            if cpu_batch == gpu_batch:
                passed += 1
            else:
                failed += 1
                print(f"FAIL batch {fn_key}: divergence")
        except Exception as e:
            failed += 1
            print(f"FAIL batch {fn_key} exception: {e}")

    print(f"\nGPU parity: {passed}/{total} passed, {failed} failed")
    return 1 if failed else 0

if __name__ == "__main__":
    raise SystemExit(main())
