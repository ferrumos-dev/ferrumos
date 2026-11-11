# Scripts

Utility scripts for the FerrumOS project.

## Contents

| Script | Purpose |
|--------|---------|
| `kernel_tests/run.sh` | Run the kernel tests under QEMU, save the results and compare them with the baseline (local and CI) |
| `kernel_tests/results.py` | Parse `cargo test` output into JSON, compare with the baseline, append to the CSV history |
| `kernel_tests/store_history.sh` | CI only: commit each run's results to the `test-results` branch |
| `kernel_tests/test_results.py` | Self-tests for `results.py` |

See [`docs/tests/AUTOMATISATION_TESTS_QEMU.md`](../docs/tests/AUTOMATISATION_TESTS_QEMU.md).

## Usage

```sh
# From repository root
bash scripts/kernel_tests/run.sh

# Self-tests of the results tool (no QEMU needed)
python3 -m unittest discover -s scripts/kernel_tests
```
