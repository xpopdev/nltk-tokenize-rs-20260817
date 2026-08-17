use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct PunktTrainer {
    pub abbrev_types: HashSet<String>,
    pub collocations: HashSet<(String, String)>,
}

impl PunktTrainer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn train(&mut self, text: &str) {
        for w in text.split_whitespace() {
            if w.ends_with('.') && w.len() > 2 {
                self.abbrev_types
                    .insert(w.trim_end_matches('.').to_lowercase());
            }
        }
        let _ = &self.collocations;
        let _ = HashMap::<String, i32>::new();
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
    fn train_basic() {
        let mut t = PunktTrainer::new();
        t.train("Mr. Smith went home. Dr. Jones stayed.");
        assert!(t.abbrev_types.contains("mr"));
    }
}
