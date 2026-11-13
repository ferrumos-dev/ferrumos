# Test environment

Memory measurements are only comparable when taken in this exact environment.
Record any change here before publishing new before/after numbers.

To reproduce the measurements from scratch, follow `docs/memoire/PROTOCOLE_DE_MESURE.md`
instead: it lists the prerequisites, the commit to check out, the expected
values and what to report back.

## Toolchain

| Component | Version | Pinned in |
|---|---|---|
| Rust | `nightly-2026-01-15` (rustc 1.94.0-nightly) | `rust-toolchain.toml` |
| `bootimage` | 0.10.3 | installed with `cargo install bootimage --version 0.10.3` |
| `bootloader` | 0.9.34 | `Cargo.lock` |
| QEMU | 11.1.0 | host install |

Newer toolchains do not work: `bootimage` 0.10.5 passes `-Zjson-target-spec`,
and `x86_64` 0.14 stops compiling once the `Step` trait changes.

## Kernel configuration

| Setting | Value | Defined in |
|---|---|---|
| Target | `x86_64-ferrumos.json`, soft-float, `x86-softfloat` ABI | repository root of the PoC |
| Heap start | `0x_4444_4444_0000` | `src/allocator.rs` |
| Heap size | 100 KiB | `src/allocator.rs` |
| Allocator | bump, frees only when the live count reaches zero | `src/allocator.rs` |
| PIC masks at boot | all interrupts masked | `src/main.rs` |

## QEMU invocation

Tests run through `bootimage runner`, which calls:

```
qemu-system-x86_64 \
  -drive format=raw,file=<image> \
  -no-reboot \
  -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
  -serial stdio \
  -display none
```

No `-smp`, no `-m`: the default QEMU machine and memory size are used. Exit
code 33 means every test passed, 35 means a test failed.

## Layout

| Target | Content |
|---|---|
| `src/lib.rs` | the kernel as a library, plus the capability tests |
| `src/main.rs` | the bootable kernel: scheduler, boot screen and shell |
| `tests/experiment_1.rs` to `tests/experiment_5.rs` | one bootable image per experiment, each with `harness = false` |

## Running

```
cargo test
```

This builds and boots seven images: one for the capability tests and one per
experiment. To run a single experiment:

```
cargo test --test experiment_2
```

## Available measurements

| Source | What it gives |
|---|---|
| `allocator::stats()` | heap size, used, free, live allocations, total allocations, failed allocations |
| `allocator::try_alloc` / `allocator::free` | allocation that reports exhaustion instead of panicking |
| `memory::frame_stats()` | usable, allocated, free and reclaimed physical frames |
| `memory::with_memory` | access to the page mapper and the frame allocator |
| `interrupts::ticks()` | timer ticks since boot, ~55 ms each, too coarse for allocator timings |
| `timing::cycles()` / `Stopwatch` / `measure` | TSC cycles, the resolution used for before/after comparisons |
| `testing::Rng` | xorshift generator, fixed seed, for reproducible allocation sequences |
| `testing::Record` | one-line measurement record: name, iterations, errors, used before/after, cycles |
| `interrupts::unmask_timer()` / `mask_all()` | run a test with or without timer interrupts |

## Measurement caveats

- QEMU runs without KVM here, so TSC values are emulated. They are stable
  enough to compare two runs in this same environment, but they are not real
  hardware timings.
- `RDTSCP` triggers an invalid opcode on the default QEMU CPU model, which
  escalates to a double fault. Only `RDTSC` is used.
- There is no handler for invalid opcode or general protection faults yet, so
  such a bug surfaces as a double fault rather than a precise message.

## Known findings

- Unmapping a page returns its frame, but the 3 page-table frames allocated by
  the first `map_to` on a fresh address range are never reclaimed. Later cycles
  on that same range are frame neutral, as experiment 4 shows over 500 cycles.
- The bump allocator never reuses freed memory unless every allocation is
  freed, which is the limitation the optimisation cycles must address.
- The scheduler simulates tasks: it has no per-task stack and no context
  switch, so the multi-task half of experiment 5 cannot run yet.
- Writing the PIC masks without disabling interrupts deadlocks the kernel;
  use `interrupts::set_masks` and friends, which guard against it.
- Each experiment is its own integration test under `tests/`, so it runs in a
  freshly booted kernel with an untouched heap. Nothing an experiment leaves
  behind can reach another one.
- A single long lived allocation disables the bump reset for everything that
  follows it inside one boot, which is why the frame reclaim list is a fixed
  array rather than a Vec.
- The timer handler does not allocate, so experiment 5 never exercises the
  allocator lock from an interrupt.
