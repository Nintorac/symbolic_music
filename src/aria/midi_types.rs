use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct NoteData {
    pub pitch: i32,
    pub start: i32,
    pub end: i32,
    pub velocity: i32,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct NoteMessage {
    pub msg_type: String,
    pub data: NoteData,
    pub tick: i32,
    pub channel: i32,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct TempoMessage {
    pub msg_type: String,
    pub data: i32,
    pub tick: i32,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct InstrumentMessage {
    pub msg_type: String,
    pub data: i32,
    pub tick: i32,
    pub channel: i32,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct PedalMessage {
    pub msg_type: String,
    pub data: i32,
    pub tick: i32,
    pub channel: i32,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct MetaMessage {
    pub msg_type: String,
    pub data: String,
    pub tick: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MidiDict {
    pub ticks_per_beat: i32,
    pub note_msgs: Vec<NoteMessage>,
    pub tempo_msgs: Vec<TempoMessage>,
    pub instrument_msgs: Vec<InstrumentMessage>,
    pub pedal_msgs: Vec<PedalMessage>,
    pub meta_msgs: Vec<MetaMessage>,
    pub metadata: HashMap<String, String>,
}

impl MidiDict {
    pub fn new() -> Self {
        Self {
            ticks_per_beat: 480,
            note_msgs: Vec::new(),
            tempo_msgs: Vec::new(),
            instrument_msgs: Vec::new(),
            pedal_msgs: Vec::new(),
            meta_msgs: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn ensure_default_tempo(&mut self) {
        // Add default tempo message if none exists (like Python implementation)
        if self.tempo_msgs.is_empty() {
            let default_tempo_msg = TempoMessage {
                msg_type: "tempo".to_string(),
                data: 500000, // 500,000 microseconds per quarter note (120 BPM)
                tick: 0,
            };
            self.tempo_msgs.push(default_tempo_msg);
        }
    }

    pub fn remove_instruments(&mut self, ignore_instruments: &HashMap<String, bool>) {
        // Find programs to remove based on ignore_instruments mapping
        let mut programs_to_remove = Vec::new();
        for program in 0..=127 {
            if let Some(instrument) = program_to_instrument(program) {
                if ignore_instruments.get(&instrument).unwrap_or(&false) == &true {
                    programs_to_remove.push(program);
                }
            }
        }
        
        // Find channels to remove based on instrument messages with those programs
        let mut channels_to_remove: std::collections::HashSet<i32> = std::collections::HashSet::new();
        for inst_msg in &self.instrument_msgs {
            if programs_to_remove.contains(&inst_msg.data) {
                channels_to_remove.insert(inst_msg.channel);
            }
        }
        
        // Remove drums (channel 9) from channels to remove
        channels_to_remove.remove(&9);
        
        // Remove unwanted messages from all message types
        self.note_msgs.retain(|msg| !channels_to_remove.contains(&msg.channel));
        self.instrument_msgs.retain(|msg| !channels_to_remove.contains(&msg.channel));
        self.pedal_msgs.retain(|msg| !channels_to_remove.contains(&msg.channel));
        // meta_msgs and tempo_msgs don't have channels, so we don't filter them
    }

    pub fn _build_pedal_intervals(&self) -> HashMap<i32, Vec<(i32, i32)>> {
        use std::collections::HashMap;
        
        // Sort pedal messages by tick
        let mut sorted_pedal_msgs = self.pedal_msgs.clone();
        sorted_pedal_msgs.sort_by_key(|msg| msg.tick);
        
        let mut channel_to_pedal_intervals: HashMap<i32, Vec<(i32, i32)>> = HashMap::new();
        let mut pedal_status: HashMap<i32, i32> = HashMap::new();
        
        // Initialize empty vectors for all channels that have notes
        for note_msg in &self.note_msgs {
            channel_to_pedal_intervals.entry(note_msg.channel).or_insert_with(Vec::new);
        }
        
        for pedal_msg in &sorted_pedal_msgs {
            let tick = pedal_msg.tick;
            let channel = pedal_msg.channel;
            let data = pedal_msg.data;
            
            if data == 1 && !pedal_status.contains_key(&channel) {
                // Start pedal interval
                pedal_status.insert(channel, tick);
            } else if data == 0 && pedal_status.contains_key(&channel) {
                // Close pedal interval
                let start_tick = pedal_status.remove(&channel).unwrap();
                let end_tick = tick;
                channel_to_pedal_intervals
                    .entry(channel)
                    .or_insert_with(Vec::new)
                    .push((start_tick, end_tick));
            }
        }
        
        // Close all unclosed pedals at end of track
        // NOTE: Our version uses max() which is more correct, but we match Python's bug
        // where it uses the last note's end time instead of the actual final time
        if !self.note_msgs.is_empty() {
            // Correct version (commented out to match Python):
            // let final_tick = self.note_msgs.iter().map(|msg| msg.data.end).max().unwrap_or(0);
            
            // Python bug version - uses last note by position, not actual final note end time:
            let final_tick = self.note_msgs.last().unwrap().data.end;
            
            for (channel, start_tick) in pedal_status {
                channel_to_pedal_intervals
                    .entry(channel)
                    .or_insert_with(Vec::new)
                    .push((start_tick, final_tick));
            }
        }
        
        channel_to_pedal_intervals
    }

    pub fn to_midi(&self) -> MidiFile {
        MidiFile { 
            midi_dict: self.clone(),
        }
    }

    pub fn tick_to_ms(&self, tick: i32) -> i32 {
        get_duration_ms(0, tick, &self.tempo_msgs, self.ticks_per_beat)
    }

    pub fn calculate_hash(&self) -> u64 {
        // Hash only the musical content, not metadata/meta_msgs/ticks_per_beat
        let mut hasher = DefaultHasher::new();
        self.note_msgs.hash(&mut hasher);
        self.tempo_msgs.hash(&mut hasher);
        self.pedal_msgs.hash(&mut hasher);
        self.instrument_msgs.hash(&mut hasher);
        hasher.finish()
    }

    pub fn resolve_pedal(self) -> Self {
        // Stub implementation - return self unchanged
        self
    }

    pub fn remove_redundant_pedals(self) -> Self {
        // Stub implementation - return self unchanged
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Special(String),                          // "<S>", "<E>", "<T>", etc.
    Prefix(String, String, String),           // ("prefix", "instrument", "piano")
    Note(String, i32, i32),                   // ("piano", 60, 80)
    Onset(String, i32),                       // ("onset", 100)
    Duration(String, i32),                    // ("dur", 500)  
    Drum(String, i32),                        // ("drum", 36)
}

// MIDI file type for to_midi() method
#[derive(Debug, Clone)]
pub struct MidiFile {
    pub midi_dict: MidiDict,
}

impl MidiFile {
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        use midly::{Smf, Header, Timing, Format, Track, TrackEvent, TrackEventKind, MetaMessage as MidlyMetaMessage, MidiMessage as MidlyMidiMessage};
        
        // Create header for multi-track format
        let header = Header {
            format: Format::Parallel,
            timing: Timing::Metrical(midly::num::u15::from(self.midi_dict.ticks_per_beat as u16)),
        };
        
        // Create separate tracks - one for tempo/meta and one for each instrument
        let mut track_events: std::collections::HashMap<u8, Vec<(i32, TrackEvent)>> = std::collections::HashMap::new();
        
        // Initialize track 0 for tempo and meta events
        track_events.insert(0, Vec::new());
        
        // Sort instruments by program number for consistent track ordering
        let mut sorted_instruments: Vec<(usize, &InstrumentMessage)> = self.midi_dict.instrument_msgs.iter().enumerate().collect();
        sorted_instruments.sort_by_key(|(_, inst)| inst.data);
        
        // Create mapping from instrument channel to track index
        let mut channel_to_track: std::collections::HashMap<i32, u8> = std::collections::HashMap::new();
        let mut channel_to_program: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();
        
        // Create a track for each instrument (starting from track 1), ordered by program number
        for (track_idx, (_, inst_msg)) in sorted_instruments.iter().enumerate() {
            let track_num = (track_idx + 1) as u8;
            track_events.insert(track_num, Vec::new());
            channel_to_track.insert(inst_msg.channel, track_num);
            channel_to_program.insert(inst_msg.channel, inst_msg.data);
        }
        
        // Add tempo events to track 0
        for tempo_msg in &self.midi_dict.tempo_msgs {
            let event = TrackEvent {
                delta: midly::num::u28::from(0), // Will be calculated later
                kind: TrackEventKind::Meta(MidlyMetaMessage::Tempo(
                    midly::num::u24::from(tempo_msg.data as u32)
                )),
            };
            track_events.get_mut(&0).unwrap().push((tempo_msg.tick, event));
        }
        
        // Add meta events to track 0
        for meta_msg in &self.midi_dict.meta_msgs {
            let event = match meta_msg.msg_type.as_str() {
                "text" => TrackEvent {
                    delta: midly::num::u28::from(0),
                    kind: TrackEventKind::Meta(MidlyMetaMessage::Text(meta_msg.data.as_bytes())),
                },
                "copyright" => TrackEvent {
                    delta: midly::num::u28::from(0),
                    kind: TrackEventKind::Meta(MidlyMetaMessage::Copyright(meta_msg.data.as_bytes())),
                },
                _ => continue,
            };
            track_events.get_mut(&0).unwrap().push((meta_msg.tick, event));
        }
        
        // Add instrument events to appropriate tracks (ordered by program number)
        for (track_idx, (_, inst_msg)) in sorted_instruments.iter().enumerate() {
            let track_num = (track_idx + 1) as u8; // Track 0 is for tempo/meta
            
            let event = TrackEvent {
                delta: midly::num::u28::from(0),
                kind: TrackEventKind::Midi {
                    channel: midly::num::u4::from(inst_msg.channel as u8),
                    message: MidlyMidiMessage::ProgramChange {
                        program: midly::num::u7::from(inst_msg.data as u8),
                    },
                },
            };
            track_events.get_mut(&track_num).unwrap().push((inst_msg.tick, event));
        }
        
        // Add pedal events to appropriate tracks based on their instrument channel
        for pedal_msg in &self.midi_dict.pedal_msgs {
            if let Some(&track_idx) = channel_to_track.get(&pedal_msg.channel) {
                let event = TrackEvent {
                    delta: midly::num::u28::from(0),
                    kind: TrackEventKind::Midi {
                        channel: midly::num::u4::from(pedal_msg.channel as u8),
                        message: MidlyMidiMessage::Controller {
                            controller: midly::num::u7::from(64), // Sustain pedal
                            value: midly::num::u7::from(if pedal_msg.data > 0 { 127 } else { 0 }),
                        },
                    },
                };
                track_events.get_mut(&track_idx).unwrap().push((pedal_msg.tick, event));
            }
        }
        
        // Add note events (convert to note-on/note-off pairs) to appropriate tracks based on their instrument channel
        for note_msg in &self.midi_dict.note_msgs {
            if let Some(&track_idx) = channel_to_track.get(&note_msg.channel) {
                // Note-on event
                let note_on = TrackEvent {
                    delta: midly::num::u28::from(0),
                    kind: TrackEventKind::Midi {
                        channel: midly::num::u4::from(note_msg.channel as u8),
                        message: MidlyMidiMessage::NoteOn {
                            key: midly::num::u7::from(note_msg.data.pitch as u8),
                            vel: midly::num::u7::from(note_msg.data.velocity as u8),
                        },
                    },
                };
                track_events.get_mut(&track_idx).unwrap().push((note_msg.data.start, note_on));
                
                // Note-off event
                let note_off = TrackEvent {
                    delta: midly::num::u28::from(0),
                    kind: TrackEventKind::Midi {
                        channel: midly::num::u4::from(note_msg.channel as u8),
                        message: MidlyMidiMessage::NoteOff {
                            key: midly::num::u7::from(note_msg.data.pitch as u8),
                            vel: midly::num::u7::from(64), // Default release velocity
                        },
                    },
                };
                track_events.get_mut(&track_idx).unwrap().push((note_msg.data.end, note_off));
            }
        }
        
        // Convert track events to tracks
        let mut tracks = Vec::new();
        let mut track_keys: Vec<u8> = track_events.keys().cloned().collect();
        track_keys.sort();
        
        for track_key in track_keys {
            let mut events = track_events.remove(&track_key).unwrap();
            
            // Sort events by tick
            events.sort_by_key(|(tick, _)| *tick);
            
            // Create track
            let mut track = Track::new();
            
            // Calculate delta times and add to track
            let mut last_tick = 0;
            for (tick, mut event) in events {
                let delta = (tick - last_tick).max(0) as u32;
                event.delta = midly::num::u28::from(delta);
                track.push(event);
                last_tick = tick;
            }
            
            // Add end of track event
            track.push(TrackEvent {
                delta: midly::num::u28::from(0),
                kind: TrackEventKind::Meta(MidlyMetaMessage::EndOfTrack),
            });
            
            tracks.push(track);
        }
        
        // Create SMF and write to file
        let smf = Smf {
            header,
            tracks,
        };
        
        let mut bytes = Vec::new();
        smf.write(&mut bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, bytes)?;
        
        Ok(())
    }
}

pub fn program_to_instrument(program: i32) -> Option<String> {
    match program {
        0..=7 => Some("piano".to_string()),
        8..=15 => Some("chromatic".to_string()),
        16..=23 => Some("organ".to_string()),
        24..=31 => Some("guitar".to_string()),
        32..=39 => Some("bass".to_string()),
        40..=47 => Some("strings".to_string()),
        48..=55 => Some("ensemble".to_string()),
        56..=63 => Some("brass".to_string()),
        64..=71 => Some("reed".to_string()),
        72..=79 => Some("pipe".to_string()),
        80..=87 => Some("synth_lead".to_string()),
        88..=95 => Some("synth_pad".to_string()),
        96..=103 => Some("synth_effect".to_string()),
        104..=111 => Some("ethnic".to_string()),
        112..=119 => Some("percussive".to_string()),
        120..=127 => Some("sfx".to_string()),
        _ => None, // Invalid program number
    }
}

pub fn get_duration_ms(
    start_tick: i32,
    end_tick: i32,
    tempo_msgs: &[TempoMessage],
    ticks_per_beat: i32,
) -> i32 {
    use super::units::tick2second;
    
    // Find the tempo message index for start_tick
    // Python logic: find idx such that tempo_msg[idx]["tick"] < start_tick <= tempo_msg[idx+1]["tick"]
    // The Python loop finds the first tempo message where start_tick <= tempo_msg["tick"], then decrements if idx > 0
    let mut idx = 0;
    for (i, tempo_msg) in tempo_msgs.iter().enumerate() {
        if start_tick <= tempo_msg.tick {
            idx = i;
            break;
        }
        idx = i;
    }
    
    // Match Python logic exactly: if idx > 0, decrement by 1 (unless we're at the very first tempo)
    if idx > 0 {
        idx -= 1;
    }

    let mut duration_s = 0.0;
    let mut curr_tick = start_tick;
    let curr_tempo = if idx < tempo_msgs.len() { tempo_msgs[idx].data } else { 500000 };

    // Sum all tempo intervals
    for i in idx..tempo_msgs.len() {
        let tempo_msg = &tempo_msgs[i];
        let next_tempo_tick = if i + 1 < tempo_msgs.len() { tempo_msgs[i + 1].tick } else { i32::MAX };
        
        let delta_tick = if end_tick < next_tempo_tick {
            end_tick - curr_tick
        } else {
            next_tempo_tick - curr_tick
        };

        if delta_tick > 0 {
            duration_s += tick2second(delta_tick, ticks_per_beat, tempo_msg.data);
        }

        if end_tick < next_tempo_tick {
            curr_tick = end_tick;
            break;
        }
        curr_tick = next_tempo_tick;
    }

    // Handle case where end_tick is beyond the last tempo change
    if curr_tick < end_tick {
        let delta_tick = end_tick - curr_tick;
        if delta_tick > 0 {
            duration_s += tick2second(delta_tick, ticks_per_beat, curr_tempo);
        }
    }

    // Convert to milliseconds and round
    let duration_ms = (duration_s * 1000.0).round() as i32;
    
    duration_ms
}