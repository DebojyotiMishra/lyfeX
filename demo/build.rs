//! Bakes the Homebrew library directory into the demo binary as an rpath so the Vulkan
//! loader is found at runtime.
//!
//! macOS has no native Vulkan; Homebrew installs the loader (and MoltenVK) outside the
//! default dynamic linker search path. `DYLD_LIBRARY_PATH` does not reliably help: SIP
//! strips `DYLD_*` variables when exec'ing SIP-restricted binaries (which `cargo` itself
//! often is), so the variable frequently never reaches spawned processes. An rpath is
//! embedded in the Mach-O file and is unaffected.
//!
//! This lives in a build script rather than `.cargo/config.toml` because cargo ignores
//! `target.<triple>.rustflags` entirely whenever the `RUSTFLAGS` environment variable is
//! set - env takes precedence over the config table and the two are not merged. Any shell
//! or CI job exporting `RUSTFLAGS` (say `-D warnings`) would silently drop the rpath, and
//! the resulting binary fails at load with "Library not loaded: libvulkan.dylib". Build
//! script link args are not subject to that precedence.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }

    let lib_dir = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "/opt/homebrew/lib",
        _ => "/usr/local/lib",
    };
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}");
}
