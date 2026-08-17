use crate::api::TokenizerI;
use crate::util::{regexp_span_tokenize, string_span_tokenize};

pub struct SpaceTokenizer;
impl TokenizerI for SpaceTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        let mut out = Vec::with_capacity(s.len().saturating_add(1) / 4);
        for tok in s.split(' ') { out.push(tok.to_string()); }
        out
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        string_span_tokenize(s, " ")
    }
}

pub struct TabTokenizer;
impl TokenizerI for TabTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        s.split('\t').map(|x| x.to_string()).collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        string_span_tokenize(s, "\t")
    }
}

pub struct CharTokenizer;
impl TokenizerI for CharTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        let mut out = Vec::with_capacity(s.chars().count());
        for c in s.chars() { out.push(c.to_string()); }
        out
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut pos = 0usize;
        for c in s.chars() {
            let next = pos + c.len_utf8();
            out.push((pos, next));
            pos = next;
        }
        out
    }
}

pub struct LineTokenizer {
    pub blanklines: String,
}
impl LineTokenizer {
    pub fn new(blanklines: &str) -> Self {
        let valid = ["discard", "keep", "discard-eof"];
        if !valid.contains(&blanklines) {
            panic!("Blank lines must be one of: {}", valid.join(" "));
        }
        Self {
            blanklines: blanklines.to_string(),
        }
    }
    fn split_keep_eof(s: &str) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        let mut start = 0usize;
        for (i, c) in s.char_indices() {
            if c == '\n' {
                v.push(s[start..i].to_string());
                start = i + 1;
            }
        }
        v.push(s[start..].to_string());
        if s.ends_with('\n') {
            v.pop();
        }
        v
    }
}
impl TokenizerI for LineTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        let parts = Self::split_keep_eof(s);
        match self.blanklines.as_str() {
            "keep" => parts,
            "discard" => parts.into_iter().filter(|l| !l.trim().is_empty()).collect(),
            "discard-eof" => {
                let mut v = parts;
                if v.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
                    v.pop();
                }
                v.into_iter().filter(|l| !l.trim().is_empty()).collect()
            }
            _ => parts,
        }
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        if self.blanklines == "keep" {
            string_span_tokenize(s, "\n")
        } else {
            regexp_span_tokenize(s, r"\n(\s+\n)*")
        }
    }
}

pub fn line_tokenize(text: &str, blanklines: &str) -> Vec<String> {
    LineTokenizer::new(blanklines).tokenize(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn space() {
        assert_eq!(SpaceTokenizer.tokenize("a b c"), vec!["a", "b", "c"]);
    }
    #[test]
    fn tab() {
        assert_eq!(TabTokenizer.tokenize("a\tb"), vec!["a", "b"]);
    }
    #[test]
    fn chars() {
        assert_eq!(CharTokenizer.tokenize("ab"), vec!["a", "b"]);
    }
    #[test]
    fn line_discard() {
        assert_eq!(
            LineTokenizer::new("discard").tokenize("a\n\nb"),
            vec!["a", "b"]
        );
    }
    #[test]
    fn line_keep() {
        assert_eq!(
            LineTokenizer::new("keep").tokenize("a\n\nb"),
            vec!["a", "", "b"]
        );
    }
    #[test]
    fn line_discard_eof() {
        assert_eq!(
            LineTokenizer::new("discard-eof").tokenize("a\nb\n"),
            vec!["a", "b"]
        );
    }
}
