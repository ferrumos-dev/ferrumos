# Getting started with FerrumOS

FerrumOS is an experimental x86_64 operating system written in Rust. The integrated kernel boots in QEMU, displays a text prompt and echoes the lines you type on a French AZERTY keyboard. The team uses separate prototypes to develop and test features such as scheduling and heap allocation.

## Get a working copy

Install Git, then clone the repository.

```sh
git clone https://github.com/ferrumos-dev/ferrumos.git
cd ferrumos
```

Run the commands below from the repository root on the default branch, `main`.

## Run the host tests

For a first check, install Python 3.11 or newer and run the boot-script tests. They exercise the launcher using Python's standard library. Use `python3` if that is the name of your Python executable.

```sh
python -m unittest discover -s scripts -p test_boot.py -v
```

The final line should be `OK`. To test the Rust drivers, install Rust using [rustup](https://rustup.rs/). On Ubuntu, install the host compiler and linker with `sudo apt install build-essential`. On Windows with the MSVC toolchain, install Visual Studio Build Tools with the **Desktop development with C++** workload and a Windows SDK.

```sh
rustup toolchain install 1.94.0 --profile minimal --component rustfmt,clippy --target x86_64-unknown-none
cargo fetch --locked
cargo test --workspace --lib --locked --offline
```

Look for `test result: ok`. These tests exercise console editing, framebuffer rendering and keyboard decoding on your machine. QEMU is only needed for the next section.

## Integrated Kernel

Install [QEMU for x86_64](https://www.qemu.org/download/) (`sudo apt install qemu-system-x86` on Ubuntu). The BIOS image builder uses a separate pinned Rust toolchain. Allow network access for the first build.

```sh
rustup toolchain install nightly-2026-03-07 --profile minimal --component rust-src,llvm-tools-preview,rustfmt,clippy
python scripts/boot.py test --timeout 30
```

The script builds the BIOS image, boots it in QEMU, types and edits a line, then checks the serial response and framebuffer. Look for `PASS: boot, framebuffer and PS/2 keyboard`. If it fails, look in `target/boot/build.log` for compilation errors and `target/boot/results/` for VM diagnostics. The [boot guide](../../../boot.md) covers QEMU discovery and keyboard settings.

To try it yourself, run `python scripts/boot.py run`. Click the QEMU window, type a line at the `>` prompt and press Enter. The console echoes the line using a French AZERTY layout. Ctrl+Alt releases the keyboard and mouse. To build the image alone, run `python scripts/boot.py build`.

## Propose a change

Pick an [open issue](https://github.com/ferrumos-dev/ferrumos/issues) or start with a setup correction or missing test. Fork the public repository, make the change on a branch and open a pull request against `ferrumos-dev/ferrumos:main`. Include what changed and how you checked it. See the [contribution guide](./CONTRIBUTING.md) and [scope of contributions](./contribution_scope.md).

