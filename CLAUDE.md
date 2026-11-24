# CLAUDE.md

Guidance for working in the FerrumOS repository.

## Project

FerrumOS is a pedagogical x86_64 operating system written in Rust (Epitech EIP,
team of 4). The goal is mastery of boot, paging, interrupts, scheduling and kernel
architecture with minimal `unsafe`. Most project documentation is in **French**;
code, rustdoc and comments are in **English**. The contributor-facing docs
(`README.md`, `CONTRIBUTING.md`, `docs/greenlights/objectif_complementaire_1/deliverables/`)
are in English.

Development happens in the public repository `ferrumos-dev/ferrumos` (issues, PRs,
project board). `.github/workflows/sync-epitech.yml` copies every branch to the
Epitech repository once `kernel.yml` and `kernel-tests.yml` pass; never push or open
PRs there directly. Code is MIT-licensed (`LICENSE`).

The repository holds independent parts that build differently:

| Part | What it is | Toolchain |
| --- | --- | --- |
| `src/` (root crate `ferrumos`) | Integrated kernel: library + `baremetal` binary. Boots in QEMU (BIOS), framebuffer text console, COM1 diagnostics, polled PS/2 keyboard (French AZERTY) with line editing. | Rust **1.94.0** (`rust-toolchain.toml`), host + `x86_64-unknown-none` |
| `tools/image-builder/` | Host tool that wraps the kernel ELF into a BIOS image with `bootloader 0.11.15` | `nightly-2026-03-07` (own manifest, lockfile, toolchain file) |
| `POC/` | Three standalone prototypes (reference, adaptive scheduler, graphics) | Nightly: `POC/rust-toolchain.toml` (`nightly-2026-01-15`); graphics PoC pins `nightly-2026-11-16` |
| `libs/ferrum_alloc` + `research/heap-allocator/bench` | Kernel heap allocators (bump, linked list, fixed blocks) and their host benchmark | Stable for host tests and bench; built into the scheduler PoC with nightly + `build-std` |

The root workspace **excludes** the PoCs, `libs/ferrum_alloc`, the bench and the image
builder; each keeps its own manifest and checks.

Not yet in the integrated kernel: heap, FerrumOS-owned paging, timer/IRQs, scheduler,
filesystem, user space. The boot loop keeps maskable interrupts disabled and polls the
keyboard. `src/ui/shell` (parser, `help`/`version`/`clear`/`stats`, `Console` and
`StatsProvider` traits) exists as a host-tested portable module but is not yet wired
into the boot loop, which only echoes lines. PoC features do not imply the root
kernel has them.

## Commands

### Root crate (run from repo root)

```sh
cargo fetch --locked
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --lib --locked --offline
cargo build --workspace --lib --target x86_64-unknown-none --locked --offline
cargo clippy --workspace --lib --bin ferrumos --features baremetal --target x86_64-unknown-none --locked --offline -- -D warnings
cargo doc --workspace --no-deps --locked --offline
python -m unittest discover -s scripts -p test_boot.py -v
python scripts/boot.py test --timeout 30
```

These mirror `.github/workflows/kernel.yml` (doc warnings are errors in CI). Run a
single Rust test with `cargo test --workspace --lib --locked --offline <name>`.
Use `python3` where that is the interpreter name; Python 3.11+ is required.

### Boot the integrated kernel

```sh
python scripts/boot.py build     # kernel ELF + target/boot/ferrumos-bios.img
python scripts/boot.py run       # rebuilds, then opens QEMU (Ctrl+Alt releases capture)
python scripts/boot.py test --timeout 30
```

`boot.py` selects each toolchain explicitly. Set `QEMU=/path/to/qemu-system-x86_64` if
QEMU is not on `PATH`. The test boots headless and waits for the serial markers
`FERRUMOS_BOOT_OK`, `DISPLAY_READY`, `KEYBOARD_READY`. It injects `t e s x ⌫ t ⏎`
through QMP and expects `INPUT_LINE: test`, then checks that the framebuffer capture
is non-uniform. Success prints `PASS: boot, framebuffer and PS/2 keyboard`.

Diagnostics go to `target/boot/`: `build.log`, `serial.log`, and `results/{serial.log,
qemu.stderr.log,screen.ppm,summary.json}`. A successful build alone does not prove the
drivers work; rerun the QEMU test after any boot or driver change.

Reference configuration: BIOS, QEMU `pc` machine, TCG, 1 CPU, 128 MiB RAM. UEFI, SMP and
real hardware are not validated.

### Prototypes (run from the PoC directory)

- **Scheduler PoC** `POC/ferrum_Poc_adaptative_sheduller/` (the only one with a test harness):
  `make install`, `make boot`, `make build`, `make run`, `make run-debug`
  (log in `/tmp/qemu_ferrumos.log`), `make run-serial`, `make test`, `make clean`.
  - `cargo test` boots 7 QEMU images: `lib` capability tests + `tests/experiment_1..6.rs` (each `harness = false`, fresh kernel per image).
  - Single experiment: `cargo test --test experiment_2`
  - Pick the heap allocator by feature: default `heap-linked-list`; `--no-default-features --features heap-fixed-block` or `heap-bump` (replays the October "before").
- **Graphics PoC** `POC/ferrum_Poc_graphic/`: `make boot`, `make build`, `make run` (root crate builds the disk image, `kernel/` is the `no_std` kernel).
- **Reference PoC** `POC/ferrumos_PoC/`: Windows script `build_and_run_ferrumos.bat`, or
  `cargo +nightly bootimage --target x86_64-ferrumos.json -Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec`

Each PoC keeps its own `x86_64-ferrumos.json` target; the root uses the built-in
`x86_64-unknown-none`.

### Scheduler-PoC kernel tests under QEMU, with baseline comparison

```sh
bash scripts/kernel_tests/run.sh
bash scripts/kernel_tests/run.sh -- --no-default-features --features heap-bump
bash scripts/kernel_tests/run.sh --previous old/results.json
python3 -m unittest discover -s scripts/kernel_tests               # results.py self-tests, no QEMU
```

Output goes to `kernel-test-results/` (git-ignored): `kernel-tests.log`, `results.json`,
`report.md`. Exit code 0 = pass, 1 = failed test or regression, 2 = missing tool.
QEMU exit code 33 = all tests passed, 35 = a test failed.

### Heap allocator library and benchmark

```sh
cd libs/ferrum_alloc && cargo test                                  # 9 unit tests
cd research/heap-allocator/bench && cargo run --release -- --output bench.json
```

Bench deterministic columns (iterations, errors, capacities, refusals) must be identical
across runs and match the QEMU runs; nanosecond columns are machine-dependent. The
TLSF row (`rlsf`) is exploration only; the kernel does not depend on it.

### Research watch

```sh
python3 scripts/veille/check_sources.py --state veille-state.json --digest digest.md
```

Sources live in `scripts/veille/sources.json`; `.github/workflows/veille.yml` runs it weekly.

## Pinned versions (do not upgrade casually)

- Root kernel: Rust `1.94.0`; `bootloader_api = "=0.11.15"`, `font8x8 = "=0.3.1"`, `pc-keyboard = "=0.7.0"`. The image builder's `bootloader` must stay on the same `0.11.15` as `bootloader_api`.
- Scheduler PoC (`POC/ferrum_Poc_adaptative_sheduller/test_environment.md`): `nightly-2026-01-15`, `bootimage` **0.10.3** (`cargo install bootimage --version 0.10.3 --locked`), `bootloader` 0.9.34, `x86_64` 0.14. `bootimage` 0.10.5 passes `-Zjson-target-spec`, which this nightly rejects; `x86_64` 0.14 breaks once the `Step` trait changes. Record any environment change in that file before publishing new measurements.
- Changes to pinned toolchains, required CI checks or the release process are team decisions.

## Architecture of `src/`

Layered, modular monolithic kernel (see `docs/architecture.md` and `docs/boot.md`):

```text
src/main.rs (bootloader_api entry, panic handler)
  -> boot::run (src/boot/runtime.rs owns the drivers for the input loop)
  -> COM1 (drivers/serial/uart.rs) -> framebuffer TextConsole (drivers/display)
  -> PS/2 keyboard (drivers/input/ps2.rs) -> InputLine<N> (drivers/console.rs)
```

Other layers (`kernel/{memory,interrupts,scheduler,executor,telemetry}`,
`drivers/{storage,network}`, `services/{filesystem,network}`, `ui/monitor`, `programs`)
are integration points with documented contracts.

Boundary rules to keep when implementing:

- `kernel` is `#![forbid(unsafe_code)]` and must not depend on UI, registers or device details. Privileged operations live in `arch/x86_64` (port I/O via `arch::x86_64::io`) and `drivers`.
- The hardware boot path compiles only for `x86_64-unknown-none` behind the `baremetal` feature; library and portable code stay host-testable.
- Keyboard and serial constructors require an exclusive owner. The framebuffer is borrowed mutably from `BootInfo`, with no global singleton: pass that borrow or an explicit output interface (`core::fmt::Write`) to consumers instead of creating a second owner.
- The shell consumes `InputEvent` / `InputLine<N>` and must not read keyboard ports directly.
- Before replacing polling with IRQs: establish the IDT, routing, acknowledgement and controller ownership first.
- The scheduler **selects** a task; the executor manages its progress and asks `arch` for the switch. A tick, a selection and an effective context switch are three distinct events (only the switch increments the context-switch counter).
- `drivers/network` = the NIC, `services/network` = protocols. `drivers/storage` = block device, `services/filesystem` = files.
- Shell and monitor consume kernel/service contracts. Snapshots are captured consistently, then displayed outside critical sections.
- Module separation is organizational only: it is neither a memory-protection boundary nor a Cargo package split.

Historical docs refer to `foundation/kernel-core/src/*.rs` from an archived evidence
bundle; it is not in this repo. The mapping to current paths is in `docs/architecture.md`.
The archived contracts (`MemorySnapshot`, `TimerTickSink`, `RoundRobin<N>`, `Counters`,
`KernelSnapshot`) are described in `docs/greenlights/objecitf_obligatoire_2/md/contrats_noyau.md`
and remain the reference for future implementations.

## Rust rules

Enforced in the root `Cargo.toml`: `missing_docs = "deny"`, `unsafe_op_in_unsafe_fn = "deny"`,
`clippy::undocumented_unsafe_blocks = "deny"`, `panic = "abort"`. From
`docs/greenlights/objecitf_obligatoire_2/md/normes_rust.md`:

- Default `rustfmt` settings; Clippy with `-D warnings`. A lint exception is scoped to the item and justified in code or in the PR.
- Every `unsafe` block gets a `// SAFETY:` comment stating its invariant; every public `unsafe fn` gets a `# Safety` rustdoc section. Keep `unsafe` behind small interfaces.
- `Result` for errors the caller must handle; `Option` only for an expected absence (e.g. empty queue). Errors must leave state unchanged and consistent.
- Keep the kernel `no_std` and portable logic host-testable. Justify any new dependency (why it is needed, proof it builds for the bare-metal target, maintenance cost).
- Naming: `snake_case` functions/modules, `UpperCamelCase` types/traits, `SCREAMING_SNAKE_CASE` constants; name types after the concept (`TaskId`).
- Document public interfaces, units and error cases. If a kernel contract changes, update the contract doc and its callers, and add a test.
- Bound hardware waits and IRQ work. In the scheduler PoC, never write PIC masks with interrupts enabled (use `interrupts::set_masks` and friends).

## Heap allocator (`libs/ferrum_alloc`)

- Decision (ADR-002, proposed): address-ordered linked list with coalescing is the default kernel heap (`heap-linked-list`). Bump and fixed-block variants stay for comparison.
- API: `Heap::init`, `allocate`, `deallocate`, `stats` / `HeapStats`. `allocate` returns null on failure (no panic) and bumps `failed_allocations`. The linked list rejects double frees / overlapping frees (`rejected_frees`); bump and fixed blocks do not. `init` and `deallocate` are `unsafe`. No allocator takes a lock: the caller serializes.
- The crate has no dependencies on purpose (must build for the bare-metal target with `build-std core` and for the host).
- Known limit: the PoC heap lock does not mask interrupts, so an interrupt handler must never allocate.

## Tests, baseline and CI

- `kernel.yml`: root host checks, Rust and Python tests, bare-metal build and lint, docs, the real BIOS image under QEMU, image-builder lint, and uploads `ferrumos-boot-<sha>` diagnostics.
- `kernel-tests.yml`: `results-tool` self-tests, `heap-allocator` unit tests + bench, scheduler-PoC `qemu-tests`, and `store-results` on `main` (commits to the CI-only `test-results` branch).
- `ci.yml`: PoC builds and lint. `veille.yml`: weekly research watch. `sync-epitech.yml`: mirror to the Epitech repo after CI passes.
- `POC/ferrum_Poc_adaptative_sheduller/tests/baseline.json` holds `min`/`max`/`equals` rules for **deterministic** values only (iterations, errors, used bytes). Cycle counts vary up to 20% under QEMU/TCG and are never blocking.
- To add a scheduler-PoC measurement: emit `testing::Record { .. }.report()` (`RECORD name=... iterations=... errors=... used_before=... used_after=... cycles=...`), add its rule to `baseline.json`, and add any new `tests/experiment_N.rs` image to `expected_suites`. When an optimization intentionally improves a value, raise its baseline rule **in the same PR**.
- Do not mask failures in CI (`|| true`, `|| echo`). Upload artifacts with `if-no-files-found: error`.
- `RDTSCP` faults on the default QEMU CPU; use `RDTSC` only.

## Contributing and documentation

From `docs/greenlights/objectif_complementaire_1/deliverables/CONTRIBUTING.md` and
`.github/pull_request_template.md`:

- Topic branch from `origin/main` (e.g. `docs/windows-setup`); one behavior, correction or decision per PR. Link the issue if there is one.
- Commit titles: `[TAG] Short description` with `[ADD]` (new code or tests), `[FIX]`, `[DOC]`, `[MOVE]`; merges use `[MERGE]`.
- Fill the PR template (French): change description, then the verification table (formatting/Clippy, build/tests, QEMU if relevant) with command, result and target. Mention checks you could not run.
- What to verify per change: docs → follow the steps and open edited links; portable Rust → fmt, Clippy, host tests; boot/input/display → Python boot tests + QEMU smoke test, describe what appeared on screen; PoC/allocator/bench → that component's own checks, naming manifest, target and allocator or scenario.
- Another team member reviews; the reviewer checks the tests on the revision being merged.
- Shared kernel interfaces, boot protocol, memory ownership, scheduling policy or a new subsystem need a design issue before implementation.
- Record structural choices in `docs/greenlights/objecitf_obligatoire_2/md/registre_decisions.md` (ADR: date `YYYY-MM-DD`, status `Proposée`/`Acceptée`/`Remplacée`, at least two options on the same criteria; never delete a superseded entry). ADR-001 to ADR-003 exist.

Where documentation goes:

| Content | Location |
| --- | --- |
| Project overview, quality checks, PoC entry points | `README.md` |
| Integrated-kernel setup for contributors | `docs/greenlights/objectif_complementaire_1/deliverables/README.md` |
| Boot, input, diagnostics, boot tests | `docs/boot.md` |
| `src/` structure and module responsibilities | `docs/architecture.md` |
| PoC-specific procedure | that PoC's `README.md` |
| Greenlight evidence | `docs/greenlights/<objective>/`; objective 2 sheets go in `md/` with PDFs in `pdf/` |
| Briefs, EIP dossiers, presentations | indexed from `docs/README.md` |
| Team roles, RACI, roadmap | `docs/ORGANIZATION.md` |

New Markdown files: short `snake_case` names, relative links, spaces encoded as `%20`,
and update the relevant index. Evidence (measurements, logs) states revision, target and
date; attach images and long logs as CI artifacts and link them. The Greenlight folders
are spelled `objecitf_obligatoire_1` / `_2` (existing typo, keep it so links stay valid).
Plain-language docs for the heap objective share the glossary at
`docs/greenlights/objecitf_obligatoire_1/tas_noyau/lexique.md`.

## Team areas (for reviewers and ownership)

| Member | Area |
| --- | --- |
| Benjamin Ale | Bootloader, drivers, VM compatibility, ELF loader, `unsafe` audits; boot init order in `src/boot/` |
| Mathias Desgranges | Kernel core: paging, heap, interrupts, scheduler, kernel unit tests; author of the objective 2 architecture notes |
| Elias Lebrun | Shell/CLI, kernel↔UI API, documentation and accessibility, CI/CD |
| Rayan Benhabiles | Task manager, resource stats and telemetry, memory measurements, optimization prototype |
