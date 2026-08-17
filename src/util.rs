use regex::Regex;

pub fn string_span_tokenize(s: &str, sep: &str) -> Vec<(usize, usize)> {
    if sep.is_empty() {
        panic!("Token delimiter must not be empty");
    }
    let mut out = Vec::new();
    let mut left_byte = 0;
    let s_len_chars = s.chars().count();
    loop {
        let left_char = s[..left_byte].chars().count();
        match s[left_byte..].find(sep) {
            Some(rel) => {
                let right_byte = left_byte + rel;
                let right_char = s[..right_byte].chars().count();
                if right_char != 0 {
                    out.push((left_char, right_char));
                }
                left_byte = right_byte + sep.len();
                if left_byte > s.len() {
                    break;
                }
            }
            None => {
                if left_char != s_len_chars {
                    out.push((left_char, s_len_chars));
                }
                break;
            }
        }
        if left_byte >= s.len() {
            break;
        }
    }
    out
}

pub fn regexp_span_tokenize(s: &str, pattern: &str) -> Vec<(usize, usize)> {
    let re = Regex::new(pattern).unwrap();
    let mut out = Vec::new();
    let mut left_byte = 0usize;
    for m in re.find_iter(s) {
        let (right_byte, next_byte) = (m.start(), m.end());
        if right_byte != left_byte {
            let char_left = s[..left_byte].chars().count();
            let char_right = char_left + s[left_byte..right_byte].chars().count();
            out.push((char_left, char_right));
        }
        left_byte = next_byte;
    }
    let char_left = s[..left_byte].chars().count();
    let char_right = char_left + s[left_byte..].chars().count();
    out.push((char_left, char_right));
    out
}

pub fn spans_to_relative(spans: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut prev = 0usize;
    let mut out = Vec::new();
    for &(left, right) in spans {
        out.push((left - prev, right - left));
        prev = right;
    }
    out
}

pub struct CjkRanges;
impl CjkRanges {
    pub const RANGES: &'static [(u32, u32)] = &[
        (4352, 4607),
        (11904, 42191),
        (43072, 43135),
        (44032, 55215),
        (63744, 64255),
        (65072, 65103),
        (65381, 65500),
        (131072, 196607),
    ];
}

pub fn is_cjk(c: char) -> bool {
    let cp = c as u32;
    CjkRanges::RANGES.iter().any(|&(s, e)| cp >= s && cp <= e)
}

pub fn xml_escape(text: &str) -> String {
    let mut s = text.replace('&', "&amp;");
    s = s.replace('<', "&lt;");
    s = s.replace('>', "&gt;");
    s = s.replace('\'', "&apos;");
    s = s.replace('"', "&quot;");
    s = s.replace('|', "&#124;");
    s = s.replace('[', "&#91;");
    s = s.replace(']', "&#93;");
    s
}

pub fn xml_unescape(text: &str) -> String {
    let mut s = text.replace("&apos;", "'");
    s = s.replace("&quot;", "\"");
    s = s.replace("&#124;", "|");
    s = s.replace("&#91;", "[");
    s = s.replace("&#93;", "]");
    s = s.replace("&lt;", "<");
    s = s.replace("&gt;", ">");
    s = s.replace("&amp;", "&");
    s
}

pub fn align_tokens(tokens: &[String], sentence: &str) -> Vec<(usize, usize)> {
    let mut point = 0usize;
    let mut offsets = Vec::new();
    for tok in tokens {
        match sentence[point..].find(tok.as_str()) {
            Some(rel) => {
                let start = point + rel;
                point = start + tok.len();
                offsets.push((start, point));
            }
            None => panic!("substring \"{}\" not found in \"{}\"", tok, sentence),
        }
    }
    offsets
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn string_span_basic() {
        let s = "a b c";
        assert_eq!(string_span_tokenize(s, " "), vec![(0, 1), (2, 3), (4, 5)]);
    }
    #[test]
    fn regexp_span_basic() {
        assert_eq!(
            regexp_span_tokenize("a b  c", r"\s+"),
            vec![(0, 1), (2, 3), (5, 6)]
        );
    }
    #[test]
    fn spans_relative() {
        assert_eq!(spans_to_relative(&[(0, 4), (5, 12)]), vec![(0, 4), (1, 7)]);
    }
    #[test]
    fn cjk_check() {
        assert!(is_cjk('\u{33FE}'));
        assert!(!is_cjk('\u{FE5F}'));
    }
    #[test]
    fn xml_roundtrip() {
        let s = "a & b <c> 'd' \"e\" [f]";
        assert_eq!(xml_unescape(&xml_escape(s)), s);
    }
    #[test]
    fn align_basic() {
        let s = "hello world";
        let toks = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(align_tokens(&toks, s), vec![(0, 5), (6, 11)]);
    }
}
