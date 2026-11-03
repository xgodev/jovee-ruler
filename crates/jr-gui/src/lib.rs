//! The JoVee Ruler window. `Ruler` owns it and fills it from `jr-core`; standalone it shows the
//! harmonic field and the scale, and a host (JoVee Jam) hands it a song to follow.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use jr_core::note::fifths;
use jr_core::{Pc, STANDARD};
use slint::{ComponentHandle, ModelRc, VecModel};

pub mod ui {
    slint::include_modules!();
}
mod choices;
pub mod language;
pub mod song;
pub mod view;

pub use jovee_theme::ThemeId;
use ui::AppWindow;

use song::Moment;
pub use song::{Song, SongChord, SongNote};
use view::Hand;

#[derive(Default)]
struct State {
    key: Pc,
    shape: usize,
    song: Option<Song>,
    moments: Vec<Moment>,
    /// The moment sounding; `None` before the first.
    now: Option<usize>,
    pentatonic: bool,
    hand: Hand,
    /// The song's neck flipped: the lowest string on top, as on the printed ruler.
    low_on_top: bool,
    /// Where the user's choices are kept; `None` keeps nothing.
    choices: Option<PathBuf>,
    on_seek: Option<Rc<dyn Fn(f64)>>,
    on_guitar: Option<Rc<dyn Fn(usize)>>,
}

pub struct Ruler {
    window: AppWindow,
    state: Rc<RefCell<State>>,
}

fn model<T: Clone + 'static>(rows: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(rows))
}

/// Redraws the song page after `change` alters the state.
fn redraw(w: &slint::Weak<AppWindow>, s: &RefCell<State>, change: impl FnOnce(&mut State)) {
    change(&mut s.borrow_mut());
    if let Some(w) = w.upgrade() {
        show_song(&w, &s.borrow());
    }
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
            let Some(last) = s.borrow().moments.len().checked_sub(1) else { return };
            let index = (i.max(0) as usize).min(last);
            redraw(&w, &s, |st| st.now = Some(index));
            // Taken out of the state first: the host may answer by calling back into the ruler.
            let (host, start) = {
                let st = s.borrow();
                (st.on_seek.clone(), st.moments[index].start)
            };
            if let Some(host) = host {
                host(start);
            }
        });
        let (w, s) = (window.as_weak(), state.clone());
        window.on_guitar_picked(move |i| {
            if let Some(w) = w.upgrade() {
                w.set_guitar(i);
            }
            let host = s.borrow().on_guitar.clone();
            if let Some(host) = host {
                host(i.max(0) as usize);
            }
        });
        let (w, s) = (window.as_weak(), state.clone());
        window.on_scale_picked(move |i| redraw(&w, &s, |st| st.pentatonic = i == 1));
        let (w, s) = (window.as_weak(), state.clone());
        window.on_hand_picked(move |i| redraw(&w, &s, |st| st.hand = Hand::from_index(i)));
        let (w, s) = (window.as_weak(), state.clone());
        window.on_strings_flipped(move || {
            redraw(&w, &s, |st| st.low_on_top = !st.low_on_top);
            let st = s.borrow();
            if let Some(root) = &st.choices {
                // the flip stays on screen; only the next opening forgets it
                if let Err(error) = choices::save(root, st.low_on_top) {
                    eprintln!("JoVee Ruler: could not keep the strings flipped: {error}");
                }
            }
        });

        show_key(&window, &state.borrow());
        Ok(Ruler { window, state })
    }

    pub fn window(&self) -> &AppWindow {
        &self.window
    }

    /// The look the ruler is drawn in. A host passes its own when it opens the ruler and again when the
    /// user switches; the change shows at once.
    pub fn set_theme(&self, look: ThemeId) {
        self.window.global::<ui::Theme>().set_index(look.index());
    }

    /// Sets the ruler's texts in the face that has the glyphs of `language` (a code of
    /// `language::LANGUAGES`); a Latin language keeps the JoVee face. A host calls it with the language it
    /// selected, since Slint keeps one language for every window but each window keeps its own face.
    pub fn set_font_for(&self, language: &str) {
        let locale = self.window.global::<ui::Locale>();
        let face = language::font_for(language).map_or_else(|| self.window.global::<ui::Theme>().get_sans(), Into::into);
        locale.set_font_family(face);
    }

    /// Keeps the user's choices under `root`, normally `jovee_theme::jovee_root()`, and shows the ones
    /// kept there: today which string the song page draws on top. Without it the highest is on top and a
    /// flip is forgotten when the ruler closes.
    pub fn keep_choices_in(&self, root: &Path) {
        {
            let mut s = self.state.borrow_mut();
            s.low_on_top = choices::low_on_top(root);
            s.choices = Some(root.to_path_buf());
        }
        show_song(&self.window, &self.state.borrow());
    }

    /// Turns the wheel and the scale to a major key.
    pub fn set_key(&self, key: Pc) {
        self.state.borrow_mut().key = key % 12;
        show_key(&self.window, &self.state.borrow());
    }

    /// Opens the song page on one guitar of a song, before its first note. The harmonic field and the
    /// scale follow its key; the scale and the shape picked stay from the guitar before.
    pub fn follow_song(&self, song: Song) {
        let title = song.title.clone();
        {
            let mut s = self.state.borrow_mut();
            s.key = song.key % 12;
            s.moments = song::moments(&song.notes);
            s.song = Some(song);
            s.now = None;
        }
        let s = self.state.borrow();
        show_key(&self.window, &s);
        show_song(&self.window, &s);
        self.window.set_page(2);
        self.window.set_context(title.into());
    }

    /// The user clicked a note of the phrase: the ruler already shows it, and the host takes the song
    /// to its start, in seconds. A closure that keeps the `Ruler` should hold it weakly, since the ruler
    /// keeps the closure.
    pub fn on_seek(&self, host: impl Fn(f64) + 'static) {
        self.state.borrow_mut().on_seek = Some(Rc::new(host));
    }

    /// The user picked guitar `index` of the song: the host answers with `follow_song` on its notes.
    pub fn on_guitar(&self, host: impl Fn(usize) + 'static) {
        self.state.borrow_mut().on_guitar = Some(Rc::new(host));
    }

    /// The song is at `seconds`. The page is redrawn only when another moment sounds.
    pub fn at(&self, seconds: f64) {
        let now = song::moment_at(&self.state.borrow().moments, seconds);
        if now == self.state.borrow().now {
            return;
        }
        self.state.borrow_mut().now = now;
        show_song(&self.window, &self.state.borrow());
    }
}

fn show_key(w: &AppWindow, s: &State) {
    let key = s.key;
    w.set_key_index(view::wheel_index(key) as i32);
    w.set_key_short(jr_core::note::key_name(key).into());
    w.set_relative_name(view::relative_name(key).into());
    w.set_relative_short(view::relative_short(key).into());
    w.set_chords(model(view::chord_cards(key)));

    let buttons = view::shape_buttons(key);
    let pick = buttons[s.shape.min(buttons.len() - 1)].clone();
    let window = (pick.first >= 0).then_some((pick.first, pick.last));
    w.set_shape(s.shape.min(buttons.len() - 1) as i32);
    w.set_scale_notes(model(view::neck(key, &STANDARD, view::FRETS, window, false, &[], &[])));
    w.set_shapes(model(buttons));
}

fn show_song(w: &AppWindow, s: &State) {
    let Some(song) = &s.song else { return };
    let page = view::song_page(song, &s.moments, s.now, s.pentatonic, s.hand);
    w.set_guitars(model(song.guitars.iter().map(|g| g.into()).collect()));
    w.set_guitar(song.guitar as i32);
    w.set_scale(s.pentatonic as i32);
    w.set_hand(s.hand.index());
    w.set_high_on_top(!s.low_on_top);
    w.set_tuning(page.tuning.into());
    w.set_song_shape(page.shape);
    w.set_song_notes(model(page.notes));
    w.set_open_names(model(page.open_names.into_iter().map(Into::into).collect()));
    w.set_song_frets(page.frets);
    let (first, last) = page.window.unwrap_or((-1, -1));
    w.set_song_first(first);
    w.set_song_last(last);
    w.set_now_note(page.now);
    w.set_phrase(model(page.phrase));
}

#[cfg(test)]
#[path = "window_tests.rs"]
mod window_tests;
