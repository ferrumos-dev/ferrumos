# FerrumOS

> **FerrumOS** is an experimental operating system written entirely in **Rust**, built for pedagogical and research purposes. It aims to provide deep, hands-on mastery of fundamental OS layers — boot, memory management (paging), interrupts, scheduling, and kernel architecture — while leveraging Rust's memory safety guarantees with minimal `unsafe` code.

**Start here** · [Build the project](#project-skeleton) · [Source architecture](./docs/architecture.md) · [Documentation](./docs/README.md) · [Greenlight](./docs/greenlights/README.md) · [Contributing](./CONTRIBUTING.md)

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

- **Boot process** — custom bootloader pipeline using `cargo-bootimage`
- **Memory management** — x86_64 paging (4-level page tables), isolation, custom heap allocator (Bump → linked-list / fixed-size blocks)
- **Interrupt handling** — IDT, PIC/APIC, timer, keyboard, exception handlers
- **Scheduling** — adaptive-priority simulation in the scheduler PoC and a priority policy proposed for the integrated kernel
- **System interface** — CLI shell with command parsing, task manager, resource monitoring

The project is structured around **4 team members**, each responsible for a dedicated technical area, working together through clearly defined interfaces.

---

## Project Status

**Reference PoC milestones reported in the EIP documentation**

- Boot stable and reproducible under QEMU
- Memory initialization without critical errors
- Hardware interrupts operational (timer, keyboard)
- CLI shell with `help`, `version`, `mem` commands

The project currently has **3 independent Proofs of Concept** and a compilable project skeleton in [`src/`](./src/). The source tree reserves the kernel, hardware, driver, service, interface and program layers for future integration. It contains module declarations and documentation, with no implemented OS functionality or bootable image yet. The next major milestone is integrating the PoCs into this common structure.

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
│   ├── ORGANIZATION.md    Team organization and project context
│   ├── dossiers/          EIP design dossiers
│   ├── eip/               Briefs and task allocation
│   ├── greenlights/       Greenlight documents, organized by objective
│   └── presentations/    Presentation materials
├── libs/         Shared no_std crates (ferrum_alloc: kernel heap allocators)
├── POC/          Independent reference, scheduler and graphics prototypes
├── research/     Host benchmarks and experiments (heap allocator)
├── scripts/      Repository utilities: kernel tests under QEMU, research watch
└── targets/      Shared target specifications
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
| **Rust 1.94.0** | Project skeleton compilation | Pinned toolchain, `no_std` library, host and `x86_64-unknown-none` builds |
| **Rust nightly** | Prototype compilation | Experimental kernel features and `build-std` used by the PoCs |
| **cargo-bootimage / bootloader** | Bootable image generation | Builds a bootable image from Rust source |
| **QEMU x86_64** | Execution & validation | Isolation, reproducibility, easy debugging |
| **GDB** | Kernel debugging | Register/memory/IDT inspection live |
| **Git + GitHub + CI/CD** | Versioning & integration | Component traceability, automated validation |
| **Make** | Build/boot automation | Targets: `install`, `boot`, `build`, `run`, `run-debug`, `run-serial`, `clean` |
| **Custom target** | `x86_64-ferrumos.json` | Bare-metal kernel specification |

---

## Getting Started

### Project Skeleton

The root Cargo package `ferrumos` uses Rust **1.94.0**, selected by
`rust-toolchain.toml`. With `rustup` installed, run all project commands from the
repository root:

```sh
cargo build --workspace --lib --locked --offline
cargo build --workspace --lib --target x86_64-unknown-none --locked --offline
```

The first invocation installs the pinned toolchain and components if needed, which
requires network access. Subsequent Cargo builds need no network: the skeleton has
no external dependencies. The results are libraries under `target/`, not bootable
images. See the [source architecture](./docs/architecture.md) for module boundaries
and the correspondence with the design documents.

### Quality Checks

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo clippy --workspace --lib --target x86_64-unknown-none --locked --offline -- -D warnings
cargo doc --workspace --no-deps --locked --offline
```

The [project CI](./.github/workflows/kernel.yml) runs these checks and both library
builds from the root, treating documentation warnings as errors as well. There are
no behavioral tests yet because the modules contain no implementation. Add tests
with each future contract or behavior; compilation alone does not validate boot or
hardware operation.

### Prototype Prerequisites

- **Rust nightly** (used by the prototype build pipelines and `build-std`)
- **QEMU** (for running the OS)
- `cargo-bootimage` and `rust-src` component

The PoCs remain independent Cargo projects, excluded from the root workspace.
`POC/rust-toolchain.toml` selects nightly for them; the graphics PoC also retains
its own toolchain file. Run prototype setup commands from the relevant PoC directory.

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
`src/` and describes the responsibilities of the future modules.

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

- **Memory**: x86_64 paging, 4-level page tables, strict isolation, custom Bump Allocator (evolving toward linked-list / fixed-size blocks)
- **Scheduling**: see the [priority scheduler proposal](./docs/greenlights/objecitf_obligatoire_1/ordonnancement_a_priorites.md) and its integration tests
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
- [`targets/x86_64-ferrumos.json`](./targets/x86_64-ferrumos.json) — Canonical bare-metal target specification
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
