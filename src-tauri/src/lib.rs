//! gcal-app — Tauri 2 shell. The real commands and state are added in the
//! shell phase; this stub keeps the workspace resolvable.

pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
