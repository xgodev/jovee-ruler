//! A song as a host hands it over: one guitar of it, note by note.

use jr_core::note::pc;
use jr_core::{Pc, STANDARD};

/// The neck the printed ruler shows; a song that goes higher gets a longer one.
const MIN_FRETS: i32 = 15;
/// No guitar has more.
const MAX_FRETS: i32 = 24;
/// Notes that start this close together are struck at once: a chord, or a strum.
const TOGETHER: f64 = 0.03;

/// One note of the guitar followed.
#[derive(Clone, Debug, PartialEq)]
pub struct SongNote {
    /// When it is struck, in seconds of the song.
    pub start: f64,
    pub end: f64,
    /// 1 = the highest string.
    pub string: u8,
    /// 0 = open.
    pub fret: u8,
    pub midi: u8,
}

/// A chord the song writes over its notes, from `start` on.
#[derive(Clone, Debug, PartialEq)]
pub struct SongChord {
    pub start: f64,
    pub name: String,
}

/// A song as JoVee Jam hands it over: its key and one of its guitars, note by note.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Song {
    pub title: String,
    pub key: Pc,
    /// The open strings as MIDI pitches, string 1 (the highest) first; empty is standard tuning.
    pub tuning: Vec<u8>,
    /// Every guitar of the song, by name.
    pub guitars: Vec<String>,
    /// The one whose notes these are.
    pub guitar: usize,
    /// The fret the capo clamps, 0 = none: the notes' frets count from it, as the tab writes them.
    pub capo: u8,
    pub notes: Vec<SongNote>,
    pub chords: Vec<SongChord>,
}

impl Song {
    /// The frets the neck shows: the ruler's fifteen, or up to the highest fret the guitar plays.
    pub fn frets(&self) -> i32 {
        let highest = self.notes.iter().map(|n| n.fret as i32).max().unwrap_or(0);
        highest.clamp(MIN_FRETS, MAX_FRETS)
    }

    /// The open strings' notes, lowest string first.
    pub fn open(&self) -> Vec<Pc> {
        if self.tuning.is_empty() {
            return STANDARD.to_vec();
        }
        self.tuning.iter().rev().map(|&m| pc(m as i32)).collect()
    }

    /// What the strings sound at the capo, lowest string first: the open strings when there is none.
    pub fn at_capo(&self) -> Vec<Pc> {
        self.open().iter().map(|&p| pc(p as i32 + self.capo as i32)).collect()
    }
}

/// The notes struck at one time: one note, or a chord.
#[derive(Clone, Debug, PartialEq)]
pub struct Moment {
    pub start: f64,
    pub notes: Vec<SongNote>,
}

impl Moment {
    pub fn is_chord(&self) -> bool {
        self.notes.len() > 1
    }
}

/// The notes in the order they are struck, those struck together as one moment. A moment holds the
/// notes that start within 30 ms of its first one.
pub fn moments(notes: &[SongNote]) -> Vec<Moment> {
    let mut sorted = notes.to_vec();
    sorted.sort_by(|a, b| a.start.total_cmp(&b.start));
    let mut out: Vec<Moment> = Vec::new();
    for n in sorted {
        match out.last_mut() {
            Some(m) if n.start - m.start < TOGETHER => m.notes.push(n),
            _ => out.push(Moment { start: n.start, notes: vec![n] }),
        }
    }
    out
}

/// The moment sounding at `seconds`: the last one struck by then. A rest keeps it; nothing has
/// sounded before the first.
pub fn moment_at(moments: &[Moment], seconds: f64) -> Option<usize> {
    moments.partition_point(|m| m.start <= seconds).checked_sub(1)
}

fn fretted(m: &Moment) -> Option<f32> {
    let frets: Vec<f32> = m.notes.iter().filter(|n| n.fret > 0).map(|n| n.fret as f32).collect();
    (!frets.is_empty()).then(|| frets.iter().sum::<f32>() / frets.len() as f32)
}

/// Where the hand is at moment `now`: the average fret it holds down in the last moment that holds
/// any. Before that (or before the first note) it waits on the first one it will hold; open strings
/// only, and it is nowhere.
pub fn hand_fret(moments: &[Moment], now: Option<usize>) -> Option<f32> {
    let upto = now.map_or(0, |i| (i + 1).min(moments.len()));
    moments[..upto].iter().rev().find_map(fretted).or_else(|| moments[upto..].iter().find_map(fretted))
}

#[cfg(test)]
#[path = "song_tests.rs"]
mod tests;
