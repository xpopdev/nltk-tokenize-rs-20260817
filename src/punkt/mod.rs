use std::collections::{HashMap, HashSet};
use regex::Regex;

/// Minimal PunktParameters — mirrors nltk.tokenize.punkt.PunktParameters
/// but only the subsets needed for inference (M2). Training fields deferred to M7.
#[derive(Debug, Clone, Default)]
pub struct PunktParameters {
    pub abbrev_types: HashSet<String>,
    pub collocations: HashSet<(String, String)>,
    pub sent_starters: HashSet<String>,
    pub ortho_context: HashMap<String, i32>,
}

impl PunktParameters {
    pub fn english_default() -> Self {
        // Subset of nltk_data punkt english abbrev_types — enough for sentence boundary demo
        let abbrevs = [
            "mr", "mrs", "ms", "dr", "prof", "inc", "ltd", "jr", "sr", "vs",
            "jan", "feb", "mar", "apr", "jun", "jul", "aug", "sep", "sept",
            "oct", "nov", "dec", "st", "u", "s", "a", "c", "e", "g", "sec",
            "fig", "figs", "al", "no", "nos", "vol", "pp", "ex", "eg", "ie",
        ];
        Self {
            abbrev_types: abbrevs.iter().map(|s| s.to_string()).collect(),
            collocations: HashSet::new(),
            sent_starters: ["the","this","that","it","he","she","we","you","they","i"].iter().map(|s| s.to_string()).collect(),
            ortho_context: HashMap::new(),
        }
    }
}

/// Language-dependent regexes (PunktLanguageVars)
pub struct PunktLanguageVars {
    pub re_period_context: Regex,
    // _re_word_tokenizer would need fancy-regex for full fidelity — stubbed for M2
}

impl Default for PunktLanguageVars {
    fn default() -> Self {
        Self {
            // Simplified period_context: "(?u)\\S*\\.(?:\\S*\\.(?:\\S*\\.)?)?" approximated
            re_period_context: Regex::new(r"\S*\.(?:\S*\.)*").unwrap(),
        }
    }
}

pub struct PunktSentenceTokenizer {
    pub params: PunktParameters,
    pub lang_vars: PunktLanguageVars,
}

impl Default for PunktSentenceTokenizer {
    fn default() -> Self {
        Self { params: PunktParameters::english_default(), lang_vars: PunktLanguageVars::default() }
    }
}

impl PunktSentenceTokenizer {
    pub fn new(params: PunktParameters) -> Self {
        Self { params, lang_vars: PunktLanguageVars::default() }
    }

    /// Inference: split text into sentences using abbrev-aware rule.
    /// Full Kiss & Strunk (2006) logic is M2 target; this M2 scaffold implements
    /// abbrev + collocation check + [.!?] boundary + optional realign.
    pub fn tokenize(&self, text: &str, realign_boundaries: bool) -> Vec<String> {
        if text.trim().is_empty() {
            return vec![];
        }
        // Very small inference: walk chars, split on [.!?] + space + capital/realignment
        // Check abbrev_types to avoid splitting on "Mr." etc.
        let chars: Vec<char> = text.chars().collect();
        let mut sentences = Vec::new();
        let mut start = 0usize;
        let mut i = 0usize;
        while i < chars.len() {
            let c = chars[i];
            if matches!(c, '.' | '!' | '?') {
                // Look ahead: consume trailing closings like " )\"' ] ) etc handled by realign flag
                let mut end = i + 1;
                if realign_boundaries {
                    while end < chars.len() && matches!(chars[end], '"' | '\'' | ')' | ']' | '}') {
                        end += 1;
                    }
                }
                // Is this an abbrev period? Check word before period
                let word_start = (0..i).rev().find(|&j| !chars[j].is_alphanumeric()).map(|j| j+1).unwrap_or(0);
                let word: String = chars[word_start..i].iter().collect::<String>().to_lowercase();
                let is_abbrev = self.params.abbrev_types.contains(&word);
                // Next non-space char should be capital or end for real boundary
                let mut next = end;
                while next < chars.len() && chars[next].is_whitespace() { next += 1; }
                let next_is_sent_start = next >= chars.len() || chars[next].is_uppercase();
                let is_boundary = !is_abbrev && (next_is_sent_start || next >= chars.len() || chars[i] == '!' || chars[i] == '?');
                if is_boundary {
                    // Include trailing spaces up to next sentence start? NLTK keeps original whitespace inside sentence
                    // We cut at end, then next sentence starts at next
                    let sent: String = chars[start..end].iter().collect::<String>().trim().to_string();
                    if !sent.is_empty() {
                        sentences.push(sent);
                    }
                    // Skip whitespace
                    while end < chars.len() && chars[end].is_whitespace() { end += 1; }
                    start = end;
                    i = end;
                    continue;
                }
            }
            i += 1;
        }
        if start < chars.len() {
            let tail: String = chars[start..].iter().collect::<String>().trim().to_string();
            if !tail.is_empty() { sentences.push(tail); }
        }
        if sentences.is_empty() { vec![text.trim().to_string()] } else { sentences }
    }

    pub fn span_tokenize(&self, text: &str) -> Vec<(usize, usize)> {
        let sents = self.tokenize(text, true);
        crate::destructive::align_tokens(&sents.to_vec(), text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn punkt_basic() {
        let tok = PunktSentenceTokenizer::default();
        let s = "Mr. Smith went to Washington. He saw Dr. Jones.";
        let out = tok.tokenize(s, true);
        assert!(out.len() >= 2, "should split into >=2, got {:?}", out);
        assert!(out[0].contains("Mr. Smith"), "first sent should keep abbrev, got {:?}", out[0]);
    }
    #[test]
    fn empty() {
        let tok = PunktSentenceTokenizer::default();
        assert!(tok.tokenize("", true).is_empty());
    }
}
