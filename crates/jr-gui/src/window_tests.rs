use super::*;
use jr_core::note::parse;
use slint::Model;

fn init() {
    i_slint_backend_testing::init_no_event_loop();
}

#[test]
fn picking_a_key_on_the_wheel_fills_both_pages() {
    init();
    let r = Ruler::new().unwrap();
    r.window().invoke_key_picked(1); // G, one fifth up from C
    let w = r.window();
    assert_eq!(w.get_key_title(), "Sol maior");
    assert_eq!(w.get_key_index(), 1);
    assert_eq!(w.get_chords().row_count(), 7);
    assert_eq!(w.get_chords().row_data(3).unwrap().name, "C");
    assert_eq!(w.get_shapes().row_data(0).unwrap().first, 3);
}

#[test]
fn picking_a_shape_fades_the_rest_of_the_neck() {
    init();
    let r = Ruler::new().unwrap();
    r.set_key(parse("G").unwrap());
    r.window().invoke_shape_picked(5); // the whole neck
    assert!(r.window().get_scale_notes().iter().all(|m| m.lit));
    r.window().invoke_shape_picked(0);
    let notes = r.window().get_scale_notes();
    assert!(notes.iter().any(|m| !m.lit));
    assert!(notes.iter().filter(|m| m.lit).all(|m| (3..=5).contains(&m.fret)));
}

#[test]
fn the_host_picks_any_of_the_three_looks() {
    init();
    let r = Ruler::new().unwrap();
    for look in ThemeId::ALL {
        r.set_theme(look);
        assert_eq!(r.window().global::<ui::Theme>().get_index(), look.index());
    }
}

fn song() -> Song {
    let chord = |name: &str, notes: &str| SongChord {
        name: name.into(),
        notes: notes.split_whitespace().map(|n| parse(n).unwrap()).collect(),
        fret: 3.0,
    };
    Song {
        title: "Exemplo".into(),
        key: parse("G").unwrap(),
        chords: vec![chord("G", "G B D"), chord("C", "C E G"), chord("F", "F A C")],
    }
}

#[test]
fn clicking_a_chord_tells_the_host_which_one() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(song());
    let heard = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    r.on_seek(move |i| h.borrow_mut().push(i));
    r.window().invoke_seek(2);
    r.window().invoke_seek(0);
    assert_eq!(*heard.borrow(), [2, 0]);
    assert_eq!(r.window().get_now(), 0);
}

#[test]
fn the_host_can_answer_a_click_by_moving_the_ruler() {
    init();
    let r = Rc::new(Ruler::new().unwrap());
    r.follow_song(song());
    let host = r.clone();
    r.on_seek(move |i| host.at(i + 1)); // the song lands on the next chord, not the one clicked
    r.window().invoke_seek(0);
    assert_eq!(r.window().get_now(), 1);
    assert_eq!(r.window().get_moment().roman, "IV");
}

#[test]
fn a_song_opens_on_its_page_and_follows_the_chords() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(song());
    let w = r.window();
    assert_eq!(w.get_page(), 2);
    assert_eq!(w.get_moment().roman, "I");
    assert_eq!(w.get_key_title(), "Sol maior");
    w.invoke_seek(2);
    assert_eq!(w.get_now(), 2);
    assert_eq!(w.get_moment().roman, "♭VII");
    r.at(1);
    assert_eq!(w.get_moment().roman, "IV");
}
