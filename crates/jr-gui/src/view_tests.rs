use super::*;
use jr_core::note::parse;

fn n(s: &str) -> Pc {
    parse(s).unwrap()
}

fn notes(s: &str) -> Vec<Pc> {
    s.split_whitespace().map(n).collect()
}

#[test]
fn titles_in_portuguese_spelled_as_in_the_key() {
    assert_eq!(key_title(n("G")), "Sol maior");
    assert_eq!(key_subtitle(n("G")), "Relativa: Mi menor");
    assert_eq!(key_title(n("Bb")), "Sib maior");
    assert_eq!(relative_short(n("C")), "relativa Am");
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
    let got: Vec<(String, i32, i32)> = b.iter().map(|s| (s.label.to_string(), s.first, s.last)).collect();
    assert_eq!(
        got,
        [
            ("1".into(), 3, 5),
            ("2".into(), 5, 7),
            ("3".into(), 7, 10),
            ("4".into(), 10, 12),
            ("5".into(), 0, 3),
            ("Braço todo".into(), -1, -1)
        ]
    );
    for key in 0..12 {
        for s in &shape_buttons(key)[..5] {
            assert!(s.first >= 0 && s.last <= FRETS, "key {key}: {s:?} off the neck");
        }
    }
    assert_eq!(shape_subtitle(n("G"), &b[0]), "Forma 1 · 2 na relativa menor · casas 3 a 5");
}

#[test]
fn the_neck_has_every_scale_note_and_fades_outside_the_shape() {
    let marks = neck(n("G"), Some((3, 5)), &[]);
    // seven notes out of twelve, on six strings, sixteen positions each
    let expected = (0..6)
        .map(|s| (0..=FRETS).filter(|&f| jr_core::scale(n("G")).contains(&note_at(STANDARD[s], f as u8))).count())
        .sum::<usize>();
    assert_eq!(marks.len(), expected);
    let tonic = marks.iter().find(|m| m.string == 0 && m.fret == 3).unwrap();
    assert_eq!((tonic.name.as_str(), tonic.mark, tonic.lit), ("G", 0, true));
    let open_e = marks.iter().find(|m| m.string == 0 && m.fret == 0).unwrap();
    assert_eq!((open_e.mark, open_e.lit), (1, false));
    assert!(marks.iter().all(|m| m.ring == 0));
}

#[test]
fn the_jam_moment_of_the_mockup() {
    // Song in G, C sounding with the hand at fret 3: IV, transition, shape 1 (3 to 5), C outside.
    let j = jam(n("G"), Some("C"), &notes("C E G"), 3.0);
    assert_eq!(j.moment.roman, "IV");
    assert_eq!(j.moment.function, 1);
    assert_eq!(j.moment.lit, 3);
    assert_eq!(j.moment.outside, "C");
    assert_eq!(j.moment.shape, "Pentatônica de Sol, forma 1 (casas 3 a 5)");
    // C on the 5th string, fret 3: in the shape, outside the pentatonic, red ring.
    let c = j.notes.iter().find(|m| m.string == 1 && m.fret == 3).unwrap();
    assert_eq!((c.mark, c.ring), (3, 2));
    // G on the 6th string, fret 3: the tonic, white ring.
    let g = j.notes.iter().find(|m| m.string == 0 && m.fret == 3).unwrap();
    assert_eq!(g.ring, 1);
    // E on the open 6th: sounding, but outside the shape, so no ring.
    let e = j.notes.iter().find(|m| m.string == 0 && m.fret == 0).unwrap();
    assert_eq!(e.ring, 0);
}

#[test]
fn a_borrowed_chord_in_the_jam() {
    let j = jam(n("G"), Some("F"), &notes("F A C"), 3.0);
    assert_eq!((j.moment.roman.as_str(), j.moment.function, j.moment.lit), ("♭VII", -1, -1));
    assert_eq!((j.moment.outside.as_str(), j.moment.outside_count), ("F e C", 2));
    // F is not in G major: drawn only because it is sounding, as an outside mark with a red ring.
    let f = j.notes.iter().find(|m| m.string == 1 && m.fret == 8).unwrap();
    assert_eq!((f.name.as_str(), f.mark, f.lit), ("F", 4, false));
    let f_in = j.notes.iter().find(|m| m.string == 3 && m.fret == 10);
    assert!(f_in.is_none() || !f_in.unwrap().lit);
}

#[test]
fn notes_with_no_name_still_show() {
    let j = jam(n("C"), None, &notes("C D"), 5.0);
    assert_eq!((j.moment.chord.as_str(), j.moment.roman.as_str()), ("C D", "?"));
    let j = jam(n("C"), None, &notes("A C E"), 5.0);
    assert_eq!((j.moment.chord.as_str(), j.moment.roman.as_str()), ("Am", "VI"));
}
