//! The harmonic field of a major key, as the wheel shows it: seven chords, each with its
//! degree in the major key and in the relative minor, and its function in both.

use crate::note::{name_in, pc, Pc};

/// What a chord does in the key. The wheel colors them green, orange and red
/// (T, S and D on the printed art).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Function {
    Rest,
    Transition,
    Tension,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Major,
    Minor,
    Diminished,
}

impl Quality {
    pub fn suffix(self) -> &'static str {
        match self {
            Quality::Major => "",
            Quality::Minor => "m",
            Quality::Diminished => "°",
        }
    }

    fn third_and_fifth(self) -> (i32, i32) {
        match self {
            Quality::Major => (4, 7),
            Quality::Minor => (3, 7),
            Quality::Diminished => (3, 6),
        }
    }
}

/// One chord of the field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    pub key: Pc,
    /// 1..=7 in the major key.
    pub degree: u8,
    /// 1..=7 in the relative minor.
    pub minor_degree: u8,
    pub root: Pc,
    pub quality: Quality,
    /// Root, third, fifth.
    pub notes: [Pc; 3],
    pub function: Function,
    pub minor_function: Function,
    /// The seventh chord with its ninth, as printed next to the degree.
    pub extension: &'static str,
}

const MAJOR: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
const QUALITY: [Quality; 7] = {
    use Quality::*;
    [Major, Minor, Minor, Major, Major, Minor, Diminished]
};
// Roman numerals are always uppercase (Brazilian notation); the quality is in the chord name.
const ROMAN: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];
const FUNCTION: [Function; 7] = {
    use Function::*;
    [Rest, Transition, Rest, Transition, Tension, Rest, Tension]
};
// Indexed by the degree in the relative minor.
const MINOR_FUNCTION: [Function; 7] = {
    use Function::*;
    [Rest, Transition, Rest, Transition, Tension, Rest, Tension]
};
const EXTENSION: [&str; 7] = [
    "7M(9)", "m7(9)", "m7(b9)", "7M(9)", "7(9)", "m7(9)", "m7(b5b9)",
];

impl Chord {
    pub fn roman(&self) -> &'static str {
        ROMAN[self.degree as usize - 1]
    }

    pub fn minor_roman(&self) -> &'static str {
        ROMAN[self.minor_degree as usize - 1]
    }

    /// The chord name spelled in its key: "F#m", "Bb", "B°".
    pub fn name(&self) -> String {
        format!("{}{}", name_in(self.key, self.root), self.quality.suffix())
    }
}

/// The seven chords of a major key, from I to VII.
pub fn harmonic_field(key: Pc) -> [Chord; 7] {
    std::array::from_fn(|i| {
        let root = pc(key as i32 + MAJOR[i]);
        let quality = QUALITY[i];
        let (third, fifth) = quality.third_and_fifth();
        // The relative minor starts on the sixth degree.
        let minor = (i + 2) % 7;
        Chord {
            key: key % 12,
            degree: i as u8 + 1,
            minor_degree: minor as u8 + 1,
            root,
            quality,
            notes: [root, pc(root as i32 + third), pc(root as i32 + fifth)],
            function: FUNCTION[i],
            minor_function: MINOR_FUNCTION[minor],
            extension: EXTENSION[i],
        }
    })
}

/// Which chord of the field these notes are, or `None` when they are not one.
///
/// The notes are what is sounding at one moment (octaves and repeats do not matter). They
/// match a degree when they are exactly its triad, or its root and fifth alone (a power
/// chord). A chord from outside the key, or a seventh that brings a note from outside it,
/// is `None`: it is not forced onto the nearest degree.
pub fn identify(key: Pc, notes: &[Pc]) -> Option<Chord> {
    let mut set: Vec<Pc> = notes.iter().map(|n| n % 12).collect();
    set.sort_unstable();
    set.dedup();
    harmonic_field(key).into_iter().find(|c| {
        let mut triad = c.notes.to_vec();
        triad.sort_unstable();
        if set == triad {
            return true;
        }
        let fifth_is_perfect = c.quality != Quality::Diminished;
        let mut power = vec![c.notes[0], c.notes[2]];
        power.sort_unstable();
        fifth_is_perfect && set == power
    })
}

/// The notes that do not belong to the major scale of the key, in the order given.
pub fn outside_scale(key: Pc, notes: &[Pc]) -> Vec<Pc> {
    outside(notes, |n| {
        MAJOR.contains(&(pc(n as i32 - key as i32) as i32))
    })
}

pub(crate) fn outside(notes: &[Pc], inside: impl Fn(Pc) -> bool) -> Vec<Pc> {
    let mut out = Vec::new();
    for n in notes.iter().map(|n| n % 12) {
        if !inside(n) && !out.contains(&n) {
            out.push(n);
        }
    }
    out
}

#[cfg(test)]
#[path = "field_tests.rs"]
mod tests;
