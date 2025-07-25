use super::tokenizer::*;
use super::midi_types::*;
use super::config::*;
use std::collections::HashMap;
use std::collections::HashSet;
use rand::Rng;

#[derive(Clone, Debug)]
struct NoteBuffer {
    note: Token,
    onset: Token,
    dur: Option<Token>,
}

pub struct AbsTokenizer {
    pub config: AbsConfig,
    pub name: String,
    pub base_tokenizer: BaseTokenizer,
    pub abs_time_step_ms: i32,
    pub max_dur_ms: i32,
    pub time_step_ms: i32,
    pub dur_time_quantizations: Vec<i32>,
    pub onset_time_quantizations: Vec<i32>,
    pub velocity_step: i32,
    pub velocity_quantizations: Vec<i32>,
    pub max_velocity: i32,
    pub instruments_nd: Vec<String>,
    pub instruments_wd: Vec<String>,
    pub prefix_tokens: Vec<Token>,
    pub time_tok: String,
    pub onset_tokens: Vec<Token>,
    pub dur_tokens: Vec<Token>,
    pub drum_tokens: Vec<Token>,
    pub note_tokens: Vec<Token>,
    pub pad_id: i32,
}

impl AbsTokenizer {
    pub fn new(config: AbsConfig) -> Self {
        let mut base_tokenizer = BaseTokenizer::new();
        
        // Calculate time quantizations (in ms)
        let abs_time_step_ms = config.abs_time_step_ms;
        let max_dur_ms = config.max_dur_ms;
        let time_step_ms = config.time_step_ms;
        
        let dur_time_quantizations: Vec<i32> = (0..=(max_dur_ms / time_step_ms))
            .map(|i| time_step_ms * i)
            .collect();
        let onset_time_quantizations: Vec<i32> = (0..(max_dur_ms / time_step_ms))
            .map(|i| time_step_ms * i)
            .collect();
        
        // Calculate velocity quantizations
        let velocity_step = config.velocity_quantization_step;
        let velocity_quantizations: Vec<i32> = (0..=(127 / velocity_step))
            .map(|i| i * velocity_step)
            .collect();
        let max_velocity = *velocity_quantizations.last().unwrap_or(&127);
        
        // _nd = no drum; _wd = with drum
        let instruments_nd: Vec<String> = config.ignore_instruments
            .iter()
            .filter(|(_, &ignore)| !ignore)
            .map(|(k, _)| k.clone())
            .collect();
        let mut instruments_wd = instruments_nd.clone();
        instruments_wd.push("drum".to_string());
        
        // Prefix tokens - match Python format: ("prefix", "instrument", x)
        let mut prefix_tokens: Vec<Token> = instruments_wd.iter()
            .map(|x| Token::Prefix("prefix".to_string(), "instrument".to_string(), x.clone()))
            .collect();
        
        // Add composer, form, genre prefix tokens
        for composer in &config.composer_names {
            prefix_tokens.push(Token::Prefix("prefix".to_string(), "composer".to_string(), composer.clone()));
        }
        for form in &config.form_names {
            prefix_tokens.push(Token::Prefix("prefix".to_string(), "form".to_string(), form.clone()));
        }
        for genre in &config.genre_names {
            prefix_tokens.push(Token::Prefix("prefix".to_string(), "genre".to_string(), genre.clone()));
        }
        
        // Build vocab
        let time_tok = "<T>".to_string();
        let onset_tokens: Vec<Token> = onset_time_quantizations.iter()
            .map(|&i| Token::Onset("onset".to_string(), i))
            .collect();
        let dur_tokens: Vec<Token> = dur_time_quantizations.iter()
            .map(|&i| Token::Duration("dur".to_string(), i))
            .collect();
        let drum_tokens: Vec<Token> = (35..82)
            .map(|i| Token::Drum("drum".to_string(), i))
            .collect();
        
        let mut note_tokens = Vec::new();
        for instrument in &instruments_nd {
            for pitch in 0..128 {
                for &velocity in &velocity_quantizations {
                    note_tokens.push(Token::Note(instrument.clone(), pitch, velocity));
                }
            }
        }
        
        // Add time token to special tokens
        base_tokenizer.special_tokens.push(time_tok.clone());
        
        // Build vocabulary
        let mut all_tokens = base_tokenizer.special_tokens.iter()
            .map(|s| Token::Special(s.clone()))
            .collect::<Vec<Token>>();
        all_tokens.extend(prefix_tokens.clone());
        all_tokens.extend(note_tokens.clone());
        all_tokens.extend(drum_tokens.clone());
        all_tokens.extend(dur_tokens.clone());
        all_tokens.extend(onset_tokens.clone());
        
        base_tokenizer.add_tokens_to_vocab(all_tokens);
        let pad_id = base_tokenizer.pad_id;
        
        Self {
            config,
            name: "abs".to_string(),
            base_tokenizer,
            abs_time_step_ms,
            max_dur_ms,
            time_step_ms,
            dur_time_quantizations,
            onset_time_quantizations,
            velocity_step,
            velocity_quantizations,
            max_velocity,
            instruments_nd,
            instruments_wd,
            prefix_tokens,
            time_tok,
            onset_tokens,
            dur_tokens,
            drum_tokens,
            note_tokens,
            pad_id,
        }
    }
    
    pub fn new_with_default_config() -> Self {
        let config = Config::load_default();
        Self::new(config.tokenizer.abs)
    }
    
    pub fn quantize_dur(&self, time: i32) -> i32 {
        let dur = BaseTokenizer::find_closest_int(time, &self.dur_time_quantizations);
        if dur != 0 { dur } else { self.time_step_ms }
    }
    
    fn quantize_onset(&self, time: i32) -> i32 {
        BaseTokenizer::find_closest_int(time, &self.onset_time_quantizations)
    }
    
    pub fn quantize_velocity(&self, velocity: i32) -> i32 {
        let velocity_quantized = BaseTokenizer::find_closest_int(velocity, &self.velocity_quantizations);
        if velocity_quantized == 0 && velocity != 0 {
            self.velocity_step
        } else {
            velocity_quantized
        }
    }
    
    fn format_tokens(&self, prefix: Vec<Token>, unformatted_seq: Vec<Token>, add_dim_tok: bool, add_eos_tok: bool) -> Vec<Token> {
        let mut result = prefix;
        result.push(Token::Special(self.base_tokenizer.bos_tok.clone()));
        
        let mut seq = unformatted_seq;
        
        // Add diminish token if sequence is long enough
        if seq.len() > 150 && add_dim_tok {
            let mut idx = seq.len().saturating_sub(100);
            
            // Make sure we don't break up Note->Onset->Duration or Drum->Onset sequences
            // Move backwards to find a safe insertion point (matching Python logic)
            while idx > 0 && idx < seq.len() {
                match &seq[idx] {
                    Token::Onset(_, _) => {
                        // Don't want: note/drum, <D>, onset
                        idx = idx.saturating_sub(1); // Move before the note/drum
                    }
                    Token::Duration(_, _) => {
                        // Don't want: note, onset, <D>, dur
                        idx = idx.saturating_sub(2); // Move before note->onset
                    }
                    _ => break, // Safe to insert here
                }
            }
            
            seq.insert(idx, Token::Special(self.base_tokenizer.dim_tok.clone()));
        }
        
        result.extend(seq);
        
        if add_eos_tok {
            result.push(Token::Special(self.base_tokenizer.eos_tok.clone()));
        }
        
        result
    }
    
    pub fn export_pitch_aug(&self, max_pitch_aug: i32) -> impl Fn(&[Token], Option<i32>) -> Vec<Token> {
        let unk_tok = Token::Special(self.base_tokenizer.unk_tok.clone());
        
        move |tokens: &[Token], pitch_aug: Option<i32>| -> Vec<Token> {
            // If pitch_aug is None, generate random value in range [-max_pitch_aug, max_pitch_aug]
            let pitch_aug = match pitch_aug {
                Some(val) => val,
                None => {
                    if max_pitch_aug > 0 {
                        let mut rng = rand::thread_rng();
                        rng.gen_range(-max_pitch_aug..=max_pitch_aug)
                    } else {
                        0
                    }
                }
            };
            
            tokens.iter().map(|tok| {
                match tok {
                    Token::Note(instrument, pitch, velocity) => {
                        let new_pitch = pitch + pitch_aug;
                        // Python logic: check if pitch is in valid range [0, 127]
                        if new_pitch >= 0 && new_pitch <= 127 {
                            Token::Note(instrument.clone(), new_pitch, *velocity)
                        } else {
                            unk_tok.clone()
                        }
                    }
                    _ => tok.clone()
                }
            }).collect()
        }
    }
    
    pub fn export_velocity_aug(&self, max_num_aug_steps: i32) -> impl Fn(&[Token], Option<i32>) -> Vec<Token> {
        let velocity_step = self.config.velocity_quantization_step;
        let max_velocity = self.max_velocity;
        
        move |tokens: &[Token], aug_step: Option<i32>| -> Vec<Token> {
            // If aug_step is None, generate random value in range [-max_num_aug_steps, max_num_aug_steps]
            let aug_step = match aug_step {
                Some(val) => val,
                None => {
                    if max_num_aug_steps > 0 {
                        let mut rng = rand::thread_rng();
                        rng.gen_range(-max_num_aug_steps..=max_num_aug_steps)
                    } else {
                        0
                    }
                }
            };
            let velocity_aug = aug_step * velocity_step;
            
            tokens.iter().map(|tok| {
                match tok {
                    Token::Note(instrument, pitch, velocity) => {
                        // Python logic: min(max(velocity + velocity_aug, velocity_step), max_velocity)
                        let new_velocity = (velocity + velocity_aug).max(velocity_step).min(max_velocity);
                        Token::Note(instrument.clone(), *pitch, new_velocity)
                    }
                    _ => tok.clone()
                }
            }).collect()
        }
    }
    
    pub fn export_tempo_aug(&self, max_tempo_aug: f64, mixup: bool) -> impl Fn(&[Token], Option<f64>) -> Vec<Token> {
        let abs_time_step_ms = self.abs_time_step_ms;
        let max_dur_ms = self.max_dur_ms;
        let time_step_ms = self.time_step_ms;
        let instruments_wd = self.instruments_wd.clone();
        let time_tok = Token::Special(self.time_tok.clone());
        let unk_tok = Token::Special(self.base_tokenizer.unk_tok.clone());
        let bos_tok = Token::Special(self.base_tokenizer.bos_tok.clone());
        let eos_tok = Token::Special(self.base_tokenizer.eos_tok.clone());
        let dim_tok = Token::Special(self.base_tokenizer.dim_tok.clone());
        
        move |tokens: &[Token], tempo_aug: Option<f64>| -> Vec<Token> {
            // If tempo_aug is None, generate random value in range [1-max_tempo_aug, 1+max_tempo_aug]
            let tempo_aug = match tempo_aug {
                Some(val) => val,
                None => {
                    if max_tempo_aug > 0.0 {
                        let mut rng = rand::thread_rng();
                        rng.gen_range((1.0 - max_tempo_aug)..=(1.0 + max_tempo_aug))
                    } else {
                        1.0
                    }
                }
            };
            // Helper function to quantize time
            let quantize_time = |n: f64| -> i32 {
                (n / time_step_ms as f64).round() as i32 * time_step_ms
            };
            
            // Data structure to hold notes organized by time_tok_count and onset
            // time_tok_count -> onset_time -> list of note structures
            let mut buffer: std::collections::HashMap<i32, std::collections::HashMap<i32, Vec<NoteBuffer>>> = 
                std::collections::HashMap::new();
            
            let mut src_time_tok_cnt = 0;
            let mut dim_tok_seen: Option<(i32, i32)> = None;
            let mut result: Vec<Token> = Vec::new();
            let mut last_note_buffer: Option<NoteBuffer> = None;
            
            // First pass: parse tokens into buffer structure
            for i in 0..tokens.len().saturating_sub(2) {
                let tok_1 = &tokens[i];
                let tok_2 = &tokens[i + 1];
                let tok_3 = &tokens[i + 2];
                
                // Handle different token types
                if *tok_1 == time_tok {
                    src_time_tok_cnt += 1;
                    continue;
                } else if *tok_1 == unk_tok {
                    // Handle unknown tokens similar to instruments
                    if let Token::Onset(_, onset) = tok_2 {
                        if let Token::Duration(_, _duration) = tok_3 {
                            let note_buf = NoteBuffer {
                                note: tok_1.clone(),
                                onset: tok_2.clone(), 
                                dur: Some(tok_3.clone()),
                            };
                            last_note_buffer = Some(note_buf.clone());
                            buffer.entry(src_time_tok_cnt)
                                .or_insert_with(std::collections::HashMap::new)
                                .entry(*onset)
                                .or_insert_with(Vec::new)
                                .push(note_buf);
                        }
                    }
                    continue;
                } else if *tok_1 == bos_tok {
                    result.push(tok_1.clone());
                    continue;
                } else if *tok_1 == dim_tok {
                    if let Some(ref nb) = last_note_buffer {
                        if let Token::Onset(_, onset) = &nb.onset {
                            dim_tok_seen = Some((src_time_tok_cnt, *onset));
                        }
                    }
                    continue;
                }
                
                // Handle prefix tokens
                if let Token::Prefix(_, _, _) = tok_1 {
                    result.push(tok_1.clone());
                    continue;
                }
                
                // Handle note tokens (instrument or drum)
                match tok_1 {
                    Token::Note(instrument, _, _) => {
                        if instruments_wd.contains(instrument) {
                            if let Token::Onset(_, onset) = tok_2 {
                                if let Token::Duration(_, _duration) = tok_3 {
                                    let note_buf = NoteBuffer {
                                        note: tok_1.clone(),
                                        onset: tok_2.clone(),
                                        dur: Some(tok_3.clone()),
                                    };
                                    last_note_buffer = Some(note_buf.clone());
                                    buffer.entry(src_time_tok_cnt)
                                        .or_insert_with(std::collections::HashMap::new)
                                        .entry(*onset)
                                        .or_insert_with(Vec::new)
                                        .push(note_buf);
                                }
                            }
                        }
                    }
                    Token::Drum(_, _) => {
                        if let Token::Onset(_, onset) = tok_2 {
                            let note_buf = NoteBuffer {
                                note: tok_1.clone(),
                                onset: tok_2.clone(),
                                dur: None, // Drums don't have duration
                            };
                            last_note_buffer = Some(note_buf.clone());
                            buffer.entry(src_time_tok_cnt)
                                .or_insert_with(std::collections::HashMap::new)
                                .entry(*onset)
                                .or_insert_with(Vec::new)
                                .push(note_buf);
                        }
                    }
                    _ => continue,
                }
            }
            
            // Second pass: reconstruct tokens with tempo augmentation
            let mut prev_tgt_time_tok_cnt = 0;
            
            // Sort by time_tok_count, then by onset
            let mut time_keys: Vec<i32> = buffer.keys().cloned().collect();
            time_keys.sort();
            
            for src_time_tok_cnt in time_keys {
                if let Some(interval_notes) = buffer.get(&src_time_tok_cnt) {
                    let mut onset_keys: Vec<i32> = interval_notes.keys().cloned().collect();
                    onset_keys.sort();
                    
                    for src_onset in onset_keys {
                        if let Some(notes_by_onset) = interval_notes.get(&src_onset) {
                            let src_time = src_time_tok_cnt * abs_time_step_ms + src_onset;
                            let tgt_time = quantize_time(src_time as f64 * tempo_aug);
                            let curr_tgt_time_tok_cnt = tgt_time / abs_time_step_ms;
                            let mut curr_tgt_onset = tgt_time % abs_time_step_ms;
                            
                            if curr_tgt_onset == abs_time_step_ms {
                                curr_tgt_onset -= time_step_ms;
                            }
                            
                            // Add time tokens as needed
                            for _ in 0..(curr_tgt_time_tok_cnt - prev_tgt_time_tok_cnt) {
                                result.push(time_tok.clone());
                            }
                            prev_tgt_time_tok_cnt = curr_tgt_time_tok_cnt;
                            
                            // Handle mixup by shuffling notes at same onset
                            let mut notes = notes_by_onset.clone();
                            if mixup && notes.len() > 1 {
                                // Proper Fisher-Yates shuffle implementation
                                let mut rng = rand::thread_rng();
                                for i in (1..notes.len()).rev() {
                                    let j = rng.gen_range(0..=i);
                                    notes.swap(i, j);
                                }
                            }
                            
                            // Add notes with adjusted timing
                            for note in notes {
                                result.push(note.note);
                                result.push(Token::Onset("onset".to_string(), curr_tgt_onset));
                                
                                if let Some(Token::Duration(_, dur)) = note.dur {
                                    let tgt_dur = quantize_time(dur as f64 * tempo_aug).min(max_dur_ms);
                                    result.push(Token::Duration("dur".to_string(), tgt_dur));
                                }
                                
                                // Add diminish token if needed
                                if let Some((dim_time, dim_onset)) = dim_tok_seen {
                                    if dim_time == src_time_tok_cnt && dim_onset == src_onset {
                                        result.push(dim_tok.clone());
                                        dim_tok_seen = None;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Add end token if present in original
            if tokens.last() == Some(&eos_tok) {
                result.push(eos_tok.clone());
            }
            
            result
        }
    }
    
    pub fn get_unk_tok(&self) -> Token {
        Token::Special(self.base_tokenizer.unk_tok.clone())
    }
    
    pub fn tokenize_with_options(&self, midi_dict: &MidiDict, remove_preceding_silence: bool) -> Result<Vec<Token>, TokenizerError> {
        self.tokenize_midi_dict(midi_dict, remove_preceding_silence, true, true)
    }
    
    fn tokenize_midi_dict(&self, midi_dict: &MidiDict, remove_preceding_silence: bool, add_dim_tok: bool, add_eos_tok: bool) -> Result<Vec<Token>, TokenizerError> {
        let mut midi_dict = midi_dict.clone();
        
        // Remove ignored instruments
        midi_dict.remove_instruments(&self.config.ignore_instruments);
        
        if midi_dict.note_msgs.is_empty() {
            return Err(TokenizerError::EmptyNoteMessages);
        }
        
        // Sort note messages by start tick (like Python implementation)
        midi_dict.note_msgs.sort_by_key(|msg| msg.data.start);
        
        // Build channel to pedal intervals mapping
        let channel_to_pedal_intervals = midi_dict._build_pedal_intervals();
        
        // Build channel to instrument mapping
        let mut channel_to_instrument = HashMap::new();
        for inst_msg in &midi_dict.instrument_msgs {
            if inst_msg.channel != 9 { // Exclude drums
                if let Some(instrument) = program_to_instrument(inst_msg.data) {
                    channel_to_instrument.insert(inst_msg.channel, instrument);
                }
            }
        }
        
        // Default missing non-drum channels to piano
        let channels_used: HashSet<i32> = midi_dict.note_msgs.iter().map(|msg| msg.channel).collect();
        for &channel in &channels_used {
            if channel != 9 && !channel_to_instrument.contains_key(&channel) {
                channel_to_instrument.insert(channel, "piano".to_string());
            }
        }
        
        // Calculate prefix tokens - include ALL unique instruments from channel mapping (Python behavior)
        let mut prefix = Vec::new();
        let instruments_used: HashSet<String> = channel_to_instrument.values().cloned().collect();
        for instrument in instruments_used {
            prefix.push(Token::Prefix("prefix".to_string(), "instrument".to_string(), instrument));
        }
        if channels_used.contains(&9) {
            prefix.push(Token::Prefix("prefix".to_string(), "instrument".to_string(), "drum".to_string()));
        }
        
        // Add metadata prefixes if available
        if let Some(composer) = midi_dict.metadata.get("composer") {
            if self.config.composer_names.contains(composer) {
                prefix.insert(0, Token::Prefix("prefix".to_string(), "composer".to_string(), composer.clone()));
            }
        }
        if let Some(form) = midi_dict.metadata.get("form") {
            if self.config.form_names.contains(form) {
                prefix.insert(0, Token::Prefix("prefix".to_string(), "form".to_string(), form.clone()));
            }
        }
        if let Some(genre) = midi_dict.metadata.get("genre") {
            if self.config.genre_names.contains(genre) {
                prefix.insert(0, Token::Prefix("prefix".to_string(), "genre".to_string(), genre.clone()));
            }
        }
        
        let mut tokenized_seq = Vec::new();
        
        let initial_onset_tick = if remove_preceding_silence && !midi_dict.note_msgs.is_empty() {
            // Use first note's start tick to remove preceding silence
            midi_dict.note_msgs[0].data.start
        } else {
            // Don't remove preceding silence - start from 0
            0
        };
        
        let mut curr_time_since_onset = 0;
        
        for note_msg in &midi_dict.note_msgs {
            let channel = note_msg.channel;
            let pitch = note_msg.data.pitch;
            let velocity = note_msg.data.velocity;
            let start_tick = note_msg.data.start;
            let end_tick = note_msg.data.end;
            
            // Calculate time data
            let prev_time_since_onset = curr_time_since_onset;
            curr_time_since_onset = get_duration_ms(
                initial_onset_tick,
                start_tick,
                &midi_dict.tempo_msgs,
                midi_dict.ticks_per_beat,
            );
            
            // Add time tokens if necessary
            let time_toks_to_append = (curr_time_since_onset / self.abs_time_step_ms) - (prev_time_since_onset / self.abs_time_step_ms);
            for _ in 0..time_toks_to_append {
                tokenized_seq.push(Token::Special(self.time_tok.clone()));
            }
            
            if channel == 9 {
                // Drum case
                let note_onset = self.quantize_onset(curr_time_since_onset % self.abs_time_step_ms);
                tokenized_seq.push(Token::Drum("drum".to_string(), pitch));
                tokenized_seq.push(Token::Onset("onset".to_string(), note_onset));
            } else {
                // Non-drum case
                let instrument = channel_to_instrument.get(&channel).unwrap_or(&"piano".to_string()).clone();
                
                // Update end_tick if affected by pedal
                let mut adjusted_end_tick = end_tick;
                if let Some(pedal_intervals) = channel_to_pedal_intervals.get(&channel) {
                    for &(pedal_start, pedal_end) in pedal_intervals {
                        if pedal_start < end_tick && end_tick < pedal_end {
                            adjusted_end_tick = pedal_end;
                            break;
                        }
                    }
                }
                
                let note_duration = get_duration_ms(
                    start_tick,
                    adjusted_end_tick,
                    &midi_dict.tempo_msgs,
                    midi_dict.ticks_per_beat,
                );
                
                let quantized_velocity = self.quantize_velocity(velocity);
                let onset_before_quantization = curr_time_since_onset % self.abs_time_step_ms;
                let note_onset = self.quantize_onset(onset_before_quantization);
                let quantized_duration = self.quantize_dur(note_duration);
                
                tokenized_seq.push(Token::Note(instrument, pitch, quantized_velocity));
                tokenized_seq.push(Token::Onset("onset".to_string(), note_onset));
                tokenized_seq.push(Token::Duration("dur".to_string(), quantized_duration));
            }
        }
        
        Ok(self.format_tokens(prefix, tokenized_seq, add_dim_tok, add_eos_tok))
    }
    
    fn detokenize_midi_dict(&self, tokenized_seq: &[Token]) -> Result<MidiDict, TokenizerError> {
        // Use fixed timing values to match Python implementation
        // This ensures 1000 ticks = 1000ms for direct conversion
        self.detokenize_midi_dict_with_timing(tokenized_seq, 500000, 500)
    }

    pub fn detokenize_midi_dict_with_timing(&self, tokenized_seq: &[Token], tempo: i32, ticks_per_beat: i32) -> Result<MidiDict, TokenizerError> {
        use super::units::second2tick;
        
        let tempo_msgs = vec![TempoMessage {
            msg_type: "tempo".to_string(),
            data: tempo,
            tick: 0,
        }];
        
        let meta_msgs = Vec::new();
        let pedal_msgs = Vec::new();
        let mut instrument_msgs = Vec::new();
        let mut instrument_to_channel = HashMap::new();
        
        // Process instrument prefix tokens sequentially (like Python implementation)
        // This matches the exact Python behavior in _detokenize_midi_dict
        let mut channel_idx = 0;
        let mut start_idx = 0;
        
        for (idx, token) in tokenized_seq.iter().enumerate() {
            // Skip drum channel (channel 9) like Python
            if channel_idx == 9 {
                channel_idx += 1;
            }
            
            match token {
                Token::Special(s) if s == &self.time_tok => {
                    continue; // Skip time tokens during prefix processing
                }
                Token::Special(_) => {
                    continue; // Skip other special tokens
                }
                Token::Prefix(prefix_type, prefix_subtype, instrument) 
                    if prefix_type == "prefix" && prefix_subtype == "instrument" 
                    && self.instruments_wd.contains(instrument) => {
                    
                    // Check for duplicates (like Python)
                    if instrument_to_channel.contains_key(instrument) {
                        continue; // Skip duplicates
                    }
                    
                    if instrument == "drum" {
                        instrument_to_channel.insert("drum".to_string(), 9);
                    } else {
                        instrument_to_channel.insert(instrument.clone(), channel_idx);
                        channel_idx += 1;
                    }
                }
                Token::Prefix(_, _, _) => {
                    continue; // Skip other prefix tokens
                }
                _ => {
                    // First non-prefix, non-special token
                    start_idx = idx;
                    break;
                }
            }
        }
        
        // Create instrument messages sorted by channel (for deterministic order)
        let mut channel_pairs: Vec<_> = instrument_to_channel.iter().collect();
        channel_pairs.sort_by_key(|(_, &channel)| channel);
        
        for (instrument, &channel) in channel_pairs {
            let program = if instrument == "drum" {
                0
            } else {
                self.config.instrument_programs.get(instrument).cloned().unwrap_or(0)
            };
            
            instrument_msgs.push(InstrumentMessage {
                msg_type: "instrument".to_string(),
                data: program,
                tick: 0,
                channel,
            });
        }
        
        // Process note tokens
        let mut note_msgs = Vec::new();
        let mut curr_time_ms = 0; // Reset time accumulator for note processing
        
        
        let mut i = start_idx;
        while i < tokenized_seq.len() {
            let token = &tokenized_seq[i];
            
            match token {
                Token::Special(s) if s == &self.time_tok => {
                    curr_time_ms += self.abs_time_step_ms; // Add time in milliseconds
                    i += 1;
                    continue;
                }
                Token::Special(_) => {
                    i += 1;
                    continue; // Skip other special tokens
                }
                Token::Drum(_, pitch) => {
                    // Check if we have a next token and it's an onset
                    if i + 1 < tokenized_seq.len() {
                        if let Token::Onset(_, onset_ms) = &tokenized_seq[i + 1] {
                            // Calculate time in milliseconds first
                            let start_time_ms = curr_time_ms + onset_ms;
                            let end_time_ms = start_time_ms + self.time_step_ms;
                            
                            // Convert to ticks only at the end
                            let start_tick = second2tick(start_time_ms as f64 / 1000.0, ticks_per_beat, tempo);
                            let end_tick = second2tick(end_time_ms as f64 / 1000.0, ticks_per_beat, tempo);
                            
                            if let Some(&channel) = instrument_to_channel.get("drum") {
                                note_msgs.push(NoteMessage {
                                    msg_type: "note".to_string(),
                                    data: NoteData {
                                        pitch: *pitch,
                                        start: start_tick,
                                        end: end_tick,
                                        velocity: self.config.drum_velocity,
                                    },
                                    tick: start_tick,
                                    channel,
                                });
                                
                            }
                            
                            i += 2; // Skip drum and onset tokens
                            continue;
                        }
                    }
                    // If no onset follows or we're at the end, just skip the drum token
                    i += 1;
                }
                Token::Note(instrument, pitch, velocity) => {
                    if i + 2 < tokenized_seq.len() {
                        if let (Token::Onset(_, onset_ms), Token::Duration(_, duration_ms)) = 
                            (&tokenized_seq[i + 1], &tokenized_seq[i + 2]) {
                        
                        // Calculate time in milliseconds first
                        let start_time_ms = curr_time_ms + onset_ms;
                        let end_time_ms = start_time_ms + duration_ms;
                        
                        // Convert to ticks only at the end
                        let start_tick = second2tick(start_time_ms as f64 / 1000.0, ticks_per_beat, tempo);
                        let end_tick = second2tick(end_time_ms as f64 / 1000.0, ticks_per_beat, tempo);
                        
                        if let Some(&channel) = instrument_to_channel.get(instrument) {
                            note_msgs.push(NoteMessage {
                                msg_type: "note".to_string(),
                                data: NoteData {
                                    pitch: *pitch,
                                    start: start_tick,
                                    end: end_tick,
                                    velocity: *velocity,
                                },
                                tick: start_tick,
                                channel,
                            });
                        }
                        
                            i += 3; // Skip instrument, onset, and duration tokens
                            continue;
                        } else {
                            i += 1; // Skip just the note token if onset/duration don't follow
                        }
                    } else {
                        i += 1; // Skip if not enough tokens
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        Ok(MidiDict {
            meta_msgs,
            tempo_msgs,
            pedal_msgs,
            instrument_msgs,
            note_msgs,
            ticks_per_beat,
            metadata: HashMap::new(),
        })
    }
}

impl Tokenizer for AbsTokenizer {
    fn tokenize(&self, midi_dict: &MidiDict) -> Result<Vec<Token>, TokenizerError> {
        self.tokenize_with_options(midi_dict, false)
    }
    
    fn detokenize(&self, tokens: &[Token]) -> Result<MidiDict, TokenizerError> {
        self.detokenize_midi_dict(tokens)
    }
    
    fn vocab_size(&self) -> usize {
        self.base_tokenizer.vocab.len()
    }
    
    fn encode(&self, tokens: &[Token]) -> Vec<i32> {
        tokens.iter().map(|token| {
            self.base_tokenizer.tok_to_id.get(token).cloned().unwrap_or(0)
        }).collect()
    }
    
    fn decode(&self, ids: &[i32]) -> Vec<Token> {
        ids.iter().map(|&id| {
            self.base_tokenizer.id_to_tok.get(&id).cloned().unwrap_or(Token::Special("<U>".to_string()))
        }).collect()
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}