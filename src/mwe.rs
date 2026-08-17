use std::collections::HashMap;

use crate::api::TokenizerI;

#[derive(Default)]
struct TrieNode {
    children: HashMap<String, TrieNode>,
    is_end: bool,
}

pub struct MWETokenizer {
    root: TrieNode,
    separator: String,
}

impl MWETokenizer {
    pub fn new(mwes: Vec<Vec<String>>, separator: &str) -> Self {
        let mut t = Self {
            root: TrieNode::default(),
            separator: separator.to_string(),
        };
        for mwe in mwes {
            t.add_mwe(mwe);
        }
        t
    }

    pub fn add_mwe(&mut self, mwe: Vec<String>) {
        let mut node = &mut self.root;
        for tok in mwe {
            node = node.children.entry(tok).or_default();
        }
        node.is_end = true;
    }

    pub fn tokenize(&self, tokens: &[String]) -> Vec<String> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < tokens.len() {
            let mut node = &self.root;
            let mut longest: Option<usize> = None;
            let mut j = i;
            while j < tokens.len() {
                if let Some(child) = node.children.get(&tokens[j]) {
                    node = child;
                    if node.is_end {
                        longest = Some(j);
                    }
                    j += 1;
                } else {
                    break;
                }
            }
            if let Some(end) = longest {
                let merged = tokens[i..=end].join(&self.separator);
                out.push(merged);
                i = end + 1;
            } else {
                out.push(tokens[i].clone());
                i += 1;
            }
        }
        out
    }
}

impl TokenizerI for MWETokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        self.tokenize(
            &s.split_whitespace()
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
        )
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
    fn mwe_basic() {
        let tok = MWETokenizer::new(
            vec![
                vec!["a".to_string(), "little".to_string()],
                vec!["a".to_string(), "lot".to_string()],
            ],
            "_",
        );
        let inp: Vec<String> = "a little or a lot"
            .split_whitespace()
            .map(|x| x.to_string())
            .collect();
        let out = tok.tokenize(&inp);
        assert_eq!(out, vec!["a_little", "or", "a_lot"]);
    }
    #[test]
    fn no_match() {
        let tok = MWETokenizer::new(vec![], "_");
        let inp = vec!["hello".to_string()];
        assert_eq!(tok.tokenize(&inp), inp);
    }
}
