//! The real window, filled by the real `Ruler`, drawn by Slint's software renderer to one PNG per page:
//! `cargo run -p jr-gui --example render -- <out dir> [page…]`.

use std::path::Path;
use std::rc::Rc;

use jr_core::note::{fifths, parse};
use jr_gui::{Ruler, Song, SongChord};
use resvg::tiny_skia::Pixmap;
use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType, Rgb565Pixel};
use slint::platform::{Platform, PlatformError, WindowAdapter};
use slint::{ComponentHandle, PhysicalSize};

const PAGES: [&str; 5] = ["field", "field-g", "scale", "jam", "jam-borrowed"];
const SIZE: (u32, u32) = (1280, 860);

struct Headless(Rc<MinimalSoftwareWindow>);

impl Platform for Headless {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.0.clone())
    }
}

fn save(window: &MinimalSoftwareWindow, out: &Path) {
    let (w, h) = SIZE;
    let mut buffer = vec![Rgb565Pixel(0); (w * h) as usize];
    for _ in 0..3 {
        slint::platform::update_timers_and_animations();
        window.request_redraw();
        window.draw_if_needed(|r| {
            r.render(&mut buffer, w as usize);
        });
    }
    let mut pixmap = Pixmap::new(w, h).unwrap();
    for (px, rgba) in buffer.iter().zip(pixmap.data_mut().chunks_exact_mut(4)) {
        let (r, g, b) = ((px.0 >> 11) & 0x1f, (px.0 >> 5) & 0x3f, px.0 & 0x1f);
        rgba.copy_from_slice(&[(r * 255 / 31) as u8, (g * 255 / 63) as u8, (b * 255 / 31) as u8, 255]);
    }
    std::fs::write(out, pixmap.encode_png().unwrap()).unwrap();
    eprintln!("{}", out.display());
}

/// A song in G with a borrowed chord: G C D Em F C, the hand around the 3rd fret.
fn song() -> Song {
    let chord = |name: &str, notes: &str| SongChord {
        name: name.into(),
        notes: notes.split_whitespace().map(|n| parse(n).unwrap()).collect(),
        fret: 3.0,
    };
    Song {
        title: "Exemplo em Sol".into(),
        key: parse("G").unwrap(),
        chords: vec![
            chord("G", "G B D"),
            chord("C", "C E G"),
            chord("D", "D F# A"),
            chord("Em", "E G B"),
            chord("F", "F A C"),
            chord("C", "C E G"),
        ],
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let out = Path::new(args.first().map_or("target/render", String::as_str)).to_path_buf();
    let pages: Vec<&str> = if args.len() > 1 { args[1..].iter().map(String::as_str).collect() } else { PAGES.to_vec() };
    std::fs::create_dir_all(&out).unwrap();
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(Headless(window.clone()))).unwrap();
    for page in pages {
        let ruler = Ruler::new().unwrap();
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
            "jam" => {
                ruler.follow_song(song());
                ruler.at(1);
            }
            "jam-borrowed" => {
                ruler.follow_song(song());
                ruler.at(4);
            }
            other => panic!("no page {other}"),
        }
        save(&window, &out.join(format!("{page}.png")));
        ruler.window().hide().unwrap();
    }
}
