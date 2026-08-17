#![allow(clippy::useless_conversion)]
use pyo3::prelude::*;

pub mod destructive;
pub mod punkt;
pub mod treebank;
pub mod util;

use destructive::NLTKWordTokenizer;
use punkt::PunktSentenceTokenizer;

pub fn add_core(a: i64, b: i64) -> i64 { a + b }

#[pyfunction]
fn add(a: i64, b: i64) -> PyResult<i64> { Ok(add_core(a, b)) }

#[pyfunction]
#[pyo3(signature = (text, convert_parentheses=None))]
fn word_tokenize(text: String, convert_parentheses: Option<bool>) -> PyResult<Vec<String>> {
    Ok(NLTKWordTokenizer::tokenize_core(&text, convert_parentheses.unwrap_or(false)))
}

#[pyfunction]
fn word_tokenize_span(text: String) -> PyResult<Vec<(usize, usize)>> {
    Ok(NLTKWordTokenizer::span_tokenize_core(&text))
}

#[pyfunction]
#[pyo3(signature = (text, language="english".to_string(), realign_boundaries=true))]
fn sent_tokenize(text: String, language: String, realign_boundaries: bool) -> PyResult<Vec<String>> {
    let _ = language; // M2 uses english_default; language-specific loading is M7 / asset pipeline
    let tok = PunktSentenceTokenizer::default();
    Ok(tok.tokenize(&text, realign_boundaries))
}

#[pymodule]
fn ported_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(word_tokenize, m)?)?;
    m.add_function(wrap_pyfunction!(word_tokenize_span, m)?)?;
    m.add_function(wrap_pyfunction!(sent_tokenize, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn add_core_basic() { assert_eq!(add_core(2,2),4); }
}
