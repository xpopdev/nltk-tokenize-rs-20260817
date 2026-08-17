use crate::api::TokenizerI;

pub struct LegalityPrincipleTokenizer;

impl LegalityPrincipleTokenizer {
    pub fn new() -> Self {
        Self
    }
    pub fn tokenize_word(&self, word: &str) -> Vec<String> {
        if word.is_empty() {
            return vec![];
        }
        vec![word.to_string()]
    }
}
impl Default for LegalityPrincipleTokenizer {
    fn default() -> Self {
        Self::new()
    }
}
impl TokenizerI for LegalityPrincipleTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        s.split_whitespace().map(|x| x.to_string()).collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        crate::util::regexp_span_tokenize(s, r"\s+")
    }
}

pub struct SonoritySequencingTokenizer {
    pub hierarchy: Vec<String>,
}
impl SonoritySequencingTokenizer {
    pub fn new() -> Self {
        Self { hierarchy: vec![] }
    }
}
impl Default for SonoritySequencingTokenizer {
    fn default() -> Self {
        Self::new()
    }
}
impl TokenizerI for SonoritySequencingTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        s.split_whitespace().map(|x| x.to_string()).collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        crate::util::regexp_span_tokenize(s, r"\s+")
    }
}

pub struct TextTilingTokenizer {
    pub w: usize,
    pub k: usize,
}
impl TextTilingTokenizer {
    pub fn new(w: usize, k: usize) -> Self {
        Self { w, k }
    }
}
impl Default for TextTilingTokenizer {
    fn default() -> Self {
        Self::new(20, 10)
    }
}
impl TokenizerI for TextTilingTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        if s.trim().is_empty() {
            return vec![];
        }
        s.split("\n\n")
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        let toks = self.tokenize(s);
        crate::util::align_tokens(&toks, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legality_basic() {
        assert_eq!(
            TokenizerI::tokenize(&LegalityPrincipleTokenizer, "hello world"),
            vec!["hello", "world"]
        );
    }
    #[test]
    fn sonority_basic() {
        assert_eq!(
            TokenizerI::tokenize(&SonoritySequencingTokenizer::default(), "a b"),
            vec!["a", "b"]
        );
    }
    #[test]
    fn texttiling_basic() {
        let t = TextTilingTokenizer::default();
        assert!(!TokenizerI::tokenize(&t, "para one\n\npara two").is_empty());
    }
}
