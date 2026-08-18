# Benchmark report

| Function | Original (µs/call) | Ported (µs/call) | Speedup |
|---|---|---|---|
| example.add | 0.11 | 0.098 | 1.12x |
| word_tokenize | 1694.672 | 37.704 | 44.95x |
| sent_tokenize | 218.186 | 49.566 | 4.4x |
| regexp_tokenize | 13.651 | 0.605 | 22.58x |
| string_span_tokenize | 1.303 | 0.365 | 3.57x |
| regexp_span_tokenize | 10.91 | 0.315 | 34.68x |
| spans_to_relative | 0.563 | 0.375 | 1.5x |
| is_cjk | 0.839 | 0.175 | 4.78x |
| xml_escape | 1.047 | 0.291 | 3.6x |
| xml_unescape | 0.997 | 0.239 | 4.17x |
| align_tokens | 0.79 | 0.545 | 1.45x |
| casual_tokenize | 736.027 | 29.999 | 24.54x |
| toktok_tokenize | 365.093 | 54.201 | 6.74x |
| mwe_tokenize | 4.322 | 1.13 | 3.83x |
| sexpr_tokenize | 4.377 | 0.61 | 7.18x |
| detokenize | 16.354 | 0.778 | 21.02x |
| space_tokenize | 0.397 | 0.325 | 1.22x |
| tab_tokenize | 0.38 | 0.284 | 1.34x |
| char_tokenize | 0.417 | 0.332 | 1.26x |
| line_tokenize | 0.831 | 0.486 | 1.71x |
| blankline_tokenize | 13.254 | 0.369 | 35.89x |
| wordpunct_tokenize | 15.388 | 0.711 | 21.63x |
| whitespace_tokenize | 14.176 | 0.398 | 35.61x |
| nist_tokenize | 6.255 | 1.524 | 4.1x |
| nist_international_tokenize | 7.632 | 1.378 | 5.54x |
| legality_tokenize | 1.196 | 0.679 | 1.76x |
| sonority_tokenize | 4.735 | 0.592 | 8.0x |
| shim_word_tokenize | 41.478 | 6.72 | 6.17x |
| shim_sent_tokenize | 32.882 | 11.372 | 2.89x |
