#!/usr/bin/env python3
"""
Script to tokenize MIDI test files from aria-utils assets folder
and dump the tokenized sequences to JSON files.
"""

import json
import logging
from pathlib import Path
from typing import Dict, List, Any

from ariautils.midi import MidiDict, normalize_midi_dict
from ariautils.tokenizer import AbsTokenizer, RelTokenizer


def setup_logging() -> logging.Logger:
    """Setup logging configuration."""
    logging.basicConfig(
        level=logging.INFO,
        format='%(asctime)s - %(levelname)s - %(message)s'
    )
    return logging.getLogger(__name__)


def tokenize_file_with_tokenizer(
    midi_path: Path, 
    tokenizer, 
    tokenizer_name: str,
    logger: logging.Logger
) -> Dict[str, Any]:
    """
    Tokenize a single MIDI file with the given tokenizer.
    
    Args:
        midi_path: Path to the MIDI file
        tokenizer: The tokenizer instance (AbsTokenizer or RelTokenizer)
        tokenizer_name: Name of the tokenizer for logging
        logger: Logger instance
        
    Returns:
        Dictionary containing tokenization results
    """
    try:
        logger.info(f"Tokenizing {midi_path.name} with {tokenizer_name}")
        
        # Load MIDI file
        midi_dict = MidiDict.from_midi(midi_path)
        
        # Normalize MIDI dict based on tokenizer type
        max_duration = (
            tokenizer.max_dur_ms if hasattr(tokenizer, 'max_dur_ms') 
            else tokenizer.max_time_ms
        )
        
        # midi_dict = normalize_midi_dict(
        #     midi_dict=midi_dict,
        #     ignore_instruments=tokenizer.config["ignore_instruments"],
        #     instrument_programs=tokenizer.config["instrument_programs"],
        #     time_step_ms=tokenizer.time_step_ms,
        #     max_duration_ms=max_duration,
        #     drum_velocity=tokenizer.config["drum_velocity"],
        #     quantize_velocity_fn=tokenizer._quantize_velocity,
        # )
        
        # Tokenize
        token_sequence = tokenizer.tokenize(
            midi_dict, 
            remove_preceding_silence=False
        )
        
        # Convert tokens to serializable format
        serializable_tokens = []
        for token in token_sequence:
            if isinstance(token, tuple):
                serializable_tokens.append(list(token))
            else:
                serializable_tokens.append(token)
        
        result = {
            "file": midi_path.name,
            "tokenizer": tokenizer_name,
            "num_tokens": len(token_sequence),
            "tokens": serializable_tokens,
            "config": tokenizer.config,
            "time_step_ms": tokenizer.time_step_ms,
            "max_duration_ms": max_duration,
        }
        
        # Test round-trip tokenization
        try:
            detokenized_midi_dict = tokenizer.detokenize(token_sequence)
            result["round_trip_success"] = True
            result["detokenized_note_count"] = len(detokenized_midi_dict.note_msgs)
        except Exception as e:
            logger.warning(f"Round-trip failed for {midi_path.name} with {tokenizer_name}: {e}")
            result["round_trip_success"] = False
            result["round_trip_error"] = str(e)
        
        logger.info(f"Successfully tokenized {midi_path.name} with {tokenizer_name}: {len(token_sequence)} tokens")
        return result
        
    except Exception as e:
        logger.error(f"Failed to tokenize {midi_path.name} with {tokenizer_name}: {e}")
        return {
            "file": midi_path.name,
            "tokenizer": tokenizer_name,
            "error": str(e),
            "success": False
        }


def main():
    """Main function to tokenize all test files."""
    logger = setup_logging()
    
    # Define paths
    assets_dir = Path("/workspace/aria-utils/tests/assets/data")
    output_dir = Path("/workspace/output")
    
    # Create output directory if it doesn't exist
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Find all MIDI files
    midi_files = list(assets_dir.glob("*.mid"))
    
    if not midi_files:
        logger.error(f"No MIDI files found in {assets_dir}")
        return
    
    logger.info(f"Found {len(midi_files)} MIDI files: {[f.name for f in midi_files]}")
    
    # Initialize tokenizers
    tokenizers = {
        "AbsTokenizer": AbsTokenizer(),
        "RelTokenizer": RelTokenizer()
    }
    
    # Results storage
    all_results = []
    
    # Process each file with each tokenizer
    for midi_file in midi_files:
        for tokenizer_name, tokenizer in tokenizers.items():
            result = tokenize_file_with_tokenizer(
                midi_file, tokenizer, tokenizer_name, logger
            )
            all_results.append(result)
    
    # Save individual tokenizer results
    abs_tokenizer_results = [r for r in all_results if r.get("tokenizer") == "AbsTokenizer"]
    abs_output_file = output_dir / "abstokenizer_results.json"
    with open(abs_output_file, 'w') as f:
        json.dump(abs_tokenizer_results, f, indent=2)
    logger.info(f"AbsTokenizer results saved to {abs_output_file}")
    
    rel_tokenizer_results = [r for r in all_results if r.get("tokenizer") == "RelTokenizer"]
    rel_output_file = output_dir / "reltokenizer_results.json"
    with open(rel_output_file, 'w') as f:
        json.dump(rel_tokenizer_results, f, indent=2)
    logger.info(f"RelTokenizer results saved to {rel_output_file}")
    
    # Generate summary
    summary = {
        "total_files": len(midi_files),
        "total_tokenizations": len(all_results),
        "successful_tokenizations": len([r for r in all_results if r.get("error") is None]),
        "failed_tokenizations": len([r for r in all_results if r.get("error") is not None]),
        "round_trip_successes": len([r for r in all_results if r.get("round_trip_success") is True]),
        "files_processed": [f.name for f in midi_files],
        "tokenizers_used": list(tokenizers.keys())
    }
    
    summary_file = output_dir / "summary.json"
    with open(summary_file, 'w') as f:
        json.dump(summary, f, indent=2)
    
    logger.info(f"Summary: {summary['successful_tokenizations']}/{summary['total_tokenizations']} tokenizations successful")
    logger.info(f"Round-trip success rate: {summary['round_trip_successes']}/{summary['total_tokenizations']}")
    logger.info(f"Summary saved to {summary_file}")


if __name__ == "__main__":
    main()