use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let kernel_binary = env::args_os().nth(1).unwrap();
    let kernel_binary = &absolute(kernel_binary);
    println!("{:?}", kernel_binary);
    let target_dir = kernel_binary.parent().unwrap();
    let esp_dir = &target_dir.join("esp");
    let efi_boot_dir = &esp_dir.join("EFI").join("Boot");
    let out_dir = Path::new(env!("OUT_DIR"));

    fs::create_dir_all(efi_boot_dir).unwrap();
    fs::copy(kernel_binary, efi_boot_dir.join("BootX64.efi")).unwrap();

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive")
        .arg(format!("format=raw,file=fat:rw:file={}", esp_dir.display()));
    qemu.arg("-bios")
        .arg(out_dir.join("OVMF.fd").to_str().unwrap());
    qemu.arg("-machine").arg("q35");
    qemu.arg("-serial").arg("stdio");
    qemu.arg("-net").arg("none");
    qemu.arg("-m").arg("256M");
    qemu.arg("-nodefaults");
    qemu.arg("-vga").arg("std");
    qemu.arg("-no-reboot").arg("-no-shutdown");
    qemu.arg("-s");
    qemu.arg("-d").arg("guest_errors,cpu_reset");

    println!("{:?}", qemu);

    // run the command
    let exit_status = qemu.status().unwrap();
    if !exit_status.success() {
        panic!("bootloader build failed");
    }
}

fn absolute(path: impl AsRef<Path>) -> PathBuf {
    let canonicalized = path.as_ref().canonicalize().unwrap();
    let canonicalized = canonicalized.to_str().unwrap();

    PathBuf::from(canonicalized.strip_prefix(r"\\?\").unwrap_or(canonicalized))
}
