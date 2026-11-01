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

/// A whole song: every chord named apart, so a row can only be the one it should be.
fn long_song(chords: usize) -> Song {
    let chord = |i| SongChord { name: format!("X{i}"), notes: vec![parse("G").unwrap()], fret: 3.0 };
    Song { title: "Longa".into(), key: parse("G").unwrap(), chords: (0..chords).map(chord).collect() }
}

fn strip(r: &Ruler) -> (i32, Vec<String>) {
    let w = r.window();
    (w.get_timeline_first(), w.get_timeline().iter().map(|n| n.to_string()).collect())
}

#[test]
fn a_long_song_shows_only_the_chords_around_the_one_sounding() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(long_song(238));
    assert_eq!(strip(&r).0, 0, "the start has nothing before it");

    r.at(100);
    let (first, names) = strip(&r);
    assert_eq!(names.len(), 8);
    assert_eq!(names[(100 - first) as usize], "X100");
    assert_eq!(r.window().get_now(), 100, "the sounding chord keeps its place in the whole song");

    r.at(237);
    let (first, names) = strip(&r);
    assert_eq!((first, names.len()), (230, 8), "the end still fills the strip");
    assert_eq!(names[7], "X237");
}

#[test]
fn a_short_song_shows_every_chord() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(song());
    r.at(2);
    assert_eq!(strip(&r), (0, vec!["G".to_string(), "C".to_string(), "F".to_string()]));
}

#[test]
fn clicking_a_chord_of_a_long_song_gives_its_place_in_the_whole_song() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(long_song(238));
    r.at(100);
    let heard = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    r.on_seek(move |i| h.borrow_mut().push(i));
    let chord = i_slint_backend_testing::ElementHandle::find_by_accessible_label(r.window(), "X102")
        .next()
        .expect("the chord two ahead is on the strip");
    chord.invoke_accessible_default_action();
    assert_eq!(*heard.borrow(), [102]);
    assert_eq!(r.window().get_now(), 102);
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
