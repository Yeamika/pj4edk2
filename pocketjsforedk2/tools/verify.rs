//! Standalone host checks; does not build PocketJS or boot firmware.
use std::{env, fs, path::{Path, PathBuf}, process::Command};

fn run(program: &str, args: &[String], root: &Path) {
    let status = Command::new(program).args(args).current_dir(root).status().expect(program);
    assert!(status.success(), "{program} failed");
}
fn args(values: &[&str]) -> Vec<String> { values.iter().map(|v| v.to_string()).collect() }
struct Stage(PathBuf);
impl Drop for Stage { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

fn main() {
    let root = PathBuf::from(env::args().nth(1).unwrap_or_else(|| ".".into())).canonicalize().unwrap();
    let stage = Stage(PathBuf::from(env::var_os("TMPDIR").expect("set TMPDIR to project tmp")).join(format!("pjb-verify-{}", std::process::id())));
    fs::create_dir(&stage.0).unwrap();
    let lib = stage.0.join("libpocketjs_boot_context_abi.rlib");
    let external = format!("pocketjs_boot_context_abi={}", lib.display());
    let mut rust = args(&["--edition=2024", "--crate-name", "pocketjs_boot_context_abi", "--crate-type", "rlib", "-D", "warnings", "src/lib.rs", "-o"]);
    rust.push(lib.display().to_string());
    run("rustc", &rust, &root);
    let generator = stage.0.join("generate-header");
    let mut rust = args(&["--edition=2024", "tools/generate_header.rs", "--extern", &external, "-o"]);
    rust.push(generator.display().to_string());
    run("rustc", &rust, &root);
    let mut generator_args = vec![root.display().to_string()];
    if !env::args().any(|v| v == "--write-header") { generator_args.push("--check".into()); }
    run(generator.to_str().unwrap(), &generator_args, &root);

    let probe = stage.0.join("c-consumer.c");
    fs::write(&probe, r#"#include "pocketjs_boot_context.h"
PJB_STATUS pjb_test_call(const PJB_CONTEXT_TABLE *table, const PJB_REQUEST *request, PJB_RESPONSE *response) {
    return pjb_call(table, request, response);
}
PJB_STATUS pjb_test_invoke(const PJB_CONTEXT_TABLE *table, const PJB_REQUEST *request, PJB_RESPONSE *response) {
    return table->invoke(table->runtime_instance, request, response);
}
void pjb_test_guid(PJB_GUID *output) { *output = PJB_CONTEXT_GUID; }
"#).unwrap();
    let object = stage.0.join("c-consumer.o");
    let mut cc = args(&["-std=c11", "-ffreestanding", "-Wall", "-Wextra", "-Werror", "-pedantic", "-I"]);
    cc.push(root.join("include").display().to_string());
    let mut native = cc.clone();
    native.extend(args(&["-c", probe.to_str().unwrap(), "-o", object.to_str().unwrap()]));
    run("cc", &native, &root);
    let mut arm = cc;
    arm.extend(args(&["-fsyntax-only", probe.to_str().unwrap()]));
    if env::args().any(|arg| arg == "--native-only") {
        println!("PASS: generated header C11 layout and calls for native X64; AArch64 C check skipped");
    } else {
        run("aarch64-linux-gnu-gcc", &arm, &root);
        println!("PASS: generated header C11 layout and calls for X64/AArch64");
    }
    let smoke = stage.0.join("ffi-smoke");
    let link = format!("link-arg={}", object.display());
    run("rustc", &args(&["--edition=2024", "-D", "warnings", "checks/ffi_smoke.rs", "--extern", &external, "-C", &link, "-o", smoke.to_str().unwrap()]), &root);
    run(smoke.to_str().unwrap(), &[], &root);
}
