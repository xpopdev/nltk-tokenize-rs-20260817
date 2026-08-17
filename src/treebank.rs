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
static DT_RE_DQ: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"''").unwrap());
static DT_RE_DASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" -- ").unwrap());
static DT_RE_LRB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-LRB-").unwrap());
static DT_RE_RRB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-RRB-").unwrap());
static DT_RE_LSB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-LSB-").unwrap());
static DT_RE_RSB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-RSB-").unwrap());
static DT_RE_LCB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-LCB-").unwrap());
static DT_RE_RCB: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"-RCB-").unwrap());
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
static DT_RE_SQ3: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"``").unwrap());

pub fn detokenize(tokens: &[String], convert_parentheses: bool) -> String {
    if tokens.is_empty() {
        return String::new();
    }
    let mut text = String::with_capacity(tokens.join(" ").len() + 4);
    text.push(' ');
    text.push_str(&tokens.join(" "));
    text.push(' ');
    text = DT_RE_C3.replace_all(&text, "$1$2 ").to_string();
    text = DT_RE_C2.replace_all(&text, "$1$2 ").to_string();
    text = DT_RE_END3.replace_all(&text, "$1$2").to_string();
    text = DT_RE_END4.replace_all(&text, "$1$2").to_string();
    text = DT_RE_DQ.replace_all(&text, "\"").to_string();
    text = text.trim().to_string();
    text = DT_RE_DASH.replace_all(&text, "--").to_string();
    if convert_parentheses {
        text = DT_RE_LRB.replace_all(&text, "(").to_string();
        text = DT_RE_RRB.replace_all(&text, ")").to_string();
        text = DT_RE_LSB.replace_all(&text, "[").to_string();
        text = DT_RE_RSB.replace_all(&text, "]").to_string();
        text = DT_RE_LCB.replace_all(&text, "{").to_string();
        text = DT_RE_RCB.replace_all(&text, "}").to_string();
    }
    text = DT_RE_P1.replace_all(&text, "$1").to_string();
    text = DT_RE_P2.replace_all(&text, "$1").to_string();
    text = DT_RE_P3.replace_all(&text, "$1$2").to_string();
    text = DT_RE_PUN1.replace_all(&text, "$1' ").to_string();
    text = DT_RE_PUN2.replace_all(&text, "$1").to_string();
    text = DT_RE_PUN3.replace_all(&text, "$1$2$3").to_string();
    text = DT_RE_PUN4.replace_all(&text, "$1").to_string();
    text = DT_RE_PUN5.replace_all(&text, "$1").to_string();
    text = DT_RE_PUN6.replace_all(&text, "...").to_string();
    text = DT_RE_PUN7.replace_all(&text, "$1").to_string();
    text = DT_RE_SQ1.replace_all(&text, "$1``").to_string();
    text = DT_RE_SQ2.replace_all(&text, "$1").to_string();
    text = DT_RE_SQ3.replace_all(&text, "\"").to_string();
    text.trim().to_string()
}
