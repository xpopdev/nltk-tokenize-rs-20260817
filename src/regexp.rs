use crate::util::regexp_span_tokenize as util_span;
use regex::Regex;

pub struct RegexpTokenizer {
    pattern: String,
    gaps: bool,
    discard_empty: bool,
    flags: String,
    regex: Option<Regex>,
}

impl RegexpTokenizer {
    pub fn new(pattern: &str, gaps: bool, discard_empty: bool) -> Self {
        let flags = "(?m)".to_string();
        Self {
            pattern: pattern.to_string(),
            gaps,
            discard_empty,
            flags,
            regex: None,
        }
    }

    fn get_regex(&mut self) -> &Regex {
        if self.regex.is_none() {
            let pat = self.pattern.clone();
            self.regex = Some(
                Regex::new(&pat).unwrap_or_else(|e| panic!("bad regexp {}: {}", self.pattern, e)),
            );
        }
        self.regex.as_ref().unwrap()
    }

    pub fn tokenize(&mut self, text: &str) -> Vec<String> {
        let re = self.get_regex().clone();
        if self.gaps {
            let parts: Vec<String> = re.split(text).map(|s| s.to_string()).collect();
            if self.discard_empty {
                parts.into_iter().filter(|s| !s.is_empty()).collect()
            } else {
                parts
            }
        } else {
            re.find_iter(text).map(|m| m.as_str().to_string()).collect()
        }
    }

    pub fn span_tokenize(&mut self, text: &str) -> Vec<(usize, usize)> {
        let re = self.get_regex().clone();
        if self.gaps {
            let mut out = Vec::new();
            let mut left = 0usize;
            for m in re.find_iter(text) {
                let (right, next) = (m.start(), m.end());
                if right != left && !(self.discard_empty && right == left) {
                    out.push((left, right));
                }
                left = next;
            }
            if left < text.len() || (left == text.len() && !self.discard_empty) {
                if left != text.len() || !self.discard_empty {
                    if left < text.len() {
                        out.push((left, text.len()));
                    }
                }
            }
            let _ = util_span(text, re.as_str());
            out
        } else {
            re.find_iter(text).map(|m| (m.start(), m.end())).collect()
        }
    }
}

pub struct WhitespaceTokenizer(RegexpTokenizer);
impl WhitespaceTokenizer {
    pub fn new() -> Self {
        Self(RegexpTokenizer::new(r"\s+", true, true))
    }
    pub fn tokenize(&mut self, s: &str) -> Vec<String> {
        self.0.tokenize(s)
    }
    pub fn span_tokenize(&mut self, s: &str) -> Vec<(usize, usize)> {
        self.0.span_tokenize(s)
    }
}
impl Default for WhitespaceTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BlanklineTokenizer(RegexpTokenizer);
impl BlanklineTokenizer {
    pub fn new() -> Self {
        Self(RegexpTokenizer::new(r"\s*\n\s*\n\s*", true, true))
    }
    pub fn tokenize(&mut self, s: &str) -> Vec<String> {
        self.0.tokenize(s)
    }
}
impl Default for BlanklineTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct WordPunctTokenizer(RegexpTokenizer);
impl WordPunctTokenizer {
    pub fn new() -> Self {
        Self(RegexpTokenizer::new(r"\w+|[^\w\s]+", false, true))
    }
    pub fn tokenize(&mut self, s: &str) -> Vec<String> {
        self.0.tokenize(s)
    }
}
impl Default for WordPunctTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn regexp_tokenize(text: &str, pattern: &str, gaps: bool, discard_empty: bool) -> Vec<String> {
    RegexpTokenizer::new(pattern, gaps, discard_empty).tokenize(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whitespace() {
        assert_eq!(
            WhitespaceTokenizer::new().tokenize("a  b\nc"),
            vec!["a", "b", "c"]
        );
    }
    #[test]
    fn wordpunct() {
        assert!(WordPunctTokenizer::new()
            .tokenize("Hello, world.")
            .contains(&"Hello".to_string()));
    }
    #[test]
    fn gaps_span() {
        let mut t = WhitespaceTokenizer::new();
        assert_eq!(t.span_tokenize("a b"), vec![(0, 1), (2, 3)]);
    }
}
