use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("features.txt");

    let features: Vec<String> = env::vars()
        .filter(|(key, _)| key.starts_with("CARGO_FEATURE_"))
        .map(|(key, _)| key.trim_start_matches("CARGO_FEATURE_").to_lowercase())
        .collect();

    fs::write(&dest_path, features.join(",")).unwrap();

    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_GPUI");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_LEPTOS");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_DIOXUS");
}
