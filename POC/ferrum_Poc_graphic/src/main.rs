use std::process::Command;

fn main() {
    let image = env!("FERRUMOS_BIOS_IMAGE");
    let serial_fallback = std::env::var("FERRUMOS_SERIAL_FALLBACK")
        .ok()
        .as_deref()
        == Some("1");

    println!("QEMU: click inside window to capture mouse, Ctrl+G to release.");
    if serial_fallback {
        println!("Input mode: serial fallback (terminal stdin -> COM1)");
    } else {
        println!("Input mode: on-screen interactive (keyboard/mouse in QEMU window)");
    }

    let mut args = vec![
        "-drive".to_string(),
        format!("format=raw,file={image}"),
        "-machine".to_string(),
        "pc".to_string(),
        "-device".to_string(),
        "i8042".to_string(),
        "-monitor".to_string(),
        "none".to_string(),
        "-m".to_string(),
        "1024M".to_string(),
        "-no-reboot".to_string(),
        "-no-shutdown".to_string(),
    ];

    if serial_fallback {
        args.push("-serial".to_string());
        args.push("stdio".to_string());
    } else {
        args.push("-serial".to_string());
        args.push("none".to_string());
        args.push("-display".to_string());
        args.push("gtk,grab-on-hover=on".to_string());
    }

    let status = Command::new("qemu-system-x86_64")
        .args(args)
        .status()
        .expect("failed to execute qemu-system-x86_64");

    if !status.success() {
        eprintln!("QEMU exited with status: {status}");
    }
}
