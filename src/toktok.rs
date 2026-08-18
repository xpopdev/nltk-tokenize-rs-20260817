use std::borrow::Cow;
use std::sync::LazyLock;
use regex::Regex;

use crate::api::TokenizerI;

static RE_NON_BREAKING: LazyLock<Regex> = LazyLock::new(|| Regex::new("\u{00A0}").unwrap());
static RE_FUNKY_1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"([،;؛¿!""\])}»›”؟¡%٪°±©®।॥…])"#).unwrap());
static RE_FUNKY_2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([({\[ˮ“‘„‚«‹「『])").unwrap());
static RE_DASHES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([–—])").unwrap());
static RE_PIPE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\|").unwrap());
static RE_OPEN_PUNCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([{\u{0f3a}\u{0f3c}\u{169b}\u{201a}\u{201e}\u{2045}\u{207d}\u{208d}\u{2329}\u{2768}\u{276a}\u{276c}\u{276e}\u{2770}\u{2772}\u{2774}\u{27c5}\u{27e6}\u{27e8}\u{27ea}\u{27ec}\u{27ee}\u{2983}\u{2985}\u{2987}\u{2989}\u{298b}\u{298d}\u{298f}\u{2991}\u{2993}\u{2995}\u{2997}\u{29d8}\u{29da}\u{29fc}\u{2e22}\u{2e24}\u{2e26}\u{2e28}\u{3008}\u{300a}\u{300c}\u{300e}\u{3010}\u{3014}\u{3016}\u{3018}\u{301a}\u{301d}\u{fd3e}\u{fe17}\u{fe35}\u{fe37}\u{fe39}\u{fe3b}\u{fe3d}\u{fe3f}\u{fe41}\u{fe43}\u{fe47}\u{fe59}\u{fe5b}\u{fe5d}\u{ff08}\u{ff3b}\u{ff5b}\u{ff5f}\u{ff62}])").unwrap()
});
static RE_CLOSE_PUNCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"([\]\)}\u{0f3b}\u{0f3d}\u{169c}\u{2046}\u{207e}\u{208e}\u{232a}\u{2769}\u{276b}\u{276d}\u{276f}\u{2771}\u{2773}\u{2775}\u{27c6}\u{27e7}\u{27e9}\u{27eb}\u{27ed}\u{27ef}\u{2984}\u{2986}\u{2988}\u{298a}\u{298c}\u{298e}\u{2990}\u{2992}\u{2994}\u{2996}\u{2998}\u{29d9}\u{29db}\u{29fd}\u{2e23}\u{2e25}\u{2e27}\u{2e29}\u{3009}\u{300b}\u{300d}\u{300f}\u{3011}\u{3015}\u{3017}\u{3019}\u{301b}\u{301e}\u{301f}\u{fd3f}\u{fe18}\u{fe36}\u{fe38}\u{fe3a}\u{fe3c}\u{fe3e}\u{fe40}\u{fe42}\u{fe44}\u{fe48}\u{fe5a}\u{fe5c}\u{fe5e}\u{ff09}\u{ff3d}\u{ff5d}\u{ff60}\u{ff63}])").unwrap()
});
static RE_CURRENCY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"([$",
        "\u{00a2}\u{00a3}\u{00a4}\u{00a5}\u{058f}\u{060b}\u{09f2}\u{09f3}\u{09fb}",
        "\u{0af1}\u{0bf9}\u{0e3f}\u{17db}\u{20a0}\u{20a1}\u{20a2}\u{20a3}",
        "\u{20a4}\u{20a5}\u{20a6}\u{20a7}\u{20a8}\u{20a9}\u{20aa}\u{20ab}",
        "\u{20ac}\u{20ad}\u{20ae}\u{20af}\u{20b0}\u{20b1}\u{20b2}\u{20b3}",
        "\u{20b4}\u{20b5}\u{20b6}\u{20b7}\u{20b8}\u{20b9}\u{20ba}\u{a838}",
        "\u{fdfc}\u{fe69}\u{ff04}\u{ffe0}\u{ffe1}\u{ffe5}\u{ffe6}",
        r"])"
    )).unwrap()
});
static RE_QUOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(['’`])").unwrap());
static RE_STUPID_1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" ` ` ").unwrap());
static RE_STUPID_2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" ' ' ").unwrap());
static RE_MULTI_COMMA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(,{2,})").unwrap());
static RE_MULTI_DASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(-{2,})").unwrap());
static RE_MULTI_DOTS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\.{2,})").unwrap());
static RE_ONE_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" {2,}").unwrap());

pub struct ToktokTokenizer;

impl ToktokTokenizer {
    pub fn new() -> Self { Self }
    pub fn tokenize_with_flag(&self, text: &str, return_str: bool) -> Vec<String> {
        let s = tokenize_inner(text);
        if return_str { vec![s] } else { s.split_whitespace().map(|x| x.to_string()).collect() }
    }
    pub fn tokenize_str(&self, text: &str) -> String { tokenize_inner(text) }
}

impl Default for ToktokTokenizer { fn default() -> Self { Self::new() } }

impl TokenizerI for ToktokTokenizer {
    fn tokenize(&self, s: &str) -> Vec<String> {
        tokenize_inner(s).split_whitespace().map(|x| x.to_string()).collect()
    }
    fn span_tokenize(&self, s: &str) -> Vec<(usize, usize)> {
        let toks = TokenizerI::tokenize(self, s);
        crate::util::align_tokens(&toks, s)
    }
}

// Manual look-around implementations (regex crate has no look-around)
fn expand_colon(s: &str) -> String {
    if !s.contains(':') { return s.to_string(); }
    let mut out = String::with_capacity(s.len() + 8);
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        if bytes[i] == b':' {
            let is_url = i + 2 < n && bytes[i + 1] == b'/' && bytes[i + 2] == b'/';
            if is_url { out.push(':'); } else { out.push_str(" : "); }
        } else {
            out.push(bytes[i] as char);
        }
        i += 1;
    }
    out
}

fn expand_question(s: &str) -> String {
    if !s.contains('?') { return s.to_string(); }
    let bytes = s.as_bytes();
    let n = bytes.len();
    let mut out = String::with_capacity(s.len() + 8);
    let mut i = 0;
    while i < n {
        if bytes[i] == b'?' {
            let next_is_space_or_end = i + 1 >= n || bytes[i + 1] == b' ' || bytes[i + 1] == b'\t' || bytes[i + 1] == b'\n' || bytes[i + 1] == b'\r';
            // ?(?!\S) means ? not followed by non-space -> ? at end or before space
            if next_is_space_or_end { out.push_str(" ? "); } else { out.push('?'); }
        } else {
            out.push(bytes[i] as char);
        }
        i += 1;
    }
    out
}

fn expand_comma_not_in_num(s: &str) -> String {
    if !s.contains(',') && !s.contains('،') { return s.to_string(); }
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len() + 8);
    for i in 0..n {
        let c = chars[i];
        if c == ',' || c == '،' {
            let prev_is_comma = i > 0 && (chars[i - 1] == ',' || chars[i - 1] == '،');
            let next_is_comma_or_digit = i + 1 < n && (chars[i + 1] == ',' || chars[i + 1] == '،' || chars[i + 1].is_ascii_digit());
            if !prev_is_comma && !next_is_comma_or_digit {
                out.push(' ');
                out.push(c);
                out.push(' ');
            } else {
                out.push(c);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn handle_final_period(s: &str) -> String {
    // FINAL_PERIOD_1: (?<!\.)\.$ -> " ."
    // FINAL_PERIOD_2: (?<!\.)\.\s*(["'’»›”]) *$ -> " . \1"
    if !s.contains('.') { return s.to_string(); }
    let trimmed = s.trim_end();
    // Check for FINAL_PERIOD_2 first (period + optional quotes at very end)
    // Pattern: not preceded by ., then . then optional spaces then quote char then spaces at end
    let quotes = ['"', '\'', '’', '»', '›', '”'];
    // Find if string ends with period optionally followed by quotes/spaces
    // Work on trimmed string
    if trimmed.ends_with('.') && !trimmed.ends_with("..") {
        // ends with single . -> FINAL_PERIOD_1
        let mut out = trimmed.to_string();
        out.pop(); // remove .
        out.push_str(" .");
        // preserve trailing spaces from original?
        let trailing = &s[trimmed.len()..];
        out.push_str(trailing);
        return out;
    }
    // Check for FINAL_PERIOD_2: \.\s*(["'’»›”]) *$ on trimmed
    // Need to find .\s*quote\s* at end where . not preceded by .
    for &q in &quotes {
        // Check if trimmed ends with quote (possibly with spaces before)
        let without_trailing_spaces = trimmed.trim_end();
        if without_trailing_spaces.ends_with(q) {
            // Find the . before the quote
            let before_quote = without_trailing_spaces[..without_trailing_spaces.len() - q.len_utf8()].trim_end();
            if before_quote.ends_with('.') && !before_quote.ends_with("..") {
                // Check char before . is not .
                let dot_pos = before_quote.len() - 1;
                if dot_pos == 0 || before_quote.as_bytes()[dot_pos - 1] != b'.' {
                    // Replace . + spaces + quote with " . quote"
                    let prefix = &before_quote[..dot_pos];
                    let mut out = prefix.to_string();
                    out.push_str(" . ");
                    out.push(q);
                    let trailing = &s[trimmed.len()..];
                    out.push_str(trailing);
                    return out;
                }
            }
        }
    }
    s.to_string()
}

fn tokenize_inner(text: &str) -> String {
    let mut s = text.to_string();

    if s.contains('\u{00A0}') {
        if let Cow::Owned(o) = RE_NON_BREAKING.replace_all(&s, " ") { s = o; }
    }
    if let Cow::Owned(o) = RE_FUNKY_1.replace_all(&s, " $1 ") { s = o; }
    if let Cow::Owned(o) = RE_FUNKY_2.replace_all(&s, " $1 ") { s = o; }
    // URL colon and question via manual expansion (look-around)
    if s.contains(':') {
        let tmp = expand_colon(&s);
        if tmp != s { s = tmp; }
    }
    if s.contains('?') {
        let tmp = expand_question(&s);
        if tmp != s { s = tmp; }
    }
    if s.contains("& ") {
        s = s.replace("& ", "&amp; ");
    }
    if s.contains('\t') {
        s = s.replace('\t', " &#9; ");
    }
    if s.contains('|') {
        if let Cow::Owned(o) = RE_PIPE.replace_all(&s, " &#124; ") { s = o; }
    }
    if let Cow::Owned(o) = RE_OPEN_PUNCT.replace_all(&s, "$1 ") { s = o; }
    if let Cow::Owned(o) = RE_CLOSE_PUNCT.replace_all(&s, " $1 ") { s = o; }
    if s.contains(",,") {
        if let Cow::Owned(o) = RE_MULTI_COMMA.replace_all(&s, " $1 ") { s = o; }
    }
    if s.contains(',') || s.contains('،') {
        let tmp = expand_comma_not_in_num(&s);
        if tmp != s { s = tmp; }
    }
    if s.contains('\'') || s.contains('’') || s.contains('`') {
        if let Cow::Owned(o) = RE_QUOTE.replace_all(&s, " $1 ") { s = o; }
    }
    if s.contains(" ` ` ") {
        if let Cow::Owned(o) = RE_STUPID_1.replace_all(&s, " `` ") { s = o; }
    }
    if s.contains(" ' ' ") {
        if let Cow::Owned(o) = RE_STUPID_2.replace_all(&s, " '' ") { s = o; }
    }
    if let Cow::Owned(o) = RE_CURRENCY.replace_all(&s, "$1 ") { s = o; }
    if s.contains('–') || s.contains('—') {
        if let Cow::Owned(o) = RE_DASHES.replace_all(&s, " $1 ") { s = o; }
    }
    if s.contains("--") {
        if let Cow::Owned(o) = RE_MULTI_DASH.replace_all(&s, " $1 ") { s = o; }
    }
    if s.contains("..") {
        if let Cow::Owned(o) = RE_MULTI_DOTS.replace_all(&s, " $1 ") { s = o; }
    }
    if s.contains('.') {
        let tmp = handle_final_period(&s);
        if tmp != s { s = tmp; }
    }
    if s.contains("  ") {
        if let Cow::Owned(o) = RE_ONE_SPACE.replace_all(&s, " ") { s = o; }
    }
    s.trim().to_string()
}

pub fn toktok_tokenize(text: &str) -> Vec<String> {
    crate::api::TokenizerI::tokenize(&ToktokTokenizer, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let t = ToktokTokenizer;
        let toks = crate::api::TokenizerI::tokenize(&t, "Hello, world.");
        assert!(toks.contains(&"Hello".to_string()));
        assert!(toks.contains(&",".to_string()));
    }
    #[test]
    fn str_mode() {
        let t = ToktokTokenizer;
        assert!(!t.tokenize_str("Hello world").is_empty());
    }
    #[test]
    fn currency() {
        let t = ToktokTokenizer;
        let toks = crate::api::TokenizerI::tokenize(&t, "It costs $5.00.");
        assert_eq!(toks, vec!["It", "costs", "$", "5.00", "."]);
    }
    #[test]
    fn comma_in_num() {
        let t = ToktokTokenizer;
        let toks = crate::api::TokenizerI::tokenize(&t, "Value 1,234.56 and hello, world");
        assert!(toks.contains(&"1,234.56".to_string()));
        assert!(toks.contains(&",".to_string()));
    }
}
