# Scripts

Utility scripts for the FerrumOS project.

## Contents

| Script | Purpose |
|--------|---------|
| `kernel_tests/run.sh` | Run the kernel tests under QEMU, save the results and compare them with the baseline (local and CI) |
| `kernel_tests/results.py` | Parse `cargo test` output into JSON, compare with the baseline, append to the CSV history |
| `kernel_tests/store_history.sh` | CI only: commit each run's results to the `test-results` branch |
| `kernel_tests/test_results.py` | Self-tests for `results.py` |
| `veille/check_sources.py` | Research watch (objective 1): checks the sources of `veille/sources.json` (crates.io, GitHub, RSS) and writes a digest of what changed, grouped by owner; run weekly by `.github/workflows/veille.yml` |
| `veille/sources.json` | The followed sources, with owner and reason |

See [`docs/tests/AUTOMATISATION_TESTS_QEMU.md`](../docs/tests/AUTOMATISATION_TESTS_QEMU.md) and [`docs/objectif1/02_dispositif_veille.md`](../docs/objectif1/02_dispositif_veille.md).

## Usage

```sh
# From repository root
bash scripts/kernel_tests/run.sh

# Same, with another heap allocator (heap-bump replays the October "before")
bash scripts/kernel_tests/run.sh -- --no-default-features --features heap-bump

# Research watch digest (state kept in veille-state.json)
python3 scripts/veille/check_sources.py --state veille-state.json --digest digest.md

# Self-tests of the results tool (no QEMU needed)
python3 -m unittest discover -s scripts/kernel_tests
```
