use std::sync::LazyLock;
use regex::Regex;

static RE_Q1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([«“‘„]|[`]+)").unwrap());
static RE_Q2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^""#).unwrap());
static RE_Q3: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(``)").unwrap());
static RE_Q4: LazyLock<Regex> = LazyLock::new(|| Regex::new(r##"([ \(\[{<])("|'{2})"##).unwrap());
static RE_CLITIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)'(\w)\b").unwrap());
static RE_P1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"([^\.])(\.)([\]\[\)}>"'»”’]*)\s*$"#).unwrap());
static RE_P2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([:,])([^\d])").unwrap());
static RE_P3: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([:,])$").unwrap());
static RE_P4: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.{2,}").unwrap());
static RE_P5: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[;@#$%&]").unwrap());
static RE_P6: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"([^\.])(\.)([\]\[\)}>"']*)\s*$"#).unwrap());
static RE_P7: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[?!]").unwrap());
static RE_P8: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^'])' ").unwrap());
static RE_P9: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[*]").unwrap());
static RE_PARENS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\]\[(){}<>]").unwrap());
static RE_LRB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\(").unwrap());
static RE_RRB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\)").unwrap());
static RE_LSB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[").unwrap());
static RE_RSB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\]").unwrap());
static RE_LCB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{").unwrap());
static RE_RCB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\}").unwrap());
static RE_DASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"--").unwrap());
static RE_RQUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([»”’])").unwrap());
static RE_DQ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"''").unwrap());
static RE_Q: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"""#).unwrap());
static RE_WS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static RE_C1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^' ])('[sS]|'[mM]|'[dD]|') ").unwrap());
static RE_C2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^' ])('ll|'LL|'re|'RE|'ve|'VE|n't|N'T) ").unwrap());
static RE_CONT_CAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(can)(not)\b").unwrap());
static RE_CONT_DYE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(d)('ye)\b").unwrap());
static RE_CONT_GIM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(gim)(me)\b").unwrap());
static RE_CONT_GON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(gon)(na)\b").unwrap());
static RE_CONT_GOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(got)(ta)\b").unwrap());
static RE_CONT_LEM: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(lem)(me)\b").unwrap());
static RE_CONT_MORE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(more)('n)\b").unwrap());
static RE_CONT_WAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\b(wan)(na)\b").unwrap());
static RE_TIS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i) ('t)(is)\b").unwrap());
static RE_TWAS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i) ('t)(was)\b").unwrap());

pub fn align_tokens(tokens: &[String], sentence: &str) -> Vec<(usize, usize)> {
    let mut point_byte = 0usize;
    let mut point_char = 0usize;
    let mut out = Vec::new();
    for tok in tokens {
        let rel = sentence[point_byte..].find(tok.as_str())
            .unwrap_or_else(|| panic!("substring \"{tok}\" not found in \"{sentence}\""));
        let start_byte = point_byte + rel;
        let skipped_chars = sentence[point_byte..start_byte].chars().count();
        point_char += skipped_chars;
        let char_start = point_char;
        let char_end = char_start + tok.chars().count();
        out.push((char_start, char_end));
        point_byte = start_byte + tok.len();
        point_char = char_end;
    }
    out
}

pub struct NLTKWordTokenizer;

impl NLTKWordTokenizer {
    pub fn tokenize_core(text: &str, convert_parentheses: bool) -> Vec<String> {
        let mut s = text.to_string();

        s = RE_Q1.replace_all(&s, " $1 ").to_string();
        s = RE_Q2.replace(&s, "``").to_string();
        s = RE_Q3.replace_all(&s, " $1 ").to_string();
        s = RE_Q4.replace_all(&s, "$1 `` ").to_string();
        {
            let excludes = ["re", "ve", "ll", "m", "t", "s", "d", "n"];
            s = RE_CLITIC
                .replace_all(&s, |caps: &regex::Captures| {
                    let ch = caps.get(1).unwrap().as_str();
                    let start = caps.get(0).unwrap().start();
                    let rest = &s[start + 1..];
                    let word_end = rest.find(|c: char| !c.is_alphanumeric()).unwrap_or(rest.len());
                    let word = rest[..word_end].to_lowercase();
                    if excludes.contains(&word.as_str()) {
                        caps[0].to_string()
                    } else {
                        format!("' {}", ch)
                    }
                })
                .to_string();
        }

        s = RE_P1.replace_all(&s, "$1 $2 $3 ").to_string();
        s = RE_P2.replace_all(&s, " $1 $2").to_string();
        s = RE_P3.replace_all(&s, " $1 ").to_string();
        s = RE_P4.replace_all(&s, " $0 ").to_string();
        s = RE_P5.replace_all(&s, " $0 ").to_string();
        s = RE_P6.replace_all(&s, "$1 $2$3 ").to_string();
        s = RE_P7.replace_all(&s, " $0 ").to_string();
        s = RE_P8.replace_all(&s, "$1 ' ").to_string();
        s = RE_P9.replace_all(&s, " $0 ").to_string();

        s = RE_PARENS.replace_all(&s, " $0 ").to_string();

        if convert_parentheses {
            s = RE_LRB.replace_all(&s, "-LRB-").to_string();
            s = RE_RRB.replace_all(&s, "-RRB-").to_string();
            s = RE_LSB.replace_all(&s, "-LSB-").to_string();
            s = RE_RSB.replace_all(&s, "-RSB-").to_string();
            s = RE_LCB.replace_all(&s, "-LCB-").to_string();
            s = RE_RCB.replace_all(&s, "-RCB-").to_string();
        }

        s = RE_DASH.replace_all(&s, " -- ").to_string();

        s.reserve(2); s.insert(0, ' '); s.push(' ');

        s = RE_RQUOTE.replace_all(&s, " $1 ").to_string();
        s = RE_DQ.replace_all(&s, " '' ").to_string();
        s = RE_Q.replace_all(&s, " '' ").to_string();
        s = RE_WS.replace_all(&s, " ").to_string();
        s = RE_C1.replace_all(&s, "$1 $2 ").to_string();
        s = RE_C2.replace_all(&s, "$1 $2 ").to_string();

        s = RE_CONT_CAN.replace_all(&s, " $1 $2 ").to_string();
        s = RE_CONT_DYE.replace_all(&s, " $1 $2 ").to_string();
        s = RE_CONT_GIM.replace_all(&s, " $1 $2 ").to_string();
        s = RE_CONT_GON.replace_all(&s, " $1 $2 ").to_string();
        s = RE_CONT_GOT.replace_all(&s, " $1 $2 ").to_string();
        s = RE_CONT_LEM.replace_all(&s, " $1 $2 ").to_string();
        s = RE_CONT_MORE.replace_all(&s, " $1 $2 ").to_string();
        s = RE_CONT_WAN.replace_all(&s, " $1 $2 ").to_string();
        s = RE_TIS.replace_all(&s, " $1 $2 ").to_string();
        s = RE_TWAS.replace_all(&s, " $1 $2 ").to_string();

        s.split_whitespace().map(|x| x.to_string()).collect()
    }

    pub fn span_tokenize_core(text: &str) -> Vec<(usize, usize)> {
        let toks = Self::tokenize_core(text, false);
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
