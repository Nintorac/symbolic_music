"""Basic tests for symbolic_music package."""
import pytest
import symbolic_music


def test_module_imports():
    """Test that the module can be imported and has expected classes."""
    # Test that aria submodule exists
    assert hasattr(symbolic_music, 'aria')
    
    # Test that classes are accessible via aria submodule
    assert hasattr(symbolic_music.aria, 'MidiDict')
    assert hasattr(symbolic_music.aria, 'AbsTokenizer')
    
    # Test that classes are callable
    assert callable(symbolic_music.aria.MidiDict)
    assert callable(symbolic_music.aria.AbsTokenizer)


def test_midi_dict_creation():
    """Test MidiDict creation and basic functionality."""
    midi_dict = symbolic_music.aria.MidiDict()
    
    # Test default values
    assert midi_dict.ticks_per_beat == 480
    assert len(midi_dict.note_msgs) == 0
    assert len(midi_dict.tempo_msgs) == 0
    assert len(midi_dict.instrument_msgs) == 0
    
    # Test adding notes
    midi_dict.add_note(60, 0, 480, 80, 0)  # Middle C
    assert len(midi_dict.note_msgs) == 1
    assert midi_dict.note_msgs[0].pitch == 60
    assert midi_dict.note_msgs[0].velocity == 80


def test_abs_tokenizer_creation():
    """Test AbsTokenizer creation and basic functionality."""
    tokenizer = symbolic_music.aria.AbsTokenizer()
    
    # Test tokenizer properties
    assert tokenizer.name() == "abs"
    assert tokenizer.vocab_size() > 0
    
    # Test with simple MIDI data
    midi_dict = symbolic_music.aria.MidiDict()
    midi_dict.add_note(60, 0, 480, 80, 0)
    midi_dict.add_instrument(0, 0, 0)  # Piano
    
    # Test tokenization
    tokens = tokenizer.tokenize(midi_dict)
    assert len(tokens) > 0
    assert isinstance(tokens, list)
    


def test_midi_from_file():
    """Test loading MIDI files using MidiDict.from_midi."""
    from pathlib import Path
    
    # Test with one of the test files
    test_file = Path("tests/assets/data/basic.mid")
    if test_file.exists():
        midi_dict = symbolic_music.aria.MidiDict.from_midi(str(test_file))
        
        # Should have loaded some data
        assert midi_dict.ticks_per_beat > 0
        assert len(midi_dict.note_msgs) > 0