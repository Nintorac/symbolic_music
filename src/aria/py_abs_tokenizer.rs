use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use super::abs_tokenizer::AbsTokenizer;
use super::config::TokenizerConfig;
use super::tokenizer::Tokenizer;
use super::py_midi_types::{PyMidiDict, convert_py_midi_dict_to_rust};
use super::midi_types::Token;

// Convert tokens to the right Python representation
fn token_to_python_value(token: &Token) -> PyObject {
    Python::with_gil(|py| {
        match token {
            Token::Special(s) => s.clone().into_py(py),
            Token::Prefix(p1, p2, p3) => (format!("{}:{}:{}", p1, p2, p3), 0, 0).into_py(py),
            Token::Note(instrument, pitch, velocity) => (instrument.clone(), *pitch, *velocity).into_py(py),
            Token::Onset(_, time) => ("onset".to_string(), *time, 0).into_py(py),
            Token::Duration(_, time) => ("dur".to_string(), *time, 0).into_py(py),
            Token::Drum(_, pitch) => ("drum".to_string(), *pitch, 0).into_py(py),
        }
    })
}

// Handle conversion from Python mixed types (strings and tuples) to Rust tokens
fn python_to_token(py: Python, py_obj: &Bound<PyAny>) -> PyResult<Token> {
    // Try to extract as string first (for special tokens)
    if let Ok(s) = py_obj.extract::<String>() {
        return Ok(Token::Special(s));
    }
    
    // Try to extract as tuple (for regular tokens)
    if let Ok(tuple) = py_obj.extract::<(String, i32, i32)>() {
        return Ok(tuple_to_token(&tuple));
    }
    
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Token must be either a string or a 3-tuple"))
}

fn tuple_to_token(tuple: &(String, i32, i32)) -> Token {
    match tuple.0.as_str() {
        s if s.starts_with("<") => Token::Special(s.to_string()),
        s if s.contains("prefix:") => {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() == 3 {
                Token::Prefix(parts[0].to_string(), parts[1].to_string(), parts[2].to_string())
            } else {
                Token::Special(s.to_string())
            }
        }
        "onset" => Token::Onset("onset".to_string(), tuple.1),
        "dur" => Token::Duration("dur".to_string(), tuple.1),
        "drum" => Token::Drum("drum".to_string(), tuple.1),
        _ => Token::Note(tuple.0.clone(), tuple.1, tuple.2),
    }
}

#[pyclass(name = "_AbsTokenizer", subclass)]
pub struct PyAbsTokenizer {
    tokenizer: AbsTokenizer,
    config: TokenizerConfig,
}

#[pymethods]
impl PyAbsTokenizer {
    #[new]
    fn new() -> Self {
        let config = TokenizerConfig::default();
        Self {
            tokenizer: AbsTokenizer::new(config.abs.clone()),
            config,
        }
    }
    
    #[pyo3(signature = (py_midi_dict, remove_preceding_silence = None))]
    fn tokenize(&self, py_midi_dict: &PyMidiDict, remove_preceding_silence: Option<bool>) -> PyResult<Vec<PyObject>> {
        let midi_dict = convert_py_midi_dict_to_rust(py_midi_dict);
        match self.tokenizer.tokenize(&midi_dict) {
            Ok(tokens) => Ok(tokens.iter().map(token_to_python_value).collect()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Tokenization failed: {}", e))),
        }
    }
    
    fn detokenize(&self, tokens: Vec<PyObject>) -> PyResult<PyMidiDict> {
        // Convert Python mixed types to Rust tokens
        let rust_tokens: Result<Vec<Token>, PyErr> = Python::with_gil(|py| {
            tokens.iter().map(|obj| python_to_token(py, obj.bind(py))).collect()
        });
        let rust_tokens = rust_tokens?;
        
        match self.tokenizer.detokenize(&rust_tokens) {
            Ok(midi_dict) => Ok(super::py_midi_types::convert_rust_midi_dict_to_py(&midi_dict)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Detokenization failed: {}", e))),
        }
    }
    
    fn encode(&self, tokens: Vec<PyObject>) -> PyResult<Vec<i32>> {
        // Convert Python mixed types to Rust tokens
        let rust_tokens: Result<Vec<Token>, PyErr> = Python::with_gil(|py| {
            tokens.iter().map(|obj| python_to_token(py, obj.bind(py))).collect()
        });
        let rust_tokens = rust_tokens?;
        
        Ok(self.tokenizer.encode(&rust_tokens))
    }
    
    fn decode(&self, ids: Vec<i32>) -> Vec<PyObject> {
        let rust_tokens = self.tokenizer.decode(&ids);
        rust_tokens.iter().map(token_to_python_value).collect()
    }
    
    fn vocab_size(&self) -> usize {
        self.tokenizer.vocab_size()
    }
    
    fn name(&self) -> &str {
        self.tokenizer.name()
    }
    
    #[getter]
    fn config(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let dict = PyDict::new(py);
            
            // Convert ignore_instruments to Python dict
            let ignore_instruments = PyDict::new(py);
            for (key, value) in &self.config.abs.ignore_instruments {
                ignore_instruments.set_item(key, *value)?;
            }
            dict.set_item("ignore_instruments", ignore_instruments)?;
            
            // Convert instrument_programs to Python dict
            let instrument_programs = PyDict::new(py);
            for (key, value) in &self.config.abs.instrument_programs {
                instrument_programs.set_item(key, *value)?;
            }
            dict.set_item("instrument_programs", instrument_programs)?;
            
            dict.set_item("drum_velocity", self.config.abs.drum_velocity)?;
            
            Ok(dict.into())
        })
    }
    
    #[getter]
    fn time_step_ms(&self) -> i32 {
        self.config.abs.time_step_ms
    }
    
    #[getter]
    fn abs_time_step_ms(&self) -> i32 {
        self.config.abs.abs_time_step_ms
    }
    
    #[getter]
    fn max_dur_ms(&self) -> i32 {
        self.config.abs.max_dur_ms
    }
    
    #[getter]
    fn velocity_step(&self) -> i32 {
        self.config.abs.velocity_quantization_step
    }
    
    #[getter]
    fn max_velocity(&self) -> i32 {
        self.tokenizer.max_velocity
    }
    
    #[getter]
    fn instruments_nd(&self) -> Vec<String> {
        self.config.abs.instrument_programs.keys().cloned().collect()
    }
    
    #[getter]
    fn unk_tok(&self) -> String {
        self.tokenizer.base_tokenizer.unk_tok.clone()
    }
    
    fn _quantize_velocity(&self, velocity: i32) -> i32 {
        // Use the same logic as the Rust tokenizer
        self.tokenizer.quantize_velocity(velocity)
    }
    
    fn apply_pitch_aug(&self, tokens: Vec<PyObject>, max_pitch_aug: i32, pitch_aug: Option<i32>) -> PyResult<Vec<PyObject>> {
        // Convert Python mixed types to Rust tokens
        let rust_tokens: Result<Vec<Token>, PyErr> = Python::with_gil(|py| {
            tokens.iter().map(|obj| python_to_token(py, obj.bind(py))).collect()
        });
        let rust_tokens = rust_tokens?;
        
        // Get the Rust augmentation function and apply it
        let rust_fn = self.tokenizer.export_pitch_aug(max_pitch_aug);
        let result_tokens = rust_fn(&rust_tokens, pitch_aug);
        
        // Convert back to Python representation (strings for special tokens, tuples for others)
        Ok(result_tokens.iter().map(token_to_python_value).collect())
    }
    
    fn apply_velocity_aug(&self, tokens: Vec<PyObject>, max_num_aug_steps: i32, aug_step: Option<i32>) -> PyResult<Vec<PyObject>> {
        // Convert Python mixed types to Rust tokens
        let rust_tokens: Result<Vec<Token>, PyErr> = Python::with_gil(|py| {
            tokens.iter().map(|obj| python_to_token(py, obj.bind(py))).collect()
        });
        let rust_tokens = rust_tokens?;
        
        // Get the Rust augmentation function and apply it
        let rust_fn = self.tokenizer.export_velocity_aug(max_num_aug_steps);
        let result_tokens = rust_fn(&rust_tokens, aug_step);
        
        // Convert back to Python representation (strings for special tokens, tuples for others)
        Ok(result_tokens.iter().map(token_to_python_value).collect())
    }
    
    fn apply_tempo_aug(&self, tokens: Vec<PyObject>, max_tempo_aug: f64, mixup: bool, tempo_aug: Option<f64>) -> PyResult<Vec<PyObject>> {
        // Convert Python mixed types to Rust tokens
        let rust_tokens: Result<Vec<Token>, PyErr> = Python::with_gil(|py| {
            tokens.iter().map(|obj| python_to_token(py, obj.bind(py))).collect()
        });
        let rust_tokens = rust_tokens?;
        
        // Get the Rust augmentation function and apply it
        let rust_fn = self.tokenizer.export_tempo_aug(max_tempo_aug, mixup);
        let result_tokens = rust_fn(&rust_tokens, tempo_aug);
        
        // Convert back to Python representation (strings for special tokens, tuples for others)
        Ok(result_tokens.iter().map(token_to_python_value).collect())
    }
}