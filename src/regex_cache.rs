use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use regex::Regex;

static CACHE: OnceLock<Mutex<HashMap<String, Regex>>> = OnceLock::new();

pub fn cached_regex(pattern: &str) -> Regex {
    let mutex = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = mutex.lock().unwrap();
    if let Some(re) = map.get(pattern) {
        return re.clone();
    }
    let re = Regex::new(pattern).unwrap_or_else(|e| panic!("bad regex {pattern:?}: {e}"));
    map.insert(pattern.to_string(), re.clone());
    re
}
