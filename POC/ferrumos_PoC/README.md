# FerrumOS

FerrumOS is a minimal Proof of Concept operating system written in Rust, demonstrating the core architecture of a Rust-based OS.

## Features

- **Boot Process**: Uses `bootloader` crate and `cargo bootimage`.
- **VGA Output**: Basic text mode driver.
- **Interrupts**: IDT for exceptions and hardware interrupts (Timer, Keyboard).
- **Memory Management**: Paging initialization and a simple Bump Allocator.
- **Shell**: A minimal command-line interface with `help`, `version`, and `mem` commands.
- **Architecture**: Modular design separating VGA, Memory, Interrupts, and Shell logic.

## Prerequisites

- Nightly Rust (required for `x86_64-unknown-none` and `build-std`)
- `cargo-bootimage` tool
- QEMU

### Install Dependencies

```sh
# Add rust-src component
rustup component add rust-src

# Install bootimage
cargo install bootimage
```

## Building and Running

On Windows, the [build and run script](./build_and_run_ferrumos.bat) sets up the MSVC Build Tools environment, builds the image and starts QEMU. It expects the paths shown in the script.

With the nightly toolchain, `rust-src`, `llvm-tools-preview`, `bootimage` and QEMU available, the equivalent commands from this directory are:

```sh
cargo +nightly bootimage --target x86_64-ferrumos.json -Zbuild-std=core,alloc,compiler_builtins -Zbuild-std-features=compiler-builtins-mem -Zjson-target-spec
qemu-system-x86_64 -no-reboot -no-shutdown -drive format=raw,file=target/x86_64-ferrumos/debug/bootimage-ferrumos.bin
```

See the [QEMU operations guide](../../docs/greenlights/objecitf_obligatoire_2/md/exploitation_qemu.md) for the distinction between the current prototypes and the independent archived test harness.

## Project Structure

- `src/main.rs`: Kernel entry point (`_start`), initialization sequence, and main loop.
- `src/vga_buffer.rs`: VGA text mode driver using `volatile` writes and a spinlock-protected global writer.
- `src/interrupts.rs`: Interrupt Descriptor Table (IDT) setup, PIC initialization, and interrupt handlers (Timer, Keyboard, Double Fault).
- `src/memory.rs`: Paging initialization and frame allocator setup using boot information.
- `src/allocator.rs`: Implementation of a simple Bump Allocator for heap memory management.
- `src/shell.rs`: Minimal shell logic, command parsing, and keyboard input handling.
- `x86_64-ferrumos.json`: Custom target specification for the kernel.

## Architecture Notes

- **Concurrency**: `spin::Mutex` is used for synchronizing access to shared resources like the VGA buffer and memory allocator.
- **Interrupts**: The IDT handles exceptions and hardware interrupts. The keyboard handler pushes input to a buffer consumed by the shell.
- **Memory**: The kernel uses recursive page tables (or offset page tables via `bootloader`) to map physical memory. The bump allocator allows dynamic memory allocation (`Vec`, `Box`, `String`).

## License

MIT
