use std::collections::HashMap;
use super::midi_types::{MidiDict, Token};

pub trait Tokenizer {
    fn tokenize(&self, midi_dict: &MidiDict) -> Result<Vec<Token>, TokenizerError>;
    fn detokenize(&self, tokens: &[Token]) -> Result<MidiDict, TokenizerError>;
    fn vocab_size(&self) -> usize;
    fn encode(&self, tokens: &[Token]) -> Vec<i32>;
    fn decode(&self, ids: &[i32]) -> Vec<Token>;
    fn name(&self) -> &str;
}

#[derive(Debug, thiserror::Error)]
pub enum TokenizerError {
    #[error("Empty note messages after filtering")]
    EmptyNoteMessages,
    #[error("Invalid token sequence: {0}")]
    InvalidTokenSequence(String),
    #[error("Token not found in vocabulary: {0:?}")]
    TokenNotFound(String),
    #[error("Invalid token ID: {0}")]
    InvalidTokenId(i32),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub struct BaseTokenizer {
    pub vocab: Vec<Token>,
    pub tok_to_id: HashMap<Token, i32>,
    pub id_to_tok: HashMap<i32, Token>,
    pub special_tokens: Vec<String>,
    pub bos_tok: String,
    pub eos_tok: String,
    pub pad_tok: String,
    pub unk_tok: String,
    pub dim_tok: String,
    pub pad_id: i32,
}

impl BaseTokenizer {
    pub fn new() -> Self {
        let special_tokens = vec![
            "<S>".to_string(),
            "<E>".to_string(),
            "<P>".to_string(),
            "<U>".to_string(),
            "<D>".to_string(),
        ];
        
        Self {
            vocab: Vec::new(),
            tok_to_id: HashMap::new(),
            id_to_tok: HashMap::new(),
            special_tokens: special_tokens.clone(),
            bos_tok: "<S>".to_string(),
            eos_tok: "<E>".to_string(),
            pad_tok: "<P>".to_string(),
            unk_tok: "<U>".to_string(),
            dim_tok: "<D>".to_string(),
            pad_id: 0,
        }
    }
    
    pub fn add_tokens_to_vocab(&mut self, tokens: Vec<Token>) {
        for token in tokens {
            if !self.vocab.contains(&token) {
                let id = self.vocab.len() as i32;
                self.vocab.push(token.clone());
                self.tok_to_id.insert(token.clone(), id);
                self.id_to_tok.insert(id, token);
            }
        }
        
        // Update pad_id
        if let Some(&id) = self.tok_to_id.get(&Token::Special("<P>".to_string())) {
            self.pad_id = id;
        }
    }
    
    pub fn find_closest_int(value: i32, quantizations: &[i32]) -> i32 {
        if quantizations.is_empty() {
            return 0;
        }
        
        // Exact implementation matching Python's _find_closest_int from ariautils/tokenizer/_base.py
        let mut left = 0;
        let mut right = quantizations.len() - 1;
        let mut closest = i32::MAX; // Python uses float("inf")
        
        while left <= right {
            let mid = (left + right) / 2;
            let diff = (quantizations[mid] - value).abs();
            
            // Python: if diff < abs(closest - n):
            if diff < (closest - value).abs() {
                closest = quantizations[mid];
            }
            
            if quantizations[mid] < value {
                left = mid + 1;
            } else {
                if mid == 0 { break; } // Prevent underflow
                right = mid - 1;
            }
        }
        
        closest
    }
}