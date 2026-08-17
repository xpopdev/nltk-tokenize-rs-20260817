use std::sync::LazyLock;

use regex::Regex;

use crate::regex_cache::cached_regex;

static RE_WS_FAST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());
static RE_WORD_FAST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\w+").unwrap());
static RE_DIGIT_FAST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+").unwrap());

pub struct RegexpTokenizer {
    pattern: String,
    gaps: bool,
    discard_empty: bool,
    regex: Option<Regex>,
}

impl RegexpTokenizer {
    pub fn new(pattern: &str, gaps: bool, discard_empty: bool) -> Self {
        Self {
            pattern: pattern.to_string(),
            gaps,
            discard_empty,
            regex: None,
        }
    }

    fn get_regex(&mut self) -> &Regex {
        if self.regex.is_none() {
            let pat = self.pattern.clone();
            self.regex = Some(cached_regex(&pat));
        }
        self.regex.as_ref().unwrap()
    }

    pub fn tokenize(&mut self, text: &str) -> Vec<String> {
        let gaps = self.gaps;
        let discard_empty = self.discard_empty;
        let re = self.get_regex() as &regex::Regex; let _ = discard_empty;
        if gaps {
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
        let gaps = self.gaps;
        let re = self.get_regex() as &regex::Regex;
        if gaps {
            let mut out = Vec::new();
            let mut left = 0usize;
            for m in re.find_iter(text) {
                let (right, next) = (m.start(), m.end());
                if right != left {
                    out.push((left, right));
                }
                left = next;
            }
            if left < text.len() {
                out.push((left, text.len()));
            }
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
    match pattern {
        r"\s+" if gaps && discard_empty => {
            return text.split_whitespace().map(|s| s.to_string()).collect();
        }
        r"\s+" if gaps => {
            let parts: Vec<String> = RE_WS_FAST.split(text).map(|s| s.to_string()).collect();
            if discard_empty {
                return parts.into_iter().filter(|s| !s.is_empty()).collect();
            }
            return parts;
        }
        r"\s+" => {
            return RE_WS_FAST.find_iter(text).map(|m| m.as_str().to_string()).collect();
        }
        r"\w+" if !gaps => {
            return RE_WORD_FAST.find_iter(text).map(|m| m.as_str().to_string()).collect();
        }
        r"\d+" if !gaps => {
            return RE_DIGIT_FAST.find_iter(text).map(|m| m.as_str().to_string()).collect();
        }
        _ => {}
    }
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
