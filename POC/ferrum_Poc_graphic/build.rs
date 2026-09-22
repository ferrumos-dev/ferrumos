use std::{env, path::PathBuf};

fn main() {
    let kernel = PathBuf::from(env::var_os("CARGO_BIN_FILE_KERNEL_kernel").expect("kernel artifact missing"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR missing"));

    let bios_path = out_dir.join("ferrumos_graphic-bios.img");
    bootloader::DiskImageBuilder::new(kernel)
        .create_bios_image(&bios_path)
        .expect("failed to create BIOS image");

    println!("cargo:rustc-env=FERRUMOS_BIOS_IMAGE={}", bios_path.display());
    println!("cargo:rerun-if-changed=kernel/src");
    println!("cargo:rerun-if-changed=kernel/Cargo.toml");
}
