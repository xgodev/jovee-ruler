//! The scale ruler: what each note of the neck is in a key, and the five shapes of the
//! pentatonic as boxes of frets.

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

/// One shape: a box of frets, both ends included. Neighbouring boxes share their edge
/// fret, which is how the shapes join up on the neck.
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

/// The box edges in frets from the tonic: `SHAPES` in `scale/art_scale.py`.
const EDGES: [i32; 6] = [0, 2, 4, 7, 9, 12];

/// Every shape box that touches the neck from the nut to `frets`, in order along the neck.
pub fn shapes(lowest: Pc, key: Pc, frets: i32) -> Vec<Shape> {
    let tonic = tonic_fret(lowest, key) as i32;
    let mut out = Vec::new();
    for octave in -1..=(frets / 12 + 1) {
        for n in 0..5 {
            let first = tonic + 12 * octave + EDGES[n];
            let last = tonic + 12 * octave + EDGES[n + 1];
            if last >= 0 && first <= frets {
                out.push(Shape {
                    major: n as u8 + 1,
                    minor: ((n + 1) % 5) as u8 + 1,
                    first,
                    last,
                });
            }
        }
    }
    out
}

/// The shape whose middle is nearest a fret: where the hand is, the box it is playing in.
/// A box that runs past the nut is left out unless it is the only one there.
pub fn shape_near(lowest: Pc, key: Pc, fret: f32) -> Shape {
    let all = shapes(lowest, key, fret.ceil().max(0.0) as i32 + 12);
    let on_neck = all.iter().filter(|s| s.first >= 0);
    let pick = |it: &mut dyn Iterator<Item = &Shape>| {
        it.min_by(|a, b| {
            (a.middle() - fret)
                .abs()
                .total_cmp(&(b.middle() - fret).abs())
        })
        .copied()
    };
    pick(&mut on_neck.clone())
        .or_else(|| pick(&mut all.iter()))
        .expect("there is always a shape")
}

#[cfg(test)]
#[path = "neck_tests.rs"]
mod tests;
