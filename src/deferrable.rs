use std::collections::{HashMap, HashSet};

use crate::api::TokenizerI;

// ── Legality Principle ──
#[derive(Clone)]
pub struct LegalityPrincipleTokenizer {
    vowels: String,
    vowel_arr: [bool; 256],
    legal_onsets: HashSet<String>,
}

impl LegalityPrincipleTokenizer {
    pub fn new(source_words: Vec<String>, vowels: &str) -> Self {
        let mut vowel_arr = [false; 256];
        for c in vowels.chars() {
            let lo = c.to_ascii_lowercase() as usize;
            if lo < 256 { vowel_arr[lo] = true; }
            let up = c.to_ascii_uppercase() as usize;
            if up < 256 { vowel_arr[up] = true; }
        }
        let mut tok = Self {
            vowels: vowels.to_string(),
            vowel_arr,
            legal_onsets: HashSet::new(),
        };
        tok.legal_onsets = tok.find_legal_onsets(&source_words);
        tok
    }

    fn find_legal_onsets(&self, words: &[String]) -> HashSet<String> {
        let mut counter: HashMap<String, usize> = HashMap::new();
        for w in words {
            let onset = self.onset(w);
            *counter.entry(onset).or_insert(0) += 1;
        }
        let total = words.len().max(1);
        let threshold = 0.001;
        counter
            .into_iter()
            .filter(|(k, c)| !k.is_empty() && (*c as f64 / total as f64) > threshold)
            .map(|(k, _)| k)
            .collect()
    }

    fn onset(&self, word: &str) -> String {
        let mut onset = String::new();
        for c in word.chars() {
            let idx = c as usize;
            let is_vowel = if idx < 256 { self.vowel_arr[idx] } else { self.vowels.contains(c.to_ascii_lowercase()) };
            if is_vowel {
                break;
            }
            onset.push(c);
        }
        onset.to_lowercase()
    }

    pub fn tokenize_word(&self, word: &str) -> Vec<String> {
        if word.is_empty() {
            return vec!["".to_string()];
        }
        if word.chars().all(|c| !c.is_alphabetic()) {
            return vec![word.to_string()];
        }
        // Mirror NLTK's LegalitySyllableTokenizer.tokenize: iterate reversed,
        // building syllables backwards then reverse at the end.
        let chars: Vec<char> = word.chars().collect();
        let is_vowel = |c: char| {
            let idx = c as usize;
            if idx < 256 { self.vowel_arr[idx] } else { self.vowels.contains(c.to_ascii_lowercase()) }
        };
        let mut syllables: Vec<String> = Vec::new();
        let mut syllable = String::new();
        let mut current_onset = String::new();
        let mut vowel = false;
        let mut onset = false;
        for &ch in chars.iter().rev() {
            let ch_lower = ch.to_ascii_lowercase();
            if !vowel {
                syllable.push(ch);
                vowel = is_vowel(ch);
            } else {
                let candidate = format!("{}{}", ch_lower, current_onset.chars().rev().collect::<String>());
                if self.legal_onsets.contains(&candidate) {
                    syllable.push(ch);
                    current_onset.push(ch_lower);
                    onset = true;
                } else if is_vowel(ch) && !onset {
                    syllable.push(ch);
                    current_onset.push(ch_lower);
                } else {
                    syllables.push(syllable.chars().rev().collect());
                    syllable = ch.to_string();
                    current_onset = String::new();
                    vowel = is_vowel(ch);
                    onset = false;
                }
            }
        }
        syllables.push(syllable.chars().rev().collect());
        syllables.reverse();
        syllables.retain(|s| !s.is_empty());
        if syllables.is_empty() { vec![word.to_string()] } else { syllables }
    }
}

impl Default for LegalityPrincipleTokenizer {
    fn default() -> Self {
        Self::new(vec![], "aeiouy")
    }
}

impl TokenizerI for LegalityPrincipleTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        if s.is_empty() { return vec!["".to_string()]; }
        s.split_whitespace().flat_map(|w| self.tokenize_word(w)).collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        crate::util::regexp_span_tokenize(s, r"\s+")
    }
}

// ── Sonority Sequencing ──
pub struct SonoritySequencingTokenizer {
    #[allow(dead_code)]
    hierarchy: Vec<String>,
    // fast array rank for ascii: -10 = unknown, 0 = vowel tier, 1,2,3...
    rank_arr: [i8; 256],
    vowels: String,
}

impl SonoritySequencingTokenizer {
    pub fn new() -> Self {
        Self::with_hierarchy(vec![
            "aeiouy".to_string(),
            "lmnrw".to_string(),
            "zvsf".to_string(),
            "bcdgtkpqxhj".to_string(),
        ])
    }

    pub fn with_hierarchy(hierarchy: Vec<String>) -> Self {
        let vowels = hierarchy.first().cloned().unwrap_or_default();
        let mut rank_arr = [-10i8; 256];
        for (i, level) in hierarchy.iter().enumerate() {
            for c in level.chars() {
                let lo = c as usize;
                if lo < 256 { rank_arr[lo] = i as i8; }
                let up = c.to_ascii_uppercase() as usize;
                if up < 256 { rank_arr[up] = i as i8; }
            }
        }
        Self { hierarchy, rank_arr, vowels }
    }

    #[inline]
    fn sonority(&self, c: char) -> i32 {
        let idx = c as usize;
        if idx < 256 { -(self.rank_arr[idx] as i32) } else { -10 }
    }

    pub fn tokenize_word(&self, word: &str) -> Vec<String> {
        if word.is_empty() { return vec!["".to_string()]; }
        if word.chars().all(|c| !c.is_alphabetic()) { return vec![word.to_string()]; }
        let chars: Vec<char> = word.chars().collect();
        if chars.len() <= 3 { return vec![word.to_string()]; }
        // SSP: syllable breaks before sonority troughs
        let sonorities: Vec<i32> = chars.iter().map(|&c| self.sonority(c)).collect();
        let mut breaks = Vec::new();
        for i in 1..chars.len() - 1 {
            // trough: sonority[i] < sonority[i-1] && sonority[i] <= sonority[i+1]
            // and not inside vowel cluster
            let is_vowel = |c: char| self.vowels.contains(c.to_ascii_lowercase());
            if sonorities[i] < sonorities[i - 1] && sonorities[i] <= sonorities[i + 1] && !is_vowel(chars[i]) {
                    breaks.push(i);
                }
        }
        if breaks.is_empty() { return vec![word.to_string()]; }
        let mut out = Vec::new();
        let mut start = 0usize;
        for b in breaks {
            let syl: String = chars[start..b].iter().collect();
            if !syl.is_empty() { out.push(syl); }
            start = b;
        }
        let tail: String = chars[start..].iter().collect();
        if !tail.is_empty() { out.push(tail); }
        out
    }
}

impl Default for SonoritySequencingTokenizer {
    fn default() -> Self { Self::new() }
}

impl TokenizerI for SonoritySequencingTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        if s.is_empty() { return vec!["".to_string()]; }
        s.split_whitespace().flat_map(|w| self.tokenize_word(w)).collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        crate::util::regexp_span_tokenize(s, r"\s+")
    }
}

// ── TextTiling ──
pub struct TextTilingTokenizer {
    pub w: usize,
    pub k: usize,
}

impl TextTilingTokenizer {
    pub fn new(w: usize, k: usize) -> Self { Self { w, k } }
    pub fn segment(&self, s: &str) -> Vec<String> {
        if s.trim().is_empty() { return vec![]; }
        // Fast path: if text is short or has clear paragraph breaks, use them
        let paras: Vec<&str> = s.split("\n\n").map(|x| x.trim()).filter(|x| !x.is_empty()).collect();
        if paras.len() > 1 && s.contains("\n\n") {
            return paras.into_iter().map(|x| x.to_string()).collect();
        }
        // Block comparison: split into token windows, compute lexical cohesion
        let tokens: Vec<&str> = s.split_whitespace().collect();
        if tokens.len() <= self.w * 2 {
            return vec![s.trim().to_string()];
        }
        // Simplified approximation: sliding window cosine over raw term
        // frequencies without TF-IDF weighting or NLTK's full depth-valley
        // smoothing. Suitable for short texts; not a full TextTiling port.
        let vocab: HashSet<&str> = tokens.iter().copied().collect();
        let vocab_list: Vec<&str> = vocab.into_iter().collect();
        let vocab_idx: HashMap<&str, usize> = vocab_list.iter().enumerate().map(|(i, &w)| (w, i)).collect();
        let vsize = vocab_list.len();
        // Score each gap
        let mut scores = Vec::new();
        for gap in self.w..tokens.len() - self.w {
            let left = &tokens[gap - self.w..gap];
            let right = &tokens[gap..gap + self.w];
            let mut lv = vec![0f32; vsize];
            let mut rv = vec![0f32; vsize];
            for &tok in left { if let Some(&idx) = vocab_idx.get(tok) { lv[idx] += 1.0; } }
            for &tok in right { if let Some(&idx) = vocab_idx.get(tok) { rv[idx] += 1.0; } }
            let dot: f32 = lv.iter().zip(rv.iter()).map(|(a,b)| a*b).sum();
            let ln: f32 = lv.iter().map(|x| x*x).sum::<f32>().sqrt();
            let rn: f32 = rv.iter().map(|x| x*x).sum::<f32>().sqrt();
            let sim = if ln > 0.0 && rn > 0.0 { dot / (ln * rn) } else { 0.0 };
            scores.push((gap, sim));
        }
        if scores.is_empty() { return vec![s.trim().to_string()]; }
        // Depth scores: valleys in similarity are boundaries
        let sims: Vec<f32> = scores.iter().map(|(_, v)| *v).collect();
        let mean_sim = sims.iter().sum::<f32>() / sims.len() as f32;
        let mut depth_scores: Vec<(usize, f32)> = Vec::new();
        for i in 1..sims.len()-1 {
            let left_peak = sims[..i].iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let right_peak = sims[i+1..].iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let depth = (left_peak - sims[i]) + (right_peak - sims[i]);
            if depth > 0.0 && sims[i] < mean_sim {
                depth_scores.push((scores[i].0, depth));
            }
        }
        depth_scores.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());
        let mut cuts: Vec<usize> = depth_scores.into_iter().take(5).map(|(g,_)| g).collect();
        cuts.sort_unstable();
        // Filter cuts that are too close
        let mut filtered = Vec::new();
        for c in cuts {
            if filtered.last().map(|&last| c - last >= self.w / 2).unwrap_or(true) {
                filtered.push(c);
            }
        }
        if filtered.is_empty() {
            vec![s.trim().to_string()]
        } else {
            let mut segments = Vec::new();
            let mut start = 0usize;
            for &cut in &filtered {
                let seg = tokens[start..cut].join(" ");
                if !seg.trim().is_empty() { segments.push(seg); }
                start = cut;
            }
            let tail = tokens[start..].join(" ");
            if !tail.trim().is_empty() { segments.push(tail); }
            if segments.is_empty() { vec![s.trim().to_string()] } else { segments }
        }
    }
}

impl Default for TextTilingTokenizer {
    fn default() -> Self { Self::new(20, 10) }
}

impl TokenizerI for TextTilingTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> { self.segment(s) }
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
        let t = LegalityPrincipleTokenizer::new(vec!["hello".to_string(), "world".to_string()], "aeiouy");
        let syl = t.tokenize_word("wonderful");
        assert!(!syl.is_empty());
    }
    #[test]
    fn sonority_basic() {
        let t = SonoritySequencingTokenizer::default();
        let syl = t.tokenize_word("justification");
        assert!(!syl.is_empty());
    }
    #[test]
    fn texttiling_basic() {
        let t = TextTilingTokenizer::default();
        assert!(!TokenizerI::tokenize(&t, "para one\n\npara two").is_empty());
    }
}
