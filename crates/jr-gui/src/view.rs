//! What each screen shows, worked out from `jr-core`. Plain functions from the key (and the
//! song's notes) to the rows the window draws, so they are tested without a window.

use std::ops::Range;

use jr_core::note::{fifths, name_in, pc};
use jr_core::{harmonic_field, mark, note_at, scale, shape_near, shapes, triad, Function, Mark, Pc, Shape, STANDARD};

use crate::song::{hand_fret, Moment, Song, SongNote};
use crate::ui::{ChordCard, NoteMark, NowNote, PhraseChip, ShapeButton};

/// The neck drawn on screen: the nut to fret 15, as in the mockup.
pub const FRETS: i32 = 15;

fn relative(key: Pc) -> Pc {
    (key + 9) % 12
}

/// The key names around the wheel, in fifths from C.
pub fn wheel_keys() -> Vec<slint::SharedString> {
    fifths().iter().map(|&k| jr_core::note::key_name(k).into()).collect()
}

/// Where a key sits on the wheel.
pub fn wheel_index(key: Pc) -> usize {
    fifths().iter().position(|&k| k == key % 12).expect("every key is on the wheel")
}

/// The tonic of the relative minor, spelled in the key: "E" under G, "F" under Ab.
pub fn relative_name(key: Pc) -> &'static str {
    name_in(key, relative(key))
}

/// The relative minor as the wheel names it under the key: "Em".
pub fn relative_short(key: Pc) -> String {
    jr_core::note::minor_key_name(relative(key)).to_string()
}

fn function(f: Function) -> i32 {
    match f {
        Function::Rest => 0,
        Function::Transition => 1,
        Function::Tension => 2,
    }
}

pub fn chord_cards(key: Pc) -> Vec<ChordCard> {
    harmonic_field(key)
        .iter()
        .map(|c| ChordCard {
            roman: c.roman().into(),
            name: c.name().into(),
            minor_roman: c.minor_roman().into(),
            function: function(c.function),
        })
        .collect()
}

/// One button per shape of the major scale, 1 to 5 in the major key, each at its first box that
/// fits whole on the neck on screen; then the whole neck.
pub fn shape_buttons(key: Pc) -> Vec<ShapeButton> {
    let boxes = shapes(STANDARD[0], key, FRETS, false);
    let mut out: Vec<ShapeButton> = (1..=5)
        .map(|n| {
            let s = boxes
                .iter()
                .find(|s| s.major == n && s.first >= 0 && s.last <= FRETS)
                .expect("every shape fits on fifteen frets");
            ShapeButton { major: s.major as i32, minor: s.minor as i32, first: s.first, last: s.last }
        })
        .collect();
    out.push(ShapeButton { major: 0, minor: 0, first: -1, last: -1 });
    out
}

/// A note the song is sounding, where the neck draws it: `row` 0 is the lowest string.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Played {
    pub row: i32,
    pub fret: i32,
}

/// A note outside the key is a lowered degree (the blue third, the flat seventh), so it is spelled flat.
fn played_name(key: Pc, p: Pc) -> &'static str {
    const F: Pc = 5;
    if mark(key, p) == Mark::Outside {
        name_in(F, p)
    } else {
        name_in(key, p)
    }
}

/// The marks on a neck tuned to `open` (lowest string first), from the nut to `frets`. With the
/// pentatonic on show the 4th and 7th are outside it. Notes outside what is on show are drawn only
/// while they sound or come next; a note sounding or next is lit wherever it is.
pub fn neck(
    key: Pc,
    open: &[Pc],
    frets: i32,
    window: Option<(i32, i32)>,
    pentatonic: bool,
    sounding: &[Played],
    next: &[Played],
) -> Vec<NoteMark> {
    let mut out = Vec::new();
    for (row, &string) in open.iter().enumerate() {
        for fret in 0..=frets {
            let p = note_at(string, fret as u8);
            let m = match mark(key, p) {
                Mark::Scale if pentatonic => Mark::Outside,
                m => m,
            };
            let here = |n: &Played| n.row == row as i32 && n.fret == fret;
            let now = sounding.iter().any(here);
            let then = next.iter().any(here);
            if m == Mark::Outside && !now && !then {
                continue;
            }
            out.push(NoteMark {
                string: row as i32,
                fret,
                name: played_name(key, p).into(),
                mark: match m {
                    Mark::Tonic => 0,
                    Mark::MinorTonic => 1,
                    Mark::Pentatonic => 2,
                    Mark::Scale => 3,
                    Mark::Outside => 4,
                },
                lit: now || then || window.is_none_or(|(a, b)| fret >= a && fret <= b),
                sounding: now,
                next: then,
            });
        }
    }
    out
}

/// What the song page shows of the hand: the shape it is in, one shape, or the whole neck.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Hand {
    /// The shape nearest the frets the hand holds down.
    #[default]
    Follow,
    /// Shape 1 to 5, the box nearest the hand.
    Shape(u8),
    Whole,
}

impl Hand {
    /// From the place of its button: follow the hand, 1 to 5, the whole neck.
    pub fn from_index(i: i32) -> Hand {
        match i {
            ..=0 => Hand::Follow,
            1..=5 => Hand::Shape(i as u8),
            _ => Hand::Whole,
        }
    }

    pub fn index(self) -> i32 {
        match self {
            Hand::Follow => 0,
            Hand::Shape(n) => n as i32,
            Hand::Whole => 6,
        }
    }
}

/// How many moments the phrase strip holds at a readable width.
const PHRASE: usize = 7;
/// Of those, how many have already gone by: the player needs more of what is coming.
const PHRASE_BEHIND: usize = 2;

/// Which of a song's `len` moments the phrase shows while moment `now` sounds. It stays full at both ends.
pub fn phrase_strip(len: usize, now: Option<usize>) -> Range<usize> {
    let first = now.map_or(0, |n| n.saturating_sub(PHRASE_BEHIND)).min(len.saturating_sub(PHRASE));
    first..len.min(first + PHRASE)
}

/// The song page at moment `now` (`None` before the first note) of the song's `moments`.
#[derive(Clone, Debug, PartialEq)]
pub struct SongPage {
    /// The shape on show, 1 to 5 in the major key; 0 = the whole neck.
    pub shape: i32,
    /// The open strings from the lowest, spaced: "E A D G B E".
    pub tuning: String,
    /// The same names, one per row of the neck.
    pub open_names: Vec<String>,
    pub frets: i32,
    /// The shape on show, both frets included; `None` = the whole neck.
    pub window: Option<(i32, i32)>,
    pub notes: Vec<NoteMark>,
    pub now: NowNote,
    pub phrase: Vec<PhraseChip>,
}

/// The string whose shapes the boxes are counted on: the low E of a standard neck, found a fourth under
/// the A string so a dropped 6th string or a 7th string do not move the boxes.
fn shape_string(open: &[Pc]) -> Pc {
    match open.len() {
        n if n >= 5 => pc(open[n - 5] as i32 - 5),
        _ => open[0],
    }
}

/// The frets held down, lowest first: where a chord is played.
fn lowest_fret(notes: &[SongNote]) -> u8 {
    notes.iter().map(|n| n.fret).filter(|&f| f > 0).min().unwrap_or(0)
}

/// What a moment is called: a note's name; a chord as the song writes it there, as its triad, or
/// its notes from the bass up.
fn moment_name(song: &Song, m: &Moment) -> String {
    if !m.is_chord() {
        return played_name(song.key, pc(m.notes[0].midi as i32)).into();
    }
    let written = song.chords.iter().rev().find(|c| c.start <= m.start + 0.03);
    if let Some(c) = written {
        return c.name.clone();
    }
    let mut notes = m.notes.clone();
    notes.sort_by_key(|n| n.midi);
    let pcs: Vec<Pc> = notes.iter().map(|n| pc(n.midi as i32)).collect();
    if let Some((root, q)) = triad(&pcs) {
        return format!("{}{}", name_in(song.key, root), q.suffix());
    }
    let mut names: Vec<&str> = Vec::new();
    for p in pcs {
        let n = played_name(song.key, p);
        if !names.contains(&n) {
            names.push(n);
        }
    }
    names.join(" ")
}

/// The song page: the guitar's neck with the scale and the hand's shape, only the moment sounding lit
/// on it, and the phrase around it.
pub fn song_page(song: &Song, moments: &[Moment], now: Option<usize>, pentatonic: bool, hand: Hand) -> SongPage {
    let key = song.key;
    let open = song.open();
    let frets = song.frets();
    let on_show: Vec<Pc> = if pentatonic { jr_core::pentatonic(key).to_vec() } else { scale(key).to_vec() };
    let outside = |m: &Moment| m.notes.iter().any(|n| !on_show.contains(&pc(n.midi as i32)));

    let fret = hand_fret(moments, now);
    let base = shape_string(&open);
    let shape: Option<Shape> = match hand {
        Hand::Follow => fret.map(|f| shape_near(base, key, f, pentatonic)),
        Hand::Shape(n) => {
            let middle = |s: &Shape| (s.first + s.last) as f32 / 2.0;
            let at = fret.unwrap_or(0.0);
            shapes(base, key, frets, pentatonic)
                .into_iter()
                .filter(|s| s.major == n && (0.0..=frets as f32).contains(&middle(s)))
                .min_by(|a, b| (middle(a) - at).abs().total_cmp(&(middle(b) - at).abs()))
        }
        Hand::Whole => None,
    };
    let window = shape.map(|s| (s.first.max(0), s.last.min(frets)));

    let row = |n: &SongNote| open.len() as i32 - n.string as i32;
    let played = |i: Option<usize>| -> Vec<Played> {
        i.and_then(|i| moments.get(i))
            .into_iter()
            .flat_map(|m| &m.notes)
            .map(|n| Played { row: row(n), fret: n.fret as i32 })
            .collect()
    };
    let sounding = played(now);
    // before the first note, the first one comes next
    let next = played(now.map_or(Some(0), |i| Some(i + 1)));
    // following the song the eye goes to the note sounding and the one after it; the scale and the
    // shape stay dark
    let mut notes = neck(key, &open, frets, window, pentatonic, &sounding, &next);
    notes.iter_mut().for_each(|m| m.lit = m.sounding || m.next);

    let now_note = match now.and_then(|i| moments.get(i)) {
        None => NowNote { empty: true, ..NowNote::default() },
        Some(m) => NowNote {
            name: moment_name(song, m).into(),
            chord: m.is_chord(),
            string: m.notes[0].string as i32,
            fret: (if m.is_chord() { lowest_fret(&m.notes) } else { m.notes[0].fret }) as i32,
            outside: outside(m),
            empty: false,
        },
    };
    let phrase = phrase_strip(moments.len(), now)
        .map(|i| {
            let m = &moments[i];
            PhraseChip {
                name: moment_name(song, m).into(),
                chord: m.is_chord(),
                string: m.notes[0].string as i32,
                fret: (if m.is_chord() { lowest_fret(&m.notes) } else { m.notes[0].fret }) as i32,
                state: match now {
                    Some(n) if i < n => 0,
                    Some(n) if i == n => 1,
                    Some(n) if i == n + 1 => 2,
                    None if i == 0 => 2,
                    _ => 3,
                },
                outside: outside(m),
                index: i as i32,
            }
        })
        .collect();

    let names: Vec<String> = open.iter().map(|&p| name_in(key, p).to_string()).collect();
    SongPage {
        shape: shape.map_or(0, |s| s.major as i32),
        tuning: names.join(" "),
        notes,
        open_names: names,
        frets,
        window,
        now: now_note,
        phrase,
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
