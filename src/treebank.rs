use std::borrow::Cow;
use std::sync::LazyLock;
use regex::Regex;

use crate::destructive::NLTKWordTokenizer;

pub fn word_tokenize_core(text: &str, convert_parentheses: bool) -> Vec<String> {
    NLTKWordTokenizer::tokenize_core(text, convert_parentheses)
}

pub fn sent_tokenize_placeholder(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

static DT_RE_C3: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([^' ])\s('[sS]|'[mM]|'[dD]|') ").unwrap());
static DT_RE_C2: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([^' ])\s('ll|'LL|'re|'RE|'ve|'VE|n't|N'T) ").unwrap());
static DT_RE_END3: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\S)\s('')").unwrap());
static DT_RE_END4: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"('')\s([.,:)\]>};%])").unwrap());
static DT_RE_P1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([\[({<])\s").unwrap());
static DT_RE_P2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s([])}>])").unwrap());
static DT_RE_P3: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([])}>])\s([:;,.])").unwrap());
static DT_RE_PUN1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([^'])\s'\s").unwrap());
static DT_RE_PUN2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s([?!])").unwrap());
static DT_RE_PUN3: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"([^\.])\s(\.)([\]\[\)}>"']*)\s*$"#).unwrap());
static DT_RE_PUN4: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([#$])\s").unwrap());
static DT_RE_PUN5: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s([;%])").unwrap());
static DT_RE_PUN6: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s\.\.\.\s").unwrap());
static DT_RE_PUN7: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s([:,])").unwrap());
static DT_RE_SQ1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([ \(\[{<])\s``").unwrap());
static DT_RE_SQ2: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(``)\s").unwrap());

pub fn detokenize(tokens: &[String], convert_parentheses: bool) -> String {
    if tokens.is_empty() {
        return String::new();
    }
    let mut text = String::with_capacity(tokens.join(" ").len() + 4);
    text.push(' ');
    text.push_str(&tokens.join(" "));
    text.push(' ');
    if let Cow::Owned(o) = DT_RE_C3.replace_all(&text, "$1$2 ") { text = o; }
    if let Cow::Owned(o) = DT_RE_C2.replace_all(&text, "$1$2 ") { text = o; }
    if let Cow::Owned(o) = DT_RE_END3.replace_all(&text, "$1$2") { text = o; }
    if let Cow::Owned(o) = DT_RE_END4.replace_all(&text, "$1$2") { text = o; }
    if text.contains("''") { text = text.replace("''", "\""); }
    text = text.trim().to_string();
    if text.contains(" -- ") { text = text.replace(" -- ", "--"); }
    if convert_parentheses {
        if text.contains("-LRB-") { text = text.replace("-LRB-", "("); }
        if text.contains("-RRB-") { text = text.replace("-RRB-", ")"); }
        if text.contains("-LSB-") { text = text.replace("-LSB-", "["); }
        if text.contains("-RSB-") { text = text.replace("-RSB-", "]"); }
        if text.contains("-LCB-") { text = text.replace("-LCB-", "{"); }
        if text.contains("-RCB-") { text = text.replace("-RCB-", "}"); }
    }
    if let Cow::Owned(o) = DT_RE_P1.replace_all(&text, "$1") { text = o; }
    if let Cow::Owned(o) = DT_RE_P2.replace_all(&text, "$1") { text = o; }
    if let Cow::Owned(o) = DT_RE_P3.replace_all(&text, "$1$2") { text = o; }
    if let Cow::Owned(o) = DT_RE_PUN1.replace_all(&text, "$1' ") { text = o; }
    if let Cow::Owned(o) = DT_RE_PUN2.replace_all(&text, "$1") { text = o; }
    if let Cow::Owned(o) = DT_RE_PUN3.replace_all(&text, "$1$2$3") { text = o; }
    if let Cow::Owned(o) = DT_RE_PUN4.replace_all(&text, "$1") { text = o; }
    if let Cow::Owned(o) = DT_RE_PUN5.replace_all(&text, "$1") { text = o; }
    if let Cow::Owned(o) = DT_RE_PUN6.replace_all(&text, "...") { text = o; }
    if let Cow::Owned(o) = DT_RE_PUN7.replace_all(&text, "$1") { text = o; }
    if let Cow::Owned(o) = DT_RE_SQ1.replace_all(&text, "$1``") { text = o; }
    if let Cow::Owned(o) = DT_RE_SQ2.replace_all(&text, "$1") { text = o; }
    if text.contains("``") { text = text.replace("``", "\""); }
    text.trim().to_string()
}
