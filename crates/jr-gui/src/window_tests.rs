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
    assert_eq!(w.get_key_title(), "G maior");
    assert_eq!(w.get_key_index(), 1);
    assert_eq!(w.get_chords().row_count(), 7);
    assert_eq!(w.get_chords().row_data(3).unwrap().name, "C");
    assert_eq!(w.get_shapes().row_data(0).unwrap().first, 2);
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
    assert!(notes.iter().filter(|m| m.lit).all(|m| (2..=5).contains(&m.fret)));
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

/// G major up the 1st string, one note a second from 1 s: every name apart, so a chip is only the one
/// it should be.
fn run() -> Song {
    let frets = [3, 5, 7, 8, 10, 12, 14];
    Song {
        title: "Exemplo em Sol".into(),
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
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(run());
    let w = r.window();
    assert_eq!(w.get_page(), 2);
    assert_eq!(w.get_key_title(), "G maior", "the other pages follow the song's key");
    assert_eq!(w.get_context(), "Exemplo em Sol");
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
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(run());
    r.at(0.5);
    assert!(r.window().get_now_note().empty);
    r.at(1.0);
    assert_eq!(now_name(&r), "G");
    r.at(3.7);
    assert_eq!(now_name(&r), "B");
    assert_eq!(r.window().get_now_note().detail, "corda 1 · casa 7");
    r.at(99.0);
    assert_eq!(now_name(&r), "F#");
}

#[test]
fn the_shape_follows_the_hand_up_the_neck() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(run());
    r.at(1.0);
    let first = r.window().get_song_first();
    r.at(7.0);
    assert!(r.window().get_song_first() > first, "fret 14 is in a higher shape than fret 3");
    assert!(r.window().get_song_subtitle().contains("a mão está na forma"));
}

#[test]
fn clicking_a_note_of_the_phrase_takes_the_song_there() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(run());
    r.at(3.0);
    let heard = Rc::new(RefCell::new(Vec::new()));
    let h = heard.clone();
    r.on_seek(move |s| h.borrow_mut().push(s));
    let chip = i_slint_backend_testing::ElementHandle::find_by_accessible_label(r.window(), "D, corda 1")
        .next()
        .expect("the note two ahead is on the phrase");
    chip.invoke_accessible_default_action();
    assert_eq!(*heard.borrow(), [5.0], "D starts at 5 s");
    assert_eq!(now_name(&r), "D", "the ruler shows it before the song gets there");
}

#[test]
fn the_host_can_answer_a_click_by_moving_the_ruler() {
    init();
    let r = Rc::new(Ruler::new().unwrap());
    r.follow_song(run());
    let host = r.clone();
    r.on_seek(move |s| host.at(s + 1.0)); // the song lands on the next note, not the one clicked
    r.window().invoke_seek(0);
    assert_eq!(now_name(&r), "A");
}

#[test]
fn picking_another_guitar_asks_the_host_for_it() {
    init();
    let r = Ruler::new().unwrap();
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
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(run());
    r.window().invoke_scale_picked(1);
    assert_eq!(r.window().get_song_title(), "G maior · pentatônica");
    assert!(r.window().get_song_notes().iter().all(|m| m.mark != 3));
    r.window().invoke_hand_picked(6);
    assert_eq!(r.window().get_song_first(), -1);
    r.follow_song(Song { guitar: 0, ..run() });
    let w = r.window();
    assert_eq!((w.get_scale(), w.get_hand(), w.get_song_first()), (1, 6, -1));
    assert_eq!(w.get_song_subtitle(), "Guitarra Base · braço todo");
}

#[test]
fn a_tuning_and_a_long_neck_come_from_the_song() {
    init();
    let r = Ruler::new().unwrap();
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
    init();
    let r = Ruler::new().unwrap();
    r.window().invoke_seek(3);
    r.window().invoke_scale_picked(1);
    r.window().invoke_guitar_picked(1);
    assert_eq!((r.window().get_scale(), r.window().get_guitar()), (0, 1), "no song yet: nothing redrawn");
    r.follow_song(run());
    r.window().invoke_seek(1);
    assert_eq!(now_name(&r), "A");
}

#[test]
fn a_song_draws_the_highest_string_on_top_and_the_key_flips_it() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(run());
    assert!(r.window().get_high_on_top(), "as a tab reads, the high E on the top line");
    let key = i_slint_backend_testing::ElementHandle::find_by_accessible_label(r.window(), "Inverter as cordas")
        .next()
        .expect("the flip key is beside the tuning");
    key.invoke_accessible_default_action();
    assert!(!r.window().get_high_on_top());
    r.follow_song(Song { guitar: 0, ..run() });
    assert!(!r.window().get_high_on_top(), "the flip stays for the next guitar");
    r.window().invoke_strings_flipped();
    assert!(r.window().get_high_on_top());
}

#[test]
fn the_flip_is_kept_for_the_next_opening() {
    init();
    let root = crate::choices::choices_tests::scratch("opening");
    let r = Ruler::new().unwrap();
    r.keep_choices_in(&root);
    r.follow_song(run());
    r.window().invoke_strings_flipped();
    drop(r);
    let again = Ruler::new().unwrap();
    again.keep_choices_in(&root);
    again.follow_song(run());
    assert!(!again.window().get_high_on_top(), "opened again with the lowest string on top");
}

#[test]
fn without_a_place_to_keep_it_the_flip_is_forgotten() {
    init();
    let r = Ruler::new().unwrap();
    r.follow_song(run());
    r.window().invoke_strings_flipped();
    let again = Ruler::new().unwrap();
    again.follow_song(run());
    assert!(again.window().get_high_on_top());
}
