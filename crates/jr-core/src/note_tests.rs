use super::*;

#[test]
fn the_wheel_reads_the_keys_in_fifths() {
    // CIRC and MIN in harmonic-field/art_field.py
    let majors: Vec<&str> = fifths().iter().map(|&k| key_name(k)).collect();
    assert_eq!(
        majors,
        ["C", "G", "D", "A", "E", "B", "F#", "Db", "Ab", "Eb", "Bb", "F"]
    );
    let minors: Vec<String> = fifths()
        .iter()
        .map(|&k| minor_key_name(pc(k as i32 + 9)))
        .collect();
    assert_eq!(
        minors,
        ["Am", "Em", "Bm", "F#m", "C#m", "G#m", "Ebm", "Bbm", "Fm", "Cm", "Gm", "Dm"]
    );
}

#[test]
fn note_names_read_back() {
    for p in 0..12 {
        assert_eq!(parse(name(p)), Some(p));
        assert_eq!(parse(FLAT[p as usize]), Some(p));
    }
    assert_eq!(parse("e#"), Some(5));
    assert_eq!(parse("Cb"), Some(11));
    assert_eq!(parse("H"), None);
    assert_eq!(parse("C7"), None);
    assert_eq!(parse(""), None);
}

#[test]
fn flat_keys_spell_with_flats() {
    assert_eq!(name_in(5, 10), "Bb"); // F major has Bb
    assert_eq!(name_in(7, 6), "F#"); // G major has F#
}
