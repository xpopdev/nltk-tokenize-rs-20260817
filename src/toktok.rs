use std::borrow::Cow;
use std::sync::LazyLock;
use regex::Regex;

use crate::api::TokenizerI;

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
    if !text.is_ascii() {
        let mut out = String::with_capacity(text.len()+8);
        for (idx,c) in text.char_indices() {
            if c == ':' {
                let rest = &text[idx+c.len_utf8()..];
                if rest.starts_with("//") { out.push(':'); } else { out.push_str(" : "); }
            } else { out.push(c); }
        }
        return out;
    }
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 8);
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        if bytes[i] == b':' {
            let is_url = i+2 < n && bytes[i+1] == b'/' && bytes[i+2] == b'/';
            if is_url { out.push(':'); } else { out.push_str(" : "); }
        } else {
            out.push(bytes[i] as char);
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
    let mut s = if text.contains('|') { text.replace('|', " &#124; ") } else { text.to_string() };
    if s.contains('\t') { s = s.replace('\t', " "); }
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
