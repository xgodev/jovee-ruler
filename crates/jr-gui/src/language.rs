//! The language the ruler speaks. Standalone it picks it as JoVee Studio does: the one chosen in JoVee,
//! else the system's, with Studio's fallbacks. Inside a host (JoVee Jam, JoVee Studio) the host has
//! already selected the language for every window; the ruler only takes the face, with `Ruler::set_font_for`.

use std::path::Path;

use crate::Ruler;

/// Every language JoVee Studio ships; the folders under `translations/` name them, `-` written `_`.
pub const LANGUAGES: [&str; 9] = ["de-DE", "en-US", "es-ES", "fr-FR", "hi-IN", "ja-JP", "ko-KR", "pt-BR", "zh-CN"];

/// The shipped language closest to a locale (`pt_BR.UTF-8`, `pt-PT`, `it`). Italian reads Spanish and
/// any language not shipped reads English, as in Studio.
pub fn normalize(raw: &str) -> &'static str {
    let head = raw.split(['.', '@']).next().unwrap_or_default();
    match head.split(['-', '_']).next().unwrap_or_default().to_ascii_lowercase().as_str() {
        "pt" => "pt-BR",
        "es" | "it" => "es-ES",
        "fr" => "fr-FR",
        "de" => "de-DE",
        "ja" => "ja-JP",
        "ko" => "ko-KR",
        "zh" => "zh-CN",
        "hi" => "hi-IN",
        _ => "en-US",
    }
}

/// A choice other than `auto` wins; else the system's language; else English.
pub fn resolve(chosen: Option<&str>, system: Option<&str>) -> &'static str {
    match chosen.filter(|code| !code.is_empty() && !code.eq_ignore_ascii_case("auto")) {
        Some(code) => normalize(code),
        None => system.map_or("en-US", normalize),
    }
}

fn language_in(file: &Path) -> Option<String> {
    let text = std::fs::read_to_string(file).ok()?;
    let config: serde_yaml::Value = serde_yaml::from_str(&text).ok()?;
    config.get("language")?.as_str().map(str::to_string)
}

/// The language picked in JoVee, under its root (`~/.jovee`): the shared file's, else the one JoVee
/// Studio saved.
pub fn chosen(root: &Path) -> Option<String> {
    language_in(&root.join("config.yaml")).or_else(|| language_in(&root.join("rig/config.yaml")))
}

/// The face that has the glyphs of a language; `None` keeps the JoVee face, which has the Latin ones.
pub fn font_for(language: &str) -> Option<&'static str> {
    match language {
        "ja-JP" => Some("Hiragino Sans"),
        // the Simplified-Chinese sibling: it has the glyphs only zh-Hans uses
        "zh-CN" => Some("Hiragino Sans GB"),
        "ko-KR" => Some("Apple SD Gothic Neo"),
        "hi-IN" => Some("Kohinoor Devanagari"),
        _ => None,
    }
}

/// The ruler on its own speaks `language` (a code of `LANGUAGES`), in a face with its glyphs. Slint
/// keeps one language for every window, so a host never calls this: it selects its own.
pub fn select(ruler: &Ruler, language: &str) -> Result<(), slint::SelectBundledTranslationError> {
    slint::select_bundled_translation(&language.replace('-', "_"))?;
    ruler.set_font_for(language);
    Ok(())
}

#[cfg(test)]
#[path = "language_tests.rs"]
mod tests;
