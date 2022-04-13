use std::{env, error::Error, fs, io::Read, path::Path, result::Result as StdResult};

const OVMF_DOWNLOAD_URL: &str =
    "https://github.com/rust-osdev/ovmf-prebuilt/releases/latest/download/OVMF-pure-efi.fd";

type Result = StdResult<(), Box<dyn Error>>;

fn main() -> Result {
    let out_dir = env::var("OUT_DIR")?;

    download_bios(&out_dir)?;

    Ok(())
}

fn download_bios(out_dir: impl AsRef<Path>) -> Result {
    let mut bytes = Vec::new();
    ureq::get(OVMF_DOWNLOAD_URL)
        .call()?
        .into_reader()
        .read_to_end(&mut bytes)?;

    fs::write(out_dir.as_ref().join("OVMF.fd"), bytes)?;

    Ok(())
}
