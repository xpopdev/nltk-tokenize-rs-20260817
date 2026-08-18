use crate::destructive::NLTKWordTokenizer;

pub fn word_tokenize_core(text: &str, convert_parentheses: bool) -> Vec<String> {
    NLTKWordTokenizer::tokenize_core(text, convert_parentheses)
}

pub fn sent_tokenize_placeholder(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// D1: contractions C3 / C2 / END3 / END4 — manual, no regex
fn fix_contractions(s: &str) -> String {
    // fast path: no '\'' and no "''"
    if !s.contains('\'') && !s.contains("''") {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut out = String::with_capacity(n);
    let mut i = 0usize;
    while i < n {
        // END4: "''" + ' ' + punctuation [.,:)\]>};%]
        if i + 3 < n && bytes[i] == b'\'' && bytes[i + 1] == b'\'' && bytes[i + 2] == b' ' && matches!(bytes[i + 3], b'.' | b',' | b':' | b')' | b']' | b'>' | b'}' | b';' | b'%') {
            out.push_str("''");
            out.push(bytes[i + 3] as char);
            i += 4;
            continue;
        }
        // END3: \S + ' ' + "''"  -> remove space
        if i + 2 < n && bytes[i] != b' ' && bytes[i] != b'\t' && bytes[i + 1] == b' ' && bytes[i + 2] == b'\'' && i + 3 < n && bytes[i + 3] == b'\'' {
            out.push(bytes[i] as char);
            out.push_str("''");
            i += 4;
            continue;
        }
        // C3 / C2: ([^' ])\s('<cont>)\s  -> remove the space before <cont>
        // suffixes to check after the space: "'s ", "'S ", "'m ", "'M ", "'d ", "'D ", "' ",
        // "'ll ", "'LL ", "'re ", "'RE ", "'ve ", "'VE ", "n't ", "N'T "
        if bytes[i] == b' ' && i + 1 < n && bytes[i + 1] == b'\'' {
            // check suffix with trailing space
            let rest = &s[i + 1..];
            let is_c3_c2 = rest.starts_with("'s ")
                || rest.starts_with("'S ")
                || rest.starts_with("'m ")
                || rest.starts_with("'M ")
                || rest.starts_with("'d ")
                || rest.starts_with("'D ")
                || (rest.starts_with("' ") && {
                    // "' " alone is C3's last alt — ensure we don't mis-handle "''" which is handled above
                    !rest.starts_with("''")
                })
                || rest.starts_with("'ll ")
                || rest.starts_with("'LL ")
                || rest.starts_with("'re ")
                || rest.starts_with("'RE ")
                || rest.starts_with("'ve ")
                || rest.starts_with("'VE ");
            let is_nt = rest.starts_with("n't ") || rest.starts_with("N'T ");
            // also handle bare "n't " without leading "'"? pattern includes "n't" as part of C2
            // our check above with "'": rest[0]=='\'' so "n't " won't match; handle separately
            let is_cont = if is_c3_c2 {
                true
            } else if bytes[i] == b' ' && i + 1 < n {
                // check for " n't " / " N'T " (space + n't + space)
                let r2 = &s[i + 1..];
                r2.starts_with("n't ") || r2.starts_with("N'T ")
            } else {
                false
            };
            // also check "n't " case where rest is "n't " (no leading ')
            let is_cont2 = is_cont || is_nt;
            if is_cont2 || is_c3_c2 {
                // need preceding char not ' and not ' '
                if !out.is_empty() {
                    let prev = out.as_bytes()[out.len() - 1];
                    if prev != b'\'' && prev != b' ' {
                        // skip the space at i, copy from i+1 onward in next iterations
                        i += 1;
                        continue;
                    }
                }
            }
            // check " n't " variant where pattern is " n't " with space before n
            if !is_c3_c2 && (rest.starts_with("n't ") || rest.starts_with("N'T ")) && !out.is_empty() {
                let prev = out.as_bytes()[out.len() - 1];
                if prev != b'\'' && prev != b' ' {
                    i += 1;
                    continue;
                }
            }
        }
        // also handle " n't " where the space is before n, not before '
        if bytes[i] == b' ' && i + 3 < n && (s[i + 1..].starts_with("n't ") || s[i + 1..].starts_with("N'T ")) && !out.is_empty() {
            let prev = out.as_bytes()[out.len() - 1];
            if prev != b'\'' && prev != b' ' {
                i += 1;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

#[inline]
fn fix_brackets_and_punct(s: &str) -> String {
    let bytes = s.as_bytes();
    let n = bytes.len();
    if n == 0 {
        return String::new();
    }
    let mut out = String::with_capacity(n);
    let mut i = 0usize;
    while i < n {
        let b = bytes[i];
        // " ... " -> "..."  (consume surrounding spaces)
        if b == b' ' && i + 4 < n && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' && bytes[i + 3] == b'.' && bytes[i + 4] == b' ' {
            out.push_str("...");
            i += 5;
            continue;
        }
        // opening bracket + space -> drop space  ([\[({<])\s
        if matches!(b, b'[' | b'(' | b'{' | b'<') && i + 1 < n && bytes[i + 1] == b' ' {
            out.push(b as char);
            i += 2;
            continue;
        }
        // closing bracket + space + [:;,.]  -> drop space between  (must check before generic space-before-close)
        if matches!(b, b']' | b')' | b'}' | b'>') && i + 2 < n && bytes[i + 1] == b' ' && matches!(bytes[i + 2], b':' | b';' | b',' | b'.') {
            out.push(b as char);
            out.push(bytes[i + 2] as char);
            i += 3;
            continue;
        }
        // space + closing bracket -> drop space  \s([])}>])
        if b == b' ' && i + 1 < n && matches!(bytes[i + 1], b']' | b')' | b'}' | b'>') {
            i += 1;
            continue;
        }
        // space + [?!] -> drop space
        if b == b' ' && i + 1 < n && matches!(bytes[i + 1], b'?' | b'!') {
            i += 1;
            continue;
        }
        // ([#$])\s -> drop space after #$
        if matches!(b, b'#' | b'$') && i + 1 < n && bytes[i + 1] == b' ' {
            out.push(b as char);
            i += 2;
            continue;
        }
        // \s([;%]) -> drop space before %;
        if b == b' ' && i + 1 < n && matches!(bytes[i + 1], b'%' | b';') {
            i += 1;
            continue;
        }
        // \s([:,]) -> drop space before :, (keep number case; numbers have no space before anyway)
        if b == b' ' && i + 1 < n && matches!(bytes[i + 1], b',' | b':') {
            i += 1;
            continue;
        }
        // ([^'])\s'\s -> drop space before single quote when surrounded: "x ' "
        if b == b' ' && i + 1 < n && bytes[i + 1] == b'\'' && i + 2 < n && bytes[i + 2] == b' ' {
            // check char before this space is not '
            let prev_is_quote = out.as_bytes().last().copied() == Some(b'\'');
            if !prev_is_quote {
                i += 1; // skip space before '
                continue;
            }
        }
        // final period: ([^\.])\s(\.)([\]\[\)}>"']*)\s*$  -> drop space before .
        // we can handle generically: if we see " ." and the rest is only brackets/quotes/spaces till end, drop space
        if b == b' ' && i + 1 < n && bytes[i + 1] == b'.' {
            // lookahead: after '.' we may have brack/quotes then only spaces till end
            let mut j = i + 2;
            while j < n && matches!(bytes[j], b']' | b'[' | b')' | b'(' | b'}' | b'{' | b'>' | b'<' | b'"' | b'\'' | b'`') {
                j += 1;
            }
            // skip trailing spaces till end
            let mut k = j;
            while k < n && bytes[k] == b' ' {
                k += 1;
            }
            if k == n {
                // we're at end-region, check char before space is not '.'
                let prev_is_dot = out.as_bytes().last().copied() == Some(b'.');
                if !prev_is_dot {
                    i += 1; // skip space before '.'
                    continue;
                }
            }
        }
        // SQ1: ([ \([{<])\s`` -> drop space before ``
        if b == b' ' && i + 2 < n && bytes[i + 1] == b'`' && bytes[i + 2] == b'`' {
            if let Some(prev) = out.as_bytes().last().copied() {
                if prev == b' ' || matches!(prev, b'(' | b'[' | b'{' | b'<') {
                    i += 1;
                    continue;
                }
            }
        }
        // SQ2: (``)\s -> drop space after ``
        if b == b'`' && i + 1 < n && bytes[i + 1] == b'`' && i + 2 < n && bytes[i + 2] == b' ' {
            out.push_str("``");
            i += 3;
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    // handle leading/trailing edge for " ... " at boundaries that we missed (no surrounding spaces)
    // the above covers interior; boundaries are trimmed anyway
    out
}

pub fn detokenize(tokens: &[String], convert_parentheses: bool) -> String {
    if tokens.is_empty() {
        return String::new();
    }
    // build " " + join + " " without extra join allocation
    let total_len: usize = tokens.iter().map(|t| t.len()).sum::<usize>() + tokens.len() + 1;
    let mut text = String::with_capacity(total_len + 2);
    text.push(' ');
    for (idx, tok) in tokens.iter().enumerate() {
        if idx > 0 {
            text.push(' ');
        }
        text.push_str(tok);
    }
    text.push(' ');

    // D1: contractions (C3/C2/END3/END4) — only if needed
    if text.contains('\'') {
        text = fix_contractions(&text);
    }
    // '' -> "
    if text.contains("''") {
        text = text.replace("''", "\"");
    }
    text = text.trim().to_string();
    if text.contains(" -- ") {
        text = text.replace(" -- ", "--");
    }
    if convert_parentheses {
        if text.contains("-LRB-") {
            text = text.replace("-LRB-", "(");
        }
        if text.contains("-RRB-") {
            text = text.replace("-RRB-", ")");
        }
        if text.contains("-LSB-") {
            text = text.replace("-LSB-", "[");
        }
        if text.contains("-RSB-") {
            text = text.replace("-RSB-", "]");
        }
        if text.contains("-LCB-") {
            text = text.replace("-LCB-", "{");
        }
        if text.contains("-RCB-") {
            text = text.replace("-RCB-", "}");
        }
    }
    // D2+D3+D4 fused: brackets, punct, quotes spacing — single pass
    // quick guard: if no punct/bracket/quote chars, skip
    if text.contains(['[', '(', '{', '<', ']', ')', '}', '>', '!', '?', '#', '$', '%', ';', ',', ':', '.', '`', '\'']) {
        text = fix_brackets_and_punct(&text);
    }
    if text.contains("``") {
        text = text.replace("``", "\"");
    }
    text.trim().to_string()
}
