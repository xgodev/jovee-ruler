//! The JoVee Ruler window. `Ruler` owns it and fills it from `jr-core`; standalone it shows the
//! harmonic field and the scale, and a host (JoVee Jam) hands it a song to follow.

use std::cell::RefCell;
use std::rc::Rc;

use jr_core::note::fifths;
use jr_core::Pc;
use slint::{ComponentHandle, ModelRc, VecModel};

pub mod ui {
    slint::include_modules!();
}
pub mod view;

use ui::AppWindow;

/// One chord of a song: its name as the song writes it, the notes sounding and the fret the hand
/// is around (for the pentatonic shape).
#[derive(Clone, Debug, PartialEq)]
pub struct SongChord {
    pub name: String,
    pub notes: Vec<Pc>,
    pub fret: f32,
}

/// A song as JoVee Jam hands it over: its title, its key and its chords in order.
#[derive(Clone, Debug, PartialEq)]
pub struct Song {
    pub title: String,
    pub key: Pc,
    pub chords: Vec<SongChord>,
}

#[derive(Default)]
struct State {
    key: Pc,
    shape: usize,
    song: Option<Song>,
    now: usize,
}

pub struct Ruler {
    window: AppWindow,
    state: Rc<RefCell<State>>,
}

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

impl Ruler {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let window = AppWindow::new()?;
        let state = Rc::new(RefCell::new(State::default()));
        window.set_keys(model(view::wheel_keys()));

        let (w, s) = (window.as_weak(), state.clone());
        window.on_key_picked(move |i| {
            let key = fifths()[i.rem_euclid(12) as usize];
            s.borrow_mut().key = key;
            if let Some(w) = w.upgrade() {
                show_key(&w, &s.borrow());
            }
        });
        let (w, s) = (window.as_weak(), state.clone());
        window.on_shape_picked(move |i| {
            s.borrow_mut().shape = i.max(0) as usize;
            if let Some(w) = w.upgrade() {
                show_key(&w, &s.borrow());
            }
        });
        let (w, s) = (window.as_weak(), state.clone());
        window.on_seek(move |i| {
            s.borrow_mut().now = i.max(0) as usize;
            if let Some(w) = w.upgrade() {
                show_song(&w, &s.borrow());
            }
        });

        show_key(&window, &state.borrow());
        Ok(Ruler { window, state })
    }

    pub fn window(&self) -> &AppWindow {
        &self.window
    }

    /// The scheme the ruler is drawn in; a host passes its own so the ruler matches it.
    pub fn set_theme(&self, mode: ui::ThemeMode) {
        self.window.global::<ui::Theme>().set_mode(mode);
    }

    /// Turns the wheel and the scale to a major key.
    pub fn set_key(&self, key: Pc) {
        self.state.borrow_mut().key = key % 12;
        show_key(&self.window, &self.state.borrow());
    }

    /// Opens the song page on the first chord. The harmonic field and the scale follow its key.
    pub fn follow_song(&self, song: Song) {
        {
            let mut s = self.state.borrow_mut();
            s.key = song.key % 12;
            s.song = Some(song);
            s.now = 0;
        }
        let s = self.state.borrow();
        show_key(&self.window, &s);
        show_song(&self.window, &s);
        self.window.set_page(2);
    }

    /// The song moved on to chord `index`.
    pub fn at(&self, index: usize) {
        self.state.borrow_mut().now = index;
        show_song(&self.window, &self.state.borrow());
    }
}

fn show_key(w: &AppWindow, s: &State) {
    let key = s.key;
    w.set_key_index(view::wheel_index(key) as i32);
    w.set_key_title(view::key_title(key).into());
    w.set_key_subtitle(view::key_subtitle(key).into());
    w.set_key_short(jr_core::note::key_name(key).into());
    w.set_relative_short(view::relative_short(key).into());
    w.set_chords(model(view::chord_cards(key)));

    let buttons = view::shape_buttons(key);
    let pick = buttons[s.shape.min(buttons.len() - 1)].clone();
    let window = (pick.first >= 0).then_some((pick.first, pick.last));
    w.set_shape(s.shape.min(buttons.len() - 1) as i32);
    w.set_shape_subtitle(view::shape_subtitle(key, &pick).into());
    w.set_scale_notes(model(view::neck(key, window, &[])));
    w.set_shape_segments(model(view::shape_segments(&buttons)));
    w.set_shapes(model(buttons));
}

fn show_song(w: &AppWindow, s: &State) {
    let Some(song) = &s.song else { return };
    let Some(chord) = song.chords.get(s.now.min(song.chords.len().saturating_sub(1))) else { return };
    let j = view::jam(song.key, Some(&chord.name), &chord.notes, chord.fret);
    w.set_context(format!("{} · {}", song.title, view::key_title(song.key)).into());
    w.set_timeline(model(song.chords.iter().map(|c| c.name.clone().into()).collect()));
    w.set_now(s.now as i32);
    w.set_moment(j.moment);
    w.set_jam_field(model(j.field));
    w.set_jam_notes(model(j.notes));
    w.set_jam_first(j.window.first.max(0));
    w.set_jam_last(j.window.last);
}

#[cfg(test)]
#[path = "window_tests.rs"]
mod window_tests;
