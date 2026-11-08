use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=POCKETJS_QUICKJS_INCLUDE");
    // Otherwise the final BSP link supplies this exact official guest archive,
    // built with the same QuickJS headers/options as its interpreter library.
    let Some(include) = env::var_os("POCKETJS_QUICKJS_INCLUDE") else {
        return;
    };
    let upstream = PathBuf::from("../../../vendor/pocketjs");
    let guest = upstream.join("engine/guest-quickjs");
    println!("cargo:rerun-if-changed={}", guest.display());
    println!(
        "cargo:rerun-if-changed={}",
        upstream.join("contracts/generated").display()
    );
    cc::Build::new()
        .file(guest.join("src/guest.c"))
        .include(include)
        .include(guest.join("include"))
        .include(guest.join("src"))
        .include(upstream.join("contracts/generated"))
        .flag_if_supported("-std=c11")
        .compile("pocket_guest_quickjs");
}
