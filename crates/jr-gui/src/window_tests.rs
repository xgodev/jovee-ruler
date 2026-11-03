use super::*;
use jr_core::note::parse;
use slint::Model;

/// A ruler in English, the source language: the first window a thread opens takes the system's language.
fn ruler() -> Ruler {
    i_slint_backend_testing::init_no_event_loop();
    let r = Ruler::new().unwrap();
    slint::select_bundled_translation("en_US").unwrap();
    r
}

#[test]
fn picking_a_key_on_the_wheel_fills_both_pages() {
    let r = ruler();
    r.window().invoke_key_picked(1); // G, one fifth up from C
    let w = r.window();
    assert_eq!(w.get_key_title(), "G major");
    assert_eq!(w.get_key_subtitle(), "Relative: E minor");
    assert_eq!(w.get_key_index(), 1);
    assert_eq!(w.get_chords().row_count(), 7);
    assert_eq!(w.get_chords().row_data(3).unwrap().name, "C");
    assert_eq!(w.get_shapes().row_data(0).unwrap().first, 2);
    assert_eq!(w.get_shape_subtitle(), "Shape 1 · 2 in the relative minor · frets 2 to 5");
}

#[test]
fn picking_a_shape_fades_the_rest_of_the_neck() {
    let r = ruler();
    r.set_key(parse("G").unwrap());
    r.window().invoke_shape_picked(5); // the whole neck
    assert!(r.window().get_scale_notes().iter().all(|m| m.lit));
    assert_eq!(r.window().get_shape_subtitle(), "Whole neck · all five shapes");
    r.window().invoke_shape_picked(0);
    let notes = r.window().get_scale_notes();
    assert!(notes.iter().any(|m| !m.lit));
    assert!(notes.iter().filter(|m| m.lit).all(|m| (2..=5).contains(&m.fret)));
}

#[test]
fn the_host_picks_any_of_the_three_looks() {
    let r = ruler();
    for look in ThemeId::ALL {
        r.set_theme(look);
        assert_eq!(r.window().global::<ui::Theme>().get_index(), look.index());
    }
}

/// G major up the 1st string, one note a second from 1 s: every name apart, so a chip is only the one
/// it should be.
fn run() -> Song {
    let frets = [3, 5, 7, 8, 10, 12, 14];
    Song {
        title: "Example in G".into(),
        key: parse("G").unwrap(),
        tuning: vec![64, 59, 55, 50, 45, 40],
        guitars: vec!["Base".into(), "Solo".into()],
        guitar: 1,
        notes: frets
            .iter()
            .enumerate()
            .map(|(i, &fret)| SongNote { start: 1.0 + i as f64, end: 1.5 + i as f64, string: 1, fret, midi: 64 + fret })
            .collect(),
        chords: Vec::new(),
    }
}

fn now_name(r: &Ruler) -> String {
    r.window().get_now_note().name.to_string()
}

#[test]
fn a_song_opens_on_its_page_before_its_first_note() {
    let r = ruler();
    r.follow_song(run());
    let w = r.window();
    assert_eq!(w.get_page(), 2);
    assert_eq!(w.get_key_title(), "G major", "the other pages follow the song's key");
    assert_eq!(w.get_context(), "Example in G");
    assert_eq!(w.get_guitars().iter().map(|g| g.to_string()).collect::<Vec<_>>(), ["Base", "Solo"]);
    assert_eq!(w.get_guitar(), 1);
    assert_eq!(w.get_tuning(), "E A D G B E");
    assert_eq!(w.get_open_names().row_count(), 6);
    assert_eq!(w.get_song_frets(), 15);
    assert!(w.get_now_note().empty);
    assert_eq!(w.get_phrase().row_data(0).unwrap().state, 2, "the first note is the next one");
}

#[test]
fn the_ruler_follows_the_song_note_by_note() {
    let r = ruler();
    r.follow_song(run());
    r.at(0.5);
    assert!(r.window().get_now_note().empty);
    r.at(1.0);
    assert_eq!(now_name(&r), "G");
    r.at(3.7);
    assert_eq!(now_name(&r), "B");
    let now = r.window().get_now_note();
    assert_eq!((now.chord, now.string, now.fret), (false, 1, 7));
    assert!(
        i_slint_backend_testing::ElementHandle::find_by_accessible_label(r.window(), "string 1 · fret 7").next().is_some(),
        "the box under the neck says where the hand plays it"
    );
    r.at(99.0);
    assert_eq!(now_name(&r), "F#");
}

#[test]
fn the_shape_follows_the_hand_up_the_neck() {
    let r = ruler();
    r.follow_song(run());
    r.at(1.0);
    let first = r.window().get_song_first();
    r.at(7.0);
    assert!(r.window().get_song_first() > first, "fret 14 is in a higher shape than fret 3");
    assert!(r.window().get_song_subtitle().contains("the hand is in shape"));
}

#[test]
fn clicking_a_note_of_the_phrase_takes_the_song_there() {
    let r = ruler();
    r.follow_song(run());
    r.at(3.0);
    let heard = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    r.on_seek(move |s| h.borrow_mut().push(s));
    let chip = i_slint_backend_testing::ElementHandle::find_by_accessible_label(r.window(), "D, string 1")
        .next()
        .expect("the note two ahead is on the phrase");
    chip.invoke_accessible_default_action();
    assert_eq!(*heard.borrow(), [5.0], "D starts at 5 s");
    assert_eq!(now_name(&r), "D", "the ruler shows it before the song gets there");
}

#[test]
fn the_host_can_answer_a_click_by_moving_the_ruler() {
    let r = Rc::new(ruler());
    r.follow_song(run());
    let host = r.clone();
    r.on_seek(move |s| host.at(s + 1.0)); // the song lands on the next note, not the one clicked
    r.window().invoke_seek(0);
    assert_eq!(now_name(&r), "A");
}

#[test]
fn picking_another_guitar_asks_the_host_for_it() {
    let r = ruler();
    r.follow_song(run());
    let heard = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    r.on_guitar(move |g| h.borrow_mut().push(g));
    r.window().invoke_guitar_picked(0);
    assert_eq!(*heard.borrow(), [0]);
    assert_eq!(r.window().get_guitar(), 0, "the select shows the pick while the host reads it");
}

#[test]
fn the_scale_and_the_shape_picked_stay_for_the_next_guitar() {
    let r = ruler();
    r.follow_song(run());
    r.window().invoke_scale_picked(1);
    assert_eq!(r.window().get_song_title(), "G major · pentatonic");
    assert!(r.window().get_song_notes().iter().all(|m| m.mark != 3));
    r.window().invoke_hand_picked(6);
    assert_eq!(r.window().get_song_first(), -1);
    r.follow_song(Song { guitar: 0, ..run() });
    let w = r.window();
    assert_eq!((w.get_scale(), w.get_hand(), w.get_song_first()), (1, 6, -1));
    assert_eq!(w.get_song_subtitle(), "Guitar Base · whole neck");
}

#[test]
fn a_tuning_and_a_long_neck_come_from_the_song() {
    let r = ruler();
    let mut song = run();
    song.tuning = vec![64, 59, 55, 50, 45, 38];
    song.notes[6].fret = 19;
    r.follow_song(song);
    let w = r.window();
    assert_eq!(w.get_tuning(), "D A D G B E");
    assert_eq!(w.get_open_names().row_data(0).unwrap(), "D");
    assert_eq!(w.get_song_frets(), 19);
}

#[test]
fn the_song_controls_wait_quietly_for_a_song_and_a_host() {
    let r = ruler();
    r.window().invoke_seek(3);
    r.window().invoke_scale_picked(1);
    r.window().invoke_guitar_picked(1);
    assert_eq!((r.window().get_scale(), r.window().get_guitar()), (0, 1), "no song yet: nothing redrawn");
    r.follow_song(run());
    r.window().invoke_seek(1);
    assert_eq!(now_name(&r), "A");
}

#[test]
fn the_title_writes_the_flat_as_a_lowercase_b() {
    let r = ruler();
    r.set_key(parse("Ab").unwrap());
    assert_eq!(r.window().get_key_title(), "Ab major");
    assert_eq!(r.window().get_key_subtitle(), "Relative: F minor");
}

#[test]
fn the_song_subtitle_says_which_guitar_and_where_the_hand_is() {
    let r = ruler();
    r.follow_song(run());
    r.at(1.0);
    let w = r.window();
    assert_eq!(w.get_song_title(), "G major");
    assert_eq!(w.get_song_subtitle(), "Guitar Solo · the hand is in shape 1, frets 2 to 5");
    w.invoke_hand_picked(2);
    assert_eq!(w.get_song_subtitle(), "Guitar Solo · shape 2, frets 4 to 8");
}

#[test]
fn a_chord_says_the_fret_it_is_barred_at() {
    let r = ruler();
    let mut song = run();
    // G barred at the 3rd fret, then the scale run
    let barre = [(6, 3, 43), (5, 5, 50), (4, 5, 55), (3, 4, 59), (2, 3, 62), (1, 3, 67)];
    song.notes.splice(0..0, barre.map(|(string, fret, midi)| SongNote { start: 0.0, end: 0.5, string, fret, midi }));
    r.follow_song(song);
    r.at(0.0);
    let find = |label| i_slint_backend_testing::ElementHandle::find_by_accessible_label(r.window(), label).next();
    assert!(find("chord · fret 3").is_some(), "the box under the neck");
    assert!(find("G, fret 3").is_some(), "the chip of the phrase");
}
