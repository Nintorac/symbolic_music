"""ARIA tokenizer module for symbolic music processing."""

import symbolic_music._symbolic_music as _sm
_aria = getattr(_sm, 'symbolic_music.aria._aria')

# Import all classes from the _aria submodule
MidiDict = _aria.MidiDict
NoteMessage = _aria.NoteMessage
TempoMessage = _aria.TempoMessage
InstrumentMessage = _aria.InstrumentMessage
PedalMessage = _aria.PedalMessage
MetaMessage = _aria.MetaMessage
normalize_midi_dict = _aria.normalize_midi_dict

# Import the AbsTokenizer from its own file
from .abs_tokenizer import AbsTokenizer