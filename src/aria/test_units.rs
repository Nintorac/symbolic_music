use super::units::{second2tick, tick2second};
use super::midi_types::Token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rounding_errors() {
        // Test the exact values used in detokenization
        let tempo = 500000; // microseconds per quarter note
        let ticks_per_beat = 500;
        
        // Testing time conversion precision
        
        // Test some specific millisecond values that might cause issues
        let test_values_ms = vec![40, 41, 1000, 1050, 140];
        
        for ms in test_values_ms {
            let seconds = ms as f64 / 1000.0;
            let ticks = second2tick(seconds, ticks_per_beat, tempo);
            let back_to_seconds = tick2second(ticks, ticks_per_beat, tempo);
            let back_to_ms = (back_to_seconds * 1000.0).round() as i32;
            
            // Time conversion cycle test
            
            if ms != back_to_ms {
                // Rounding error detected
            }
        }
        
        // Test the specific case from our failing test
        // Testing specific 40ms duration case
        let duration_ms = 40;
        let onset_ms = 0;
        let curr_time_ms = 0;
        
        let start_time_ms = curr_time_ms + onset_ms;
        let end_time_ms = start_time_ms + duration_ms;
        
        // Calculating start time
        // Calculating end time
        
        let start_tick = second2tick(start_time_ms as f64 / 1000.0, ticks_per_beat, tempo);
        let end_tick = second2tick(end_time_ms as f64 / 1000.0, ticks_per_beat, tempo);
        
        // Converting to start tick
        // Converting to end tick
        // Calculating duration in ticks
        
        // Convert back to verify
        let back_start_seconds = tick2second(start_tick, ticks_per_beat, tempo);
        let back_end_seconds = tick2second(end_tick, ticks_per_beat, tempo);
        let back_duration_ms = ((back_end_seconds - back_start_seconds) * 1000.0).round() as i32;
        
        // Converting back to milliseconds
        
        if duration_ms != back_duration_ms {
            // Duration mismatch detected
        }
    }

    #[test]
    fn test_rounding_errors_real_midi_values() {
        // Test with actual MIDI file values from our failing test
        let tempo = 500000; // Default tempo from the test
        let ticks_per_beat = 120; // From arabesque.mid
        
        // Testing with real MIDI values
        // Using real tempo and ticks per beat
        
        // Test the specific case from our failing test: 40 ticks duration
        let test_values_ticks = vec![40, 41, 120, 240, 480];
        
        for ticks in test_values_ticks {
            let seconds = tick2second(ticks, ticks_per_beat, tempo);
            let back_to_ticks = second2tick(seconds, ticks_per_beat, tempo);
            let ms = (seconds * 1000.0).round() as i32;
            
            // Tick conversion cycle test
            
            if ticks != back_to_ticks {
                // Rounding error in tick conversion
            }
        }
        
        // Test the specific failing case: 40 ticks at 120 TPQN
        // Testing specific failing case
        let original_duration_ticks = 40;
        let original_start_tick = 0;
        let original_end_tick = original_start_tick + original_duration_ticks;
        
        // Analyzing original timing
        
        // Convert to seconds (like tokenization does)
        let start_seconds = tick2second(original_start_tick, ticks_per_beat, tempo);
        let end_seconds = tick2second(original_end_tick, ticks_per_beat, tempo);
        let duration_seconds = end_seconds - start_seconds;
        let duration_ms = (duration_seconds * 1000.0).round() as i32;
        
        // Converting to milliseconds
        
        // Convert back to ticks (like detokenization does)
        let reconstructed_start_tick = second2tick(start_seconds, ticks_per_beat, tempo);
        let reconstructed_end_tick = second2tick(end_seconds, ticks_per_beat, tempo);
        let reconstructed_duration_ticks = reconstructed_end_tick - reconstructed_start_tick;
        
        // Analyzing reconstructed timing
        
        if original_duration_ticks != reconstructed_duration_ticks {
            // Duration mismatch in tick conversion
        }
    }

    #[test]
    fn test_tokenization_precision_loss() {
        use crate::aria::{midi_loader::load_midi_from_file, abs_tokenizer::AbsTokenizer, config::AbsConfig};
        
        // Tracing tokenization precision loss
        
        // Load arabesque.mid and examine the original note
        let midi_dict = load_midi_from_file("tests/assets/data/arabesque.mid")
            .expect("Failed to load arabesque.mid");
        
        let tokenizer = AbsTokenizer::new(AbsConfig::default());
        
        if !midi_dict.note_msgs.is_empty() {
            let first_note = &midi_dict.note_msgs[0];
            
            // Show first few tokens from our tokenization
            let tokens = tokenizer.tokenize_with_options(&midi_dict, false)
                .expect("Failed to tokenize");
            // Analyzing first tokens from Rust
            for (i, token) in tokens.iter().take(20).enumerate() {
                // Token analysis
            }
            let tempo = midi_dict.tempo_msgs.first().map(|t| t.data).unwrap_or(500000);
            let ticks_per_beat = midi_dict.ticks_per_beat;
            
            // Analyzing original note in MIDI file
            // Note timing analysis
            
            // Convert original ticks to milliseconds (what tokenization does)
            let start_seconds = tick2second(first_note.data.start, ticks_per_beat, tempo);
            let end_seconds = tick2second(first_note.data.end, ticks_per_beat, tempo);
            let duration_seconds = end_seconds - start_seconds;
            let duration_ms_exact = duration_seconds * 1000.0;
            
            // Tokenization timing calculations
            
            // Apply tokenizer quantization
            let tokenizer = AbsTokenizer::new(AbsConfig::default());
            let quantized_duration = tokenizer.quantize_dur(duration_ms_exact.round() as i32);
            
            // Quantization results
            
            // Debug the quantization arrays
            // Quantization configuration analysis
            
            // Find what values are closest to 167
            let test_values = [165, 166, 167, 168, 169, 170, 171];
            for &val in &test_values {
                let quantized = tokenizer.quantize_dur(val);
                // Quantization mapping
            }
            
            // Now convert back to ticks (what detokenization does)
            let reconstructed_duration_seconds = quantized_duration as f64 / 1000.0;
            let reconstructed_duration_ticks = second2tick(reconstructed_duration_seconds, ticks_per_beat, tempo);
            
            // Detokenization results
            
            let original_duration_ticks = first_note.data.end - first_note.data.start;
            if original_duration_ticks != reconstructed_duration_ticks {
                // Precision loss detected
                
                // Check where the precision loss occurs
                if (duration_ms_exact.round() as i32) != quantized_duration {
                    // Quantization loss analysis
                }
                
                let back_to_original_ms = tick2second(reconstructed_duration_ticks, ticks_per_beat, tempo) * 1000.0;
                if quantized_duration as f64 != back_to_original_ms {
                    // Tick conversion loss analysis
                }
            } else {
                // Perfect round trip achieved
            }
        }
    }

    #[test]
    fn test_find_closest_int_debug() {
        use crate::aria::tokenizer::BaseTokenizer;
        
                
        let quantizations = vec![0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190];
        
        // Test the specific problematic values
        let test_values = [165, 166, 167];
        
        for &value in &test_values {
                        
            // Find distances to nearby values manually
            let nearby: Vec<_> = quantizations.iter()
                .map(|&q| (q, (q - value as i32).abs()))
                .filter(|(_, dist)| *dist <= 10)
                .collect();
            
                        
            let result = BaseTokenizer::find_closest_int(value, &quantizations);
                        
            // Check if there are multiple values with the same minimum distance
            let min_distance = nearby.iter().map(|(_, dist)| *dist).min().unwrap_or(0);
            let closest_values: Vec<_> = nearby.iter()
                .filter(|(_, dist)| *dist == min_distance)
                .map(|(val, _)| *val)
                .collect();
            
            if closest_values.len() > 1 {
                                            }
        }
    }
}