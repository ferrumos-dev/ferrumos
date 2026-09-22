# ferrum_Poc_graphic

Rust OS graphics PoC for FerrumOS.

This project now uses a modern boot pipeline (`bootloader 0.11` + `bootloader_api`) and renders on a linear framebuffer.

## Goals

- Linear framebuffer rendering (not legacy VGA 320x200 mode)
- Cleaner modern desktop style
- Bottom search bar and centered terminal
- Keyboard + mouse interaction

## Commands

```bash
make boot
make build
make run
```

## Notes

The workspace contains two crates:
- root crate: builds disk image and runs QEMU
- `kernel/`: no_std kernel using `bootloader_api::BootInfo` framebuffer
