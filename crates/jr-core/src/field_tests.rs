use super::*;
use crate::note::{fifths, parse};
use Function::*;

fn notes(s: &str) -> Vec<Pc> {
    s.split_whitespace().map(|n| parse(n).unwrap()).collect()
}

#[test]
fn c_major_matches_the_wheel() {
    // L1, L2, L3 in harmonic-field/art_field.py: (major degree, minor degree, extension,
    // major function, minor function), T = rest, S = transition, D = tension.
    let wheel = [
        ("C", "I", "III", "7M(9)", Rest, Rest),
        ("Dm", "II", "IV", "m7(9)", Transition, Transition),
        ("Em", "III", "V", "m7(b9)", Rest, Tension),
        ("F", "IV", "VI", "7M(9)", Transition, Rest),
        ("G", "V", "VII", "7(9)", Tension, Tension),
        ("Am", "VI", "I", "m7(9)", Rest, Rest),
        ("B°", "VII", "II", "m7(b5b9)", Tension, Transition),
    ];
    for (c, w) in harmonic_field(0).iter().zip(wheel) {
        assert_eq!(
            (
                c.name().as_str(),
                c.roman(),
                c.minor_roman(),
                c.extension,
                c.function,
                c.minor_function
            ),
            w
        );
    }
}

#[test]
fn the_diminished_ring_matches_the_wheel() {
    // DIM in art_field.py: the VII° of each key, in fifths order.
    let dim: Vec<String> = fifths()
        .iter()
        .map(|&k| harmonic_field(k)[6].name())
        .collect();
    assert_eq!(
        dim,
        ["B°", "F#°", "C#°", "G#°", "D#°", "A#°", "F°", "C°", "G°", "D°", "A°", "E°"]
    );
}

#[test]
fn every_key_has_its_relative_minor_as_vi() {
    for k in 0..12 {
        let f = harmonic_field(k);
        assert_eq!(f[5].minor_degree, 1);
        assert_eq!(f[5].root, (k + 9) % 12);
        assert_eq!(f[5].notes, [(k + 9) % 12, k, (k + 4) % 12]);
    }
}

#[test]
fn the_example_of_the_design_doc() {
    // Song in G: G-B-D, C-E-G, D-F#-A light I, IV, V.
    let g = 7;
    assert_eq!(identify(g, &notes("G B D")).map(|c| c.roman()), Some("I"));
    assert_eq!(identify(g, &notes("C E G")).map(|c| c.roman()), Some("IV"));
    assert_eq!(identify(g, &notes("D F# A")).map(|c| c.roman()), Some("V"));
}

#[test]
fn octaves_order_and_repeats_do_not_matter() {
    assert_eq!(
        identify(7, &notes("D G B G D G")).map(|c| c.degree),
        Some(1)
    );
}

#[test]
fn power_chords_match_their_root() {
    assert_eq!(
        identify(7, &notes("E B")).map(|c| c.name()),
        Some("Em".into())
    );
    // B and F# is the III of G (Bm); in C the VII is B° (B D F), so there it is outside.
    assert_eq!(identify(7, &notes("B F#")).map(|c| c.degree), Some(3));
    assert_eq!(identify(0, &notes("B F#")), None);
}

#[test]
fn chords_outside_the_key_are_not_forced_onto_a_degree() {
    // G7 in G: the F is borrowed (V7 of IV).
    assert_eq!(identify(7, &notes("G B D F")), None);
    // D major in C: secondary dominant.
    assert_eq!(identify(0, &notes("D F# A")), None);
    // A single note is not a chord.
    assert_eq!(identify(0, &notes("C")), None);
    assert_eq!(identify(0, &[]), None);
}

#[test]
fn notes_outside_the_scale() {
    assert_eq!(outside_scale(7, &notes("G B D F F")), notes("F"));
    assert!(outside_scale(0, &notes("C D E F G A B")).is_empty());
}
