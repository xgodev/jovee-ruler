//! A capo on the neck: the shape the hand makes for a chord that sounds higher, and how many of a
//! song's chords need a barre with the capo on each fret.
//!
//! The capo on fret 4 makes every string sound four semitones higher, so the E the song plays is made
//! with the hand in the C shape. A shape is open, or needs a barre: the open ones are the chords a
//! beginner learns first (C, D, E, G, A, Am, Dm, Em, their 7ths, sus and maj7, Cadd9, Gadd9) and every
//! power chord; a slash chord is judged by the chord before the slash.

use crate::note::{parse, pc, Pc};

/// Chords are spelled as players write them: a major chord on a black key takes the flat (but F#), a
/// minor one the sharp (but Eb and Bb).
const MAJOR: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];
const MINOR: [&str; 12] = ["C", "C#", "D", "Eb", "E", "F", "F#", "G", "G#", "A", "Bb", "B"];

/// The shapes made without a barre, as root and suffix.
const OPEN: [(Pc, &str); 30] = [
    (0, ""),
    (2, ""),
    (4, ""),
    (7, ""),
    (9, ""),
    (9, "m"),
    (2, "m"),
    (4, "m"),
    (0, "7"),
    (2, "7"),
    (4, "7"),
    (7, "7"),
    (9, "7"),
    (11, "7"),
    (9, "m7"),
    (2, "m7"),
    (4, "m7"),
    (0, "maj7"),
    (2, "maj7"),
    (4, "maj7"),
    (5, "maj7"),
    (7, "maj7"),
    (9, "maj7"),
    (9, "sus2"),
    (9, "sus4"),
    (2, "sus2"),
    (2, "sus4"),
    (4, "sus4"),
    (0, "add9"),
    (7, "add9"),
];

/// One chord as the hand makes it with the capo on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapoShape {
    pub name: String,
    pub barre: bool,
}

/// The capo on one fret (0 = no capo): the shapes of the song's chords there and how many need a barre.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapoFret {
    pub fret: u8,
    pub shapes: Vec<CapoShape>,
    pub barres: usize,
    /// One of the frets with the fewest barres, when the song has a barre without a capo.
    pub fewest: bool,
}

/// A chord name taken apart: its root, what follows it, and the bass after the slash.
struct Chord<'a> {
    root: Pc,
    suffix: &'a str,
    bass: Option<&'a str>,
}

fn chord(name: &str) -> Option<Chord<'_>> {
    let (head, bass) = match name.split_once('/') {
        Some((head, bass)) => (head, Some(bass)),
        None => (name, None),
    };
    let letter = head.chars().next().filter(char::is_ascii_uppercase)?;
    let accidental = head[1..].starts_with(['#', 'b']);
    let root_len = letter.len_utf8() + usize::from(accidental);
    let root = parse(&head[..root_len])?;
    Some(Chord { root, suffix: &head[root_len..], bass })
}

fn is_minor(suffix: &str) -> bool {
    suffix.starts_with('m') && !suffix.starts_with("maj")
}

/// The chord `name` as the hand makes it with the capo on fret `capo`; a name that is not a chord stays.
pub fn shape_name(name: &str, capo: u8) -> String {
    let Some(c) = chord(name).filter(|_| capo % 12 != 0) else {
        return name.to_string();
    };
    let down = |p: Pc| pc(p as i32 - capo as i32) as usize;
    let names = if is_minor(c.suffix) { MINOR } else { MAJOR };
    let mut out = format!("{}{}", names[down(c.root)], c.suffix);
    if let Some(bass) = c.bass {
        out.push('/');
        match parse(bass) {
            Some(p) => out.push_str(MAJOR[down(p)]),
            None => out.push_str(bass),
        }
    }
    out
}

/// What a suffix means, written one way: "7M" and "M7" are maj7, and the extensions in brackets go.
fn plain(suffix: &str) -> &str {
    match suffix.split('(').next().unwrap_or_default() {
        "7M" | "M7" | "Δ" => "maj7",
        s => s,
    }
}

/// Whether the shape `name` needs a barre; a name that is not a chord does not.
pub fn is_barre(name: &str) -> bool {
    let Some(c) = chord(name) else { return false };
    let suffix = plain(c.suffix);
    suffix != "5" && !OPEN.contains(&(c.root, suffix))
}

/// The capo on each fret from none to `last`, for the song's `chords` as written: each distinct chord
/// once, in the order it first comes.
pub fn capo_frets(chords: &[String], last: u8) -> Vec<CapoFret> {
    let mut rows: Vec<CapoFret> = (0..=last)
        .map(|fret| {
            let mut shapes: Vec<CapoShape> = Vec::new();
            for name in chords.iter().filter(|name| chord(name).is_some()) {
                let shape = shape_name(name, fret);
                if !shapes.iter().any(|s| s.name == shape) {
                    shapes.push(CapoShape { barre: is_barre(&shape), name: shape });
                }
            }
            let barres = shapes.iter().filter(|s| s.barre).count();
            CapoFret { fret, shapes, barres, fewest: false }
        })
        .collect();
    let fewest = rows.iter().map(|r| r.barres).min().unwrap_or(0);
    if rows.first().is_some_and(|r| r.barres > 0) {
        rows.iter_mut().filter(|r| r.barres == fewest).for_each(|r| r.fewest = true);
    }
    rows
}

#[cfg(test)]
#[path = "capo_tests.rs"]
mod tests;
