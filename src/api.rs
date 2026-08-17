pub trait TokenizerI {
    fn tokenize(&self, s: &str) -> Vec<String>;
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)>;
    fn tokenize_sents(&self, sents: &[String]) -> Vec<Vec<String>> {
        sents.iter().map(|s| self.tokenize(s)).collect()
    }
}
