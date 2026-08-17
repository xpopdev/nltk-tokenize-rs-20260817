pub fn string_span_tokenize(s: &str, sep: &str) -> Vec<(usize, usize)> {
    if sep.is_empty() {
        panic!("Token delimiter must not be empty");
    }
    let mut out = Vec::new();
    let mut left = 0;
    loop {
        match s[left..].find(sep) {
            Some(rel) => {
                let right = left + rel;
                if right != 0 {
                    out.push((left, right));
                }
                left = right + sep.len();
                if left > s.len() { break; }
            }
            None => {
                if left != s.len() { out.push((left, s.len())); }
                break;
            }
        }
        if left >= s.len() { break; }
    }
    out
}
