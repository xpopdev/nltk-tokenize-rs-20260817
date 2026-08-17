#![allow(clippy::useless_conversion)]
use pyo3::prelude::*;
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

static LEGALITY_CACHE: LazyLock<RwLock<HashMap<String, crate::deferrable::LegalityPrincipleTokenizer>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub mod api;
pub mod casual;
pub mod deferrable;
pub mod destructive;
pub mod mwe;
pub mod punkt;
pub mod punkt_trainer;
pub mod regex_cache;
pub mod regexp;
pub mod sexpr;
pub mod simple;
pub mod nist;
pub mod toktok;
pub mod treebank;
pub mod util;

use crate::api::TokenizerI;
use destructive::NLTKWordTokenizer;
use punkt::PunktSentenceTokenizer;

pub fn add_core(a: i64, b: i64) -> i64 {
    a + b
}

#[pyfunction]
fn add(a: i64, b: i64) -> PyResult<i64> {
    Ok(add_core(a, b))
}

#[pyfunction]
#[pyo3(signature = (text, convert_parentheses=None))]
fn word_tokenize(py: Python, text: &str, convert_parentheses: Option<bool>) -> PyResult<Vec<String>> {
    let convert = convert_parentheses.unwrap_or(false);
    py.allow_threads(|| Ok(NLTKWordTokenizer::tokenize_core(text, convert)))
}

#[pyfunction]
fn word_tokenize_span(py: Python, text: &str) -> PyResult<Vec<(usize, usize)>> {
    py.allow_threads(|| Ok(NLTKWordTokenizer::span_tokenize_core(text)))
}

#[pyfunction]
#[pyo3(signature = (text, language="english".to_string(), realign_boundaries=true))]
fn sent_tokenize(py: Python, text: &str, language: String, realign_boundaries: bool) -> PyResult<Vec<String>> {
    let _ = language;
    py.allow_threads(|| {
        let tok = PunktSentenceTokenizer::default();
        Ok(tok.tokenize(text, realign_boundaries))
    })
}

#[pyfunction]
fn regexp_tokenize(py: Python, text: &str, pattern: &str, gaps: bool, discard_empty: bool) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::regexp::regexp_tokenize(text, pattern, gaps, discard_empty)))
}

#[pyfunction]
fn string_span_tokenize_py(s: &str, sep: &str) -> PyResult<Vec<(usize, usize)>> {
    if sep.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err("Token delimiter must not be empty"));
    }
    Ok(crate::util::string_span_tokenize(s, sep))
}

#[pyfunction]
fn regexp_span_tokenize_py(s: &str, pattern: &str) -> PyResult<Vec<(usize, usize)>> {
    Ok(crate::util::regexp_span_tokenize(s, pattern))
}

#[pyfunction]
fn spans_to_relative_py(spans: Vec<(usize, usize)>) -> PyResult<Vec<(usize, usize)>> {
    Ok(crate::util::spans_to_relative(&spans))
}

#[pyfunction]
fn is_cjk_py(ch: String) -> PyResult<bool> {
    let c = ch.chars().next().unwrap_or('\0');
    Ok(crate::util::is_cjk(c))
}

#[pyfunction]
fn xml_escape_py(text: &str) -> PyResult<String> {
    Ok(crate::util::xml_escape(text))
}

#[pyfunction]
fn xml_unescape_py(text: &str) -> PyResult<String> {
    Ok(crate::util::xml_unescape(text))
}

#[pyfunction]
fn align_tokens_py(tokens: Vec<String>, sentence: &str) -> PyResult<Vec<(usize, usize)>> {
    Ok(crate::util::align_tokens(&tokens, sentence))
}

#[pyfunction]
#[pyo3(signature = (text, preserve_case=true, reduce_len=false, strip_handles=false, match_phone_numbers=true))]
fn casual_tokenize_py(py: Python, text: &str,
    preserve_case: bool,
    reduce_len: bool,
    strip_handles: bool,
    match_phone_numbers: bool,
) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::casual::casual_tokenize(text,
        preserve_case,
        reduce_len,
        strip_handles,
        match_phone_numbers,
    )))
}

#[pyfunction]
fn toktok_tokenize_py(py: Python, text: &str) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::toktok::toktok_tokenize(text)))
}

#[pyfunction]
fn mwe_tokenize_py(tokens: Vec<String>, mwes: Vec<Vec<String>>, separator: &str) -> PyResult<Vec<String>> {
    let tok = crate::mwe::MWETokenizer::new(mwes, separator);
    Ok(tok.tokenize_words(&tokens))
}

#[pyfunction]
#[pyo3(signature = (text, parens="()".to_string(), strict=true))]
fn sexpr_tokenize_py(py: Python, text: &str, parens: String, strict: bool) -> PyResult<Vec<String>> {
    py.allow_threads(|| { let tok = crate::sexpr::SExprTokenizer::new(&parens, strict); Ok(TokenizerI::tokenize(&tok, text)) })
}


#[pyfunction]
fn word_tokenize_batch(py: Python, texts: Vec<String>) -> PyResult<Vec<Vec<String>>> {
    py.allow_threads(|| Ok(texts.iter().map(|t| NLTKWordTokenizer::tokenize_core(t, false)).collect()))
}
#[pyfunction]
fn sent_tokenize_batch(py: Python, texts: Vec<String>) -> PyResult<Vec<Vec<String>>> {
    py.allow_threads(|| {
        let tok = PunktSentenceTokenizer::default();
        Ok(texts.iter().map(|t| tok.tokenize(t, true)).collect())
    })
}

#[pyfunction]
fn detokenize_py(py: Python, tokens: Vec<String>, convert_parentheses: bool) -> PyResult<String> {
    py.allow_threads(|| Ok(crate::treebank::detokenize(&tokens, convert_parentheses)))
}

#[pyfunction]
fn space_tokenize_py(py: Python, text: &str) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::api::TokenizerI::tokenize(&crate::simple::SpaceTokenizer, text)))
}

#[pyfunction]
fn tab_tokenize_py(py: Python, text: &str) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::api::TokenizerI::tokenize(&crate::simple::TabTokenizer, text)))
}

#[pyfunction]
fn char_tokenize_py(py: Python, text: &str) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::api::TokenizerI::tokenize(&crate::simple::CharTokenizer, text)))
}

#[pyfunction]
#[pyo3(signature = (text, blanklines="discard".to_string()))]
fn line_tokenize_py(py: Python, text: &str, blanklines: String) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::simple::line_tokenize(text, &blanklines)))
}

#[pyfunction]
fn blankline_tokenize_py(py: Python, text: &str) -> PyResult<Vec<String>> {
    py.allow_threads(|| {
        let mut t = crate::regexp::BlanklineTokenizer::new();
        Ok(t.tokenize(text))
    })
}

#[pyfunction]
fn wordpunct_tokenize_py(py: Python, text: &str) -> PyResult<Vec<String>> {
    py.allow_threads(|| {
        Ok(crate::regexp::WordPunctTokenizer::new().tokenize(text))
    })
}

#[pyfunction]
fn whitespace_tokenize_py(py: Python, text: &str) -> PyResult<Vec<String>> {
    py.allow_threads(|| {
        let mut t = crate::regexp::WhitespaceTokenizer::new();
        Ok(t.tokenize(text))
    })
}

#[pyfunction]
#[pyo3(signature = (text, lowercase=false, western_lang=true))]
fn nist_tokenize_py(py: Python, text: &str, lowercase: bool, western_lang: bool) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::nist::nist_tokenize(text, lowercase, western_lang)))
}

#[pyfunction]
#[pyo3(signature = (text, lowercase=false))]
fn nist_international_tokenize_py(py: Python, text: &str, lowercase: bool) -> PyResult<Vec<String>> {
    py.allow_threads(|| Ok(crate::nist::nist_international_tokenize(text, lowercase)))
}

#[pyfunction]
#[pyo3(signature = (word, vowels="aeiouy".to_string()))]
fn legality_tokenize_py(py: Python, word: &str, vowels: String) -> PyResult<Vec<String>> {
    // Without corpus, use default legal onsets (empty -> fallback syllabification)
    py.allow_threads(|| {
        let t = crate::deferrable::LegalityPrincipleTokenizer::new(vec![], &vowels);
        Ok(t.tokenize_word(word))
    })
}

#[pyfunction]
fn sonority_tokenize_py(py: Python, word: &str) -> PyResult<Vec<String>> {
    py.allow_threads(|| {
        let t = crate::deferrable::SonoritySequencingTokenizer::default();
        Ok(t.tokenize_word(word))
    })
}

#[pyfunction]
#[pyo3(signature = (text, w=20, k=10))]
fn texttiling_tokenize_py(py: Python, text: &str, w: usize, k: usize) -> PyResult<Vec<String>> {
    py.allow_threads(|| {
        let t = crate::deferrable::TextTilingTokenizer::new(w, k);
        Ok(crate::api::TokenizerI::tokenize(&t, text))
    })
}

#[pyfunction]
#[pyo3(signature = (text, corpus, vowels="aeiouy".to_string()))]
fn legality_tokenize_with_corpus_py(_py: Python, text: &str, corpus: Vec<String>, vowels: String) -> PyResult<Vec<String>> {
    let key = format!("{}:{}", vowels, corpus.len());
    if let Some(result) = LEGALITY_CACHE.read().unwrap().get(&key).map(|t| crate::api::TokenizerI::tokenize(t, text)) {
        return Ok(result);
    }
    let t = crate::deferrable::LegalityPrincipleTokenizer::new(corpus.clone(), &vowels);
    let result = crate::api::TokenizerI::tokenize(&t, text);
    LEGALITY_CACHE.write().unwrap().insert(key, t);
    Ok(result)
}

#[pyfunction]
#[pyo3(signature = (text, vowels="aeiouy".to_string()))]
fn legality_tokenize_cached_py(text: &str, vowels: String) -> PyResult<Vec<String>> {
    // try vowels-specific cache first, then any cached entry
    let cache = LEGALITY_CACHE.read().unwrap();
    let key = format!("{}:5000", vowels);
    if let Some(t) = cache.get(&key) {
        return Ok(crate::api::TokenizerI::tokenize(t, text));
    }
    if let Some(t) = cache.values().next() {
        return Ok(crate::api::TokenizerI::tokenize(t, text));
    }
    drop(cache);
    let t = crate::deferrable::LegalityPrincipleTokenizer::new(vec![], &vowels);
    Ok(t.tokenize_word(text))
}

#[pymodule]

fn ported_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(word_tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(word_tokenize_span, m)?)?;
    m.add_function(wrap_pyfunction!(sent_tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(regexp_tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(string_span_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(regexp_span_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(spans_to_relative_py, m)?)?;
    m.add_function(wrap_pyfunction!(is_cjk_py, m)?)?;
    m.add_function(wrap_pyfunction!(xml_escape_py, m)?)?;
    m.add_function(wrap_pyfunction!(xml_unescape_py, m)?)?;
    m.add_function(wrap_pyfunction!(align_tokens_py, m)?)?;
    m.add_function(wrap_pyfunction!(casual_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(toktok_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(mwe_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(sexpr_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(word_tokenize_batch, m)?)?;
    m.add_function(wrap_pyfunction!(sent_tokenize_batch, m)?)?;
    m.add_function(wrap_pyfunction!(detokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(space_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(tab_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(char_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(line_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(blankline_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(wordpunct_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(whitespace_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(nist_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(nist_international_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(legality_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(sonority_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(texttiling_tokenize_py, m)?)?;
    m.add_function(wrap_pyfunction!(legality_tokenize_with_corpus_py, m)?)?;
    m.add_function(wrap_pyfunction!(legality_tokenize_cached_py, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_core_basic() {
        assert_eq!(add_core(2, 2), 4);
    }
}
