use std::collections::{HashMap, HashSet};

/// Training is intentionally a Python fallback in this port.
/// The Kiss & Strunk unsupervised learner (Dunning log-likelihood,
/// collocation/ortho counts) lives in NLTK's Python `punkt_trainer.py`
/// and is not reimplemented in Rust — use `nltk.tokenize.punkt.PunktTrainer`
/// directly for training, then export params via `params_english.rs`.
#[derive(Debug, Clone, Default)]
pub struct PunktTrainer {
    pub abbrev_types: HashSet<String>,
    pub collocations: HashSet<(String, String)>,
}

impl PunktTrainer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn train(&mut self, _text: &str) {
        // No-op: training requires the full Python trainer. See module doc.
    }
    pub fn finalize(self) -> crate::punkt::PunktParameters {
        crate::punkt::PunktParameters {
            abbrev_types: self.abbrev_types,
            collocations: self.collocations,
            sent_starters: HashSet::new(),
            ortho_context: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn train_is_noop_fallback() {
        let mut t = PunktTrainer::new();
        t.train("Mr. Smith went home. Dr. Jones stayed.");
        assert!(t.abbrev_types.is_empty(), "Rust trainer is a fallback — training is Python-side");
    }
}
