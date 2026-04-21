use std::path::PathBuf;

fn main() {
    let metadata_path = PathBuf::from("../metadata/finney.scale");

    if metadata_path.exists() {
        println!("cargo::rerun-if-changed={}", metadata_path.display());
    } else {
        println!(
            "cargo::warning=Metadata file not found at {}. \
             Run: subxt metadata --url wss://entrypoint-finney.opentensor.ai:443 -f bytes > {}",
            metadata_path.display(),
            metadata_path.display()
        );
    }
}
