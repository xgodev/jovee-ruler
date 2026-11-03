//! The ruler speaks the language selected, in a face with its glyphs; inside a host, the host's.

use jr_gui::{language, ui::Locale, ui::Theme, Ruler};
use slint::ComponentHandle;

#[test]
fn standalone_every_text_and_the_face_follow_the_language_selected() {
    i_slint_backend_testing::init_no_event_loop();
    let ruler = Ruler::new().unwrap();
    language::select(&ruler, "ja-JP").unwrap();
    let w = ruler.window();
    assert_eq!(w.get_key_title(), "Cメジャー");
    assert_eq!(w.get_key_subtitle(), "平行調：Aマイナー");
    assert_eq!(w.global::<Locale>().get_font_family(), "Hiragino Sans");
}

#[test]
fn a_latin_language_keeps_the_jovee_face() {
    i_slint_backend_testing::init_no_event_loop();
    let ruler = Ruler::new().unwrap();
    language::select(&ruler, "ja-JP").unwrap();
    language::select(&ruler, "de-DE").unwrap();
    let w = ruler.window();
    assert_eq!(w.get_key_title(), "C-Dur");
    assert_eq!(w.global::<Locale>().get_font_family(), w.global::<Theme>().get_sans());
}

#[test]
fn inside_a_host_the_ruler_speaks_what_the_host_selected() {
    i_slint_backend_testing::init_no_event_loop();
    let ruler = Ruler::new().unwrap();
    // what JoVee Jam does once its own window exists; the ruler is handed only the face
    slint::select_bundled_translation("pt_BR").unwrap();
    ruler.set_font_for("pt-BR");
    let w = ruler.window();
    assert_eq!(w.get_key_title(), "C maior");
    assert_eq!(w.get_key_subtitle(), "Relativa: A menor");
    assert_eq!(w.global::<Locale>().get_font_family(), w.global::<Theme>().get_sans());
}
