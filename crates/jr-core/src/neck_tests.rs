use super::*;
use crate::note::parse;

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

#[test]
fn the_shapes_of_g_match_the_label() {
    // The tonic of G on the 6th string is fret 3; boxes 3-5, 5-7, 7-10, 10-12, 12-15.
    let in_first_octave: Vec<(u8, u8, i32, i32)> = shapes(STANDARD[0], n("G"), 15)
        .into_iter()
        .filter(|s| s.first >= 3 && s.last <= 15)
        .map(|s| (s.major, s.minor, s.first, s.last))
        .collect();
    assert_eq!(in_first_octave, [(1, 2, 3, 5), (2, 3, 5, 7), (3, 4, 7, 10), (4, 5, 10, 12), (5, 1, 12, 15)]);
}

#[test]
fn only_shapes_3_and_5_fill_the_four_fret_window() {
    // how-to-use.md: shapes 3 and 5 are 4 frets wide, 1, 2 and 4 are 3.
    for key in 0..12 {
        for s in shapes(STANDARD[0], key, 24) {
            assert_eq!(s.frets(), if s.major == 3 || s.major == 5 { 4 } else { 3 });
        }
    }
}

#[test]
fn shape_1_of_a_minor_is_on_fret_5_and_17() {
    // how-to-use.md: Am (key C), shape 1 of the minor starts on fret 5, and again on 17.
    let starts: Vec<i32> =
        shapes(STANDARD[0], n("C"), 24).into_iter().filter(|s| s.minor == 1 && s.first >= 0).map(|s| s.first).collect();
    assert_eq!(starts, [5, 17]);
}

#[test]
fn shapes_cover_the_whole_neck_and_join_up() {
    for key in 0..12 {
        let all = shapes(STANDARD[0], key, 24);
        assert!(all.first().unwrap().first <= 0, "key {key}: nothing at the nut");
        assert!(all.last().unwrap().last >= 24, "key {key}: nothing at fret 24");
        for pair in all.windows(2) {
            assert_eq!(pair[0].last, pair[1].first, "key {key}: boxes do not share a fret");
        }
    }
}

#[test]
fn the_shape_near_the_hand() {
    // Design doc: a song in G played around the 3rd fret sits in shape 1 (frets 3-5).
    let s = shape_near(STANDARD[0], n("G"), 3.0);
    assert_eq!((s.major, s.first, s.last), (1, 3, 5));
    let s = shape_near(STANDARD[0], n("G"), 13.5);
    assert_eq!((s.major, s.first, s.last), (5, 12, 15));
    // At the nut in G the nearest whole box is shape 5 an octave down (0-3).
    let s = shape_near(STANDARD[0], n("G"), 0.0);
    assert_eq!((s.major, s.first, s.last), (5, 0, 3));
}
