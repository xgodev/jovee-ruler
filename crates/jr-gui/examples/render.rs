//! The real window, filled by the real `Ruler`, drawn by Slint's software renderer to one PNG per page
//! and look (`field-neon.png`, `field-rack.png`, `field-classic.png`, …):
//! `cargo run -p jr-gui --example render -- <out dir> [page…] [--language ja-JP]`. English unless a language is
//! given, then its code ends each file name (`field-neon-ja-JP.png`).

use std::path::Path;
use std::rc::Rc;

use jr_core::note::{fifths, parse};
use jr_gui::{Ruler, Song, SongChord, SongNote, ThemeId};
use resvg::tiny_skia::Pixmap;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, PlatformError, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize, Rgb8Pixel};

const PAGES: [&str; 8] =
    ["field", "field-g", "scale", "song-solo", "song-flipped", "song-base", "song-seven", "song-flat"];
const SIZE: (u32, u32) = (1280, 860);

struct Headless(Rc<MinimalSoftwareWindow>);

impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.0.clone())
    }
}

fn save(window: &MinimalSoftwareWindow, out: &Path) {
    let (w, h) = SIZE;
    let mut buffer = vec![Rgb8Pixel { r: 0, g: 0, b: 0 }; (w * h) as usize];
    for _ in 0..3 {
        slint::platform::update_timers_and_animations();
        window.request_redraw();
        window.draw_if_needed(|r| {
            r.render(&mut buffer, w as usize);
        });
    }
    let mut pixmap = Pixmap::new(w, h).unwrap();
    for (px, rgba) in buffer.iter().zip(pixmap.data_mut().chunks_exact_mut(4)) {
        rgba.copy_from_slice(&[px.r, px.g, px.b, 255]);
    }
    std::fs::write(out, pixmap.encode_png().unwrap()).unwrap();
    eprintln!("{}", out.display());
}

/// The open strings of a standard guitar, the 1st first, as a tab gives them.
const STANDARD: [u8; 6] = [64, 59, 55, 50, 45, 40];

/// The notes struck together at `start`, as (string, fret).
fn struck(tuning: &[u8], start: f64, at: &[(u8, u8)]) -> Vec<SongNote> {
    at.iter()
        .map(|&(string, fret)| SongNote {
            start,
            end: start + 0.9,
            string,
            fret,
            midi: tuning[string as usize - 1] + fret,
        })
        .collect()
}

fn song(guitar: usize, tuning: &[u8], moments: &[&[(u8, u8)]], chords: &[&str]) -> Song {
    Song {
        title: "Example in G".into(),
        key: parse("G").unwrap(),
        tuning: tuning.to_vec(),
        guitars: vec!["Base".into(), "Solo".into()],
        guitar,
        notes: moments.iter().enumerate().flat_map(|(i, at)| struck(tuning, i as f64, at)).collect(),
        chords: chords
            .iter()
            .enumerate()
            .map(|(i, name)| SongChord { start: i as f64, name: (*name).into() })
            .collect(),
    }
}

/// The solo in G pentatonic, a Bb bent out of the scale first; the hand in shape 1 at the 3rd fret.
fn solo() -> Song {
    song(1, &STANDARD, &[&[(3, 3)], &[(3, 4)], &[(2, 3)], &[(2, 5)], &[(2, 3)], &[(3, 4)], &[(4, 5)]], &[])
}

/// The rhythm guitar: barre chords around the 3rd fret.
fn base() -> Song {
    let em: &[(u8, u8)] = &[(6, 0), (5, 2), (4, 2), (3, 0), (2, 0), (1, 0)];
    let g: &[(u8, u8)] = &[(6, 3), (5, 5), (4, 5), (3, 4), (2, 3), (1, 3)];
    let c: &[(u8, u8)] = &[(5, 3), (4, 5), (3, 5), (2, 5), (1, 3)];
    let d: &[(u8, u8)] = &[(5, 5), (4, 7), (3, 7), (2, 7), (1, 5)];
    song(0, &STANDARD, &[em, g, c, d, g, em, c], &["Em", "G", "C", "D", "G", "Em", "C"])
}

/// The solo a half step up, in Ab: a flat in the title, "Ab major · pentatonic".
fn flat() -> Song {
    let up: &[&[(u8, u8)]] = &[&[(3, 4)], &[(3, 5)], &[(2, 4)], &[(2, 6)], &[(2, 4)], &[(3, 5)], &[(4, 6)]];
    Song { key: parse("Ab").unwrap(), ..song(1, &STANDARD, up, &[]) }
}

/// A seven-string, the low B under the E, the riff climbing past the 15th fret.
fn seven() -> Song {
    let tuning = [64, 59, 55, 50, 45, 40, 35];
    song(1, &tuning, &[&[(7, 3)], &[(6, 3)], &[(5, 5)], &[(2, 15)], &[(1, 17)], &[(1, 19)]], &[])
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let language = args.iter().position(|a| a == "--language").map(|at| {
        let code = args.get(at + 1).cloned().expect("--language takes a code, as ja-JP");
        args.drain(at..at + 2);
        code
    });
    let out = Path::new(args.first().map_or("target/render", String::as_str)).to_path_buf();
    let pages: Vec<&str> = if args.len() > 1 { args[1..].iter().map(String::as_str).collect() } else { PAGES.to_vec() };
    std::fs::create_dir_all(&out).unwrap();
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(window.clone()))).unwrap();
    for (page, look) in pages.iter().flat_map(|p| ThemeId::ALL.map(|look| (*p, look))) {
        let ruler = Ruler::new().unwrap();
        jr_gui::language::select(&ruler, language.as_deref().unwrap_or("en-US")).unwrap();
        ruler.set_theme(look);
        ruler.window().show().unwrap();
        window.set_size(PhysicalSize::new(SIZE.0, SIZE.1));
        let g = fifths().iter().position(|&k| k == parse("G").unwrap()).unwrap() as i32;
        match page {
            "field" => {}
            "field-g" => ruler.window().invoke_key_picked(g),
            "scale" => {
                ruler.window().invoke_key_picked(g);
                ruler.window().set_page(1);
            }
            "song-solo" => {
                ruler.follow_song(solo());
                ruler.window().invoke_scale_picked(1);
                ruler.at(2.0);
            }
            // the strings flipped by the key beside the tuning: the lowest on top
            "song-flipped" => {
                ruler.follow_song(solo());
                ruler.window().invoke_scale_picked(1);
                ruler.window().invoke_strings_flipped();
                ruler.at(2.0);
            }
            "song-base" => {
                ruler.follow_song(base());
                ruler.at(1.0);
            }
            "song-seven" => {
                ruler.follow_song(seven());
                ruler.at(3.0);
            }
            "song-flat" => {
                ruler.follow_song(flat());
                ruler.window().invoke_scale_picked(1);
                ruler.at(2.0);
            }
            "song-start" => ruler.follow_song(solo()),
            // the "Now" box at its tightest, a long chord name: one per step of the name's size
            tight if tight.starts_with("tight-") => {
                let mut s = base();
                s.chords[1].name = tight["tight-".len()..].into();
                ruler.follow_song(s);
                ruler.at(1.0);
            }
            other => panic!("no page {other}"),
        }
        let suffix = language.as_deref().map_or_else(String::new, |code| format!("-{code}"));
        save(&window, &out.join(format!("{page}-{}{suffix}.png", look.name())));
        ruler.window().hide().unwrap();
    }
}
