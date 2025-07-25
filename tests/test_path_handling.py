"""Tests for path handling in MidiDict loading."""

import unittest
from pathlib import Path
import tempfile
import os

from symbolic_music.aria import MidiDict


class TestPathHandling(unittest.TestCase):
    def setUp(self) -> None:
        self.test_midi_path = Path("tests/assets/data/arabesque.mid")
        self.assertTrue(self.test_midi_path.exists(), f"Test file not found: {self.test_midi_path}")
        
    def test_load_from_string_path(self) -> None:
        """Test loading MIDI from string path."""
        midi_dict = MidiDict.from_midi(str(self.test_midi_path))
        self.assertIsInstance(midi_dict, MidiDict)
        self.assertGreater(len(midi_dict.note_msgs), 0)
        
    def test_load_from_pathlib_path(self) -> None:
        """Test loading MIDI from pathlib.Path object."""
        midi_dict = MidiDict.from_midi(self.test_midi_path)
        self.assertIsInstance(midi_dict, MidiDict)
        self.assertGreater(len(midi_dict.note_msgs), 0)
        
    def test_save_to_string_path(self) -> None:
        """Test saving MIDI to string path."""
        midi_dict = MidiDict.from_midi(str(self.test_midi_path))
        
        with tempfile.NamedTemporaryFile(suffix='.mid', delete=False) as tmp:
            tmp_path = tmp.name
            
        try:
            midi_dict.to_midi().save(tmp_path)
            self.assertTrue(os.path.exists(tmp_path))
            
            # Verify we can load it back
            reloaded = MidiDict.from_midi(tmp_path)
            self.assertEqual(len(midi_dict.note_msgs), len(reloaded.note_msgs))
        finally:
            if os.path.exists(tmp_path):
                os.unlink(tmp_path)
                
    def test_save_to_pathlib_path(self) -> None:
        """Test saving MIDI to pathlib.Path object."""
        midi_dict = MidiDict.from_midi(str(self.test_midi_path))
        
        with tempfile.NamedTemporaryFile(suffix='.mid', delete=False) as tmp:
            tmp_path = Path(tmp.name)
            
        try:
            midi_dict.to_midi().save(tmp_path)
            self.assertTrue(tmp_path.exists())
            
            # Verify we can load it back
            reloaded = MidiDict.from_midi(tmp_path)
            self.assertEqual(len(midi_dict.note_msgs), len(reloaded.note_msgs))
        finally:
            if tmp_path.exists():
                tmp_path.unlink()
                
    def test_compare_loading_methods(self) -> None:
        """Test that all loading methods produce identical results."""
        str_path = str(self.test_midi_path)
        pathlib_path = self.test_midi_path
        
        midi_from_str = MidiDict.from_midi(str_path)
        midi_from_pathlib = MidiDict.from_midi(pathlib_path)
        
        # Compare note counts
        self.assertEqual(len(midi_from_str.note_msgs), len(midi_from_pathlib.note_msgs))
        
        # Compare tempo counts
        self.assertEqual(len(midi_from_str.tempo_msgs), len(midi_from_pathlib.tempo_msgs))
        
        # Compare hashes (should be identical)
        self.assertEqual(midi_from_str.calculate_hash(), midi_from_pathlib.calculate_hash())
        
    def test_invalid_path_type(self) -> None:
        """Test that invalid path types raise appropriate errors."""
        with self.assertRaises(Exception):
            MidiDict.from_midi(123)  # Invalid type
            
        with self.assertRaises(Exception):
            MidiDict.from_midi(['not', 'a', 'path'])  # Invalid type
            
    def test_nonexistent_file(self) -> None:
        """Test that nonexistent files raise appropriate errors."""
        with self.assertRaises(Exception):
            MidiDict.from_midi("nonexistent_file.mid")
            
        with self.assertRaises(Exception):
            MidiDict.from_midi(Path("nonexistent_file.mid"))


if __name__ == '__main__':
    unittest.main()