#[path = "../tools/ui-input.rs"]
mod ui_input;

fn main() {
    let inputs = ui_input::load(env!("CARGO_MANIFEST_DIR"));
    let mut source = String::from("&[\n");
    for (name, path) in inputs.scripts {
        source.push_str(&format!(
            "Script {{ name: {name:?}, source: include_str!({:?}) }},\n",
            path.to_str().expect("UTF-8 UI path"),
        ));
    }
    source.push_str("]\n");
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(out.join("scripts.rs"), source).expect("write embedded UI scripts");
}
