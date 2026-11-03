//! Every text the ruler shows is translated in every language JoVee Studio ships.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

const LANGUAGES: [&str; 9] = ["de_DE", "en_US", "es_ES", "fr_FR", "hi_IN", "ja_JP", "ko_KR", "pt_BR", "zh_CN"];

fn translations() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("translations")
}

fn slint_files(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            slint_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "slint") {
            found.push(path);
        }
    }
}

/// The text of the string literal `rest` opens with, escapes kept as written.
fn literal(rest: &str) -> Option<&str> {
    let mut escaped = false;
    for (at, c) in rest.char_indices() {
        match c {
            '\\' if !escaped => escaped = true,
            '"' if !escaped => return Some(&rest[..at]),
            _ => escaped = false,
        }
    }
    None
}

/// Every `@tr("…")` text of the window.
fn shown_texts() -> BTreeSet<String> {
    let mut files = Vec::new();
    slint_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"), &mut files);
    let mut texts = BTreeSet::new();
    for file in files {
        let source = std::fs::read_to_string(&file).unwrap();
        for (at, _) in source.match_indices("@tr(\"") {
            texts.extend(literal(&source[at + 5..]).map(str::to_string));
        }
    }
    texts
}

/// A catalog's texts and their translations, multi-line ones joined.
fn catalog(language: &str) -> HashMap<String, String> {
    let path = translations().join(language).join("LC_MESSAGES/jr-gui.po");
    let mut entries = HashMap::new();
    let (mut id, mut text, mut in_text) = (String::new(), String::new(), false);
    for line in std::fs::read_to_string(path).unwrap().lines() {
        if let Some(rest) = line.strip_prefix("msgid \"") {
            if in_text {
                entries.insert(std::mem::take(&mut id), std::mem::take(&mut text));
            }
            id = rest.trim_end_matches('"').to_string();
            in_text = false;
        } else if let Some(rest) = line.strip_prefix("msgstr \"").or_else(|| line.strip_prefix("msgstr[0] \"")) {
            text = rest.trim_end_matches('"').to_string();
            in_text = true;
        } else if let Some(rest) = line.strip_prefix('"') {
            let rest = rest.trim_end_matches('"');
            if in_text {
                text.push_str(rest)
            } else {
                id.push_str(rest)
            }
        }
    }
    if in_text {
        entries.insert(id, text);
    }
    entries
}

#[test]
fn every_shown_text_is_translated_in_every_language() {
    let texts = shown_texts();
    assert!(texts.len() > 30, "the window was not read: {} texts", texts.len());
    for language in LANGUAGES {
        let translated = catalog(language);
        let missing: Vec<_> = texts.iter().filter(|text| translated.get(*text).is_none_or(|t| t.is_empty())).collect();
        assert!(missing.is_empty(), "{language} lacks {missing:?}");
    }
}

/// Inside JoVee Jam or JoVee Studio the host's language list is the one Slint keeps, and every window
/// looks its texts up by their place in it. The ruler follows the host only while both lists are the same.
#[test]
fn the_catalogs_are_the_nine_languages_and_nothing_else() {
    let mut found: Vec<String> = std::fs::read_dir(translations())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .collect();
    found.sort();
    assert_eq!(found, LANGUAGES);
}
