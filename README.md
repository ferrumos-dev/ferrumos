# FerrumOS

> **FerrumOS** is an experimental operating system written entirely in **Rust**, built for pedagogical and research purposes. It aims to provide deep, hands-on mastery of fundamental OS layers — boot, memory management (paging), interrupts, scheduling, and kernel architecture — while leveraging Rust's memory safety guarantees with minimal `unsafe` code.

**Start here** · [Run FerrumOS](#integrated-kernel) · [Boot guide](./docs/boot.md) · [Source architecture](./docs/architecture.md) · [Documentation](./docs/README.md) · [Contributing](./CONTRIBUTING.md)

---

## Table of Contents

- [About](#about)
- [Project Status](#project-status)
- [Repository Layout](#repository-layout)
- [Proofs of Concept](#proofs-of-concept)
- [Tech Stack](#tech-stack)
- [Getting Started](#getting-started)
- [Architecture](#architecture)
- [Roadmap](#roadmap)
- [Team](#team)
- [Documentation](#documentation)
- [Changelog](#changelog)
- [License](#license)

---

## About

FerrumOS is not intended to compete with existing operating systems. Instead, its purpose is to **master the internals** of a modern OS from the ground up:

- **Boot process** — integrated BIOS image using `bootloader 0.11.15`; older PoCs retain their own pipelines
- **Memory management** — validated physical frames, dedicated heap pages and reusable linked-list allocation
- **Interrupt handling** — IDT, legacy PIC IRQ0, PIT timer and terminal exception diagnostics
- **Scheduling** — fixed priorities and FIFO ties in the integrated kernel, with bounded cooperative task steps
- **System interface** — CLI shell with command parsing, task manager, resource monitoring

The project is structured around **4 team members**, each responsible for a dedicated technical area, working together through clearly defined interfaces.

---

## Project Status

**Integrated kernel in `src/`**

- BIOS boot under QEMU from the root project, with a framebuffer text console and COM1 diagnostics
- Polled PS/2 keyboard input using French AZERTY, editable UTF-8 lines, Backspace, Enter and Escape
- Host driver tests and an automated QEMU check of boot, keyboard editing and a nonblank framebuffer
- Boot memory-map validation with usable-byte and whole-frame counts
- A 1 MiB heap mapped at `0x4444_4444_0000`, with owned allocations, automatic free on drop and real usage snapshots
- Periodic PIT interrupts near 100 Hz, copied uptime observations and timer wakeups from CPU halt
- Allocation-free priority scheduling and cooperative dispatch of the input and startup diagnostic tasks

The input console echoes editable lines. Command parsing, CPU context switching,
filesystem services and user space remain integration work.
See the [boot guide](./docs/boot.md) for the supported configuration and verification limits.

**Reference PoC milestones reported in the EIP documentation**

- Boot stable and reproducible under QEMU
- Memory initialization without critical errors
- Hardware interrupts operational (timer, keyboard)
- CLI shell with `help`, `version`, `mem` commands

The **3 independent Proofs of Concept** remain available as integration references.
Their reported features do not imply that those features already run in the root kernel.

---

## Repository Layout

```text
.
├── Cargo.toml            Root package and workspace configuration
├── Cargo.lock            Dependency resolution
├── rust-toolchain.toml   Pinned Rust toolchain
├── .github/              GitHub Actions and pull request template
├── CONTRIBUTING.md
├── src/
│   ├── lib.rs            FerrumOS library root (no_std)
│   ├── main.rs           Bare-metal bootloader entry point
│   ├── boot/             Bootloader integration and initialization
│   ├── arch/x86_64/      CPU, memory and interrupt mechanisms
│   ├── kernel/           Memory contracts, scheduling, execution and telemetry
│   ├── drivers/          Serial, display, input, storage and network devices
│   ├── services/         Filesystems and networking services
│   ├── ui/               Shell and monitoring
│   └── programs/         Future user programs
├── docs/
│   ├── README.md          Documentation index
│   ├── architecture.md   Source layout and module responsibilities
│   ├── boot.md           Build, launch, diagnostics and boot tests
│   ├── ORGANIZATION.md    Team organization and project context
│   ├── dossiers/          EIP design dossiers
│   ├── eip/               Briefs and task allocation
│   ├── greenlights/       Greenlight documents, organized by objective
│   └── presentations/    Presentation materials
├── libs/         Shared no_std crates (ferrum_alloc: kernel heap allocators)
├── POC/          Independent reference, scheduler and graphics prototypes
├── research/     Host benchmarks and experiments (heap allocator)
├── tools/image-builder/  Isolated host tool producing the BIOS image
└── scripts/      Root boot/run/test commands, PoC tests and research watch
```

---

## Proofs of Concept

| PoC | Role | Key contents |
|-----|------|-------------|
| `ferrumos_PoC/` | **Reference PoC (MVP)** | Boot (bootloader + `cargo bootimage`), VGA text, IDT (timer/keyboard), paging + Bump Allocator, CLI shell (`help`, `version`, `mem`) |
| `ferrum_Poc_adaptative_sheduller/` | **Adaptive scheduler** | Base kernel + scheduler + UI, VGA welcome screen, QEMU automation via `Makefile` |
| `ferrum_Poc_graphic/` | **Graphic rendering** | Modern pipeline (`bootloader 0.11` + `bootloader_api`), linear framebuffer, desktop, search bar, mouse + keyboard input |

---

## Tech Stack

| Tool | Role | Justification |
|------|------|---------------|
| **Rust 1.94.0** | Root library and kernel compilation | Pinned stable toolchain and built-in `x86_64-unknown-none` target |
| **nightly-2026-03-07** | Isolated image builder | Builds the BIOS stages required by `bootloader 0.11.15` |
| **bootloader / bootloader_api 0.11.15** | BIOS image and kernel ABI | Matching pinned versions for the boot interface |
| **ferrum_alloc / spin / x86_64** | Root heap and page mappings | Reuses the project's linked-list allocator with interrupt-safe locking and page-table adapters |
| **Python 3.11+** | Root build, launch and smoke-test commands | Standard-library script shared by local development and CI |
| **Rust nightly / cargo-bootimage** | Independent PoCs | Their versions and build commands remain scoped to each prototype |
| **QEMU x86_64** | Execution & validation | Isolation, reproducibility, easy debugging |
| **GDB** | Kernel debugging | Register/memory/IDT inspection live |
| **Git + GitHub + CI/CD** | Versioning & integration | Component traceability, automated validation |
| **Make** | Prototype build/boot automation | Existing PoC targets remain independent of the root build |
| **Custom targets in each PoC** | `x86_64-ferrumos.json` | Prototype specifications; the root uses the built-in target |

---

## Getting Started

### Integrated Kernel

Install **rustup**, **Python 3.11+** and **QEMU x86_64**, then prepare the pinned
toolchains using the [boot guide](./docs/boot.md#préparer-les-outils). From the
repository root:

```sh
python scripts/boot.py build
python scripts/boot.py run
python scripts/boot.py test --timeout 30
```

On systems where Python 3 is named `python3`, use that command instead.
`run` opens the QEMU console; click inside it to type, and use Ctrl+Alt to release
the keyboard and mouse. Each command rebuilds the root kernel and BIOS image,
failing instead of booting an old image when compilation fails.

The kernel, image and logs stay under `target/`, ignored by Git. Initial toolchain
and dependency downloads require the network. The [boot guide](./docs/boot.md)
describes the AZERTY input console, QEMU discovery and diagnostics.

### Quality Checks

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

The [project CI](./.github/workflows/kernel.yml) runs host checks and the BIOS
smoke test for heap allocate/free, repeated timer delivery, keyboard and framebuffer.
It preserves boot diagnostics. Documentation warnings
are errors in CI. The image builder and heap crates retain their own manifests
and lockfiles; the root workspace excludes them. See the [boot guide](./docs/boot.md#contrôles-et-preuves)
for the distinction between host tests, QEMU observations and remaining work.

### Prototype Prerequisites

- **Rust nightly** (used by the prototype build pipelines and `build-std`)
- **QEMU** (for running the OS)
- `cargo-bootimage` and `rust-src` component

The PoCs remain independent Cargo projects, excluded from the root workspace.
`POC/rust-toolchain.toml` selects nightly for them; the graphics PoC also retains
its own toolchain file, and the scheduler PoC pins its tested nightly. Run prototype
setup commands from the relevant PoC directory.

```sh
# Add the rust-src component
rustup component add rust-src

# Install bootimage
cargo install bootimage
```

### Build and Run

| Prototype | Entry point |
| --- | --- |
| [Reference PoC](./POC/ferrumos_PoC/README.md) | [Windows build and run script](./POC/ferrumos_PoC/build_and_run_ferrumos.bat), with the documented prerequisites |
| [Scheduler PoC](./POC/ferrum_Poc_adaptative_sheduller/README.md) | From its directory, `make build` and `make run` on a compatible Linux host |
| [Graphics PoC](./POC/ferrum_Poc_graphic/README.md) | From its directory, `make boot`, `make build` and `make run` on a compatible Linux host |

See the [QEMU operations guide](./docs/greenlights/objecitf_obligatoire_2/md/exploitation_qemu.md) for prerequisites, logs and the archived test procedure. The [CI guide](./docs/greenlights/objecitf_obligatoire_2/md/ci_cd.md) describes how to configure and verify the checks.

---

## Architecture

The [source architecture guide](./docs/architecture.md) maps these layers to
`src/` and distinguishes the implemented boot/driver path from future modules.

Layered architecture inspired by the C4 model:

```
+---------------------------+
|        Programs           |   User applications (future)
+---------------------------+
|         Services          |   Filesystem, networking (progressive addition)
+---------------------------+
|          UI / Shell       |   CLI shell, task manager, resource stats
+---------------------------+
|          Drivers          |   Keyboard, mouse, VGA, framebuffer
+---------------------------+
|          Kernel           |   CPU | Memory (paging) | IRQ | Scheduler
+---------------------------+
|        Bootloader         |   Hardware initialization
+---------------------------+
```

Key design decisions:

- **Memory**: the bootloader's physical direct map gives the paging adapter access to the active page tables. The kernel claims whole usable frames for its 1 MiB heap. `KernelHeap::allocate` returns zero-initialized `HeapBlock` handles that free their exact allocation on drop. Shared usage observations come from `MemoryReporter` under the same heap lock.
- **Timer**: the IDT routes legacy PIC IRQ0 to a bounded PIT handler. The handler records a tick and acknowledges the PIC. Foreground code reads `clock_snapshot()` and formats its copied `TickSnapshot` after the IRQ has returned.
- **Scheduling**: [`PriorityScheduler`](./src/kernel/scheduler/mod.rs) chooses ready IDs and [`CooperativeExecutor`](./src/kernel/executor/mod.rs) invokes task functions. The boot input task uses this path. A yielding task moves behind its peers, and a blocked task waits for an explicit wakeup.
- **Safety**: Rust's ownership model enforced; `unsafe` blocks strictly limited to IRQ handlers, context switches, and physical memory access

---

## Roadmap

### Three phases

| Phase | Description |
|-------|-------------|
| **Foundations** | System base (boot, VGA, memory) — *achieved via PoC* |
| **Kernel** | Functional + performant kernel (IRQ, paging, scheduler) — *in progress* |
| **Maturity** | Stabilization + advanced features (userspace, syscalls, networking) |

### 1-year objectives (H2 — 6 to 18 months)

- Isolated user space (Ring 3)
- POSIX-like syscall table
- ELF binary loader
- Minimal network driver (e1000)
- Priority scheduling integrated with real task execution
- Minimal FAT16 filesystem
- Real-time task manager with CPU/RAM/storage statistics
- Automated kernel unit tests in CI/CD

---

## Team

| Member | Role / Area |
|--------|-------------|
| **Benjamin Ale** | Bootloader, drivers, VM compatibility, ELF loader, low-level `unsafe` audits |
| **Mathias Desgranges** | Kernel core: memory (paging), interrupts, scheduler evolution, unit tests |
| **Elias Lebrun** | Shell/CLI, kernel↔UI liaison API, documentation & accessibility |
| **Rayan Benhabiles** | Task manager, resource stats (CPU/RAM/storage), AI optimization prototype |

See [`docs/ORGANIZATION.md`](./docs/ORGANIZATION.md) for the full task breakdown, RACI matrix, collaboration model, and per-member commitments.

---

## Documentation

- [Root kernel boot, input and verification](./docs/boot.md)
- [Source architecture and module responsibilities](./docs/architecture.md)
- [Documentation index](./docs/README.md) and [Greenlight dossier](./docs/greenlights/README.md)
- [Contribution and review guide](./CONTRIBUTING.md)
- [Kernel heap](./docs/greenlights/objecitf_obligatoire_1/tas_noyau/README.md) — Greenlight mandatory objective 1: kernel heap hard problem (99 → 10 000), research watch, prototypes, decision, integration
- [`libs/ferrum_alloc/`](./libs/ferrum_alloc/) — Kernel heap allocators (linked list by default), shared by the kernel and the host benchmark
- [`docs/ORGANIZATION.md`](./docs/ORGANIZATION.md) — Task distribution, roadmap, team commitments
- [`docs/dossiers/FerrumOS_Dossiers_Complets.md`](./docs/dossiers/FerrumOS_Dossiers_Complets.md) — Compiled design dossiers (requirements, context, architecture, budgeting, risks)
- [`docs/presentations/FerrumOS_Presentations_Summary.md`](./docs/presentations/FerrumOS_Presentations_Summary.md) — Presentation summaries
- [`docs/eip/enonce_suivi1.txt`](./docs/eip/enonce_suivi1.txt) — EIP (student project) brief
- [`docs/eip/repartitions.txt`](./docs/eip/repartitions.txt) — Initial task allocation
- Bare-metal targets: the root project builds for the built-in `x86_64-unknown-none` target; each PoC keeps its own `x86_64-ferrumos.json` specification
- Individual PoC READMEs in `POC/` — Build/run instructions per PoC

---

## Changelog

### `426d4e8` — [ADD] Documentation

- Added comprehensive project documentation: `ORGANIZATION.md`, `README.md`, EIP dossiers, presentations, and task allocation
- Repository structure reorganization: documentation consolidated under `docs/`, added CI/CD pipeline, root `.gitignore`, shared target specs, and scripts directory

### `c548875` — Initial commit

- Repository initialization with 3 Proofs of Concept: `ferrumos_PoC` (reference/MVP), `ferrum_Poc_adaptative_sheduller` (scheduler), `ferrum_Poc_graphic` (graphics/framebuffer)

---

## License

The [reference PoC README](./POC/ferrumos_PoC/README.md#license) mentions MIT. Confirm the license for the whole repository with the maintainers before distribution.
