# Symbolic Music

A high-performance Rust library with Python bindings for fast symbolic music format conversion, designed specifically for high performance machine learning applications. 


## Setup and Development

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **Python 3.8+**: Required for the Python bindings
- **uv**: Fast Python package manager

### Steps

1. **Clone and navigate to the project:**
   ```bash
   git clone git@github.com:Nintorac/symbolic_music.git
   cd symbolic_midi
   ```

2. **Install dependencies:**
   ```bash
   uv sync
   ```

3. **Build the project for development:**
   ```bash
   uv run maturin develop
   ```

4. **Test the installation:**
   ```bash
   uv run python -c "import symbolic_music; print(symbolic_music.hello_world())"
   ```

## Usage

### ARIA Tokenizer - AbsTokenizer

The AbsTokenizer provides absolute time-based tokenization for MIDI data:

```python
from symbolic_music.aria import MidiDict, AbsTokenizer, normalize_midi_dict

# Initialize the tokenizer
tokenizer = AbsTokenizer()

# Load MIDI file
midi_dict = MidiDict.from_midi("path/to/your/file.mid")

# Normalize MIDI data (recommended preprocessing step)
normalized_midi_dict = normalize_midi_dict(
    midi_dict=midi_dict,
    ignore_instruments=tokenizer.config["ignore_instruments"],
    instrument_programs=tokenizer.config["instrument_programs"],
    time_step_ms=tokenizer.time_step_ms,
    max_duration_ms=tokenizer.max_dur_ms,
    drum_velocity=tokenizer.config["drum_velocity"],
    quantize_velocity_fn=tokenizer._quantize_velocity,
)

# Tokenize MIDI data to sequence
sequence = tokenizer.tokenize(normalized_midi_dict, remove_preceding_silence=False)

# Detokenize sequence back to MIDI
reconstructed_midi_dict = tokenizer.detokenize(sequence)

# Save reconstructed MIDI
reconstructed_midi_dict.to_midi().save("path/to/output.mid")
```

## Building for Production

To build wheels for distribution:
```bash
uv run maturin build --release
```

## Development Commands

- `uv sync` - Install all dependencies
- `uv run maturin develop` - Build and install for development
- `uv run maturin build` - Build wheel packages
- `uv run python -m pytest` - Run tests (when added)