use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let source_css = manifest_dir.join("../../assets/styles/generated.css");
    let bundled_assets_dir = manifest_dir.join("assets");
    let bundled_css = bundled_assets_dir.join("generated.css");

    println!("cargo:rerun-if-changed={}", source_css.display());

    fs::create_dir_all(&bundled_assets_dir).expect("failed to create app assets directory");

    if source_css.exists() {
        fs::copy(&source_css, &bundled_css)
            .expect("failed to copy compiled stylesheet into desktop bundle assets");
    } else {
        fs::write(
            &bundled_css,
            "/* generated at build time placeholder; run `bun run build:styles` to sync */",
        )
        .expect("failed to write placeholder stylesheet");
    }
}
