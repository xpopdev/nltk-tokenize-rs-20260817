# Benchmark report

| Function | Original (µs/call) | Ported (µs/call) | Speedup |
|---|---|---|---|
| example.add | 0.112 | 0.099 | 1.13x |
| word_tokenize | 1680.501 | 37.379 | 44.96x |
| sent_tokenize | 215.205 | 50.222 | 4.29x |
| regexp_tokenize | 13.675 | 0.62 | 22.04x |
| string_span_tokenize | 1.314 | 0.359 | 3.66x |
| regexp_span_tokenize | 10.775 | 0.315 | 34.22x |
| spans_to_relative | 0.562 | 0.362 | 1.55x |
| is_cjk | 0.849 | 0.186 | 4.57x |
| xml_escape | 1.046 | 0.276 | 3.79x |
| xml_unescape | 1.006 | 0.235 | 4.28x |
| align_tokens | 0.792 | 0.509 | 1.56x |
| casual_tokenize | 741.3 | 30.226 | 24.53x |
| toktok_tokenize | 362.034 | 53.46 | 6.77x |
| mwe_tokenize | 4.293 | 1.049 | 4.09x |
| sexpr_tokenize | 4.399 | 0.589 | 7.47x |
| detokenize | 16.39 | 0.767 | 21.36x |
| space_tokenize | 0.4 | 0.424 | 0.94x |
| tab_tokenize | 0.388 | 0.363 | 1.07x |
| char_tokenize | 0.423 | 0.402 | 1.05x |
| line_tokenize | 0.833 | 0.473 | 1.76x |
| blankline_tokenize | 13.239 | 0.388 | 34.16x |
| wordpunct_tokenize | 15.359 | 0.737 | 20.83x |
| whitespace_tokenize | 14.15 | 0.403 | 35.11x |
| nist_tokenize | 6.203 | 1.516 | 4.09x |
| nist_international_tokenize | 7.599 | 1.356 | 5.6x |
| legality_tokenize | 1.205 | 0.673 | 1.79x |
| sonority_tokenize | 4.692 | 0.599 | 7.83x |
