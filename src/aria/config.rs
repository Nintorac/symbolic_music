use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbsConfig {
    pub abs_time_step_ms: i32,
    pub max_dur_ms: i32,
    pub time_step_ms: i32,
    pub velocity_quantization_step: i32,
    pub ignore_instruments: HashMap<String, bool>,
    pub instrument_programs: HashMap<String, i32>,
    pub drum_velocity: i32,
    pub composer_names: Vec<String>,
    pub form_names: Vec<String>,
    pub genre_names: Vec<String>,
}

impl Default for AbsConfig {
    fn default() -> Self {
        let mut ignore_instruments = HashMap::new();
        ignore_instruments.insert("piano".to_string(), false);
        ignore_instruments.insert("guitar".to_string(), false);
        ignore_instruments.insert("drum".to_string(), false);

        let mut instrument_programs = HashMap::new();
        instrument_programs.insert("piano".to_string(), 0);
        instrument_programs.insert("guitar".to_string(), 24);
        instrument_programs.insert("drum".to_string(), 0);

        Self {
            abs_time_step_ms: 5000,
            max_dur_ms: 5000,
            time_step_ms: 10,
            velocity_quantization_step: 4,
            ignore_instruments,
            instrument_programs,
            drum_velocity: 127,
            composer_names: vec!["Mozart".to_string(), "Bach".to_string()],
            form_names: vec!["sonata".to_string(), "fugue".to_string()],
            genre_names: vec!["classical".to_string(), "jazz".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    pub abs: AbsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub tokenizer: TokenizerConfig,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            abs: AbsConfig::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tokenizer: TokenizerConfig::default(),
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub fn load_default() -> Self {
        let default_config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/aria/default_config.json");
        
        if default_config_path.exists() {
            match Self::load_from_file(&default_config_path) {
                Ok(config) => return config,
                Err(e) => {
                    // Warning: Failed to load default config, using fallback
                }
            }
        }
        
        // Fallback to hardcoded default
        Self::default()
    }
}