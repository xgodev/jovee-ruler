//! Pitch classes and how they are spelled.

/// A pitch class: 0 = C, 1 = C#/Db … 11 = B.
pub type Pc = u8;

const SHARP: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const FLAT: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

/// Major keys spelled with flats: F, Bb, Eb, Ab, Db. The wheel prints them that way.
const FLAT_KEYS: [Pc; 5] = [5, 10, 3, 8, 1];

/// Any number of semitones folded into one octave.
pub fn pc(semitones: i32) -> Pc {
    semitones.rem_euclid(12) as Pc
}

/// The name with sharps, as on the scale ruler's paper.
pub fn name(p: Pc) -> &'static str {
    SHARP[(p % 12) as usize]
}

/// The name as it is spelled in a major key: flats in the flat keys, sharps otherwise.
pub fn name_in(key: Pc, p: Pc) -> &'static str {
    if FLAT_KEYS.contains(&(key % 12)) {
        FLAT[(p % 12) as usize]
    } else {
        SHARP[(p % 12) as usize]
    }
}

/// The name of a major key, as on the wheel ("Gb/F#" is shown as F#).
pub fn key_name(key: Pc) -> &'static str {
    name_in(key, key)
}

/// The name of a minor key from its root, as on the wheel: Ebm and Bbm, sharps otherwise.
pub fn minor_key_name(root: Pc) -> String {
    let n = match root % 12 {
        3 => "Eb",
        10 => "Bb",
        r => SHARP[r as usize],
    };
    format!("{n}m")
}

/// Reads a note name: a letter, then any number of `#` or `b` ("C", "f#", "Bb", "E#").
pub fn parse(s: &str) -> Option<Pc> {
    let mut chars = s.trim().chars();
    let base: i32 = match chars.next()?.to_ascii_uppercase() {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let mut shift = 0;
    for c in chars {
        match c {
            '#' | '♯' => shift += 1,
            'b' | '♭' => shift -= 1,
            _ => return None,
        }
    }
    Some(pc(base + shift))
}

/// The keys in the order of the wheel: the circle of fifths, starting at C.
pub fn fifths() -> [Pc; 12] {
    std::array::from_fn(|i| pc(7 * i as i32))
}

#[cfg(test)]
#[path = "note_tests.rs"]
mod tests;
