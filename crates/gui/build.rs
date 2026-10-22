fn main() {
    embed_windows_icon();
}

// `winres` is only a build-dependency under `[target.'cfg(windows)']` in
// Cargo.toml, so referencing it has to be gated at compile time with
// `#[cfg(windows)]` — a runtime `CARGO_CFG_TARGET_OS` check alone still
// makes rustc resolve the `winres::` path on every host, which fails to
// link on non-Windows builds even though that branch would never run.
#[cfg(windows)]
fn embed_windows_icon() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("../../assets/razerline.ico");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn embed_windows_icon() {}
