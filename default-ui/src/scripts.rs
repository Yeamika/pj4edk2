/// One ordered, source-only script evaluated in the same PocketJS realm.
pub struct Script {
    pub name: &'static str,
    pub source: &'static str,
}

/// Evaluate these scripts once, then call `rimuksInit` and `rimuksTick`.
pub fn scripts() -> &'static [Script] {
    &[
        Script {
            name: "theme.js",
            source: include_str!("../pages/theme.js"),
        },
        Script {
            name: "battery-status.js",
            source: include_str!("../pages/battery-status.js"),
        },
        Script {
            name: "chrome.js",
            source: include_str!("../pages/chrome.js"),
        },
        Script {
            name: "settings-debug.js",
            source: include_str!("../pages/settings-debug.js"),
        },
        Script {
            name: "settings.js",
            source: include_str!("../pages/settings.js"),
        },
        Script {
            name: "boot-geometry.js",
            source: include_str!("../pages/boot-geometry.js"),
        },
        Script {
            name: "boot-image.js",
            source: include_str!("../pages/boot-image.js"),
        },
        Script {
            name: "boot-motion.js",
            source: include_str!("../pages/boot-motion.js"),
        },
        Script {
            name: "icons.js",
            source: include_str!("../pages/icons.js"),
        },
        Script {
            name: "boot-default.js",
            source: include_str!("../pages/boot-default.js"),
        },
        Script {
            name: "boot-actions.js",
            source: include_str!("../pages/boot-actions.js"),
        },
        Script {
            name: "boot.js",
            source: include_str!("../pages/boot.js"),
        },
        Script {
            name: "recovery.js",
            source: include_str!("../pages/recovery.js"),
        },
        Script {
            name: "charging.js",
            source: include_str!("../pages/charging.js"),
        },
        Script {
            name: "low-battery.js",
            source: include_str!("../pages/low-battery.js"),
        },
        Script {
            name: "main.js",
            source: include_str!("../pages/main.js"),
        },
        Script {
            name: "stage.js",
            source: include_str!("../pages/stage.js"),
        },
    ]
}
