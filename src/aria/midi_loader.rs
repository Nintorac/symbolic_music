use super::midi_types::*;
use std::io;
use std::collections::HashMap;
use midly::{Smf, TrackEventKind, MetaMessage as MidlyMetaMessage, MidiMessage as MidlyMidiMessage};

pub fn load_midi_from_file(path: &str) -> Result<MidiDict, io::Error> {
    // Load MIDI file using midly
    let data = std::fs::read(path)?;
    let smf = Smf::parse(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    let mut midi_dict = MidiDict::new();
    midi_dict.ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(ticks_per_beat) => ticks_per_beat.as_int() as i32,
        midly::Timing::Timecode(_, _) => 480, // Default fallback
    };
    
    // Add metadata
    midi_dict.metadata.insert("abs_load_path".to_string(), 
        std::path::Path::new(path).canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(path))
            .to_string_lossy().to_string());
    
    // Track note-on events to pair with note-off
    let mut note_on_events: HashMap<(u8, u8), Vec<(i32, u8)>> = HashMap::new(); // (channel, note) -> [(tick, velocity)]
    
    // Process all tracks
    for (track_idx, track) in smf.tracks.iter().enumerate() {
        let mut current_tick = 0;
        
        for event in track.iter() {
            current_tick += event.delta.as_int() as i32;
            
            match event.kind {
                TrackEventKind::Meta(meta_msg) => {
                    match meta_msg {
                        MidlyMetaMessage::Text(text) => {
                            if let Ok(text_str) = std::str::from_utf8(text) {
                                midi_dict.meta_msgs.push(MetaMessage {
                                    msg_type: "text".to_string(),
                                    data: text_str.to_string(),
                                    tick: current_tick,
                                });
                            }
                        }
                        MidlyMetaMessage::Copyright(text) => {
                            if let Ok(text_str) = std::str::from_utf8(text) {
                                midi_dict.meta_msgs.push(MetaMessage {
                                    msg_type: "copyright".to_string(),
                                    data: text_str.to_string(),
                                    tick: current_tick,
                                });
                            }
                        }
                        MidlyMetaMessage::Tempo(tempo) => {
                            midi_dict.tempo_msgs.push(TempoMessage {
                                msg_type: "tempo".to_string(),
                                data: tempo.as_int() as i32,
                                tick: current_tick,
                            });
                        }
                        _ => {}
                    }
                }
                TrackEventKind::Midi { channel, message } => {
                    match message {
                        MidlyMidiMessage::ProgramChange { program } => {
                            // Program change event
                            midi_dict.instrument_msgs.push(InstrumentMessage {
                                msg_type: "instrument".to_string(),
                                data: program.as_int() as i32,
                                tick: current_tick,
                                channel: channel.as_int() as i32, // Use actual MIDI channel
                            });
                        }
                        MidlyMidiMessage::Controller { controller, value } => {
                            if controller.as_int() == 64 { // Sustain pedal
                                midi_dict.pedal_msgs.push(PedalMessage {
                                    msg_type: "pedal".to_string(),
                                    data: if value.as_int() >= 64 { 1 } else { 0 },
                                    tick: current_tick,
                                    channel: channel.as_int() as i32, // Use actual MIDI channel
                                });
                            }
                        }
                        MidlyMidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() > 0 {
                                // Real note-on
                                let channel_note = (channel.as_int() as u8, key.as_int());
                                note_on_events.entry(channel_note)
                                    .or_insert_with(Vec::new)
                                    .push((current_tick, vel.as_int()));
                            } else {
                                // Note-on with velocity 0 is note-off
                                let channel_note = (channel.as_int() as u8, key.as_int());
                                if let Some(note_ons) = note_on_events.get_mut(&channel_note) {
                                    // Find notes to close (not starting at current tick)
                                    let mut notes_to_close = Vec::new();
                                    let mut notes_to_keep = Vec::new();
                                    
                                    for (start_tick, velocity) in note_ons.drain(..) {
                                        if start_tick != current_tick {
                                            notes_to_close.push((start_tick, velocity));
                                        } else {
                                            notes_to_keep.push((start_tick, velocity));
                                        }
                                    }
                                    
                                    // Close the notes
                                    for (start_tick, velocity) in notes_to_close {
                                        midi_dict.note_msgs.push(NoteMessage {
                                            msg_type: "note".to_string(),
                                            data: NoteData {
                                                pitch: key.as_int() as i32,
                                                start: start_tick,
                                                end: current_tick,
                                                velocity: velocity as i32,
                                            },
                                            tick: start_tick,
                                            channel: channel_note.0 as i32, // Use stored MIDI channel
                                        });
                                    }
                                    
                                    // Keep the notes that started at the same tick
                                    *note_ons = notes_to_keep;
                                    if note_ons.is_empty() {
                                        note_on_events.remove(&channel_note);
                                    }
                                }
                            }
                        }
                        MidlyMidiMessage::NoteOff { key, vel: _ } => {
                            let channel_note = (channel.as_int() as u8, key.as_int());
                            if let Some(note_ons) = note_on_events.get_mut(&channel_note) {
                                // Find notes to close (not starting at current tick)
                                let mut notes_to_close = Vec::new();
                                let mut notes_to_keep = Vec::new();
                                
                                for (start_tick, velocity) in note_ons.drain(..) {
                                    if start_tick != current_tick {
                                        notes_to_close.push((start_tick, velocity));
                                    } else {
                                        notes_to_keep.push((start_tick, velocity));
                                    }
                                }
                                
                                // Close the notes
                                for (start_tick, velocity) in notes_to_close {
                                    midi_dict.note_msgs.push(NoteMessage {
                                        msg_type: "note".to_string(),
                                        data: NoteData {
                                            pitch: key.as_int() as i32,
                                            start: start_tick,
                                            end: current_tick,
                                            velocity: velocity as i32,
                                        },
                                        tick: start_tick,
                                        channel: channel_note.0 as i32, // Use stored MIDI channel
                                    });
                                }
                                
                                // Keep the notes that started at the same tick
                                *note_ons = notes_to_keep;
                                if note_ons.is_empty() {
                                    note_on_events.remove(&channel_note);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    
    // If no tempo messages were found, add default
    if midi_dict.tempo_msgs.is_empty() {
        midi_dict.tempo_msgs.push(TempoMessage {
            msg_type: "tempo".to_string(),
            data: 500000, // 120 BPM
            tick: 0,
        });
    }
    
    // If no instrument messages were found, add default (piano)
    if midi_dict.instrument_msgs.is_empty() {
        midi_dict.instrument_msgs.push(InstrumentMessage {
            msg_type: "instrument".to_string(),
            data: 0, // Piano
            tick: 0,
            channel: 0,
        });
    }
    
    // Sort messages by tick (matching Python implementation)
    midi_dict.note_msgs.sort_by_key(|msg| msg.tick);
    midi_dict.tempo_msgs.sort_by_key(|msg| msg.tick);
    midi_dict.pedal_msgs.sort_by_key(|msg| msg.tick);
    midi_dict.instrument_msgs.sort_by_key(|msg| msg.tick);
    
    
    Ok(midi_dict)
}

pub fn normalize_midi_dict<F>(
    mut midi_dict: MidiDict,
    ignore_instruments: &HashMap<String, bool>,
    instrument_programs: &HashMap<String, i32>,
    time_step_ms: i32,
    max_duration_ms: i32,
    drum_velocity: i32,
    quantize_velocity_fn: F,
) -> MidiDict
where
    F: Fn(i32) -> i32,
{
    // Helper function to create channel mappings
    fn create_channel_mappings(
        midi_dict: &MidiDict,
        instruments: &[String],
    ) -> (HashMap<String, i32>, HashMap<i32, String>) {
        let mut new_instrument_to_channel = HashMap::new();
        for (idx, instrument) in instruments.iter().enumerate() {
            let channel = if instrument == "drum" {
                9
            } else if idx >= 9 {
                idx as i32 + 1
            } else {
                idx as i32
            };
            new_instrument_to_channel.insert(instrument.clone(), channel);
        }

        let mut old_channel_to_instrument = HashMap::new();
        for msg in &midi_dict.instrument_msgs {
            if let Some(instrument) = program_to_instrument(msg.data) {
                old_channel_to_instrument.insert(msg.channel, instrument);
            }
        }
        old_channel_to_instrument.insert(9, "drum".to_string());

        (new_instrument_to_channel, old_channel_to_instrument)
    }

    // Helper function to create instrument messages
    fn create_instrument_messages(
        instrument_programs: &HashMap<String, i32>,
        instrument_to_channel: &HashMap<String, i32>,
    ) -> Vec<InstrumentMessage> {
        let mut pairs: Vec<_> = instrument_to_channel.iter().collect();
        pairs.sort_by_key(|(_, &channel)| channel); // Sort by channel for deterministic order
        
        pairs
            .into_iter()
            .map(|(instrument, &channel)| InstrumentMessage {
                msg_type: "instrument".to_string(),
                data: if instrument == "drum" {
                    0
                } else {
                    *instrument_programs.get(instrument).unwrap_or(&0)
                },
                tick: 0,
                channel,
            })
            .collect()
    }

    // Helper function to normalize note messages
    fn normalize_note_messages<F>(
        midi_dict: &MidiDict,
        old_channel_to_instrument: &HashMap<i32, String>,
        new_instrument_to_channel: &HashMap<String, i32>,
        time_step_ms: i32,
        max_duration_ms: i32,
        drum_velocity: i32,
        quantize_velocity_fn: F,
    ) -> Vec<NoteMessage>
    where
        F: Fn(i32) -> i32,
    {
        // Helper function to quantize time
        fn quantize_time(n: i32, time_step_ms: i32) -> i32 {
            ((n as f64 / time_step_ms as f64).round() as i32) * time_step_ms
        }

        let mut note_msgs = Vec::new();
        for msg in &midi_dict.note_msgs {
            let msg_channel = msg.channel;
            if let Some(instrument) = old_channel_to_instrument.get(&msg_channel) {
                if let Some(&new_msg_channel) = new_instrument_to_channel.get(instrument) {
                    let start_tick = quantize_time(midi_dict.tick_to_ms(msg.data.start), time_step_ms);
                    let end_tick = quantize_time(midi_dict.tick_to_ms(msg.data.end), time_step_ms);
                    let velocity = quantize_velocity_fn(msg.data.velocity);

                    let mut new_msg = msg.clone();
                    new_msg.channel = new_msg_channel;
                    new_msg.tick = start_tick;
                    new_msg.data.start = start_tick;

                    if new_msg_channel != 9 {
                        // Non-drum instrument
                        new_msg.data.end = (start_tick + max_duration_ms).min(end_tick);
                        new_msg.data.velocity = velocity;
                    } else {
                        // Drum instrument
                        new_msg.data.end = start_tick + time_step_ms;
                        new_msg.data.velocity = drum_velocity;
                    }

                    note_msgs.push(new_msg);
                }
            }
        }

        note_msgs
    }

    // Remove unwanted instruments (stub - would need full implementation)
    // midi_dict.remove_instruments(ignore_instruments);

    // Resolve pedal effects (stub - would need full implementation)  
    midi_dict = midi_dict.resolve_pedal();
    
    // Clear pedal messages
    midi_dict.pedal_msgs.clear();

    // Create list of instruments to keep (sorted for deterministic order)
    let mut instruments: Vec<String> = ignore_instruments
        .iter()
        .filter(|(_, &ignored)| !ignored)
        .map(|(name, _)| name.clone())
        .collect();
    instruments.sort(); // Sort for deterministic order
    instruments.push("drum".to_string()); // Add drum at the end

    // Create channel mappings
    let (new_instrument_to_channel, old_channel_to_instrument) =
        create_channel_mappings(&midi_dict, &instruments);

    // Create new instrument messages
    let instrument_msgs = create_instrument_messages(instrument_programs, &new_instrument_to_channel);

    // Normalize note messages
    let note_msgs = normalize_note_messages(
        &midi_dict,
        &old_channel_to_instrument,
        &new_instrument_to_channel,
        time_step_ms,
        max_duration_ms,
        drum_velocity,
        quantize_velocity_fn,
    );

    // Create normalized MidiDict
    MidiDict {
        meta_msgs: Vec::new(),
        tempo_msgs: vec![TempoMessage {
            msg_type: "tempo".to_string(),
            data: 500000,
            tick: 0,
        }],
        pedal_msgs: Vec::new(),
        instrument_msgs,
        note_msgs,
        ticks_per_beat: 500,
        metadata: HashMap::new(),
    }
}