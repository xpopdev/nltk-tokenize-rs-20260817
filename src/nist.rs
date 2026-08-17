use std::sync::LazyLock;
use regex::Regex;

static RE_SKIP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<skipped>").unwrap());
static RE_EOL_HYPHEN: LazyLock<Regex> = LazyLock::new(|| Regex::new("\u{2028}").unwrap());
static RE_PUNCT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([\{\-\~\[-\` \-\&\(\-\+\:-\@\/])").unwrap());
static RE_PERIOD_COMMA_PRE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^0-9])([\.,])").unwrap());
static RE_PERIOD_COMMA_POST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([\.,])([^0-9])").unwrap());
static RE_DASH_DIGIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([0-9])(-)").unwrap());

fn xml_unescape_local(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
     .replace("&quot;", "\"").replace("&apos;", "'")
}

fn lang_independent_sub(text: &str) -> String {
    let t = RE_SKIP.replace_all(text, "").to_string();
    let t = xml_unescape_local(&t);
    RE_EOL_HYPHEN.replace_all(&t, " ").to_string()
}

pub fn nist_tokenize(text: &str, lowercase: bool, western_lang: bool) -> Vec<String> {
    let mut t = lang_independent_sub(text);
    if western_lang {
        t = format!(" {} ", t);
        if lowercase { t = t.to_lowercase(); }
        t = RE_PUNCT.replace_all(&t, " $1 ").to_string();
        t = RE_PERIOD_COMMA_PRE.replace_all(&t, "$1 $2 ").to_string();
        t = RE_PERIOD_COMMA_POST.replace_all(&t, " $1 $2").to_string();
        t = RE_DASH_DIGIT.replace_all(&t, "$1 $2 ").to_string();
    }
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    let t = t.trim().to_string();
    if t.is_empty() { vec![] } else { t.split_whitespace().map(|s| s.to_string()).collect() }
}

pub fn nist_international_tokenize(text: &str, lowercase: bool) -> Vec<String> {
    let mut t = lang_independent_sub(text);
    t = format!(" {} ", t);
    if lowercase { t = t.to_lowercase(); }
    // Simplified international: split non-ascii runs, tokenize punctuation
    // Full perluniprops tables are corpus-dependent; this covers the common path
    // by inserting spaces around non-ascii ↔ ascii boundaries and punctuation
    let mut out = String::with_capacity(t.len() * 2);
    let mut prev_ascii: Option<bool> = None;
    for c in t.chars() {
        let is_ascii = c.is_ascii();
        if let Some(pa) = prev_ascii {
            if pa != is_ascii { out.push(' '); }
        }
        // pad punctuation/symbol chars
        if c.is_ascii_punctuation() && c != '\'' && c != '"' {
            out.push(' '); out.push(c); out.push(' ');
        } else {
            out.push(c);
        }
        prev_ascii = Some(is_ascii);
    }
    // apply western punct rules on the ascii parts
    let mut s = RE_PUNCT.replace_all(&out, " $1 ").to_string();
    s = RE_PERIOD_COMMA_PRE.replace_all(&s, "$1 $2 ").to_string();
    s = RE_PERIOD_COMMA_POST.replace_all(&s, " $1 $2").to_string();
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let s = s.trim().to_string();
    if s.is_empty() { vec![] } else { s.split_whitespace().map(|s| s.to_string()).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let toks = nist_tokenize("Good muffins cost $3.88 in New York.", false, true);
        assert!(toks.contains(&"Good".to_string()));
        assert!(toks.contains(&"$".to_string()));
    }
    #[test]
    fn lower() {
        let toks = nist_tokenize("Hello World", true, true);
        assert_eq!(toks, vec!["hello", "world"]);
    }
}
