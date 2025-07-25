pub mod aria;

pub use aria::tokenizer::Tokenizer;
pub use aria::midi_types::*;
pub use aria::abs_tokenizer::AbsTokenizer;
pub use aria::config::*;
pub use aria::midi_loader::load_midi_from_file;

use pyo3::prelude::*;

#[pymodule]
#[pyo3(name="_symbolic_music")]
fn symbolic_music(m: &Bound<'_, PyModule>) -> PyResult<()> {
    aria::py_mod::register_python_classes(m)?;
    Ok(())
}

