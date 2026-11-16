# BIOS image builder

This standalone host tool combines the root kernel ELF and `bootloader` 0.11.15
into a BIOS disk image. It is excluded from the root Cargo workspace and uses
nightly-2026-03-07 only for the bootloader's build scripts. The root kernel remains
on Rust 1.94.0. Its lockfile is separate from the kernel's lockfile.

Use `python scripts/boot.py build` from the repository root. The script selects the
toolchains explicitly, builds the kernel and writes `target/boot/ferrumos-bios.img`
only after the image builder succeeds. `run` opens QEMU and `test` runs the hidden
boot/display/keyboard smoke test.

Direct use, from this directory, after compiling the root kernel:

```sh
cargo run --locked -- ../../target/x86_64-unknown-none/debug/ferrumos ../../target/boot/ferrumos-bios.img
```

The first build downloads the BIOS stages and compiles them with `rust-src` and
`llvm-tools-preview`. On Windows, install Visual Studio C++ Build Tools for host
linking. QEMU is required only for `run` and `test`.
