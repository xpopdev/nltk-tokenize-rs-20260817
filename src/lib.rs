#![allow(clippy::useless_conversion)]
use pyo3::prelude::*;

pub mod api;
pub mod casual;
pub mod destructive;
pub mod mwe;
pub mod punkt;
pub mod regexp;
pub mod sexpr;
pub mod simple;
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
fn word_tokenize(text: String, convert_parentheses: Option<bool>) -> PyResult<Vec<String>> {
    Ok(NLTKWordTokenizer::tokenize_core(
        &text,
        convert_parentheses.unwrap_or(false),
    ))
}

#[pyfunction]
fn word_tokenize_span(text: String) -> PyResult<Vec<(usize, usize)>> {
    Ok(NLTKWordTokenizer::span_tokenize_core(&text))
}

#[pyfunction]
#[pyo3(signature = (text, language="english".to_string(), realign_boundaries=true))]
fn sent_tokenize(
    text: String,
    language: String,
    realign_boundaries: bool,
) -> PyResult<Vec<String>> {
    let _ = language;
    let tok = PunktSentenceTokenizer::default();
    Ok(tok.tokenize(&text, realign_boundaries))
}

#[pyfunction]
fn regexp_tokenize(
    text: String,
    pattern: String,
    gaps: bool,
    discard_empty: bool,
) -> PyResult<Vec<String>> {
    Ok(crate::regexp::regexp_tokenize(
        &text,
        &pattern,
        gaps,
        discard_empty,
    ))
}

#[pyfunction]
fn string_span_tokenize_py(s: String, sep: String) -> PyResult<Vec<(usize, usize)>> {
    if sep.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Token delimiter must not be empty",
        ));
    }
    Ok(crate::util::string_span_tokenize(&s, &sep))
}

#[pyfunction]
fn regexp_span_tokenize_py(s: String, pattern: String) -> PyResult<Vec<(usize, usize)>> {
    Ok(crate::util::regexp_span_tokenize(&s, &pattern))
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
fn xml_escape_py(text: String) -> PyResult<String> {
    Ok(crate::util::xml_escape(&text))
}

#[pyfunction]
fn xml_unescape_py(text: String) -> PyResult<String> {
    Ok(crate::util::xml_unescape(&text))
}

#[pyfunction]
fn align_tokens_py(tokens: Vec<String>, sentence: String) -> PyResult<Vec<(usize, usize)>> {
    Ok(crate::util::align_tokens(&tokens, &sentence))
}

#[pyfunction]
#[pyo3(signature = (text, preserve_case=true, reduce_len=false, strip_handles=false, match_phone_numbers=true))]
fn casual_tokenize_py(
    text: String,
    preserve_case: bool,
    reduce_len: bool,
    strip_handles: bool,
    match_phone_numbers: bool,
) -> PyResult<Vec<String>> {
    Ok(crate::casual::casual_tokenize(
        &text,
        preserve_case,
        reduce_len,
        strip_handles,
        match_phone_numbers,
    ))
}

#[pyfunction]
fn toktok_tokenize_py(text: String) -> PyResult<Vec<String>> {
    Ok(crate::toktok::toktok_tokenize(&text))
}

#[pyfunction]
fn mwe_tokenize_py(
    tokens: Vec<String>,
    mwes: Vec<Vec<String>>,
    separator: String,
) -> PyResult<Vec<String>> {
    let tok = crate::mwe::MWETokenizer::new(mwes, &separator);
    Ok(tok.tokenize_words(&tokens))
}

#[pyfunction]
#[pyo3(signature = (text, parens="()".to_string(), strict=true))]
fn sexpr_tokenize_py(text: String, parens: String, strict: bool) -> PyResult<Vec<String>> {
    let tok = crate::sexpr::SExprTokenizer::new(&parens, strict);
    Ok(TokenizerI::tokenize(&tok, &text))
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
