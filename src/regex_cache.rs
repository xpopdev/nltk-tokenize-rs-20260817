use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use regex::Regex;
static CACHE: OnceLock<RwLock<HashMap<String, Regex>>> = OnceLock::new();
pub fn cached_regex(pattern: &str) -> Regex {
    let lock = CACHE.get_or_init(|| RwLock::new(HashMap::new()));
    {
        let map = lock.read().unwrap();
        if let Some(re) = map.get(pattern) {
            return re.clone();
        }
    }
    let re = Regex::new(pattern).unwrap_or_else(|e| panic!("bad regex {pattern:?}: {e}"));
    let mut map = lock.write().unwrap();
    // double check
    if let Some(existing) = map.get(pattern) {
        return existing.clone();
    }
    map.insert(pattern.to_string(), re.clone());
    re
}
