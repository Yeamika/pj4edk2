use std::{env, fs, path::PathBuf};

fn replace(source: String, from: &str, to: &str, count: usize) -> String {
    assert_eq!(source.matches(from).count(), count, "upstream browser embedding changed: {from}");
    source.replace(from, to)
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let upstream = root.join("../components/pj4edk2/vendor/pocketjs/programs/web/wasm/src")
        .canonicalize().expect("initialize the PocketJS source submodule");
    let source_path = upstream.join("lib.rs");
    println!("cargo:rerun-if-changed={}", source_path.display());
    println!("cargo:rerun-if-changed={}", upstream.join("browser.rs").display());
    let mut source = fs::read_to_string(source_path).unwrap();
    source = source.lines().map(|line| line.strip_prefix("//!").map_or_else(|| line.to_owned(), |doc| format!("//{doc}"))).collect::<Vec<_>>().join("\n");
    source = replace(source, "#![allow(clippy::missing_safety_doc)]", "", 1);
    source = replace(source, "mod browser;", &format!("#[path = {:?}] mod browser;", upstream.join("browser.rs")), 1);
    source = replace(source, "ui.set_viewport(size[0] as f32, size[1] as f32);",
        "pj4edk2_rendering::configure(&mut ui);\n    ui.set_viewport(size[0] as f32, size[1] as f32);", 1);
    // Only the host's capture/presentation resource view changes. The core,
    // command stream, rasterizer, package admission and bindings stay upstream.
    source = replace(source, "&c.ui,", "&crate::PixelResources(&c.ui),", 5);
    source = replace(source, "&Auxiliary(&c.ui),", "&crate::PixelResources(&Auxiliary(&c.ui)),", 1);
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(output.join("browser_embedding.rs"), source).unwrap();
}
