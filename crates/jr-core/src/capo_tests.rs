use super::*;

fn names(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn a_capo_on_the_fourth_fret_turns_e_into_the_c_shape() {
    assert_eq!(shape_name("E", 4), "C");
    assert_eq!(shape_name("B", 4), "G");
    assert_eq!(shape_name("C#m", 4), "Am");
    assert_eq!(shape_name("A", 4), "F");
}

#[test]
fn no_capo_keeps_the_name_as_written() {
    assert_eq!(shape_name("C#m", 0), "C#m");
    assert_eq!(shape_name("Gb7", 0), "Gb7");
}

#[test]
fn the_suffix_and_the_bass_follow_the_root() {
    assert_eq!(shape_name("E7", 2), "D7");
    assert_eq!(shape_name("Emaj7", 4), "Cmaj7");
    assert_eq!(shape_name("Bsus4", 4), "Gsus4");
    assert_eq!(shape_name("B/D#", 4), "G/B");
    assert_eq!(shape_name("C#m7(9)", 4), "Am7(9)");
}

#[test]
fn shapes_are_spelled_as_players_write_them() {
    // a major chord on a black key takes the flat, but F#; a minor one the sharp, but Eb and Bb
    let majors: Vec<String> = (0..12).map(|c| shape_name("B", c)).collect();
    assert_eq!(majors, ["B", "Bb", "A", "Ab", "G", "F#", "F", "E", "Eb", "D", "Db", "C"]);
    let minors: Vec<String> = (0..12).map(|c| shape_name("Bm", c)).collect();
    assert_eq!(minors, ["Bm", "Bbm", "Am", "G#m", "Gm", "F#m", "Fm", "Em", "Ebm", "Dm", "C#m", "Cm"]);
}

#[test]
fn a_name_that_is_not_a_chord_stays_as_it_is() {
    assert_eq!(shape_name("N.C.", 4), "N.C.");
    assert_eq!(shape_name("", 4), "");
    assert_eq!(shape_name("riff", 4), "riff");
}

#[test]
fn open_shapes_need_no_barre() {
    for shape in [
        "C", "D", "E", "G", "A", "Am", "Dm", "Em", "C7", "D7", "E7", "G7", "A7", "B7", "Am7", "Dm7", "Em7", "Cmaj7",
        "Fmaj7", "Asus2", "Asus4", "Dsus2", "Dsus4", "Esus4", "Cadd9", "Gadd9", "E5", "F5", "C#5", "G/B", "D/F#",
    ] {
        assert!(!is_barre(shape), "{shape}");
    }
}

#[test]
fn the_rest_need_a_barre() {
    for shape in ["F", "B", "Bb", "Bm", "F#m", "C#m", "Cm", "Gm", "F7", "Bm7", "Ab", "F/A", "Bdim"] {
        assert!(is_barre(shape), "{shape}");
    }
}

#[test]
fn a_name_that_is_not_a_chord_is_no_barre() {
    assert!(!is_barre("N.C."));
}

#[test]
fn each_fret_lists_the_shapes_and_counts_the_barres() {
    let rows = capo_frets(&names(&["E", "B", "C#m", "A", "E"]), 7);
    let counts: Vec<usize> = rows.iter().map(|r| r.barres).collect();
    assert_eq!(counts, [2, 4, 1, 4, 1, 3, 4, 1]);
    let fourth = &rows[4];
    assert_eq!(fourth.fret, 4);
    let shapes: Vec<(&str, bool)> = fourth.shapes.iter().map(|s| (s.name.as_str(), s.barre)).collect();
    assert_eq!(shapes, [("C", false), ("G", false), ("Am", false), ("F", true)]);
}

#[test]
fn the_frets_with_the_fewest_barres_are_marked() {
    let rows = capo_frets(&names(&["E", "B", "C#m", "A"]), 7);
    let fewest: Vec<u8> = rows.iter().filter(|r| r.fewest).map(|r| r.fret).collect();
    assert_eq!(fewest, [2, 4, 7]);
}

#[test]
fn a_song_with_no_barre_marks_nothing() {
    let rows = capo_frets(&names(&["G", "C", "D", "Em"]), 7);
    assert_eq!(rows[0].barres, 0);
    assert!(rows.iter().all(|r| !r.fewest));
}

#[test]
fn a_song_with_no_chords_lists_the_frets_bare() {
    let rows = capo_frets(&[], 3);
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|r| r.shapes.is_empty() && r.barres == 0 && !r.fewest));
}

#[test]
fn what_is_not_a_chord_is_left_out_of_the_list() {
    let rows = capo_frets(&names(&["N.C.", "E"]), 0);
    let shapes: Vec<&str> = rows[0].shapes.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(shapes, ["E"]);
}
