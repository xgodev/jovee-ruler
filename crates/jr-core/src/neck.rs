//! The scale ruler: what each note of the neck is in a key, and the five shapes of the
//! pentatonic and of the major scale as boxes of frets.

use crate::field::outside;
use crate::note::{pc, Pc};

/// Standard tuning, lowest string first (the 6th string, drawn on top as on the paper).
pub const STANDARD: [Pc; 6] = [4, 9, 2, 7, 11, 4];

const MAJOR: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];
const PENTATONIC: [u8; 5] = [0, 2, 4, 7, 9];

/// The hole the slider has over a note, in `scale/scale.scad`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// Square: the tonic of the major key.
    Tonic,
    /// Diamond: the tonic of the relative minor.
    MinorTonic,
    /// Circle: the rest of the pentatonic.
    Pentatonic,
    /// Triangle: the two notes the major scale adds to the pentatonic (4th and 7th).
    Scale,
    /// No hole: outside the scale.
    Outside,
}

/// What a note is in a major key.
pub fn mark(key: Pc, note: Pc) -> Mark {
    let interval = pc(note as i32 - key as i32);
    match interval {
        0 => Mark::Tonic,
        9 => Mark::MinorTonic,
        i if PENTATONIC.contains(&i) => Mark::Pentatonic,
        i if MAJOR.contains(&i) => Mark::Scale,
        _ => Mark::Outside,
    }
}

/// The major scale of the key, from the tonic.
pub fn scale(key: Pc) -> [Pc; 7] {
    MAJOR.map(|i| pc(key as i32 + i as i32))
}

/// The major pentatonic of the key, from the tonic. The same notes are the minor
/// pentatonic of the relative minor.
pub fn pentatonic(key: Pc) -> [Pc; 5] {
    PENTATONIC.map(|i| pc(key as i32 + i as i32))
}

/// The notes that are not in the pentatonic of the key, in the order given.
pub fn outside_pentatonic(key: Pc, notes: &[Pc]) -> Vec<Pc> {
    outside(notes, |n| PENTATONIC.contains(&pc(n as i32 - key as i32)))
}

/// The note at a fret of a string tuned to `open`.
pub fn note_at(open: Pc, fret: u8) -> Pc {
    pc(open as i32 + fret as i32)
}

/// The first fret (0..=11) where the lowest string plays the tonic. The shapes are
/// counted from there, as the ruler counts them from its KEY window.
pub fn tonic_fret(lowest: Pc, key: Pc) -> u8 {
    pc(key as i32 - lowest as i32)
}

/// One shape: a box of frets, both ends included, from the lowest to the highest fret its
/// notes are played on across the six strings. Neighbouring boxes overlap, which is how the
/// shapes join up on the neck.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    /// 1..=5, counted in the major key (the top row of the label).
    pub major: u8,
    /// 1..=5, the same box counted in the relative minor (the bottom row).
    pub minor: u8,
    pub first: i32,
    pub last: i32,
}

impl Shape {
    pub fn frets(&self) -> i32 {
        self.last - self.first + 1
    }

    fn middle(&self) -> f32 {
        (self.first + self.last) as f32 / 2.0
    }
}

/// The boxes of the pentatonic in frets from the tonic on the lowest string, shape 1 to 5. Shape n
/// plays two notes on each string, from degree n on the lowest one; the box is every fret they use
/// on a standard neck, so shape 1 takes the fret under the tonic, where the 5th string plays the 3rd.
const PENTATONIC_BOXES: [(i32, i32); 5] = [(-1, 2), (1, 5), (4, 7), (6, 9), (9, 12)];

/// The boxes of the major scale: the pentatonic ones with the 4th and 7th added. Where a note has
/// no place inside the pentatonic box, it goes on the string that keeps its notes closest: shape 4
/// reaches up for the 4th on the 2nd string, shape 5 back for the 7th on the 3rd.
const MAJOR_BOXES: [(i32, i32); 5] = [(-1, 2), (1, 5), (4, 7), (6, 10), (8, 12)];

/// Every shape box that touches the neck from the nut to `frets`, in order along the neck, as
/// the pentatonic plays it or as the major scale does.
pub fn shapes(lowest: Pc, key: Pc, frets: i32, pentatonic: bool) -> Vec<Shape> {
    let tonic = tonic_fret(lowest, key) as i32;
    let boxes = if pentatonic { PENTATONIC_BOXES } else { MAJOR_BOXES };
    let mut out = Vec::new();
    for octave in -1..=(frets / 12 + 1) {
        for (n, &(a, b)) in boxes.iter().enumerate() {
            let first = tonic + 12 * octave + a;
            let last = tonic + 12 * octave + b;
            if last >= 0 && first <= frets {
                out.push(Shape { major: n as u8 + 1, minor: ((n + 1) % 5) as u8 + 1, first, last });
            }
        }
    }
    out
}

/// The shape whose middle is nearest a fret: where the hand is, the box it is playing in.
/// A box mostly past the nut is left out unless it is the only one there.
pub fn shape_near(lowest: Pc, key: Pc, fret: f32, pentatonic: bool) -> Shape {
    let all = shapes(lowest, key, fret.ceil().max(0.0) as i32 + 12, pentatonic);
    let on_neck = all.iter().filter(|s| s.middle() >= 0.0);
    let pick = |it: &mut dyn Iterator<Item = &Shape>| {
        it.min_by(|a, b| (a.middle() - fret).abs().total_cmp(&(b.middle() - fret).abs())).copied()
    };
    pick(&mut on_neck.clone()).or_else(|| pick(&mut all.iter())).expect("there is always a shape")
}

#[cfg(test)]
#[path = "neck_tests.rs"]
mod tests;
