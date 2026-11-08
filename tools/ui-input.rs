//! Shared by the script and font build scripts. Paths are relative to ui.json.
use std::{env, fs, path::{Component, Path, PathBuf}};

// Each build script uses a different part of the validated UI inputs.
#[allow(dead_code)]
pub struct UiInputs {
    pub scripts: Vec<(String, PathBuf)>,
    pub font: PathBuf,
}

pub fn load(default_root: impl AsRef<Path>) -> UiInputs {
    println!("cargo:rerun-if-env-changed=PJB_DEFAULT_UI");
    let root = env::var_os("PJB_DEFAULT_UI")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_root.as_ref().to_owned())
        .canonicalize().expect("default UI directory must exist");
    let manifest = root.join("ui.json");
    assert!(manifest.canonicalize().expect("default UI must contain ui.json").starts_with(&root),
        "ui.json must stay inside the UI directory");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let config: serde_json::Value = serde_json::from_slice(
        &fs::read(&manifest).expect("default UI must contain ui.json"),
    ).expect("default UI ui.json must be valid JSON");
    assert_eq!(config["schema"].as_u64(), Some(1), "unsupported UI schema");
    let resolve = |value: &serde_json::Value| {
        let name = value.as_str().expect("UI file paths must be strings");
        let path = Path::new(name);
        assert!(!name.is_empty()
            && !name.chars().any(|ch| matches!(ch, '\\' | '\n' | '\r' | '\0'))
            && name.split('/').all(|part| !matches!(part, "" | "." | ".."))
            && path.components().all(|part| matches!(part, Component::Normal(_))),
            "UI file paths must stay inside the UI directory: {name}");
        let file = root.join(path).canonicalize().expect("UI input file must exist");
        assert!(file.starts_with(&root) && file.is_file(),
            "UI input must be a file inside the UI directory: {name}");
        println!("cargo:rerun-if-changed={}", root.join(path).display());
        // Also track a symlink's current target.
        println!("cargo:rerun-if-changed={}", file.display());
        (name.to_owned(), file)
    };
    let scripts = config["scripts"].as_array().expect("UI scripts must be an ordered array")
        .iter().map(resolve).collect::<Vec<_>>();
    assert!(!scripts.is_empty(), "UI scripts must not be empty");
    let mut names = std::collections::HashSet::new();
    for (name, _) in &scripts {
        assert!(names.insert(name), "duplicate UI script: {name}");
    }
    let (_, font) = resolve(&config["font"]);
    UiInputs { scripts, font }
}
