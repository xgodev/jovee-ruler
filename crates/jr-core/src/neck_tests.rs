use super::*;
use crate::note::{parse, pc};

fn n(s: &str) -> Pc {
    parse(s).unwrap()
}

#[test]
fn holes_match_the_slider() {
    // scale.scad: degree 0 square, 9 diamond, PENT circles, the rest of DIAT triangles.
    let g = n("G");
    let low = STANDARD[0];
    assert_eq!(mark(g, note_at(low, 3)), Mark::Tonic);
    assert_eq!(mark(g, note_at(low, 0)), Mark::MinorTonic);
    assert_eq!(mark(g, note_at(low, 5)), Mark::Pentatonic);
    let triangles: Vec<Pc> = (0..12).filter(|&p| mark(g, p) == Mark::Scale).collect();
    assert_eq!(triangles, [n("C"), n("F#")]);
    assert_eq!(mark(g, n("F")), Mark::Outside);
}

#[test]
fn every_hole_of_the_slider_in_every_key() {
    // scale.scad cuts the hole of string i, column c from degree(i,c) = (TUNING[i]+c+OFF)%12
    // with OFF = 8, and the KEY window reads the note of the 6th string under column 0.
    const OFF: i32 = 8;
    for f0 in 0..12u8 {
        let key = note_at(STANDARD[0], f0);
        for (i, &open) in STANDARD.iter().enumerate() {
            for c in 0..13u8 {
                let printed = match (STANDARD[i] as i32 + c as i32 + OFF) % 12 {
                    0 => Mark::Tonic,
                    9 => Mark::MinorTonic,
                    2 | 4 | 7 => Mark::Pentatonic,
                    5 | 11 => Mark::Scale,
                    _ => Mark::Outside,
                };
                assert_eq!(mark(key, note_at(open, f0 + c)), printed, "slider at {f0}, string {i}, column {c}");
            }
        }
    }
}

#[test]
fn the_pentatonic_of_g() {
    assert_eq!(pentatonic(n("G")), ["G", "A", "B", "D", "E"].map(n));
    assert_eq!(scale(n("G")), ["G", "A", "B", "C", "D", "E", "F#"].map(n));
    assert_eq!(outside_pentatonic(n("G"), &["G", "C", "B", "F#", "C"].map(n)), ["C", "F#"].map(n));
}

/// Semitones from the open 6th string to each open string of a standard neck.
const OPEN: [i32; 6] = [0, 5, 10, 15, 19, 24];

/// The five boxes of one octave of `key`, from shape 1 on: (shape, first, last).
fn octave(key: Pc, pentatonic: bool) -> Vec<(u8, i32, i32)> {
    let tonic = tonic_fret(STANDARD[0], key) as i32;
    shapes(STANDARD[0], key, 30, pentatonic)
        .into_iter()
        .filter(|s| s.first >= tonic - 1 && s.first < tonic + 11)
        .map(|s| (s.major, s.first, s.last))
        .collect()
}

#[test]
fn the_shapes_of_g() {
    // G on the 6th string is fret 3. The pentatonic boxes, and the major scale's five positions.
    assert_eq!(octave(n("G"), true), [(1, 2, 5), (2, 4, 8), (3, 7, 10), (4, 9, 12), (5, 12, 15)]);
    assert_eq!(octave(n("G"), false), [(1, 2, 5), (2, 4, 8), (3, 7, 10), (4, 9, 13), (5, 11, 15)]);
}

#[test]
fn the_shapes_of_ab_take_fret_3() {
    // The owner, on Ab: shape 1 plays C on the 5th string, F on the 4th and Bb on the 3rd at fret 3.
    let three: Vec<Pc> = (1..4).map(|s| note_at(STANDARD[s], 3)).collect();
    assert_eq!(three, ["C", "F", "Bb"].map(n));
    assert_eq!(octave(n("Ab"), true), [(1, 3, 6), (2, 5, 9), (3, 8, 11), (4, 10, 13), (5, 13, 16)]);
    assert_eq!(octave(n("Ab"), false), [(1, 3, 6), (2, 5, 9), (3, 8, 11), (4, 10, 14), (5, 12, 16)]);
}

/// The pentatonic as shape `major` of `key` plays it: from that degree on the 6th string inside the
/// box, the scale upwards, two notes on each string. Pitches in semitones over the open 6th string.
fn two_per_string(key: Pc, major: u8, first: i32, last: i32) -> Vec<i32> {
    let degree = pentatonic(key)[major as usize - 1];
    let start = (first..=last).find(|&f| name(f) == degree).expect("the shape starts inside its box");
    (start..).filter(|&p| pentatonic(key).contains(&name(p))).take(12).collect()
}

/// The note `p` semitones over the open 6th string.
fn name(p: i32) -> Pc {
    pc(STANDARD[0] as i32 + p)
}

/// Whether every note of a shape has a fret inside `first..=last`: the pentatonic two per string,
/// and with the major scale every note of the scale between its lowest note and its highest.
fn fits(notes: &[i32], key: Pc, pentatonic: bool, first: i32, last: i32) -> bool {
    let inside = |f: i32| (first..=last).contains(&f);
    let pairs = notes.iter().enumerate().all(|(i, p)| inside(p - OPEN[i / 2]));
    let added = pentatonic
        || (notes[0]..notes[11]).filter(|&p| scale(key).contains(&name(p))).all(|p| OPEN.iter().any(|o| inside(p - o)));
    pairs && added
}

#[test]
fn every_box_holds_every_note_of_its_shape_and_no_more_frets() {
    for key in 0..12 {
        for pentatonic in [true, false] {
            for s in shapes(STANDARD[0], key, 24, pentatonic) {
                let notes = two_per_string(key, s.major, s.first, s.last);
                let what = format!("key {key}, pentatonic {pentatonic}, {s:?}");
                assert!(fits(&notes, key, pentatonic, s.first, s.last), "{what}: a note is left out");
                assert!(!fits(&notes, key, pentatonic, s.first + 1, s.last), "{what}: the first fret is not needed");
                assert!(!fits(&notes, key, pentatonic, s.first, s.last - 1), "{what}: the last fret is not needed");
            }
        }
    }
}

#[test]
fn shape_1_of_a_minor_is_on_fret_5_and_17() {
    // how-to-use.md: Am (key C), shape 1 of the minor starts on fret 5, and again on 17.
    let starts: Vec<i32> = shapes(STANDARD[0], n("C"), 24, true)
        .into_iter()
        .filter(|s| s.minor == 1 && s.first >= 0)
        .map(|s| s.first)
        .collect();
    assert_eq!(starts, [5, 17]);
}

#[test]
fn shapes_cover_the_whole_neck_and_join_up() {
    for key in 0..12 {
        for pentatonic in [true, false] {
            let all = shapes(STANDARD[0], key, 24, pentatonic);
            assert!(all.first().unwrap().first <= 0, "key {key}: nothing at the nut");
            assert!(all.last().unwrap().last >= 24, "key {key}: nothing at fret 24");
            for pair in all.windows(2) {
                assert!(pair[0].first < pair[1].first, "key {key}: boxes out of order");
                assert!(pair[1].first <= pair[0].last, "key {key}: a gap between boxes");
            }
        }
    }
}

#[test]
fn the_shape_near_the_hand() {
    // Design doc: a song in G played around the 3rd fret sits in shape 1 (frets 2-5).
    let s = shape_near(STANDARD[0], n("G"), 3.0, true);
    assert_eq!((s.major, s.first, s.last), (1, 2, 5));
    let s = shape_near(STANDARD[0], n("G"), 13.5, true);
    assert_eq!((s.major, s.first, s.last), (5, 12, 15));
    // At the nut in G the nearest whole box is shape 5 an octave down (0-3).
    let s = shape_near(STANDARD[0], n("G"), 0.0, true);
    assert_eq!((s.major, s.first, s.last), (5, 0, 3));
    // The major scale's shape 5 reaches back for the 7th: an octave down, one fret under the nut.
    let s = shape_near(STANDARD[0], n("G"), 0.0, false);
    assert_eq!((s.major, s.first, s.last), (5, -1, 3));
    // In E shape 1 takes the fret under the nut, and is still the shape of the open strings.
    let s = shape_near(STANDARD[0], n("E"), 1.0, true);
    assert_eq!((s.major, s.first, s.last), (1, -1, 2));
}
