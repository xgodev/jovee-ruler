use super::*;

#[test]
fn a_locale_becomes_the_shipped_language_closest_to_it() {
    assert_eq!(normalize("pt_BR.UTF-8"), "pt-BR");
    assert_eq!(normalize("pt-PT"), "pt-BR", "any Portuguese reads Brazilian");
    assert_eq!(normalize("en-GB"), "en-US");
    assert_eq!(normalize("zh-Hant-TW"), "zh-CN");
    assert_eq!(normalize("ja"), "ja-JP");
    assert_eq!(normalize("ko_KR"), "ko-KR");
    assert_eq!(normalize("hi-IN"), "hi-IN");
    assert_eq!(normalize("de_AT@euro"), "de-DE");
    assert_eq!(normalize("fr-CA"), "fr-FR");
    assert_eq!(normalize("it-IT"), "es-ES", "Italian reads Spanish, as in Studio");
    assert_eq!(normalize("ru-RU"), "en-US", "a language not shipped reads English");
    assert_eq!(normalize(""), "en-US");
}

#[test]
fn a_choice_wins_over_the_system_unless_it_is_auto() {
    assert_eq!(resolve(Some("fr-FR"), Some("pt_BR")), "fr-FR");
    assert_eq!(resolve(Some("auto"), Some("pt_BR")), "pt-BR");
    assert_eq!(resolve(Some("AUTO"), Some("ja_JP")), "ja-JP");
    assert_eq!(resolve(Some(""), Some("de_DE")), "de-DE");
    assert_eq!(resolve(None, Some("ko_KR")), "ko-KR");
    assert_eq!(resolve(None, None), "en-US");
    for language in LANGUAGES {
        assert_eq!(resolve(Some(language), None), language, "every shipped code reads as itself");
    }
}

#[test]
fn the_scripts_the_jovee_face_lacks_get_a_face_with_their_glyphs() {
    assert_eq!(font_for("ja-JP"), Some("Hiragino Sans"));
    assert_eq!(font_for("zh-CN"), Some("Hiragino Sans GB"));
    assert_eq!(font_for("ko-KR"), Some("Apple SD Gothic Neo"));
    assert_eq!(font_for("hi-IN"), Some("Kohinoor Devanagari"));
    for latin in ["de-DE", "en-US", "es-ES", "fr-FR", "pt-BR"] {
        assert_eq!(font_for(latin), None, "{latin} keeps the JoVee face");
    }
}

fn write(root: &Path, file: &str, text: &str) {
    let path = root.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn the_shared_choice_wins_over_the_one_studio_saved() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(chosen(root.path()), None, "nothing saved yet");
    write(root.path(), "rig/config.yaml", "audio:\n  buffer: 64\nlanguage: ja-JP\n");
    assert_eq!(chosen(root.path()).as_deref(), Some("ja-JP"));
    write(root.path(), "config.yaml", "theme: rack\nlanguage: \"de-DE\"\n");
    assert_eq!(chosen(root.path()).as_deref(), Some("de-DE"));
}

#[test]
fn a_file_with_no_language_leaves_the_choice_to_the_next() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "config.yaml", "theme: neon\n");
    write(root.path(), "rig/config.yaml", "language: ko-KR\n");
    assert_eq!(chosen(root.path()).as_deref(), Some("ko-KR"));
    write(root.path(), "config.yaml", "language: null\n");
    assert_eq!(chosen(root.path()).as_deref(), Some("ko-KR"));
    write(root.path(), "config.yaml", "{ not yaml\n");
    assert_eq!(chosen(root.path()).as_deref(), Some("ko-KR"), "a broken file is no choice");
}
