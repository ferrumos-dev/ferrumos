# Scripts

Utility scripts for the FerrumOS project.

## Contents

| Script | Purpose |
|--------|---------|
| `boot.py` | Build, launch or test the integrated root kernel BIOS image; Python 3.11+, outputs under `target/boot/` |
| `test_boot.py` | Host self-tests for the root image builder orchestration and QEMU test client |
| `kernel_tests/run.sh` | Run the kernel tests under QEMU, save the results and compare them with the baseline (local and CI) |
| `kernel_tests/results.py` | Parse `cargo test` output into JSON, compare with the baseline, append to the CSV history |
| `kernel_tests/store_history.sh` | CI only: commit each run's results to the `test-results` branch |
| `kernel_tests/test_results.py` | Self-tests for `results.py` |
| `veille/check_sources.py` | Research watch (objective 1): checks the sources of `veille/sources.json` (crates.io, GitHub, RSS) and writes a digest of what changed, grouped by owner; run weekly by `.github/workflows/veille.yml` |
| `veille/sources.json` | The followed sources, with owner and reason |

See [`docs/greenlights/objecitf_obligatoire_2/md/automatisation_tests_qemu.md`](../docs/greenlights/objecitf_obligatoire_2/md/automatisation_tests_qemu.md) and [`docs/greenlights/objecitf_obligatoire_1/tas_noyau/02_dispositif_veille.md`](../docs/greenlights/objecitf_obligatoire_1/tas_noyau/02_dispositif_veille.md).

## Usage

The root kernel has its own build and boot commands. They do not launch a PoC:

```sh
python scripts/boot.py build
python scripts/boot.py run
python scripts/boot.py test --timeout 30
python -m unittest discover -s scripts -p test_boot.py -v
```

Use `python3` where required by the host. See the [boot guide](../docs/boot.md)
for pinned toolchains, QEMU discovery, keyboard controls and diagnostic files.
`run` and `test` rebuild the image before launching it. The test injects PS/2
keyboard events through QMP and checks serial acknowledgement and framebuffer output.

The historical scheduler/heap PoC tests and research watch keep their own commands:

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
