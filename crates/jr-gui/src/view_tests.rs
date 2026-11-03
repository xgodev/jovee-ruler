use super::*;
use crate::song::{moments, Song, SongChord, SongNote};
use jr_core::note::parse;

fn n(s: &str) -> Pc {
    parse(s).unwrap()
}

const STANDARD_MIDI: [u8; 6] = [64, 59, 55, 50, 45, 40];

/// A note on `string` (1 = highest) at `fret`, in standard tuning, `start` seconds in.
fn note(start: f64, string: u8, fret: u8) -> SongNote {
    SongNote { start, end: start + 0.5, string, fret, midi: STANDARD_MIDI[string as usize - 1] + fret }
}

fn chord(start: f64, frets: &[(u8, u8)]) -> Vec<SongNote> {
    frets.iter().map(|&(s, f)| note(start, s, f)).collect()
}

fn song(notes: Vec<SongNote>, chords: &[(f64, &str)]) -> Song {
    Song {
        title: "Exemplo em Sol".into(),
        key: n("G"),
        tuning: STANDARD_MIDI.to_vec(),
        guitars: vec!["Base".into(), "Solo".into()],
        guitar: 1,
        notes,
        chords: chords.iter().map(|&(start, name)| SongChord { start, name: name.into() }).collect(),
    }
}

/// The lead of the mockup: Bb B on the 3rd string, D E D on the 2nd, B, then G on the 4th.
fn solo() -> Song {
    let frets = [(3, 3), (3, 4), (2, 3), (2, 5), (2, 3), (3, 4), (4, 5)];
    song(frets.iter().enumerate().map(|(i, &(s, f))| note(i as f64, s, f)).collect(), &[])
}

/// The rhythm of the mockup: G then C, both barred at the 3rd fret.
fn rhythm() -> Song {
    let mut notes = chord(0.0, &[(6, 3), (5, 5), (4, 5), (3, 4), (2, 3), (1, 3)]);
    notes.extend(chord(1.0, &[(5, 3), (4, 5), (3, 5), (2, 5), (1, 3)]));
    song(notes, &[(0.0, "G"), (1.0, "C")])
}

fn page(song: &Song, now: Option<usize>, pentatonic: bool, hand: Hand) -> SongPage {
    song_page(song, &moments(&song.notes), now, pentatonic, hand)
}

fn at(marks: &[NoteMark], row: i32, fret: i32) -> &NoteMark {
    marks.iter().find(|m| m.string == row && m.fret == fret).unwrap_or_else(|| panic!("no mark at {row}:{fret}"))
}

#[test]
fn the_relative_minor_is_named_in_the_key() {
    assert_eq!(relative_name(n("G")), "E");
    assert_eq!(relative_name(n("Ab")), "F");
    assert_eq!(relative_name(n("F")), "D");
    assert_eq!(relative_name(n("Eb")), "C");
    assert_eq!(relative_short(n("C")), "Am");
}

#[test]
fn the_wheel_starts_at_c_and_goes_in_fifths() {
    let keys = wheel_keys();
    assert_eq!(keys.len(), 12);
    assert_eq!((keys[0].as_str(), keys[1].as_str(), keys[11].as_str()), ("C", "G", "F"));
    assert_eq!(wheel_index(n("G")), 1);
    assert_eq!(wheel_index(n("F")), 11);
}

#[test]
fn shape_buttons_fit_on_the_screen_neck() {
    let b = shape_buttons(n("G"));
    let got: Vec<(i32, i32, i32, i32)> = b.iter().map(|s| (s.major, s.minor, s.first, s.last)).collect();
    // each box counted in the major key and in the relative minor; the whole neck last
    assert_eq!(got, [(1, 2, 3, 5), (2, 3, 5, 7), (3, 4, 7, 10), (4, 5, 10, 12), (5, 1, 0, 3), (0, 0, -1, -1)]);
    for key in 0..12 {
        for s in &shape_buttons(key)[..5] {
            assert!(s.first >= 0 && s.last <= FRETS, "key {key}: {s:?} off the neck");
        }
    }
}

#[test]
fn the_neck_has_every_scale_note_and_fades_outside_the_shape() {
    let marks = neck(n("G"), &STANDARD, FRETS, Some((3, 5)), false, &[]);
    // seven notes out of twelve, on six strings, sixteen positions each
    let expected = (0..6)
        .map(|s| (0..=FRETS).filter(|&f| jr_core::scale(n("G")).contains(&note_at(STANDARD[s], f as u8))).count())
        .sum::<usize>();
    assert_eq!(marks.len(), expected);
    let tonic = at(&marks, 0, 3);
    assert_eq!((tonic.name.as_str(), tonic.mark, tonic.lit), ("G", 0, true));
    let open_e = at(&marks, 0, 0);
    assert_eq!((open_e.mark, open_e.lit), (1, false));
    assert!(marks.iter().all(|m| !m.sounding));
}

#[test]
fn the_neck_follows_the_tuning_and_the_frets_it_is_given() {
    let drop_d = neck(n("G"), &[2, 9, 2, 7, 11, 4], 19, None, false, &[]);
    assert_eq!(at(&drop_d, 0, 0).name, "D");
    assert_eq!(at(&drop_d, 0, 5).mark, 0, "G is two frets higher on a string tuned down");
    assert!(drop_d.iter().any(|m| m.fret == 19));
    let seven = neck(n("G"), &[11, 4, 9, 2, 7, 11, 4], FRETS, None, false, &[]);
    assert_eq!(seven.iter().map(|m| m.string).max(), Some(6));
}

#[test]
fn the_pentatonic_hides_the_4th_and_7th_unless_they_are_played() {
    let marks = neck(n("G"), &STANDARD, FRETS, Some((3, 5)), true, &[]);
    assert!(marks.iter().all(|m| m.mark != 3));
    assert!(marks.iter().all(|m| !(m.string == 1 && m.fret == 3)), "C is not in the pentatonic");
    // played, C shows as outside the scale on show, lit though the shape is elsewhere
    let sounding = [Played { row: 1, fret: 15 }];
    let marks = neck(n("G"), &STANDARD, FRETS, Some((3, 5)), true, &sounding);
    let c = at(&marks, 1, 15);
    assert_eq!((c.name.as_str(), c.mark, c.sounding, c.lit), ("C", 4, true, true));
}

/// The marks lit on a song page: (row, fret) of each, top row first.
fn lit(p: &SongPage) -> Vec<(i32, i32)> {
    p.notes.iter().filter(|m| m.lit).map(|m| (m.string, m.fret)).collect()
}

#[test]
fn following_a_song_only_the_note_sounding_stands_out() {
    // D on the 2nd string sounds; B before it and E after it stay as dark as the rest of the scale
    let p = page(&solo(), Some(2), true, Hand::Follow);
    assert_eq!(lit(&p), [(4, 3)]);
    assert!(p.notes.iter().all(|m| m.sounding == m.lit));
    assert!(p.notes.iter().all(|m| !(m.string == 3 && m.fret == 3)), "the blue third went with its note");
    // the shape still shows where the hand is
    assert_eq!(p.window, Some((3, 5)));
}

#[test]
fn the_lead_of_the_mockup() {
    let p = page(&solo(), Some(2), true, Hand::Follow);
    assert_eq!((p.shape, p.window), (1, Some((3, 5))), "the hand is in shape 1, frets 3 to 5");
    assert_eq!(p.tuning, "E A D G B E");
    assert_eq!(p.open_names, ["E", "A", "D", "G", "B", "E"]);
    assert_eq!((p.frets, p.window), (FRETS, Some((3, 5))));
    let now = &p.now;
    assert_eq!(
        (now.name.as_str(), now.chord, now.string, now.fret, now.outside, now.empty),
        ("D", false, 2, 3, false, false),
        "D on the 2nd string, 3rd fret"
    );
    let chips: Vec<(String, bool, i32, i32, bool)> =
        p.phrase.iter().map(|c| (c.name.to_string(), c.chord, c.string, c.state, c.outside)).collect();
    assert_eq!(
        chips,
        [
            // a note outside the key is a lowered degree: the blue third of G
            ("Bb".into(), false, 3, 0, true),
            ("B".into(), false, 3, 0, false),
            ("D".into(), false, 2, 1, false),
            ("E".into(), false, 2, 2, false),
            ("D".into(), false, 2, 3, false),
            ("B".into(), false, 3, 3, false),
            ("G".into(), false, 4, 3, false),
        ]
    );
    assert_eq!(p.phrase.iter().map(|c| c.index).collect::<Vec<_>>(), [0, 1, 2, 3, 4, 5, 6]);
    // the 2nd string is the 5th row from the top
    assert!(at(&p.notes, 4, 3).sounding);
    assert!(p.notes.iter().all(|m| m.mark != 3));
}

#[test]
fn the_rhythm_of_the_mockup() {
    let p = page(&rhythm(), Some(0), false, Hand::Follow);
    assert_eq!((p.now.name.as_str(), p.now.chord, p.now.fret), ("G", true, 3), "a chord, barred at the 3rd fret");
    let chip = |c: &PhraseChip| (c.name.to_string(), c.chord, c.fret, c.state);
    assert_eq!(chip(&p.phrase[0]), ("G".into(), true, 3, 1));
    assert_eq!(chip(&p.phrase[1]), ("C".into(), true, 3, 2));
    assert_eq!(p.phrase.len(), 2);
    assert_eq!(
        lit(&p),
        [(0, 3), (1, 5), (2, 5), (3, 4), (4, 3), (5, 3)],
        "the whole G chord, nothing of the C after it"
    );
    assert!(p.notes.iter().any(|m| m.mark == 3), "the major scale shows the 4th and 7th");
}

#[test]
fn a_chord_the_song_does_not_name_names_itself() {
    let triad = song(chord(0.0, &[(6, 3), (5, 2), (4, 0)]), &[]);
    assert_eq!(page(&triad, Some(0), false, Hand::Follow).now.name, "G");
    let fifth = song(chord(0.0, &[(6, 3), (4, 0)]), &[]);
    assert_eq!(page(&fifth, Some(0), false, Hand::Follow).now.name, "G D");
    let power = song(chord(0.0, &[(6, 3), (5, 5), (4, 5)]), &[]);
    assert_eq!(page(&power, Some(0), false, Hand::Follow).now.name, "G D", "the octave is not named twice");
}

#[test]
fn a_shape_can_be_picked_or_the_whole_neck_shown() {
    let p = page(&solo(), Some(2), true, Hand::Shape(2));
    assert_eq!((p.window, p.shape), (Some((5, 7)), 2));
    let p = page(&solo(), Some(2), true, Hand::Whole);
    assert_eq!((p.window, p.shape), (None, 0), "the whole neck is no shape");
    assert_eq!(lit(&p), [(4, 3)], "the whole neck shows the scale, still only the note sounding is lit");
    assert_eq!(
        (Hand::from_index(0), Hand::from_index(3), Hand::from_index(6)),
        (Hand::Follow, Hand::Shape(3), Hand::Whole)
    );
    assert_eq!(Hand::Shape(3).index(), 3);
}

#[test]
fn a_shape_picked_on_a_long_neck_is_the_box_nearest_the_hand() {
    let frets = [(3, 15), (3, 16), (2, 15), (2, 17), (2, 19)];
    let high = song(frets.iter().enumerate().map(|(i, &(s, f))| note(i as f64, s, f)).collect(), &[]);
    let p = page(&high, Some(4), true, Hand::Shape(2));
    assert_eq!((p.frets, p.window), (19, Some((17, 19))), "not the box at 5 to 7");
}

#[test]
fn before_the_first_note_nothing_on_the_neck_stands_out() {
    let p = page(&solo(), None, true, Hand::Follow);
    assert!(p.now.empty);
    assert_eq!(p.phrase.iter().map(|c| c.state).collect::<Vec<_>>(), [2, 3, 3, 3, 3, 3, 3]);
    assert!(lit(&p).is_empty());
    assert!(p.notes.iter().all(|m| !m.sounding));
    assert_eq!((p.shape, p.window), (1, Some((3, 5))), "the hand waits where the first note is");
}

#[test]
fn a_song_with_no_notes_still_shows_its_scale() {
    let p = page(&song(Vec::new(), &[]), None, false, Hand::Follow);
    assert!(p.now.empty && p.phrase.is_empty());
    assert!(!p.notes.is_empty());
}

#[test]
fn the_phrase_keeps_its_size_at_both_ends() {
    assert_eq!(phrase_strip(3, Some(1)), 0..3);
    assert_eq!(phrase_strip(100, Some(50)), 48..55);
    assert_eq!(phrase_strip(100, Some(99)), 93..100);
    assert_eq!(phrase_strip(100, None), 0..7);
}

#[test]
fn a_four_string_bass_counts_its_shapes_on_its_low_e() {
    let bass = Song {
        tuning: vec![43, 38, 33, 28],
        notes: vec![SongNote { start: 0.0, end: 0.5, string: 4, fret: 3, midi: 31 }],
        ..song(Vec::new(), &[])
    };
    let p = page(&bass, Some(0), true, Hand::Follow);
    assert_eq!((p.tuning.as_str(), p.window), ("E A D G", Some((3, 5))));
    assert!(at(&p.notes, 0, 3).sounding);
}
