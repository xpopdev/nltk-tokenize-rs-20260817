use crate::api::TokenizerI;

pub struct SExprTokenizer {
    pub open_paren: String,
    pub close_paren: String,
    pub strict: bool,
}

impl SExprTokenizer {
    pub fn new(parens: &str, strict: bool) -> Self {
        assert!(
            parens.chars().count() == 2,
            "parens must contain exactly two strings"
        );
        let mut chars = parens.chars();
        Self {
            open_paren: chars.next().unwrap().to_string(),
            close_paren: chars.next().unwrap().to_string(),
            strict,
        }
    }
}

impl Default for SExprTokenizer {
    fn default() -> Self {
        Self::new("()", true)
    }
}

impl TokenizerI for SExprTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let mut pos = 0usize;
        let mut depth = 0usize;
        let bytes = s.as_bytes();
        let open_b = self.open_paren.as_bytes()[0];
        let close_b = self.close_paren.as_bytes()[0];

        let mut starts: Vec<usize> = Vec::new();
        let mut i = 0usize;
        while i < bytes.len() {
            let b = bytes[i];
            if b == open_b || b == close_b {
                let paren = if b == open_b {
                    &self.open_paren
                } else {
                    &self.close_paren
                };
                if depth == 0 && b == open_b {
                    let prefix = s[pos..i]
                        .split_whitespace()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>();
                    result.extend(prefix);
                    pos = i;
                    let _ = paren;
                } else if depth == 0 && b == close_b {
                    let prefix = s[pos..i]
                        .split_whitespace()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>();
                    result.extend(prefix);
                    pos = i;
                    if self.strict {
                        panic!("Un-matched close paren at char {}", i);
                    }
                    result.push(s[i..i + 1].to_string());
                    pos = i + 1;
                    i += 1;
                    continue;
                }
                if b == open_b {
                    depth += 1;
                    starts.push(i);
                } else {
                    if depth == 0 && self.strict {
                        panic!("Un-matched close paren at char {}", i);
                    }
                    if depth > 0 {
                        depth -= 1;
                        starts.pop();
                    }
                    if depth == 0 {
                        result.push(s[pos..i + 1].to_string());
                        pos = i + 1;
                    }
                }
            }
            i += 1;
        }
        if self.strict && depth > 0 {
            panic!("Un-matched open paren at char {}", pos);
        }
        if pos < s.len() {
            let tail = &s[pos..];
            if depth > 0 {
                result.push(tail.to_string());
            } else {
                result.extend(tail.split_whitespace().map(|x| x.to_string()));
            }
        }
        result.into_iter().filter(|x| !x.is_empty()).collect()
    }

    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        let toks = TokenizerI::tokenize(self, s);
        crate::util::align_tokens(&toks, s)
    }
}

pub fn sexpr_tokenize(text: &str) -> Vec<String> {
    TokenizerI::tokenize(&SExprTokenizer::default(), text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let t = SExprTokenizer::default();
        assert_eq!(
            TokenizerI::tokenize(&t, "(a b (c d)) e f (g)"),
            vec!["(a b (c d))", "e", "f", "(g)"]
        );
    }
    #[test]
    fn strict_close() {
        let t = SExprTokenizer::new("()", false);
        let out = TokenizerI::tokenize(&t, "c) d");
        assert!(out.contains(&")".to_string()));
    }
    #[test]
    fn custom_parens() {
        let t = SExprTokenizer::new("{}", true);
        assert_eq!(TokenizerI::tokenize(&t, "{a b} c"), vec!["{a b}", "c"]);
    }
}
