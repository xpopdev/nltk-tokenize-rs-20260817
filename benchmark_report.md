# Benchmark report

| Function | Original (µs/call) | Ported (µs/call) | Speedup |
|---|---|---|---|
| example.add | 0.111 | 0.099 | 1.13x |
| word_tokenize | 1682.35 | 37.649 | 44.68x |
| sent_tokenize | 216.625 | 49.185 | 4.4x |
| regexp_tokenize | 13.871 | 0.601 | 23.08x |
| string_span_tokenize | 1.304 | 0.36 | 3.63x |
| regexp_span_tokenize | 11.116 | 0.322 | 34.54x |
| spans_to_relative | 0.554 | 0.371 | 1.49x |
| is_cjk | 0.849 | 0.184 | 4.63x |
| xml_escape | 1.046 | 0.275 | 3.81x |
| xml_unescape | 0.995 | 0.235 | 4.24x |
| align_tokens | 0.783 | 0.534 | 1.47x |
| casual_tokenize | 736.255 | 30.516 | 24.13x |
| toktok_tokenize | 358.076 | 55.429 | 6.46x |
| mwe_tokenize | 4.315 | 1.07 | 4.03x |
| sexpr_tokenize | 4.239 | 0.58 | 7.31x |
| detokenize | 15.896 | 0.768 | 20.71x |
| space_tokenize | 0.401 | 0.327 | 1.23x |
| tab_tokenize | 0.386 | 0.287 | 1.35x |
| char_tokenize | 0.425 | 0.328 | 1.3x |
| line_tokenize | 0.832 | 0.465 | 1.79x |
| blankline_tokenize | 13.414 | 0.388 | 34.59x |
| wordpunct_tokenize | 15.582 | 0.706 | 22.07x |
| whitespace_tokenize | 14.348 | 0.413 | 34.71x |
| nist_tokenize | 6.151 | 1.525 | 4.03x |
| nist_international_tokenize | 7.58 | 1.376 | 5.51x |
| legality_tokenize | 1.166 | 0.664 | 1.76x |
| sonority_tokenize | 4.617 | 0.591 | 7.81x |
