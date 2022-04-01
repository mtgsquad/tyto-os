use std::{env, error::Error, fs, path::Path};

const OVMF_DOWNLOAD_URL: &str =
    "https://github.com/rust-osdev/ovmf-prebuilt/releases/latest/download/OVMF-pure-efi.fd";

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);

    fs::write(
        out_dir.join("OMVF-pure-efi.fd"),
        ureq::get(OVMF_DOWNLOAD_URL).call()?.into_string()?,
    )?;

    Ok(())
}
