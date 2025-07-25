# Aria-Utils Test Suite Generator

This Docker setup generates a test suite from the aria-utils repository by tokenizing test MIDI files and dumping the results to JSON files.

The script is fixed to a specific version of aria-utils and uses only the default configuration provided in that commit.

## Usage

### Build the Docker Image

```bash
make build
```

Or manually:
```bash
docker build -t aria-test-suite .
```

### Run the Container

```bash
make run
```

This will:
1. Use the aria-utils submodule which lives on a specific commit
2. Install the package and dependencies
3. Run tokenization on all MIDI files in `tests/assets/data/`
4. Save results to `/workspace/output/` (mounted to `../tests/assets/aria_targets/` on host)

### Other Commands

- `make help` - Show available commands
- `make run-only` - Run without rebuilding
- `make clean` - Remove Docker image
- `make rebuild` - Clean and rebuild

## Output Files

The script generates several output files:

- `abstokenizer_results.json`: Results specifically for AbsTokenizer
- `reltokenizer_results.json`: Results specifically for RelTokenizer  
- `summary.json`: Summary statistics

## Test Files Processed

The test will run against all the test midi files found in the aria-utils library

## Tokenizers Used

- **AbsTokenizer**: Absolute tokenization approach
- **RelTokenizer**: Relative tokenization approach

## Output Format

Each tokenization result includes:
- Original file name
- Tokenizer used
- Number of tokens generated
- Full token sequence
- Tokenizer configuration
- Round-trip tokenization test results
- Error information (if any)

## Dependencies

- Python 3.11+
- mido (for MIDI processing)
- aria-utils (cloned from GitHub)