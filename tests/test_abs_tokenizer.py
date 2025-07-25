"""Tests for tokenizers."""

import unittest
import copy

from importlib import resources
from pathlib import Path
from typing import Final

from symbolic_music.aria import MidiDict, normalize_midi_dict
from symbolic_music.aria.abs_tokenizer import AbsTokenizer
# RelTokenizer not available yet
import logging

def get_logger(name):
    return logging.getLogger(name)

TEST_DATA_DIRECTORY: Final[Path] = Path(
    str(resources.files("tests").joinpath("assets", "data"))
)
RESULTS_DATA_DIRECTORY: Final[Path] = Path(
    str(resources.files("tests").joinpath("assets", "results"))
)


class TestAbsTokenizer(unittest.TestCase):
    def setUp(self) -> None:
        self.logger = get_logger(__name__ + ".TestAbsTokenizer")

    def test_normalize_midi_dict(self) -> None:
        def _test_normalize_midi_dict(
            _load_path: Path, _save_path: Path
        ) -> None:
            tokenizer = AbsTokenizer()
            midi_dict = MidiDict.from_midi(str(_load_path))
            # Create a fresh copy by reloading from the same file
            midi_dict_copy = MidiDict.from_midi(str(_load_path))

            normalized_midi_dict = normalize_midi_dict(
                midi_dict=midi_dict,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )
            normalized_twice_midi_dict = normalize_midi_dict(
                normalized_midi_dict,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )
            self.assertEqual(
                normalized_midi_dict,
                normalized_twice_midi_dict,
            )
            self.assertEqual(
                midi_dict,
                midi_dict_copy,
            )
            normalized_midi_dict.to_midi().save(_save_path)

        load_path = TEST_DATA_DIRECTORY.joinpath("arabesque.mid")
        save_path = RESULTS_DATA_DIRECTORY.joinpath("arabesque_norm.mid")
        _test_normalize_midi_dict(load_path, save_path)
        load_path = TEST_DATA_DIRECTORY.joinpath("pop.mid")
        save_path = RESULTS_DATA_DIRECTORY.joinpath("pop_norm.mid")
        _test_normalize_midi_dict(load_path, save_path)
        load_path = TEST_DATA_DIRECTORY.joinpath("basic.mid")
        save_path = RESULTS_DATA_DIRECTORY.joinpath("basic_norm.mid")
        _test_normalize_midi_dict(load_path, save_path)

    def test_tokenize_detokenize(self) -> None:
        def _test_tokenize_detokenize(_load_path: Path) -> None:
            tokenizer = AbsTokenizer()
            midi_dict = MidiDict.from_midi(str(_load_path))

            midi_dict_1 = normalize_midi_dict(
                midi_dict=midi_dict,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )

            midi_dict_2 = normalize_midi_dict(
                midi_dict=tokenizer.detokenize(
                    tokenizer.tokenize(
                        midi_dict_1, remove_preceding_silence=False
                    )
                ),
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )

            self.assertEqual(
                midi_dict_1,
                midi_dict_2,
            )

        load_path = TEST_DATA_DIRECTORY.joinpath("arabesque.mid")
        _test_tokenize_detokenize(_load_path=load_path)
        load_path = TEST_DATA_DIRECTORY.joinpath("pop.mid")
        _test_tokenize_detokenize(_load_path=load_path)
        load_path = TEST_DATA_DIRECTORY.joinpath("basic.mid")
        _test_tokenize_detokenize(_load_path=load_path)

    def test_pitch_aug(self) -> None:
        def _test_out_of_bounds(
            tokenizer: AbsTokenizer,
            midi_dict: MidiDict,
            pitch_aug: int,
        ) -> None:
            pitch_aug_fn = tokenizer.export_pitch_aug(pitch_aug)
            seq = tokenizer.tokenize(midi_dict, remove_preceding_silence=False)
            augmented_seq = pitch_aug_fn(seq, pitch_aug=pitch_aug)

            for tok_1, tok_2 in zip(seq, augmented_seq):
                if (
                    isinstance(tok_1, tuple)
                    and tok_1[0] in tokenizer.instruments_nd
                    and tok_2 == tokenizer.unk_tok
                ):
                    self.assertTrue(
                        tok_1[1] + pitch_aug not in set(range(0, 128))
                    )

        def _test_pitch_aug(
            tokenizer: AbsTokenizer,
            midi_dict: MidiDict,
            pitch_aug: int,
        ) -> None:
            midi_dict = normalize_midi_dict(
                midi_dict=midi_dict,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )

            pitch_aug_fn = tokenizer.export_pitch_aug(pitch_aug)
            seq = tokenizer.tokenize(midi_dict, remove_preceding_silence=False)
            augmented_seq = pitch_aug_fn(seq, pitch_aug=pitch_aug)
            midi_dict_aug = tokenizer.detokenize(augmented_seq)

            self.assertEqual(len(seq), len(augmented_seq))
            if tokenizer.unk_tok in augmented_seq:
                # Skip cases with unk tok
                self.logger.info(
                    f"Seen unk_tok on {load_path.name} for pitch_aug={pitch_aug}"
                )
                return

            midi_dict_aug = normalize_midi_dict(
                midi_dict=midi_dict_aug,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )

            for msg_no_aug, msg_aug in zip(
                midi_dict.note_msgs, midi_dict_aug.note_msgs
            ):
                if msg_no_aug.channel != 9:
                    self.assertEqual(
                        msg_no_aug.pitch + pitch_aug,
                        msg_aug.pitch,
                    )

                    # Compare all fields except pitch
                    self.assertEqual(msg_no_aug.msg_type, msg_aug.msg_type)
                    self.assertEqual(msg_no_aug.start, msg_aug.start)
                    self.assertEqual(msg_no_aug.end, msg_aug.end)
                    self.assertEqual(msg_no_aug.velocity, msg_aug.velocity)
                    self.assertEqual(msg_no_aug.tick, msg_aug.tick)
                    self.assertEqual(msg_no_aug.channel, msg_aug.channel)
                else:
                    # For drums, all fields should be equal
                    self.assertEqual(msg_no_aug.msg_type, msg_aug.msg_type)
                    self.assertEqual(msg_no_aug.pitch, msg_aug.pitch)
                    self.assertEqual(msg_no_aug.start, msg_aug.start)
                    self.assertEqual(msg_no_aug.end, msg_aug.end)
                    self.assertEqual(msg_no_aug.velocity, msg_aug.velocity)
                    self.assertEqual(msg_no_aug.tick, msg_aug.tick)
                    self.assertEqual(msg_no_aug.channel, msg_aug.channel)

        tokenizer = AbsTokenizer()
        load_path = TEST_DATA_DIRECTORY.joinpath("arabesque.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for pitch_aug in range(-30, 30):
            _test_pitch_aug(tokenizer, midi_dict, pitch_aug)
            _test_out_of_bounds(tokenizer, midi_dict, pitch_aug)

        load_path = TEST_DATA_DIRECTORY.joinpath("pop.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for pitch_aug in range(-30, 30):
            _test_pitch_aug(tokenizer, midi_dict, pitch_aug)
            _test_out_of_bounds(tokenizer, midi_dict, pitch_aug)

        load_path = TEST_DATA_DIRECTORY.joinpath("basic.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for pitch_aug in range(-30, 30):
            _test_pitch_aug(tokenizer, midi_dict, pitch_aug)
            _test_out_of_bounds(tokenizer, midi_dict, pitch_aug)

    def test_velocity_aug(self) -> None:
        def _test_velocity_aug(
            tokenizer: AbsTokenizer,
            midi_dict: MidiDict,
            velocity_aug_step: int,
        ) -> None:
            midi_dict = normalize_midi_dict(
                midi_dict=midi_dict,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )

            velocity_aug_fn = tokenizer.export_velocity_aug(velocity_aug_step)
            seq = tokenizer.tokenize(midi_dict, remove_preceding_silence=False)
            augmented_seq = velocity_aug_fn(seq, aug_step=velocity_aug_step)
            midi_dict_aug = tokenizer.detokenize(augmented_seq)

            self.assertEqual(len(seq), len(augmented_seq))
            self.assertTrue(tokenizer.unk_tok not in augmented_seq)

            midi_dict_aug = normalize_midi_dict(
                midi_dict=midi_dict_aug,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )

            for msg_no_aug, msg_aug in zip(
                midi_dict.note_msgs, midi_dict_aug.note_msgs
            ):
                if msg_no_aug.channel == 9:
                    # For drums, all fields should be equal
                    self.assertEqual(msg_no_aug.msg_type, msg_aug.msg_type)
                    self.assertEqual(msg_no_aug.pitch, msg_aug.pitch)
                    self.assertEqual(msg_no_aug.start, msg_aug.start)
                    self.assertEqual(msg_no_aug.end, msg_aug.end)
                    self.assertEqual(msg_no_aug.velocity, msg_aug.velocity)
                    self.assertEqual(msg_no_aug.tick, msg_aug.tick)
                    self.assertEqual(msg_no_aug.channel, msg_aug.channel)
                else:
                    _velocity = min(
                        max(
                            msg_no_aug.velocity
                            + velocity_aug_step * tokenizer.velocity_step,
                            tokenizer.velocity_step,
                        ),
                        tokenizer.max_velocity,
                    )
                    print(_velocity)
                    self.assertEqual(msg_aug.velocity, _velocity)

                # Compare all fields except velocity
                self.assertEqual(msg_no_aug.msg_type, msg_aug.msg_type)
                self.assertEqual(msg_no_aug.pitch, msg_aug.pitch)
                self.assertEqual(msg_no_aug.start, msg_aug.start)
                self.assertEqual(msg_no_aug.end, msg_aug.end)
                self.assertEqual(msg_no_aug.tick, msg_aug.tick)
                self.assertEqual(msg_no_aug.channel, msg_aug.channel)

        tokenizer = AbsTokenizer()
        load_path = TEST_DATA_DIRECTORY.joinpath("arabesque.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for velocity_aug in range(-10, 10):
            _test_velocity_aug(tokenizer, midi_dict, velocity_aug)

        load_path = TEST_DATA_DIRECTORY.joinpath("pop.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for velocity_aug in range(-10, 10):
            _test_velocity_aug(tokenizer, midi_dict, velocity_aug)

        load_path = TEST_DATA_DIRECTORY.joinpath("basic.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for velocity_aug in range(-10, 10):
            _test_velocity_aug(tokenizer, midi_dict, velocity_aug)

    @unittest.skip("Known issue with tempo augmentation - see https://github.com/EleutherAI/aria-utils/issues/32")
    def test_tempo_aug(self) -> None:
        def _quantize_time(_n: int | float, time_step: int) -> int:
            return round(_n / time_step) * time_step

        def _test_tempo_aug(
            tokenizer: AbsTokenizer,
            midi_dict: MidiDict,
            tempo_aug: float,
        ) -> None:
            midi_dict = normalize_midi_dict(
                midi_dict=midi_dict,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )

            tempo_aug_fn = tokenizer.export_tempo_aug(tempo_aug, mixup=False)
            seq = tokenizer.tokenize(midi_dict, remove_preceding_silence=False)
            augmented_seq = tempo_aug_fn(seq, tempo_aug=tempo_aug)
            midi_dict_aug = tokenizer.detokenize(augmented_seq)

            # self.assertEqual(len(seq), len(augmented_seq))
            self.assertTrue(tokenizer.unk_tok not in augmented_seq)

            midi_dict_aug = normalize_midi_dict(
                midi_dict=midi_dict_aug,
                ignore_instruments=tokenizer.config["ignore_instruments"],
                instrument_programs=tokenizer.config["instrument_programs"],
                time_step_ms=tokenizer.time_step_ms,
                max_duration_ms=tokenizer.max_dur_ms,
                drum_velocity=tokenizer.config["drum_velocity"],
                quantize_velocity_fn=tokenizer._quantize_velocity,
            )

            for msg_no_aug, msg_aug in zip(
                midi_dict.note_msgs, midi_dict_aug.note_msgs
            ):
                _start_tick = _quantize_time(
                    msg_no_aug.start * tempo_aug,
                    time_step=tokenizer.time_step_ms,
                )
                _end_tick = min(
                    _start_tick + tokenizer.max_dur_ms,
                    _quantize_time(
                        msg_no_aug.end * tempo_aug,
                        time_step=tokenizer.time_step_ms,
                    ),
                )

                self.assertLessEqual(abs(msg_aug.tick - _start_tick), 10)
                self.assertLessEqual(
                    abs(msg_aug.start - _start_tick), 10
                )
                self.assertLessEqual(
                    abs(msg_aug.end - _end_tick), 10
                )

                # Compare all fields except timing fields
                self.assertEqual(msg_no_aug.msg_type, msg_aug.msg_type)
                self.assertEqual(msg_no_aug.pitch, msg_aug.pitch)
                self.assertEqual(msg_no_aug.velocity, msg_aug.velocity)
                self.assertEqual(msg_no_aug.channel, msg_aug.channel)

        tokenizer = AbsTokenizer()
        tempo_range = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0]

        load_path = TEST_DATA_DIRECTORY.joinpath("arabesque.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for tempo_aug in tempo_range:
            _test_tempo_aug(tokenizer, midi_dict, tempo_aug)

        load_path = TEST_DATA_DIRECTORY.joinpath("pop.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for tempo_aug in tempo_range:
            _test_tempo_aug(tokenizer, midi_dict, tempo_aug)

        load_path = TEST_DATA_DIRECTORY.joinpath("basic.mid")
        midi_dict = MidiDict.from_midi(str(load_path))
        for tempo_aug in tempo_range:
            _test_tempo_aug(tokenizer, midi_dict, tempo_aug)
