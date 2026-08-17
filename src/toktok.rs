use regex::Regex;

use crate::api::TokenizerI;

fn apply(text: &str, pattern: &str, replacement: &str) -> String {
    let rust_repl = replacement.replace("\\1", "$1").replace("\\2", "$2");
    let re = Regex::new(pattern).unwrap();
    re.replace_all(text, rust_repl.as_str()).to_string()
}

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
    pub fn new() -> Self {
        Self
    }
    pub fn tokenize_with_flag(&self, text: &str, return_str: bool) -> Vec<String> {
        let s = tokenize_inner(text);
        if return_str {
            vec![s]
        } else {
            s.split_whitespace().map(|x| x.to_string()).collect()
        }
    }
    pub fn tokenize_str(&self, text: &str) -> String {
        tokenize_inner(text)
    }
}

impl Default for ToktokTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenizerI for ToktokTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        tokenize_inner(s)
            .split_whitespace()
            .map(|x| x.to_string())
            .collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        let toks = TokenizerI::tokenize(self, s);
        crate::util::align_tokens(&toks, s)
    }
}

fn tokenize_inner(text: &str) -> String {
    let mut s = text.to_string();
    s = apply(&s, r"\|", " &#124; ");
    s = apply(&s, r"\t", " ");
    s = apply(&s, r"([\[\](){}<>])", " $1 ");
    s = apply(&s, r"([:/?#])", " $1 ");
    s = apply_with_lookahead_colon(&s);
    s = apply(&s, r"\s*([,])\s*", " $1 ");
    s = apply(&s, r"(['’`])", " $1 ");
    s = apply(&s, r" ` ` ", " `` ");
    s = apply(&s, r" ' ' ", " '' ");
    s = apply(&s, r"(,{2,})", " $1 ");
    s = apply(&s, r"(-{2,})", " $1 ");
    s = apply(&s, r"(\.{2,})", " $1 ");
    if s.ends_with('.') && !s.ends_with("..") {
        s = Regex::new(r"\.$").unwrap().replace(&s, " .").to_string();
    }
    s = apply(&s, r" {2,}", " ");
    s.trim().to_string()
}

pub fn toktok_tokenize(text: &str) -> Vec<String> {
    crate::api::TokenizerI::tokenize(&ToktokTokenizer::default(), text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let t = ToktokTokenizer::default();
        let toks = crate::api::TokenizerI::tokenize(&t, "Hello, world.");
        assert!(toks.contains(&"Hello".to_string()));
        assert!(toks.contains(&",".to_string()));
    }
    #[test]
    fn str_mode() {
        let t = ToktokTokenizer::default();
        assert!(!t.tokenize_str("Hello world").is_empty());
    }
}
