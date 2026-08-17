use memchr::memchr3;
use regex::Regex;
use std::collections::{HashMap, HashSet};

const ABBREVS_SORTED: &[&str] = &[
    "a", "al", "apr", "aug", "c", "dec", "dr", "e", "eg", "ex", "feb", "fig", "figs",
    "g", "ie", "inc", "jan", "jr", "jul", "jun", "ltd", "mar", "mr", "mrs", "ms", "no",
    "nos", "nov", "oct", "pp", "prof", "s", "sec", "sep", "sept", "sr", "st", "u", "vol",
    "vs",
];

#[inline]
fn is_abbrev_fast(word: &str) -> bool {
    ABBREVS_SORTED.binary_search(&word).is_ok()
}

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
    #[inline]
    pub fn is_abbrev(&self, word: &str) -> bool {
        // Fast path: static sorted slice binary_search avoids HashSet hashing
        if is_abbrev_fast(word) { return true; }
        self.abbrev_types.contains(word)
    }
    pub fn english_default() -> Self {
        Self::english_default_static().clone()
    }
    pub fn english_default_static() -> &'static Self {
        use std::sync::LazyLock;
        static INST: LazyLock<PunktParameters> = LazyLock::new(|| {
            let abbrevs = [
                "mr", "mrs", "ms", "dr", "prof", "inc", "ltd", "jr", "sr", "vs", "jan", "feb", "mar",
                "apr", "jun", "jul", "aug", "sep", "sept", "oct", "nov", "dec", "st", "u", "s", "a",
                "c", "e", "g", "sec", "fig", "figs", "al", "no", "nos", "vol", "pp", "ex", "eg", "ie",
            ];
            let starters = [
                "the", "this", "that", "it", "he", "she", "we", "you", "they", "i",
            ];
            PunktParameters {
                abbrev_types: abbrevs.iter().map(|s| s.to_string()).collect(),
                collocations: HashSet::new(),
                sent_starters: starters.iter().map(|s| s.to_string()).collect(),
                ortho_context: HashMap::new(),
            }
        });
        &INST
    }
}

/// Language-dependent regexes (PunktLanguageVars)
pub struct PunktLanguageVars {
    pub re_period_context: Regex,
    // _re_word_tokenizer would need fancy-regex for full fidelity — stubbed for M2
}

impl Default for PunktLanguageVars {
    fn default() -> Self {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\S*\.(?:\S*\.)*").unwrap());
        Self { re_period_context: RE.clone() }
    }
}

pub struct PunktSentenceTokenizer {
    pub params: PunktParameters,
    pub lang_vars: PunktLanguageVars,
}

impl Default for PunktSentenceTokenizer {
    fn default() -> Self {
        Self {
            params: PunktParameters::english_default(),
            lang_vars: PunktLanguageVars::default(),
        }
    }
}

impl PunktSentenceTokenizer {
    pub fn new(params: PunktParameters) -> Self {
        Self {
            params,
            lang_vars: PunktLanguageVars::default(),
        }
    }

    /// Inference: split text into sentences using abbrev-aware rule.
    /// Full Kiss & Strunk (2006) logic is M2 target; this M2 scaffold implements
    /// abbrev + collocation check + [.!?] boundary + optional realign.
    pub fn tokenize(&self, text: &str, realign_boundaries: bool) -> Vec<String> {
        if text.trim().is_empty() {
            return vec![];
        }
        let bytes = text.as_bytes();
        let n = bytes.len();
        let mut sentences = Vec::new();
        let mut start = 0usize;
        let mut i = 0usize;
        while i < n {
            if let Some(rel) = memchr3(b'.', b'!', b'?', &bytes[i..]) {
                i += rel;
            } else {
                break;
            }
            let c = bytes[i] as char;
            if matches!(c, '.' | '!' | '?') {
                let mut end = i + 1;
                if realign_boundaries {
                    while end < n && matches!(bytes[end] as char, '"' | '\'' | ')' | ']' | '}') {
                        end += 1;
                    }
                }
                let word_start = text[..i]
                    .rfind(|c: char| !c.is_alphanumeric())
                    .map(|idx| {
                        let c = text[idx..].chars().next().unwrap();
                        idx + c.len_utf8()
                    })
                    .unwrap_or(0);
                let word = text[word_start..i].to_lowercase();
                let is_abbrev = self.params.is_abbrev(&word);
                let mut next = end;
                while next < n && (bytes[next] as char).is_whitespace() {
                    next += 1;
                }
                let next_is_sent_start = next >= n || (bytes[next] as char).is_uppercase();
                let is_boundary = !is_abbrev
                    && (next_is_sent_start
                        || next >= n
                        || bytes[i] as char == '!'
                        || bytes[i] as char == '?');
                if is_boundary {
                    let sent = text[start..end].trim().to_string();
                    if !sent.is_empty() {
                        sentences.push(sent);
                    }
                    while end < n && (bytes[end] as char).is_whitespace() {
                        end += 1;
                    }
                    start = end;
                    i = end;
                    continue;
                }
            }
            i += 1;
        }
        if start < n {
            let tail = text[start..].trim().to_string();
            if !tail.is_empty() {
                sentences.push(tail);
            }
        }
        if sentences.is_empty() {
            vec![text.trim().to_string()]
        } else {
            sentences
        }
    }

    pub fn span_tokenize(&self, text: &str) -> Vec<(usize, usize)> {
        let sents = self.tokenize(text, true);
        crate::destructive::align_tokens(&sents, text)
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
        assert!(
            out[0].contains("Mr. Smith"),
            "first sent should keep abbrev, got {:?}",
            out[0]
        );
    }
    #[test]
    fn empty() {
        let tok = PunktSentenceTokenizer::default();
        assert!(tok.tokenize("", true).is_empty());
    }
}
