use crate::{AbsTokenizer, Tokenizer, Token, load_midi_from_file, AbsConfig};
use serde_json::Value;
use std::collections::HashMap;

#[test]
fn test_abs_tokenizer_against_targets() {
    // Load the target results
    let target_results_path = "tests/assets/aria_targets/abstokenizer_results.json";
    let target_content = std::fs::read_to_string(target_results_path)
        .expect("Failed to read abstokenizer_results.json");
    let target_results: Vec<Value> = serde_json::from_str(&target_content)
        .expect("Failed to parse target results JSON");
    
    // Get config from the first target result and create tokenizer
    let first_result = &target_results[0];
    let config_json = &first_result["config"];
    let config: AbsConfig = serde_json::from_value(config_json.clone())
        .expect("Failed to deserialize config from target");
    let tokenizer = AbsTokenizer::new(config);
    
    // Tokenize all files first and collect results
    let mut actual_results = Vec::new();
    let mut failed_files = Vec::new();
    
    for target_result in &target_results {
        let test_file = target_result["file"].as_str()
            .expect("Target result missing file field");
            
        let midi_path = format!("tests/assets/data/{}", test_file);
        
        // Load MIDI file
        let midi_dict = load_midi_from_file(&midi_path)
            .expect(&format!("Failed to load {}", midi_path));
        
        // Tokenize using AbsTokenizer
        let tokens = tokenizer.tokenize(&midi_dict)
            .expect(&format!("Failed to tokenize {}", test_file));
        
        // Convert tokens to comparable format
        let tokens_json = tokens_to_json(&tokens);
        
        // Create actual result in same format as target
        let actual_result: Value = serde_json::json!({
            "file": test_file,
            "tokenizer": "AbsTokenizer",
            "num_tokens": tokens.len(),
            "tokens": tokens_json,
            "config": target_result["config"].clone(),
            "time_step_ms": target_result.get("time_step_ms").unwrap_or(&Value::Null),
            "max_duration_ms": target_result.get("max_duration_ms").unwrap_or(&Value::Null),
            "round_trip_success": target_result.get("round_trip_success").unwrap_or(&Value::Null),
            "detokenized_note_count": target_result.get("detokenized_note_count").unwrap_or(&Value::Null)
        });
        
        actual_results.push(actual_result.clone());
        
        // Skip prefix tokens for comparison (issue https://github.com/EleutherAI/aria-utils/issues/29)
        let actual_tokens_filtered = filter_out_prefix_tokens(&tokens_json);
        let actual_tokens_filtered = filter_out_diminish_tokens(&actual_tokens_filtered);
        let expected_tokens = &target_result["tokens"];
        let expected_tokens_filtered = filter_out_prefix_tokens(expected_tokens);
        let expected_tokens_filtered = filter_out_diminish_tokens(&expected_tokens_filtered);
        
        // Check if this file has differences
        if actual_tokens_filtered != expected_tokens_filtered {
            // Detokenize and save as MIDI file for debugging
            if let Ok(detokenized_midi) = tokenizer.detokenize(&tokens) {
                let failed_midi_path = format!("tests/assets/results/{}_failed.mid", 
                    test_file.strip_suffix(".mid").unwrap_or(test_file));
                let midi_file = detokenized_midi.to_midi();
                let _ = midi_file.save(&failed_midi_path);
                println!("Saved failed detokenized MIDI to: {}", failed_midi_path);
            }
            
            failed_files.push((test_file.to_string(), actual_result));
        }
    }
    
    // If any files failed, write the actual results to file and report differences
    if !failed_files.is_empty() {
        // Ensure results directory exists
        std::fs::create_dir_all("tests/assets/results").unwrap();
        
        // Write actual results for failed files
        let actual_results_json = serde_json::to_string_pretty(&actual_results).unwrap();
        std::fs::write("tests/assets/results/actual_abstokenizer_results.json", actual_results_json).unwrap();
        
        // Collect all differences for writing to file
        let mut diff_output = String::new();
        
        // Report differences
        for (test_file, actual_result) in &failed_files {
            let target_result = target_results.iter()
                .find(|r| r["file"].as_str() == Some(test_file))
                .unwrap();
            
            let actual_tokens = &actual_result["tokens"];
            let expected_tokens = &target_result["tokens"];
            
            // Filter out prefix and diminish tokens for comparison
            let actual_tokens_filtered = filter_out_prefix_tokens(actual_tokens);
            let actual_tokens_filtered = filter_out_diminish_tokens(&actual_tokens_filtered);
            let expected_tokens_filtered = filter_out_prefix_tokens(expected_tokens);
            let expected_tokens_filtered = filter_out_diminish_tokens(&expected_tokens_filtered);
            
            // Print first 10 differences for this file
            if let (Value::Array(actual_array), Value::Array(expected_array)) = (&actual_tokens_filtered, &expected_tokens_filtered) {
                let header = format!("=============\nToken differences for {}:\n  Total tokens: actual={}, expected={}\n", 
                    test_file, actual_array.len(), expected_array.len());
                println!("{}", header.trim());
                diff_output.push_str(&header);
                
                let mut diff_count = 0;
                
                for i in 0..std::cmp::max(actual_array.len(), expected_array.len()) {
                    let actual_token = actual_array.get(i);
                    let expected_token = expected_array.get(i);
                    
                    if actual_token != expected_token {
                        let from_end_actual = actual_array.len().saturating_sub(i + 1);
                        let from_end_expected = expected_array.len().saturating_sub(i + 1);
                        
                        let mut diff_entry = String::new();
                        
                        // Show previous 2 tokens for context
                        if i >= 2 {
                            let context = format!("    Context: [{}-2]={:?}, [{}-1]={:?}\n", 
                                i, actual_array.get(i-2), i, actual_array.get(i-1));
                            if diff_count < 10 {
                                println!("{}", context.trim());
                            }
                            diff_entry.push_str(&context);
                        }
                        
                        let diff_line = format!("  [{}]: expected={:?}, actual={:?} (from_end: exp={}, act={})\n---\n", 
                            i, expected_token, actual_token, from_end_expected, from_end_actual);
                        
                        if diff_count < 10 {
                            println!("{}", diff_line.trim_end_matches('\n'));
                        }
                        diff_entry.push_str(&diff_line);
                        diff_output.push_str(&diff_entry);
                        
                        diff_count += 1;
                    }
                }
            }
        }
        
        // Write all differences to file
        if !diff_output.is_empty() {
            let diff_file_path = "tests/assets/results/token_differences.txt";
            std::fs::write(diff_file_path, &diff_output)
                .expect("Failed to write token differences to file");
            println!("\nAll token differences written to: {}", diff_file_path);
        }
        
        panic!("Found {} files with token mismatches. Actual results written to tests/assets/results/actual_abstokenizer_results.json", failed_files.len());
    }
    
    println!("✓ All {} files passed tokenization tests", target_results.len());
}

fn tokens_to_json(tokens: &[Token]) -> Value {
    
    let json_tokens: Vec<Value> = tokens.iter().enumerate().map(|(idx, token)| {
        match token {
            Token::Special(s) => Value::String(s.clone()),
            Token::Prefix(prefix_type, prefix_subtype, value) => {
                Value::Array(vec![
                    Value::String(prefix_type.clone()),
                    Value::String(prefix_subtype.clone()),
                    Value::String(value.clone())
                ])
            },
            Token::Note(instrument, pitch, velocity) => {
                Value::Array(vec![
                    Value::String(instrument.clone()),
                    Value::Number((*pitch).into()),
                    Value::Number((*velocity).into())
                ])
            },
            Token::Drum(drum_type, pitch) => {
                Value::Array(vec![
                    Value::String(drum_type.clone()),
                    Value::Number((*pitch).into())
                ])
            },
            Token::Onset(onset_type, time) => {
                Value::Array(vec![
                    Value::String(onset_type.clone()),
                    Value::Number((*time).into())
                ])
            },
            Token::Duration(dur_type, time) => {
                Value::Array(vec![
                    Value::String(dur_type.clone()),
                    Value::Number((*time).into())
                ])
            }
        }
    }).collect();
    
    Value::Array(json_tokens)
}

fn filter_out_prefix_tokens(tokens: &Value) -> Value {
    if let Value::Array(token_array) = tokens {
        let filtered: Vec<Value> = token_array.iter().filter(|token| {
            // Skip prefix tokens - they are arrays with first element "prefix"
            if let Value::Array(arr) = token {
                if arr.len() >= 1 {
                    if let Value::String(first) = &arr[0] {
                        return first != "prefix";
                    }
                }
            }
            true
        }).cloned().collect();
        
        Value::Array(filtered)
    } else {
        tokens.clone()
    }
}

fn filter_out_diminish_tokens(tokens: &Value) -> Value {
    if let Value::Array(token_array) = tokens {
        let filtered: Vec<Value> = token_array.iter().filter(|token| {
            // Skip diminish tokens - they are strings that contain "diminish"
            if let Value::String(s) = token {
                return !s.contains("<D>");
            }
            true
        }).cloned().collect();
        
        Value::Array(filtered)
    } else {
        tokens.clone()
    }
}