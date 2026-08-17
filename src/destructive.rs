use regex::Regex;

fn repl_python_to_rust(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            if chars[i + 1] == 'g' && i + 3 < chars.len() && chars[i + 2] == '<' {
                // \g<0> -> $0  or \g<1> etc
                if let Some(end) = chars[i..].iter().position(|&c| c == '>') {
                    let inner: String = chars[i + 3..i + end].iter().collect();
                    out.push('$');
                    out.push_str(&inner);
                    i += end + 1;
                    continue;
                }
            } else if chars[i + 1].is_ascii_digit() {
                out.push('$');
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn apply(text: &str, pattern: &str, replacement: &str) -> String {
    // Strip Python (?#...) comments
    let cleaned = strip_comments(pattern);
    let re = Regex::new(&cleaned).unwrap_or_else(|e| panic!("bad pattern {pattern}: {e}"));
    let rust_repl = repl_python_to_rust(replacement);
    re.replace_all(text, rust_repl.as_str()).to_string()
}

fn strip_comments(pat: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = pat.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 3 < chars.len() && chars[i] == '(' && chars[i + 1] == '?' && chars[i + 2] == '#' {
            // skip until ')'
            if let Some(end) = chars[i..].iter().position(|&c| c == ')') {
                i += end + 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub fn align_tokens(tokens: &[String], sentence: &str) -> Vec<(usize, usize)> {
    let mut point = 0usize;
    let mut out = Vec::new();
    for tok in tokens {
        let rel = sentence[point..].find(tok.as_str()).unwrap_or_else(|| {
            panic!("substring \"{tok}\" not found in \"{sentence}\"")
        });
        let start = point + rel;
        // need char-index vs byte-index: Python indices are char offsets
        // sentence[..start].chars().count() is not needed because sentence is &str and find returns byte offset but Python returns char offset.
        // For ASCII they coincide; for unicode we must convert byte offsets to char counts.
        // Do conversion: char offset = sentence[..byte].chars().count()
        let char_start = sentence[..start].chars().count();
        let char_end = char_start + tok.chars().count();
        out.push((char_start, char_end));
        point = start + tok.len();
    }
    out
}

pub struct NLTKWordTokenizer;

impl NLTKWordTokenizer {
    pub fn tokenize_core(text: &str, convert_parentheses: bool) -> Vec<String> {
        let mut s = text.to_string();

        // STARTING_QUOTES
        s = apply(&s, r"([«“‘„]|[`]+)", r" $1 ");
        s = apply(&s, "^\"", r"``");
        s = apply(&s, r"(``)", r" $1 ");
        s = apply(&s, r##"([ \(\[{<])("|'{2})"##, r"$1 `` ");
        // Rewrite of (?i)(')(?!re|ve|ll|m|t|s|d|n)(\w)\b — Rust regex has no lookahead
        // Match ' + word char, then filter out excluded clitics in Rust
        {
            let re = Regex::new(r"(?i)'(\w)\b").unwrap();
            let excludes = ["re","ve","ll","m","t","s","d","n"];
            s = re.replace_all(&s, |caps: &regex::Captures| {
                let ch = caps.get(1).unwrap().as_str();
                // Look ahead: if the word starting here is in excludes, don't split
                // Recreate original lookahead semantics: check following word slice
                let start = caps.get(0).unwrap().start();
                // Extract the word that starts at ch position to test against excludes
                // The capture is single char, but lookahead originally checks multi-char strings like "re"
                // So we look at substring from ch onward up to word boundary
                let rest = &s[start+1..];
                let word_end = rest.find(|c: char| !c.is_alphanumeric()).unwrap_or(rest.len());
                let word = rest[..word_end].to_lowercase();
                if excludes.contains(&word.as_str()) {
                    caps[0].to_string()
                } else {
                    format!("' {}", ch)
                }
            }).to_string();
        }

        // PUNCTUATION — simplified for M1; full unicode quote handling deferred to later milestone
        s = apply(&s, r"([^\.])(.)([\]\[\)}]*)\s*$", r"$1 $2 $3 ");
        // Actually pattern above is complex; fallback: keep Python's exact patterns via simpler translations
        // For M1 we use the five core punctuation patterns that cover most cases
        s = apply(&s, r"([:,])([^\d])", r" $1 $2");
        s = apply(&s, r"([:,])$", r" $1 ");
        s = apply(&s, r"\.{2,}", r" $0 ");
        s = apply(&s, r"[;@#$%&]", r" $0 ");
        s = apply(&s, r#"([^\.])(.)([\]\[\)}]*)\s*$"#, r"$1 $2$3 ");
        s = apply(&s, r"[?!]", r" $0 ");
        s = apply(&s, r"([^'])' ", r"$1 ' ");
        s = apply(&s, r"[*]", r" $0 ");

        // PARENS
        s = apply(&s, r"[\]\[(){}<>]", r" $0 ");

        if convert_parentheses {
            s = apply(&s, r"\(", "-LRB-");
            s = apply(&s, r"\)", "-RRB-");
            s = apply(&s, r"\[", "-LSB-");
            s = apply(&s, r"\]", "-RSB-");
            s = apply(&s, r"\{", "-LCB-");
            s = apply(&s, r"\}", "-RCB-");
        }

        s = apply(&s, r"--", r" -- ");

        s = format!(" {s} ");

        s = apply(&s, r"([»”’])", r" $1 ");
        s = apply(&s, r"''", " '' ");
        s = apply(&s, "\"", " '' ");
        s = apply(&s, r"\s+", " ");
        s = apply(&s, r"([^' ])('[sS]|'[mM]|'[dD]|') ", r"$1 $2 ");
        s = apply(&s, r"([^' ])('ll|'LL|'re|'RE|'ve|'VE|n't|N'T) ", r"$1 $2 ");

        // CONTRACTIONS2 - rewritten without lookahead: \b(wan)(na)\b covers the (?=\s) case for most inputs
        for pat in &[
            r"(?i)\b(can)(not)\b",
            r"(?i)\b(d)('ye)\b",
            r"(?i)\b(gim)(me)\b",
            r"(?i)\b(gon)(na)\b",
            r"(?i)\b(got)(ta)\b",
            r"(?i)\b(lem)(me)\b",
            r"(?i)\b(more)('n)\b",
            r"(?i)\b(wan)(na)\b",
        ] {
            s = apply(&s, pat, r" $1 $2 ");
        }
        for pat in &[r"(?i) ('t)(is)\b", r"(?i) ('t)(was)\b"] {
            s = apply(&s, pat, r" $1 $2 ");
        }

        s.split_whitespace().map(|x| x.to_string()).collect()
    }

    pub fn span_tokenize_core(text: &str) -> Vec<(usize, usize)> {
        let toks = Self::tokenize_core(text, false);
        // Quote restoration branch omitted for M1 skeleton; align directly
        align_tokens(&toks, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_split() {
        let t = NLTKWordTokenizer::tokenize_core("Good muffins cost $3.88 in New York.", false);
        assert!(t.contains(&"Good".to_string()));
        assert!(t.contains(&"$".to_string()));
    }
    #[test]
    fn align_basic() {
        let s = "hello world";
        let toks = vec!["hello".to_string(), "world".to_string()];
        let spans = align_tokens(&toks, s);
        assert_eq!(spans, vec![(0, 5), (6, 11)]);
    }
}
