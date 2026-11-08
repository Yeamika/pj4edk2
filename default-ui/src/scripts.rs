/// One ordered, source-only script evaluated in the same PocketJS realm.
pub struct Script {
    pub name: &'static str,
    pub source: &'static str,
}

/// Evaluate these scripts once, then call `rimuksInit` and `rimuksTick`.
pub fn scripts() -> &'static [Script] {
    include!(concat!(env!("OUT_DIR"), "/scripts.rs"))
}
