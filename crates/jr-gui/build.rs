fn main() {
    // Debug info only in debug builds: it is what lets a test find an element by its id.
    let debug = std::env::var("PROFILE").is_ok_and(|p| p == "debug");
    let config = slint_build::CompilerConfiguration::new().with_style("fluent".into()).with_debug_info(debug);
    slint_build::compile_with_config("ui/main.slint", config).expect("the screens compile");
}
