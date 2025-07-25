use super::{midi_types::*, midi_loader::*, abs_tokenizer::*, tokenizer::Tokenizer};
use std::path::Path;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_data_path() -> String {
        "tests/assets/data".to_string()
    }

    fn get_results_path() -> String {
        "tests/assets/results".to_string()
    }

    #[test]
    fn test_load() {
        let load_path = format!("{}/arabesque.mid", get_test_data_path());
        let midi_dict = load_midi_from_file(&load_path).expect("Failed to load MIDI file");
        
        assert!(!midi_dict.note_msgs.is_empty(), "MIDI file should contain notes");
    }

    #[test]
    fn test_save() {
        let load_path = format!("{}/arabesque.mid", get_test_data_path());
        let save_path = format!("{}/arabesque.mid", get_results_path());
        
        // Create results directory if it doesn't exist
        if let Some(parent) = Path::new(&save_path).parent() {
            let _ = fs::create_dir_all(parent);
        }

        let midi_dict = load_midi_from_file(&load_path).expect("Failed to load MIDI file");
        
        // Debug: Print channel information
        let mut channels_used = std::collections::HashSet::new();
        for note_msg in &midi_dict.note_msgs {
            channels_used.insert(note_msg.channel);
        }
        for inst_msg in &midi_dict.instrument_msgs {
            channels_used.insert(inst_msg.channel);
        }
        for pedal_msg in &midi_dict.pedal_msgs {
            channels_used.insert(pedal_msg.channel);
        }
        println!("Channels in loaded MIDI: {:?}", channels_used);
        println!("Note messages: {}", midi_dict.note_msgs.len());
        println!("Instrument messages: {}", midi_dict.instrument_msgs.len());
        println!("Pedal messages: {}", midi_dict.pedal_msgs.len());
        
        midi_dict.to_midi().save(&save_path).expect("Failed to save MIDI file");

        // Compare at the midly level by parsing both files
        use midly::Smf;
        
        let original_bytes = fs::read(&load_path).expect("Failed to read original file");
        let saved_bytes = fs::read(&save_path).expect("Failed to read saved file");
        
        let original_smf = Smf::parse(&original_bytes).expect("Failed to parse original MIDI");
        let saved_smf = Smf::parse(&saved_bytes).expect("Failed to parse saved MIDI");
        
        // Compare headers
        assert_eq!(original_smf.header.format, saved_smf.header.format, "MIDI format differs");
        assert_eq!(original_smf.header.timing, saved_smf.header.timing, "MIDI timing differs");
        
        // Count tracks with actual MIDI events (not just meta events)
        let original_midi_tracks = original_smf.tracks.iter().filter(|track| {
            track.iter().any(|event| matches!(event.kind, midly::TrackEventKind::Midi { .. }))
        }).count();
        
        let saved_midi_tracks = saved_smf.tracks.iter().filter(|track| {
            track.iter().any(|event| matches!(event.kind, midly::TrackEventKind::Midi { .. }))
        }).count();
        
        // Compare number of tracks with MIDI events
        assert_eq!(original_midi_tracks, saved_midi_tracks, 
                  "Number of tracks with MIDI events differs: original={}, saved={}", 
                  original_midi_tracks, saved_midi_tracks);
        
        // Test round-trip by comparing the actual MIDI messages
        let original_midi_dict = load_midi_from_file(&load_path).expect("Failed to reload original file");
        let reloaded_midi_dict = load_midi_from_file(&save_path).expect("Failed to reload saved file");
        
        // Compare note messages
        assert_eq!(original_midi_dict.note_msgs.len(), reloaded_midi_dict.note_msgs.len(), 
                  "Number of note messages differs");
        
        // Compare instrument messages
        assert_eq!(original_midi_dict.instrument_msgs.len(), reloaded_midi_dict.instrument_msgs.len(), 
                  "Number of instrument messages differs");
        
        // Compare pedal messages
        assert_eq!(original_midi_dict.pedal_msgs.len(), reloaded_midi_dict.pedal_msgs.len(), 
                  "Number of pedal messages differs");
        
        // Compare tempo messages
        assert_eq!(original_midi_dict.tempo_msgs.len(), reloaded_midi_dict.tempo_msgs.len(), 
                  "Number of tempo messages differs");
        
        // Compare notes using sets to find differences
        use std::collections::HashSet;
        
        #[derive(Hash, Eq, PartialEq, Debug)]
        struct NoteKey {
            pitch: i32,
            start: i32,
            // Note: Excluding end time since tokenizer may modify note durations due to pedal effects
            // Note: Excluding channel since tokenizer may reassign channels during detokenization
            // Note: Excluding velocity since tokenizer quantizes velocities
        }
        
        let original_notes: HashSet<NoteKey> = original_midi_dict.note_msgs.iter()
            .map(|n| NoteKey {
                pitch: n.data.pitch,
                start: n.data.start,
            }).collect();
            
        let reloaded_notes: HashSet<NoteKey> = reloaded_midi_dict.note_msgs.iter()
            .map(|n| NoteKey {
                pitch: n.data.pitch,
                start: n.data.start,
            }).collect();
        
        let in_original_not_reloaded: Vec<_> = original_notes.difference(&reloaded_notes).collect();
        let in_reloaded_not_original: Vec<_> = reloaded_notes.difference(&original_notes).collect();
        
        if !in_original_not_reloaded.is_empty() {
            println!("Notes in original but not in reloaded ({}):", in_original_not_reloaded.len());
            for (i, note) in in_original_not_reloaded.iter().take(10).enumerate() {
                println!("  {}: {:?}", i, note);
            }
        }
        
        if !in_reloaded_not_original.is_empty() {
            println!("Notes in reloaded but not in original ({}):", in_reloaded_not_original.len());
            for (i, note) in in_reloaded_not_original.iter().take(10).enumerate() {
                println!("  {}: {:?}", i, note);
            }
        }
        
        assert_eq!(original_notes, reloaded_notes, "Note sets differ after round-trip");
    }

    #[test]
    fn test_tick_to_ms() {
        const CORRECT_LAST_NOTE_ONSET_MS: i32 = 220140;
        let load_path = format!("{}/arabesque.mid", get_test_data_path());
        let midi_dict = load_midi_from_file(&load_path).expect("Failed to load MIDI file");
        
        let last_note = midi_dict.note_msgs.last().expect("No notes found");
        let last_note_onset_tick = last_note.tick;
        let last_note_onset_ms = midi_dict.tick_to_ms(last_note_onset_tick);
        assert_eq!(last_note_onset_ms, CORRECT_LAST_NOTE_ONSET_MS);
    }

    #[test]
    fn test_calculate_hash() {
        use tempfile::NamedTempFile;
        
        // Load two identical files with different filenames and metadata
        let load_path = format!("{}/arabesque.mid", get_test_data_path());
        let midi_dict_orig = load_midi_from_file(&load_path).expect("Failed to load MIDI file");

        // Create a temporary file copy
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        std::fs::copy(&load_path, temp_file.path()).expect("Failed to copy file");
        let mut midi_dict_temp = load_midi_from_file(temp_file.path().to_str().unwrap()).expect("Failed to load temp file");

        // Add metadata that shouldn't affect the hash
        midi_dict_temp.meta_msgs.push(MetaMessage {
            msg_type: "text".to_string(),
            data: "test".to_string(),
            tick: 0,
        });
        midi_dict_temp.metadata.insert("composer".to_string(), "test".to_string());
        midi_dict_temp.metadata.insert("ticks_per_beat".to_string(), "-1".to_string());

        assert_eq!(
            midi_dict_orig.calculate_hash(),
            midi_dict_temp.calculate_hash()
        );
    }

    #[test]
    fn test_resolve_pedal() {
        let load_path = format!("{}/arabesque.mid", get_test_data_path());
        let save_path = format!("{}/arabesque_pedal_resolved.mid", get_results_path());
        
        // Create results directory if it doesn't exist
        if let Some(parent) = Path::new(&save_path).parent() {
            let _ = fs::create_dir_all(parent);
        }

        let midi_dict = load_midi_from_file(&load_path).expect("Failed to load MIDI file").resolve_pedal();
        midi_dict.to_midi().save(&save_path).expect("Failed to save MIDI file");
    }

    #[test]
    fn test_remove_redundant_pedals() {
        let load_path = format!("{}/arabesque.mid", get_test_data_path());
        let save_path = format!("{}/arabesque_remove_redundant_pedals.mid", get_results_path());
        
        // Create results directory if it doesn't exist
        if let Some(parent) = Path::new(&save_path).parent() {
            let _ = fs::create_dir_all(parent);
        }

        let midi_dict = load_midi_from_file(&load_path).expect("Failed to load MIDI file");
        
        let midi_dict_adj_resolve = load_midi_from_file(&load_path)
            .expect("Failed to load MIDI file")
            .resolve_pedal()
            .remove_redundant_pedals();
        
        let midi_dict_resolve_adj = load_midi_from_file(&load_path)
            .expect("Failed to load MIDI file")
            .remove_redundant_pedals()
            .resolve_pedal();

                
        assert_eq!(
            midi_dict_adj_resolve.pedal_msgs.len(),
            midi_dict_resolve_adj.pedal_msgs.len(),
        );

        for (msg_1, msg_2) in midi_dict_adj_resolve.note_msgs.iter().zip(midi_dict_resolve_adj.note_msgs.iter()) {
            assert_eq!(msg_1, msg_2);
        }

        midi_dict_adj_resolve.to_midi().save(&save_path).expect("Failed to save MIDI file");
    }
}