use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let kernel_binary = env::args_os().nth(1).unwrap();
    let kernel_binary = &absolute(kernel_binary);
    let target_dir = kernel_binary.parent().unwrap();
    let qemu_fs_dir = &target_dir.join("qemu_fs");
    let efi_boot_dir = &qemu_fs_dir.join("EFI").join("Boot");
    let ovmf = Path::new(concat!(env!("OUT_DIR"), "/OVMF.fd"));

    fs::create_dir_all(efi_boot_dir).unwrap();
    fs::copy(kernel_binary, efi_boot_dir.join("BootX64.efi")).unwrap();

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!(
        "format=raw,file=fat:rw:file={}",
        qemu_fs_dir.display()
    ));
    qemu.arg("-bios").arg(ovmf.to_str().unwrap());
    qemu.arg("-machine").arg("q35");
    qemu.arg("-serial").arg("stdio");
    qemu.arg("-net").arg("none");
    qemu.arg("-m").arg("1024");

    // run the command
    let exit_status = qemu.status().unwrap();
    if !exit_status.success() {
        panic!("bootloader build failed");
    }
}

fn absolute(path: impl AsRef<Path>) -> PathBuf {
    let canonicalized = path.as_ref().canonicalize().unwrap();
    let canonicalized = canonicalized
        .strip_prefix(r"\\?\")
        .unwrap_or(&canonicalized);
    canonicalized.to_owned()
}
