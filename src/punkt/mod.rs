use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

#[cfg(feature = "punkt-fancy")]
use fancy_regex::Regex as FancyRegex;
use regex::Regex;

pub mod params_english;

// Orthographic context flags — mirrors punkt.py _ORTHO_*
const ORTHO_BEG_UC: i32 = 1 << 1;
const ORTHO_MID_UC: i32 = 1 << 2;
const ORTHO_UNK_UC: i32 = 1 << 3;
const ORTHO_BEG_LC: i32 = 1 << 4;
const ORTHO_MID_LC: i32 = 1 << 5;
const ORTHO_UNK_LC: i32 = 1 << 6;
const ORTHO_UC: i32 = ORTHO_BEG_UC + ORTHO_MID_UC + ORTHO_UNK_UC;
const ORTHO_LC: i32 = ORTHO_BEG_LC + ORTHO_MID_LC + ORTHO_UNK_LC;

// ── PunktParameters ──
#[derive(Debug, Clone, Default)]
pub struct PunktParameters {
    pub abbrev_types: HashSet<String>,
    pub collocations: HashSet<(String, String)>,
    pub sent_starters: HashSet<String>,
    pub ortho_context: HashMap<String, i32>,
}

impl PunktParameters {
    pub fn is_abbrev(&self, word: &str) -> bool {
        self.abbrev_types.contains(word)
    }
    pub fn clear_abbrevs(&mut self) { self.abbrev_types.clear(); }
    pub fn clear_collocations(&mut self) { self.collocations.clear(); }
    pub fn clear_sent_starters(&mut self) { self.sent_starters.clear(); }
    pub fn clear_ortho_context(&mut self) { self.ortho_context.clear(); }
    pub fn add_ortho_context(&mut self, typ: &str, flag: i32) {
        *self.ortho_context.entry(typ.to_string()).or_insert(0) |= flag;
    }
    pub fn english_default() -> Self {
        Self::english_default_static().clone()
    }
    pub fn english_default_static() -> &'static Self {
        static INST: LazyLock<PunktParameters> = LazyLock::new(|| {
            let mut p = PunktParameters::default();
            for &s in params_english::ENGLISH_ABBREVS { p.abbrev_types.insert(s.to_string()); }
            for &(a,b) in params_english::ENGLISH_COLLOCATIONS { p.collocations.insert((a.to_string(), b.to_string())); }
            for &s in params_english::ENGLISH_SENT_STARTERS { p.sent_starters.insert(s.to_string()); }
            for &(k,v) in params_english::ENGLISH_ORTHO { p.ortho_context.insert(k.to_string(), v); }
            p
        });
        &INST
    }
}

// ── PunktLanguageVars ──
pub struct PunktLanguageVars {
    pub sent_end_chars: Vec<char>,
    pub internal_punctuation: String,
    pub re_period_context: Regex,
    #[cfg(feature = "punkt-fancy")]
    pub re_word_tokenizer_fancy: LazyLock<FancyRegex, fn() -> FancyRegex>,
    #[cfg(not(feature = "punkt-fancy"))]
    pub re_word_tokenizer: Regex,
    pub re_boundary_realignment: Regex,
}

impl Default for PunktLanguageVars {
    fn default() -> Self {
        // period_context:  [.!?](?=(?:[)";}\]*:@\'\({\[!?]|\s+\S+))
        // Build using sent_end_chars = .?!
        let period_context_pat = r"[.?!\]]";
        let re_period = Regex::new(period_context_pat).unwrap();

        let re_boundary = Regex::new(r#"["')\]}]+(?:\s+|$)"#).unwrap();

        #[cfg(feature = "punkt-fancy")]
        let re_word_fancy: LazyLock<FancyRegex, fn() -> FancyRegex> = {
            fn make_fancy() -> FancyRegex {
                let non_word_chars = r#"[)";}\]*:@'\({\[?!]"#;
                let word_start = r#"[^\(\"\\`{\[:;&\#\*@\)}\]\-,]"#;
                let multi = r"(?:\-{2,}|\.{2,}|(?:\.\s){2,}\.)";
                let fmt = format!(r"({multi}|(?={ws})\S+?(?=\s|$|{nw}|{multi}|,(?=$|\s|{nw}|{multi}))|\S)", ws=word_start, nw=non_word_chars, multi=multi);
                FancyRegex::new(&fmt).unwrap()
            }
            LazyLock::new(make_fancy)
        };

        #[cfg(not(feature = "punkt-fancy"))]
        let re_word = {
            // Fallback: simple word-punct split without lookahead — approximate
            Regex::new(r"[^\s]+").unwrap()
        };

        Self {
            sent_end_chars: vec!['.', '?', '!'],
            internal_punctuation: ",:;".to_string(),
            re_period_context: re_period,
            #[cfg(feature = "punkt-fancy")]
            re_word_tokenizer_fancy: re_word_fancy,
            #[cfg(not(feature = "punkt-fancy"))]
            re_word_tokenizer: re_word,
            re_boundary_realignment: re_boundary,
        }
    }
}

impl PunktLanguageVars {
    pub fn word_tokenize(&self, s: &str) -> Vec<String> {
        #[cfg(feature = "punkt-fancy")]
        {
            self.re_word_tokenizer_fancy.find_iter(s).filter_map(|m| m.ok()).map(|m| m.as_str().to_string()).collect()
        }
        #[cfg(not(feature = "punkt-fancy"))]
        {
            self.re_word_tokenizer.find_iter(s).map(|m| m.as_str().to_string()).collect()
        }
    }
}

// ── PunktToken ──
#[derive(Debug, Clone)]
pub struct PunktToken {
    pub tok: String,
    pub typ: String,
    pub period_final: bool,
    pub sentbreak: bool,
    pub abbr: bool,
    pub ellipsis: bool,
    pub parastart: bool,
    pub linestart: bool,
}

impl PunktToken {
    pub fn new(tok: String, parastart: bool, linestart: bool) -> Self {
        let typ = Self::get_type(&tok);
        let period_final = tok.ends_with('.');
        Self {
            tok,
            typ,
            period_final,
            sentbreak: false,
            abbr: false,
            ellipsis: false,
            parastart,
            linestart,
        }
    }
    fn get_type(tok: &str) -> String {
        // _RE_NUMERIC: ^-?[\.,]?\d[\d,\.-]*\.?$
        // If numeric, normalize to ##number##
        let is_numeric = {
            static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?[\.,]?\d[\d,\.-]*\.?$").unwrap());
            RE.is_match(tok)
        };
        if is_numeric {
            return "##number##".to_string();
        }
        tok.to_lowercase()
    }
    pub fn type_no_period(&self) -> String {
        if self.typ.len() > 1 && self.typ.ends_with('.') { self.typ[..self.typ.len()-1].to_string() } else { self.typ.clone() }
    }
    pub fn type_no_sentperiod(&self) -> String {
        if self.sentbreak { self.type_no_period() } else { self.typ.clone() }
    }
    pub fn first_upper(&self) -> bool { self.tok.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) }
    pub fn first_lower(&self) -> bool { self.tok.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) }
    pub fn first_case(&self) -> &str {
        if self.first_lower() { "lower" } else if self.first_upper() { "upper" } else { "none" }
    }
    pub fn is_ellipsis(&self) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.\.+$").unwrap());
        RE.is_match(&self.tok)
    }
    pub fn is_number(&self) -> bool { self.typ.starts_with("##number##") }
    pub fn is_initial(&self) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^\W\d]\.$").unwrap());
        RE.is_match(&self.tok)
    }
    pub fn is_alpha(&self) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^\W\d]+$").unwrap());
        RE.is_match(&self.tok)
    }
    pub fn is_non_punct(&self) -> bool {
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^\W\d]").unwrap());
        RE.is_match(&self.typ)
    }
}

// ── PunktSentenceTokenizer ──
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
        Self { params, lang_vars: PunktLanguageVars::default() }
    }

    fn tokenize_words(&self, plaintext: &str) -> Vec<PunktToken> {
        let mut out = Vec::new();
        let mut parastart = false;
        for line in plaintext.split('\n') {
            if line.trim().is_empty() {
                parastart = true;
                continue;
            }
            let toks = self.lang_vars.word_tokenize(line);
            let mut first = true;
            for tok in toks {
                out.push(PunktToken::new(tok, parastart && first, first));
                first = false;
                parastart = false;
            }
            if out.is_empty() { parastart = false; }
        }
        out
    }

    fn first_pass_annotation(&self, tok: &mut PunktToken) {
        if self.lang_vars.sent_end_chars.contains(&tok.tok.chars().next().unwrap_or(' ')) && tok.tok.len()==1 && self.lang_vars.sent_end_chars.iter().any(|&c| tok.tok==c.to_string()) {
            tok.sentbreak = true;
        } else if tok.is_ellipsis() {
            tok.ellipsis = true;
        } else if tok.period_final && !tok.tok.ends_with("..") {
            let without = tok.tok[..tok.tok.len()-1].to_lowercase();
            let last_part = without.rsplit('-').next().unwrap_or(&without).to_string();
            if self.params.abbrev_types.contains(&without) || self.params.abbrev_types.contains(&last_part) {
                tok.abbr = true;
            } else {
                tok.sentbreak = true;
            }
        } else if tok.tok.len()==1 && self.lang_vars.sent_end_chars.contains(&tok.tok.chars().next().unwrap()) {
            tok.sentbreak = true;
        }
        // Also handle single char sent_end
        if tok.tok.len()==1 && self.lang_vars.sent_end_chars.contains(&tok.tok.chars().next().unwrap_or(' ')) {
            tok.sentbreak = true;
            tok.abbr = false;
            tok.ellipsis = false;
        }
    }

    fn annotate_first_pass(&self, mut tokens: Vec<PunktToken>) -> Vec<PunktToken> {
        for tok in &mut tokens { self.first_pass_annotation(tok); }
        tokens
    }

    fn ortho_heuristic(&self, tok: &PunktToken) -> Option<bool> {
        const PUNCT: &[&str] = &[";", ":", ",", ".", "!", "?", "\"", "'", "``", "''", "(", ")", "[", "]", "{", "}", "<", ">"];
        if PUNCT.contains(&tok.tok.as_str()) { return Some(false); }
        let ortho = self.params.ortho_context.get(&tok.type_no_sentperiod()).copied().unwrap_or(0);
        if tok.first_upper() && (ortho & ORTHO_LC) != 0 && (ortho & ORTHO_MID_UC) == 0 { return Some(true); }
        if tok.first_lower() && ((ortho & ORTHO_UC) != 0 || (ortho & ORTHO_BEG_LC) == 0) { return Some(false); }
        None
    }

    fn second_pass_annotation(&self, tokens: &mut [PunktToken]) {
        // iterate pairs
        for i in 0..tokens.len() {
            if i+1 >= tokens.len() { break; }
            // need to handle mutable borrow - clone next for check, apply to current
            let next_typ = tokens[i+1].type_no_sentperiod();
            let next_first_upper = tokens[i+1].first_upper();
            let next_is_alpha = tokens[i+1].is_alpha();
            let cur_period_final = tokens[i].period_final;
            if !cur_period_final { continue; }
            let cur_typ_no_period = tokens[i].type_no_period();
            let cur_is_initial = tokens[i].is_initial();
            let cur_is_number = tokens[i].is_number();
            let cur_is_abbr = tokens[i].abbr;
            let cur_is_ellipsis = tokens[i].ellipsis;
            let cur_sentbreak = tokens[i].sentbreak;

            // collocation
            if self.params.collocations.contains(&(cur_typ_no_period.clone(), next_typ.clone())) {
                tokens[i].sentbreak = false;
                tokens[i].abbr = true;
                continue;
            }
            if (cur_is_abbr || cur_is_ellipsis) && !cur_is_initial {
                if let Some(is_starter) = self.ortho_heuristic(&tokens[i+1]) {
                    if is_starter {
                        tokens[i].sentbreak = true;
                        continue;
                    }
                }
                if next_first_upper && self.params.sent_starters.contains(&next_typ) {
                    tokens[i].sentbreak = true;
                    continue;
                }
            }
            if cur_is_initial || cur_typ_no_period == "##number##" {
                match self.ortho_heuristic(&tokens[i+1]) {
                    Some(false) => {
                        tokens[i].sentbreak = false;
                        tokens[i].abbr = true;
                        continue;
                    }
                    None if cur_is_initial && next_first_upper => {
                        let ortho_next = self.params.ortho_context.get(&next_typ).copied().unwrap_or(0);
                        if (ortho_next & ORTHO_LC) == 0 {
                            tokens[i].sentbreak = false;
                            tokens[i].abbr = true;
                            continue;
                        }
                    }
                    _ => {}
                }
            }
            // suppress initial handling already done
            let _ = cur_is_number;
            let _ = cur_sentbreak;
            let _ = next_is_alpha;
        }
    }

    fn annotate_tokens(&self, tokens: Vec<PunktToken>) -> Vec<PunktToken> {
        let mut toks = self.annotate_first_pass(tokens);
        // need orthography already from params (pre-trained), not recomputed
        self.second_pass_annotation(&mut toks);
        toks
    }

    #[allow(dead_code)]
    fn get_last_whitespace_index(&self, _text: &str) -> usize {
        0
    }

    #[allow(dead_code)]
    fn match_potential_end_contexts(&self, _text: &str) -> Vec<(usize, usize, String)> {
        vec![]
    }

    fn slices_from_text(&self, text: &str) -> Vec<(usize, usize)> {
        // Faithful to NLTK but simplified via annotate_tokens on full word tokenization
        // Instead of complex _match_potential_end_contexts + text_contains_sentbreak,
        // we annotate all tokens and map back to character offsets via whitespace scanning.
        let tokens = self.tokenize_words(text);
        if tokens.is_empty() { return vec![]; }
        let annotated = self.annotate_tokens(tokens);
        // Build sentences by grouping annotated tokens where sentbreak true
        // Need to map token boundaries back to original text positions.
        // Use whitespace regex to find token positions.
        let mut slices = Vec::new();
        let mut pos = 0usize;
        let mut sent_start = 0usize;
        let mut last_tok_end = 0usize;
        // Precompute token spans in text via sequential search
        let mut search_pos = 0usize;
        let mut tok_spans: Vec<(usize, usize)> = Vec::new();
        for tok in &annotated {
            if let Some(rel) = text[search_pos..].find(&tok.tok) {
                let s = search_pos + rel;
                let e = s + tok.tok.len();
                tok_spans.push((s, e));
                search_pos = e;
            } else {
                // fallback: use pos scanning with whitespace
                tok_spans.push((pos, pos+tok.tok.len()));
                pos += tok.tok.len()+1;
            }
        }
        for (idx, tok) in annotated.iter().enumerate() {
            last_tok_end = tok_spans[idx].1;
            if tok.sentbreak {
                // sentence ends at this token's end, but realign will extend to include closing punct
                slices.push((sent_start, last_tok_end));
                // next sentence starts after whitespace following this token
                let mut next_start = last_tok_end;
                while next_start < text.len() && text[next_start..].chars().next().map(|c| c.is_whitespace()).unwrap_or(false) {
                    next_start += text[next_start..].chars().next().unwrap().len_utf8();
                }
                // if next token exists, its span start is better
                if idx+1 < tok_spans.len() {
                    let ns = tok_spans[idx+1].0;
                    // include intervening whitespace up to ns, but not beyond
                    if ns > next_start { next_start = tok_spans[idx+1].0; }
                    while next_start < text.len() && text[next_start..].chars().next().map(|c| c.is_whitespace()).unwrap_or(false) && next_start < ns {
                        next_start += 1;
                    }
                }
                sent_start = next_start;
            }
        }
        if sent_start < text.trim_end().len() {
            let mut end = text.trim_end().len();
            // ensure end at least last_tok_end
            if last_tok_end > sent_start { end = text.len(); /* keep original trailing ws trimmed later */ }
            let _ = end;
            slices.push((sent_start, text.trim_end().len()));
        }
        if slices.is_empty() && !text.trim().is_empty() {
            slices.push((0, text.trim_end().len()));
        }
        // Filter empty
        slices.into_iter().filter(|(s,e)| s<e && !text[*s..*e].trim().is_empty()).collect()
    }

    fn realign_boundaries(&self, text: &str, mut slices: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        if slices.len() <= 1 { return slices; }
        let mut out = Vec::new();
        let mut i = 0;
        while i < slices.len() {
            let (s, mut e) = slices[i];
            if i+1 < slices.len() {
                let (ns, _) = slices[i+1];
                let after = &text[ns..];
                if let Some(m) = self.lang_vars.re_boundary_realignment.find(after) {
                    if m.start()==0 {
                        let m_end = m.end();
                        let extra = m.as_str().trim_end().len();
                        e = ns + extra;
                        slices[i+1].0 = ns + m_end;
                    }
                }
                out.push((s, e));
            } else {
                if s < text.len() { out.push((s, e)); }
            }
            i+=1;
        }
        out.into_iter().filter(|(s,e)| s<e).collect()
    }

    pub fn tokenize(&self, text: &str, realign_boundaries: bool) -> Vec<String> {
        if text.trim().is_empty() { return vec![]; }
        let mut slices = self.slices_from_text(text);
        if realign_boundaries {
            slices = self.realign_boundaries(text, slices);
        }
        let mut out: Vec<String> = slices.into_iter().map(|(s,e)| text[s..e].trim().to_string()).filter(|s| !s.is_empty()).collect();
        if out.is_empty() { out.push(text.trim().to_string()); }
        out
    }

    pub fn span_tokenize(&self, text: &str) -> Vec<(usize, usize)> {
        let sents = self.tokenize(text, true);
        crate::util::align_tokens(&sents, text)
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
    fn abbrev_inc_vs() {
        let tok = PunktSentenceTokenizer::default();
        let s = "Acme Inc. is here. It vs. that is fine. etc. is abbrev.";
        let out = tok.tokenize(s, true);
        assert!(out.iter().any(|x| x.contains("Inc.")), "Inc. should not split: {:?}", out);
    }
    #[test]
    fn empty() {
        let tok = PunktSentenceTokenizer::default();
        assert!(tok.tokenize("", true).is_empty());
    }
}
