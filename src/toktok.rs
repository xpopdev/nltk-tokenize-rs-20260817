use std::borrow::Cow;
use std::sync::LazyLock;
use regex::Regex;

use crate::api::TokenizerI;

static RE_PIPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\|").unwrap());
static RE_BRACKETS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([\[\](){}<>])").unwrap());
static RE_URL_PUNCT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([:/?#])").unwrap());
static RE_COMMA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*([,])\s*").unwrap());
static RE_QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(['\u{2019}`])").unwrap());
static RE_CC1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" ` ` ").unwrap());
static RE_CC2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" ' ' ").unwrap());
static RE_COMMA2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(,{2,})").unwrap());
static RE_DASH2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(-{2,})").unwrap());
static RE_DOTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\.{2,})").unwrap());
static RE_FINAL_DOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.$").unwrap());
static RE_WS2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" {2,}").unwrap());

fn apply_with_lookahead_colon(text: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ':'
            && (i + 1 >= chars.len()
                || chars[i + 1] != '/'
                || (i + 2 < chars.len() && chars[i + 2] != '/')
                || i + 2 >= chars.len())
        {
            if i + 1 < chars.len()
                && chars[i + 1] == '/'
                && i + 2 < chars.len()
                && chars[i + 2] == '/'
            {
                out.push(':');
            } else {
                out.push_str(" : ");
            }
        } else {
            out.push(chars[i]);
        }
        i += 1;
    }
    out
}

pub struct ToktokTokenizer;

impl ToktokTokenizer {
    pub fn new() -> Self { Self }
    pub fn tokenize_with_flag(&self, text: &str, return_str: bool) -> Vec<String> {
        let s = tokenize_inner(text);
        if return_str { vec![s] } else { s.split_whitespace().map(|x| x.to_string()).collect() }
    }
    pub fn tokenize_str(&self, text: &str) -> String { tokenize_inner(text) }
}

impl Default for ToktokTokenizer { fn default() -> Self { Self::new() } }

impl TokenizerI for ToktokTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        tokenize_inner(s).split_whitespace().map(|x| x.to_string()).collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        let toks = TokenizerI::tokenize(self, s);
        crate::util::align_tokens(&toks, s)
    }
}

fn tokenize_inner(text: &str) -> String {
    let mut s = RE_PIPE.replace_all(text, " &#124; ").to_string();
    if s.contains('	') { s = s.replace('	', " "); }
    if let Cow::Owned(o) = RE_BRACKETS.replace_all(&s, " $1 ") { s = o; }
    if let Cow::Owned(o) = RE_URL_PUNCT.replace_all(&s, " $1 ") { s = o; }
    s = apply_with_lookahead_colon(&s);
    if let Cow::Owned(o) = RE_COMMA.replace_all(&s, " $1 ") { s = o; }
    if let Cow::Owned(o) = RE_QUOTE.replace_all(&s, " $1 ") { s = o; }
    if let Cow::Owned(o) = RE_CC1.replace_all(&s, " `` ") { s = o; }
    if let Cow::Owned(o) = RE_CC2.replace_all(&s, " '' ") { s = o; }
    if let Cow::Owned(o) = RE_COMMA2.replace_all(&s, " $1 ") { s = o; }
    if let Cow::Owned(o) = RE_DASH2.replace_all(&s, " $1 ") { s = o; }
    if let Cow::Owned(o) = RE_DOTS.replace_all(&s, " $1 ") { s = o; }
    if s.ends_with('.') && !s.ends_with("..") {
        if let Cow::Owned(o) = RE_FINAL_DOT.replace(&s, " .") { s = o; }
    }
    if let Cow::Owned(o) = RE_WS2.replace_all(&s, " ") { s = o; }
    s.trim().to_string()
}

pub fn toktok_tokenize(text: &str) -> Vec<String> {
    crate::api::TokenizerI::tokenize(&ToktokTokenizer, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let t = ToktokTokenizer;
        let toks = crate::api::TokenizerI::tokenize(&t, "Hello, world.");
        assert!(toks.contains(&"Hello".to_string()));
        assert!(toks.contains(&",".to_string()));
    }
    #[test]
    fn str_mode() {
        let t = ToktokTokenizer;
        assert!(!t.tokenize_str("Hello world").is_empty());
    }
}
