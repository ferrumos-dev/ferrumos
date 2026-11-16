//! Host-only BIOS image builder. The kernel itself uses the root stable toolchain.

use std::{env, error::Error, fs, path::PathBuf, process::ExitCode};

fn build() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let kernel = PathBuf::from(arguments.next().ok_or("missing kernel ELF path")?);
    let image = PathBuf::from(arguments.next().ok_or("missing output image path")?);
    if arguments.next().is_some() {
        return Err("usage: ferrumos-image-builder <kernel-elf> <bios-image>".into());
    }
    if !kernel.is_file() || fs::metadata(&kernel)?.len() == 0 {
        return Err(format!("kernel ELF is missing or empty: {}", kernel.display()).into());
    }
    if kernel.canonicalize()? == image.canonicalize().unwrap_or_else(|_| image.clone()) {
        return Err("kernel and output image must be different files".into());
    }
    if let Some(parent) = image.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    bootloader::DiskImageBuilder::new(kernel).create_bios_image(&image)?;
    if fs::metadata(&image)?.len() == 0 {
        return Err("bootloader produced an empty image".into());
    }
    println!("BIOS image: {}", image.display());
    Ok(())
}

fn main() -> ExitCode {
    match build() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Image build failed: {error}");
            ExitCode::FAILURE
        }
    }
}
