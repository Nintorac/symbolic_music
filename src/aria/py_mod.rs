use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;
use super::py_midi_types::{PyMidiDict, PyNoteMessage, PyTempoMessage, PyInstrumentMessage, PyPedalMessage, PyMetaMessage, convert_py_midi_dict_to_rust, convert_rust_midi_dict_to_py};
use super::py_abs_tokenizer::PyAbsTokenizer;
use super::midi_loader::normalize_midi_dict;

#[pyfunction]
#[pyo3(name = "normalize_midi_dict")]
fn py_normalize_midi_dict(
    midi_dict: PyMidiDict,
    ignore_instruments: &Bound<'_, PyDict>,
    instrument_programs: &Bound<'_, PyDict>,
    time_step_ms: i32,
    max_duration_ms: i32,
    drum_velocity: i32,
    quantize_velocity_fn: &Bound<'_, PyAny>,
) -> PyResult<PyMidiDict> {
    // Convert Python dictionaries to Rust HashMaps
    let mut ignore_instruments_map = HashMap::new();
    for (key, value) in ignore_instruments.iter() {
        let key_str: String = key.extract()?;
        let value_bool: bool = value.extract()?;
        ignore_instruments_map.insert(key_str, value_bool);
    }
    
    let mut instrument_programs_map = HashMap::new();
    for (key, value) in instrument_programs.iter() {
        let key_str: String = key.extract()?;
        let value_i32: i32 = value.extract()?;
        instrument_programs_map.insert(key_str, value_i32);
    }
    
    // Create a closure that calls the Python function
    let quantize_fn = |velocity: i32| -> i32 {
        let result = quantize_velocity_fn.call1((velocity,));
        match result {
            Ok(py_result) => py_result.extract().unwrap_or(velocity),
            Err(_) => velocity,
        }
    };
    
    // Convert PyMidiDict to Rust MidiDict
    let rust_midi_dict = convert_py_midi_dict_to_rust(&midi_dict);
    
    // Call the Rust normalize_midi_dict function
    let normalized_midi_dict = normalize_midi_dict(
        rust_midi_dict,
        &ignore_instruments_map,
        &instrument_programs_map,
        time_step_ms,
        max_duration_ms,
        drum_velocity,
        quantize_fn,
    );
    
    // Convert back to PyMidiDict
    Ok(convert_rust_midi_dict_to_py(&normalized_midi_dict))
}

pub fn register_python_classes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    
    // Create aria submodule
    let aria_module = PyModule::new(py, "symbolic_music.aria._aria")?;
    
    // Add classes to aria submodule
    aria_module.add_class::<PyMidiDict>()?;
    aria_module.add_class::<PyNoteMessage>()?;
    aria_module.add_class::<PyTempoMessage>()?;
    aria_module.add_class::<PyInstrumentMessage>()?;
    aria_module.add_class::<PyPedalMessage>()?;
    aria_module.add_class::<PyMetaMessage>()?;
    aria_module.add_class::<PyAbsTokenizer>()?;
    
    // Add functions to aria submodule
    aria_module.add_function(wrap_pyfunction!(py_normalize_midi_dict, &aria_module)?)?;
    
    // Add aria submodule to main module
    m.add_submodule(&aria_module)?;
    
    // Register in sys.modules
    py.import("sys")?
        .getattr("modules")?
        .set_item("symbolic_music.aria._aria", &aria_module)?;
    
    Ok(())
}