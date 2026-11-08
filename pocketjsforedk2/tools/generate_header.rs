//! Build with rustc, linking this project's ABI rlib; see docs/abi.md.
use pocketjs_boot_context_abi as abi;
use std::{env, fs, mem, path::PathBuf, process::Command};

fn main() {
    let root = PathBuf::from(env::args().nth(1).unwrap_or_else(|| ".".into()));
    let root = root.canonicalize().expect("project directory");
    let tool = env::var("CBINDGEN").unwrap_or_else(|_| "cbindgen".into());
    let version = Command::new(&tool).arg("--version").output().expect("cbindgen");
    assert!(version.status.success());
    assert_eq!(String::from_utf8_lossy(&version.stdout).trim(), "cbindgen 0.29.2");
    let result = Command::new(tool)
        .arg("--config").arg(root.join("cbindgen.toml"))
        .arg(&root).output().expect("generate C header");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let mut header = String::from_utf8(result.stdout).expect("UTF-8 header");
    // cbindgen 0.29.2 drops BareFn ABI metadata. Add only the efiapi marker;
    // argument types and all struct/constant definitions come from Rust.
    for name in ["PJB_INVOKE", "PJB_READ_POWER", "PJB_SET_QUIESCED", "PJB_READ_CHARGE_ENABLED", "PJB_SET_CHARGE_ENABLED"] {
        let marker = format!("(*{name})");
        assert_eq!(header.matches(&marker).count(), 1, "unexpected callback emission");
        header = header.replacen(&marker, &format!("(PJB_EFIAPI *{name})"), 1);
    }
    // cbindgen cannot emit a struct constant containing an array. Generate
    // the initializer from the compiled Rust value rather than duplicate it.
    let guid = abi::CONTEXT_GUID;
    let bytes = guid.data4.iter().map(|b| format!("0x{b:02x}")).collect::<Vec<_>>().join(", ");
    let mut checks = format!("\n#define PJB_CONTEXT_GUID ((PJB_GUID){{0x{:08x}, 0x{:04x}, 0x{:04x}, {{{bytes}}}}})\n", guid.data1, guid.data2, guid.data3);
    let guid = abi::PLATFORM_GUID;
    let bytes = guid.data4.iter().map(|b| format!("0x{b:02x}")).collect::<Vec<_>>().join(", ");
    checks.push_str(&format!("#define PJB_PLATFORM_GUID ((PJB_GUID){{0x{:08x}, 0x{:04x}, 0x{:04x}, {{{bytes}}}}})\n", guid.data1, guid.data2, guid.data3));
    let guid = abi::CHARGE_CONTROL_GUID;
    let bytes = guid.data4.iter().map(|b| format!("0x{b:02x}")).collect::<Vec<_>>().join(", ");
    checks.push_str(&format!("#define PJB_CHARGE_CONTROL_GUID ((PJB_GUID){{0x{:08x}, 0x{:04x}, 0x{:04x}, {{{bytes}}}}})\n", guid.data1, guid.data2, guid.data3));
    checks.push_str("\n#if defined(__cplusplus)\n#define PJB_ASSERT static_assert\n#else\n#define PJB_ASSERT _Static_assert\n#endif\n");
    let guid = abi::BOOT_DISPLAY_GUID;
    let bytes = guid.data4.iter().map(|b| format!("0x{b:02x}")).collect::<Vec<_>>().join(", ");
    checks.push_str(&format!("#define PJB_BOOT_DISPLAY_GUID ((PJB_GUID){{0x{:08x}, 0x{:04x}, 0x{:04x}, {{{bytes}}}}})\n", guid.data1, guid.data2, guid.data3));
    for (name, size) in [
        ("PJB_BOOT_DISPLAY", mem::size_of::<abi::BootDisplay>()),
        ("void *", mem::size_of::<usize>()),
        ("PJB_GUID", mem::size_of::<abi::Guid>()),
        ("PJB_HEAP_RANGE", mem::size_of::<abi::HeapRange>()),
        ("PJB_REQUEST", mem::size_of::<abi::Request>()),
        ("PJB_RESPONSE", mem::size_of::<abi::Response>()),
        ("PJB_CONTEXT_TABLE", mem::size_of::<abi::ContextTable>()),
        ("PJB_PLATFORM_PROTOCOL", mem::size_of::<abi::PlatformProtocol>()),
        ("PJB_CHARGE_CONTROL_PROTOCOL", mem::size_of::<abi::ChargeControlProtocol>()),
        ("PJB_POWER_SAMPLE", mem::size_of::<abi::PowerSample>()),
    ] {
        checks.push_str(&format!("PJB_ASSERT(sizeof({name}) == {size}, \"Rust/C layout: {name}\");\n"));
    }
    for (field, offset) in [
        ("runtime_instance", mem::offset_of!(abi::ContextTable, runtime_instance)),
        ("invoke", mem::offset_of!(abi::ContextTable, invoke)),
    ] {
        checks.push_str(&format!("PJB_ASSERT(offsetof(PJB_CONTEXT_TABLE, {field}) == {offset}, \"Rust/C field: {field}\");\n"));
    }
    checks.push_str("#undef PJB_ASSERT\n\n");
    let end = header.rfind("#endif").expect("header guard");
    header.insert_str(end, &checks);
    let target = root.join("include/pocketjs_boot_context.h");
    if env::args().any(|arg| arg == "--check") {
        assert_eq!(fs::read_to_string(target).expect("existing header"), header, "generated header is stale");
        println!("PASS: generated header matches Rust source");
    } else {
        fs::write(target, header).expect("write generated header");
        println!("Generated pocketjs_boot_context.h");
    }
}
