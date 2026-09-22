# ferrum_Poc_adaptative_sheduller

Minimal Rust OS PoC (FerrumOS base), prepared for Garuda Linux, with QEMU boot through a `Makefile`.

## Steps (Garuda Linux)

1. Install system dependencies:
   ```bash
   make install
   ```

2. Install the Rust toolchain and bootimage:
   ```bash
   make boot
   ```

3. Build the bootable image:
   ```bash
   make build
   ```

4. Launch the VM and boot the OS:
   ```bash
   make run
   ```

At boot, FerrumOS now shows a minimal VGA welcome screen before entering the terminal. Press `Enter` or `T` to open the shell.

## Useful Make targets

- `make run`: launch QEMU with VGA window.
- `make run-debug`: launch QEMU with debug logs in `/tmp/qemu_ferrumos.log`.
- `make run-serial`: launch QEMU without window, serial output in terminal.
- `make clean`: clean build artifacts.

## Project content

- `src/`: base kernel code (VGA, interrupts, memory, shell, scheduler, minimal boot UI).
- `x86_64-ferrumos.json`: custom bare-metal x86_64 target.
- `Makefile`: build and VM boot automation.
- `requirements.txt`: dependency list for Garuda Linux.
