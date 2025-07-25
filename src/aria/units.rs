// SPDX-FileCopyrightText: 2017 Ole Martin Bjorndalen <ombdalen@gmail.com>
//
// SPDX-License-Identifier: MIT
// This file an LLM port of mido.units python file

/// Convert absolute time in ticks to seconds.
/// 
/// Returns absolute time in seconds for a chosen MIDI file time resolution
/// (ticks/pulses per quarter note, also called PPQN) and tempo (microseconds
/// per quarter note).
pub fn tick2second(tick: i32, ticks_per_beat: i32, tempo: i32) -> f64 {
    let scale = tempo as f64 * 1e-6 / ticks_per_beat as f64;
    tick as f64 * scale
}

/// Convert absolute time in seconds to ticks.
/// 
/// Returns absolute time in ticks for a chosen MIDI file time resolution
/// (ticks/pulses per quarter note, also called PPQN) and tempo (microseconds
/// per quarter note). Normal rounding applies.
pub fn second2tick(second: f64, ticks_per_beat: i32, tempo: i32) -> i32 {
    let scale = tempo as f64 * 1e-6 / ticks_per_beat as f64;
    (second / scale).round() as i32
}

/// Convert BPM (beats per minute) to MIDI file tempo (microseconds per
/// quarter note).
/// 
/// Depending on the chosen time signature a bar contains a different number of
/// beats. These beats are multiples/fractions of a quarter note, thus the
/// returned BPM depend on the time signature. Normal rounding applies.
pub fn bpm2tempo(bpm: f64, time_signature: (i32, i32)) -> i32 {
    (60.0 * 1e6 / bpm * time_signature.1 as f64 / 4.0).round() as i32
}

/// Convert MIDI file tempo (microseconds per quarter note) to BPM (beats
/// per minute).
/// 
/// Depending on the chosen time signature a bar contains a different number of
/// beats. The beats are multiples/fractions of a quarter note, thus the
/// returned tempo depends on the time signature denominator.
pub fn tempo2bpm(tempo: i32, time_signature: (i32, i32)) -> f64 {
    60.0 * 1e6 / tempo as f64 * time_signature.1 as f64 / 4.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick2second_basic() {
        // Test the specific case from our debug output
        let tick = 480;
        let ticks_per_beat = 480;
        let tempo = 500000;
        
        let result = tick2second(tick, ticks_per_beat, tempo);
        // Testing tick2second conversion
        
        // This should be 0.5 seconds (500ms)
        assert!((result - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_specific_failing_cases() {
        // Test the specific cases that are failing with 10ms offset
        let test_cases = vec![
            (560, 500000, 480, "Duration case: 560 ticks"),
            (4340, 500000, 480, "Onset case: 4340 ticks"), 
            (40, 500000, 120, "Simple 40 tick case"),
            (480, 500000, 480, "One beat case"),
        ];
        
        println!("\nRust timing calculations:");
        println!("========================================");
        
        for (ticks, tempo, ticks_per_beat, description) in test_cases {
            let seconds = tick2second(ticks, ticks_per_beat, tempo);
            let ms = seconds * 1000.0;
            let rounded_ms = ms.round() as i32;
            let roundtrip = second2tick(seconds, ticks_per_beat, tempo);
            
            println!("\n{}", description);
            println!("  Ticks: {}", ticks);
            println!("  Seconds: {}", seconds);
            println!("  Milliseconds: {}", ms);
            println!("  Rounded ms: {}", rounded_ms);
            println!("  Round-trip ticks: {}", roundtrip);
            println!("  Round-trip diff: {}", roundtrip - ticks);
        }
    }
    
    #[test]
    fn test_quantization_edge_cases() {
        use crate::aria::tokenizer::BaseTokenizer;
        
        // Test quantization around the failing edge cases
        let dur_quantizations: Vec<i32> = (0..=500).map(|i| i * 10).collect();
        let onset_quantizations: Vec<i32> = (0..500).map(|i| i * 10).collect();
        
        println!("\nRust quantization edge cases:");
        println!("========================================");
        
        // Test cases around our failing values
        let test_cases = vec![
            (555, &dur_quantizations, "Duration ~555ms"),
            (925, &dur_quantizations, "Duration ~925ms"), 
            (4345, &onset_quantizations, "Onset ~4345ms"),
        ];
        
        for (value, quantizations, description) in test_cases {
            let result = BaseTokenizer::find_closest_int(value, quantizations);
            println!("\n{}: {} -> {}", description, value, result);
            
            // Test nearby values to see where the boundary is
            for offset in [-5, -2, -1, 0, 1, 2, 5] {
                let test_val = value + offset;
                let test_result = BaseTokenizer::find_closest_int(test_val, quantizations);
                println!("  {} -> {}", test_val, test_result);
            }
        }
    }
}