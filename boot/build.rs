use std::{env, error::Error, fs, io::Read, path::Path};

const OVMF_DOWNLOAD_ROOT: &str =
    "https://github.com/rust-osdev/ovmf-prebuilt/releases/latest/download";

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;

    download_file("OVMF_CODE-pure-efi.fd", &out_dir)?;

    download_file("OVMF_VARS-pure-efi.fd", &out_dir)?;

    Ok(())
}

fn download_file(name: &str, out_dir: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let mut bytes = Vec::new();
    ureq::get(&format!("{OVMF_DOWNLOAD_ROOT}/{name}"))
        .call()?
        .into_reader()
        .read_to_end(&mut bytes)?;

    fs::write(out_dir.as_ref().join(name.replace("-pure-efi", "")), bytes)?;

    Ok(())
}
