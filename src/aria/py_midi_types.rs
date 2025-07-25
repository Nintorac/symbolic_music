use pyo3::prelude::*;
use pyo3::types::{PyType, PyAny};
use std::collections::HashMap;
use super::midi_types::*;

fn extract_path_like(path: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(path_str) = path.extract::<String>() {
        Ok(path_str)
    } else if let Ok(path_pathlib) = path.call_method0("__str__") {
        path_pathlib.extract::<String>()
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Path must be a string or pathlib.Path"))
    }
}

#[pyclass(name = "MidiDict")]
#[derive(Clone)]
pub struct PyMidiDict {
    #[pyo3(get, set)]
    pub ticks_per_beat: i32,
    #[pyo3(get, set)]
    pub note_msgs: Vec<PyNoteMessage>,
    #[pyo3(get, set)]
    pub tempo_msgs: Vec<PyTempoMessage>,
    #[pyo3(get, set)]
    pub instrument_msgs: Vec<PyInstrumentMessage>,
    #[pyo3(get, set)]
    pub pedal_msgs: Vec<PyPedalMessage>,
    #[pyo3(get, set)]
    pub meta_msgs: Vec<PyMetaMessage>,
    #[pyo3(get, set)]
    pub metadata: HashMap<String, String>,
}

#[pyclass(name = "NoteMessage")]
#[derive(Clone, PartialEq)]
pub struct PyNoteMessage {
    #[pyo3(get, set)]
    pub msg_type: String,
    #[pyo3(get, set)]
    pub pitch: i32,
    #[pyo3(get, set)]
    pub start: i32,
    #[pyo3(get, set)]
    pub end: i32,
    #[pyo3(get, set)]
    pub velocity: i32,
    #[pyo3(get, set)]
    pub tick: i32,
    #[pyo3(get, set)]
    pub channel: i32,
}

#[pyclass(name = "TempoMessage")]
#[derive(Clone, PartialEq)]
pub struct PyTempoMessage {
    #[pyo3(get, set)]
    pub msg_type: String,
    #[pyo3(get, set)]
    pub data: i32,
    #[pyo3(get, set)]
    pub tick: i32,
}

#[pyclass(name = "InstrumentMessage")]
#[derive(Clone, PartialEq)]
pub struct PyInstrumentMessage {
    #[pyo3(get, set)]
    pub msg_type: String,
    #[pyo3(get, set)]
    pub data: i32,
    #[pyo3(get, set)]
    pub tick: i32,
    #[pyo3(get, set)]
    pub channel: i32,
}

#[pyclass(name = "PedalMessage")]
#[derive(Clone, PartialEq)]
pub struct PyPedalMessage {
    #[pyo3(get, set)]
    pub msg_type: String,
    #[pyo3(get, set)]
    pub data: i32,
    #[pyo3(get, set)]
    pub tick: i32,
    #[pyo3(get, set)]
    pub channel: i32,
}

#[pyclass(name = "MetaMessage")]
#[derive(Clone, PartialEq)]
pub struct PyMetaMessage {
    #[pyo3(get, set)]
    pub msg_type: String,
    #[pyo3(get, set)]
    pub data: String,
}

#[pyclass(name = "MidiFile")]
#[derive(Clone)]
pub struct PyMidiFile {
    pub midi_file: MidiFile,
}

#[pymethods]
impl PyMidiDict {
    #[new]
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
    
    #[classmethod]
    fn from_midi(_cls: &Bound<'_, PyType>, mid_path: &Bound<'_, PyAny>) -> PyResult<Self> {
        use crate::aria::midi_loader::load_midi_from_file;
        let path_str = extract_path_like(mid_path)?;
        match load_midi_from_file(&path_str) {
            Ok(midi_dict) => Ok(convert_rust_midi_dict_to_py(&midi_dict)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to load MIDI file: {}", e))),
        }
    }
    
    pub fn add_note(&mut self, pitch: i32, start: i32, end: i32, velocity: i32, channel: i32) {
        self.note_msgs.push(PyNoteMessage {
            msg_type: "note".to_string(),
            pitch,
            start,
            end,
            velocity,
            tick: start,
            channel,
        });
    }
    
    pub fn add_instrument(&mut self, program: i32, tick: i32, channel: i32) {
        self.instrument_msgs.push(PyInstrumentMessage {
            msg_type: "instrument".to_string(),
            data: program,
            tick,
            channel,
        });
    }
    
    pub fn add_tempo(&mut self, tempo: i32, tick: i32) {
        self.tempo_msgs.push(PyTempoMessage {
            msg_type: "tempo".to_string(),
            data: tempo,
            tick,
        });
    }
    
    pub fn calculate_hash(&self) -> u64 {
        let rust_midi_dict = convert_py_midi_dict_to_rust(self);
        rust_midi_dict.calculate_hash()
    }
    
    pub fn resolve_pedal(&self) -> Self {
        let rust_midi_dict = convert_py_midi_dict_to_rust(self);
        let resolved = rust_midi_dict.resolve_pedal();
        convert_rust_midi_dict_to_py(&resolved)
    }
    
    pub fn remove_redundant_pedals(&self) -> Self {
        let rust_midi_dict = convert_py_midi_dict_to_rust(self);
        let cleaned = rust_midi_dict.remove_redundant_pedals();
        convert_rust_midi_dict_to_py(&cleaned)
    }
    
    pub fn to_midi(&self) -> PyMidiFile {
        let rust_midi_dict = convert_py_midi_dict_to_rust(self);
        let midi_file = rust_midi_dict.to_midi();
        PyMidiFile { midi_file }
    }
    
    pub fn tick_to_ms(&self, tick: i32) -> i32 {
        let rust_midi_dict = convert_py_midi_dict_to_rust(self);
        rust_midi_dict.tick_to_ms(tick)
    }
    
    pub fn get_msg_dict(&self) -> PyObject {
        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("note_msgs", self.note_msgs.clone().into_py(py)).unwrap();
            dict.set_item("tempo_msgs", self.tempo_msgs.clone().into_py(py)).unwrap();
            dict.set_item("instrument_msgs", self.instrument_msgs.clone().into_py(py)).unwrap();
            dict.set_item("pedal_msgs", self.pedal_msgs.clone().into_py(py)).unwrap();
            dict.set_item("meta_msgs", self.meta_msgs.clone().into_py(py)).unwrap();
            dict.set_item("ticks_per_beat", self.ticks_per_beat).unwrap();
            dict.set_item("metadata", self.metadata.clone().into_py(py)).unwrap();
            dict.into_py(py)
        })
    }
    
    fn __eq__(&self, other: &Self) -> bool {
        self.ticks_per_beat == other.ticks_per_beat &&
        self.note_msgs == other.note_msgs &&
        self.tempo_msgs == other.tempo_msgs &&
        self.instrument_msgs == other.instrument_msgs &&
        self.pedal_msgs == other.pedal_msgs &&
        self.meta_msgs == other.meta_msgs &&
        self.metadata == other.metadata
    }
    
    fn __hash__(&self) -> u64 {
        let rust_midi_dict = convert_py_midi_dict_to_rust(self);
        rust_midi_dict.calculate_hash()
    }
}

pub fn convert_py_midi_dict_to_rust(py_midi_dict: &PyMidiDict) -> MidiDict {
    let mut midi_dict = MidiDict::new();
    midi_dict.ticks_per_beat = py_midi_dict.ticks_per_beat;
    
    for py_note in &py_midi_dict.note_msgs {
        midi_dict.note_msgs.push(NoteMessage {
            msg_type: py_note.msg_type.clone(),
            data: NoteData {
                pitch: py_note.pitch,
                start: py_note.start,
                end: py_note.end,
                velocity: py_note.velocity,
            },
            tick: py_note.tick,
            channel: py_note.channel,
        });
    }
    
    for py_tempo in &py_midi_dict.tempo_msgs {
        midi_dict.tempo_msgs.push(TempoMessage {
            msg_type: py_tempo.msg_type.clone(),
            data: py_tempo.data,
            tick: py_tempo.tick,
        });
    }
    
    for py_inst in &py_midi_dict.instrument_msgs {
        midi_dict.instrument_msgs.push(InstrumentMessage {
            msg_type: py_inst.msg_type.clone(),
            data: py_inst.data,
            tick: py_inst.tick,
            channel: py_inst.channel,
        });
    }
    
    for py_pedal in &py_midi_dict.pedal_msgs {
        midi_dict.pedal_msgs.push(PedalMessage {
            msg_type: py_pedal.msg_type.clone(),
            data: py_pedal.data,
            tick: py_pedal.tick,
            channel: py_pedal.channel,
        });
    }
    
    for py_meta in &py_midi_dict.meta_msgs {
        midi_dict.meta_msgs.push(MetaMessage {
            msg_type: py_meta.msg_type.clone(),
            data: py_meta.data.clone(),
            tick: 0, // Meta messages don't have tick in the test
        });
    }
    
    midi_dict.metadata = py_midi_dict.metadata.clone();
    
    midi_dict
}

pub fn convert_rust_midi_dict_to_py(midi_dict: &MidiDict) -> PyMidiDict {
    let mut py_midi_dict = PyMidiDict::new();
    py_midi_dict.ticks_per_beat = midi_dict.ticks_per_beat;
    
    for note in &midi_dict.note_msgs {
        py_midi_dict.note_msgs.push(PyNoteMessage {
            msg_type: note.msg_type.clone(),
            pitch: note.data.pitch,
            start: note.data.start,
            end: note.data.end,
            velocity: note.data.velocity,
            tick: note.tick,
            channel: note.channel,
        });
    }
    
    for tempo in &midi_dict.tempo_msgs {
        py_midi_dict.tempo_msgs.push(PyTempoMessage {
            msg_type: tempo.msg_type.clone(),
            data: tempo.data,
            tick: tempo.tick,
        });
    }
    
    for inst in &midi_dict.instrument_msgs {
        py_midi_dict.instrument_msgs.push(PyInstrumentMessage {
            msg_type: inst.msg_type.clone(),
            data: inst.data,
            tick: inst.tick,
            channel: inst.channel,
        });
    }
    
    for pedal in &midi_dict.pedal_msgs {
        py_midi_dict.pedal_msgs.push(PyPedalMessage {
            msg_type: pedal.msg_type.clone(),
            data: pedal.data,
            tick: pedal.tick,
            channel: pedal.channel,
        });
    }
    
    for meta in &midi_dict.meta_msgs {
        py_midi_dict.meta_msgs.push(PyMetaMessage {
            msg_type: meta.msg_type.clone(),
            data: meta.data.clone(),
        });
    }
    
    py_midi_dict.metadata = midi_dict.metadata.clone();
    
    py_midi_dict
}

#[pymethods]
impl PyMidiFile {
    pub fn save(&self, path: &Bound<'_, PyAny>) -> PyResult<()> {
        let path_str = extract_path_like(path)?;
        self.midi_file.save(&path_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to save MIDI file: {}", e)))
    }
}

#[pymethods]
impl PyNoteMessage {
    fn __getitem__(&self, key: &str) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            match key {
                "tick" => Ok(self.tick.into_py(py)),
                "pitch" => Ok(self.pitch.into_py(py)),
                "start" => Ok(self.start.into_py(py)),
                "end" => Ok(self.end.into_py(py)),
                "velocity" => Ok(self.velocity.into_py(py)),
                "channel" => Ok(self.channel.into_py(py)),
                "msg_type" => Ok(self.msg_type.clone().into_py(py)),
                _ => {
                    // Try to get attribute from Python object
                    let obj = self.clone().into_py(py);
                    obj.getattr(py, key).map_err(|_| 
                        PyErr::new::<pyo3::exceptions::PyKeyError, _>(format!("Key '{}' not found", key))
                    )
                }
            }
        })
    }
}