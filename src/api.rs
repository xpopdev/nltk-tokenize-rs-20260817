use regex::Regex;

use crate::util::{regexp_span_tokenize as util_regexp_span, string_span_tokenize};

pub trait TokenizerI {
    fn tokenize(&self, s: &str) -> Vec<String>;
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)>;
    fn tokenize_sents(&self, sents: &[String]) -> Vec<Vec<String>> {
        sents.iter().map(|s| self.tokenize(s)).collect()
    }
}
