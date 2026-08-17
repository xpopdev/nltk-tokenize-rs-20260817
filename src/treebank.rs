use crate::destructive::NLTKWordTokenizer;

pub fn word_tokenize_core(text: &str, convert_parentheses: bool) -> Vec<String> {
    NLTKWordTokenizer::tokenize_core(text, convert_parentheses)
}

pub fn sent_tokenize_placeholder(text: &str) -> Vec<String> {
    // M2 will replace with Punkt inference; for M1 just split on double-newline as stub
    text.split("\n\n").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}
