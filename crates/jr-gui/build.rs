use std::path::PathBuf;

/// The package directory of the clone being built. Read when the script runs: clones share one target
/// directory and cargo reuses a compiled build script across them, so `env!` would keep the clone that
/// compiled it.
fn manifest_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"))
}

fn main() {
    let dir = manifest_dir();
    let translations = dir.join("translations");
    println!("cargo:rerun-if-changed={}", translations.display());
    println!("cargo:rerun-if-changed={}", dir.join("ui").display());
    // Debug info only in debug builds: it is what lets a test find an element by its id.
    let debug = std::env::var("PROFILE").is_ok_and(|p| p == "debug");
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent".into())
        .with_debug_info(debug)
        .with_library_paths(jovee_theme::library_paths())
        // the .po files carry no context: without this every text would be looked up under its component
        .with_default_translation_context(slint_build::DefaultTranslationContext::None)
        // the gettext domain is the package's name: translations/<xx_YY>/LC_MESSAGES/jr-gui.po
        .with_bundled_translations(translations);
    slint_build::compile_with_config(dir.join("ui/main.slint"), config).expect("the screens compile");
}
