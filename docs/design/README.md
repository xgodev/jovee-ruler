# JoVee Ruler — design

JoVee Ruler is the two music-theory tools of this repo — the **harmonic field wheel** and the
**scale ruler** — in two forms:

- **Printed**: the 3D-printed pieces with paper art (`harmonic-field/`, `scale/`). Unchanged.
- **Digital**: the same tools as interactive screens. One codebase used in three ways.

## Three ways to run the digital ruler

| where | how |
|---|---|
| **Standalone** (`jovee-ruler` app) | Its own window. Needs nothing else running. |
| **Inside JoVee Rig** | JoVee Rig imports the JoVee Ruler crates from GitHub and opens the ruler from its main screen. That button is JoVee Rig's work; this repo only provides the screens. The button on a song belongs to JoVee Jam, so it comes along wherever Jam goes. |
| **Inside JoVee Studio** | Same as inside JoVee Rig: imported as a library, with its own button on the main screen. |

Same pattern as JoVee Jam and JoVee Tone (library inside the host, its own app outside).

Every host can open the plain ruler, with no song involved: the harmonic field and the scale, the key picked by hand, exactly as in the standalone app. The song page is an extra that appears only when the ruler is opened from a song in Jam.

## What the digital version does

The digital tools keep the gestures of the printed ones, so a player who knows one knows the other:

- **Harmonic field wheel**: turn the wheel (drag, or pick the key) and the tonic lands in the middle
  window; the seven chords of the key show with their degree in the major and in the relative minor,
  colored by function (rest, transition, tension).
- **Scale ruler**: the 24-fret note map with the slider over it. Slide until the tonic lands in a hole
  and the scale and its five CAGED shapes appear. The cassette and the blade become one control:
  pick a shape and the rest of the neck goes dark.

What paper cannot do, the digital version adds:

- Any tuning and any number of strings, instead of the one printed on the track.
- Other scales and modes, not only major / relative minor / pentatonic.
- Click a chord or a note to hear it.
- Inside a host: when the tuner hears a note, the ruler marks it on the neck.

## Inside JoVee Jam: one guitar of the song

The ruler is embedded in JoVee Jam's song screen. Jam hands it one guitar of the song: its notes from
the tab (string, fret, when), its tuning, the song's key and the names of the other guitars. The page
([`mockup/neck.html`](mockup/neck.html)) shows:

- **The guitar**, picked from the song's tracks in a select; Jam answers a pick with that guitar's notes.
- **The scale**: the major scale of the key or its pentatonic, on a neck in the guitar's tuning (any
  number of strings, open notes named on the left) and as long as the guitar plays, 15 to 24 frets.
- **The shape the hand is in**: "Segue a mão" frames the CAGED box nearest the frets held down, and
  moves with the hand; 1 to 5 fixes one box near the hand; "Braço todo" frames nothing.
- **The notes as the tab plays them**: only the one sounding is lit, with a thick ring and a halo; the
  scale and the shape stay dark around it, so the eye has one place to go. A note outside the scale on
  show is drawn while it is played, in red. Notes struck within 30 ms are one moment, a chord.
- **Agora and the phrase**: what sounds now (name, where the hand plays it, inside the scale or not)
  and seven moments around it, two behind. A click on one takes the song there.

Example: song in G, the solo plays Bb–B–D on strings 3 and 2 around the 3rd fret: the neck holds the G
pentatonic with shape 1 framed at frets 3 to 5, Bb ringed in red, D ringed as the note sounding.

The matching (notes + key → degree, notes + key → pentatonic shape) lives in `jr-core`, so it is the
same in the standalone app (where the user types the key and plays the notes) and inside Jam.

## Boundary

- One core crate holds the theory: notes, intervals, scales, harmonic field, CAGED shapes, tunings.
  It has no UI and no host types. The printed art can later be generated from it too, which ends the
  duplicated `CW`/`RS`/`TUNING` between `scale/scale.scad` and `scale/art_scale.py`.
- Screens are Slint, on the shared `xgodev/jovee-theme` tokens, fonts and controls (card, header,
  segmented, caption, tag), so they look native inside every JoVee program. One layout, three looks (Neon, Rack, Classic): the standalone app takes the one
  saved for JoVee, a host picks with `Ruler::set_theme(ThemeId)`. `jovee-theme` is a private repository,
  so building the screens needs access to it.
- Talking to a host (song key, tuner note, playing a note) goes through one trait. Standalone has a
  version that does nothing but play notes on its own output; nothing in the ruler depends on a host
  being there.

## Layout

- `crates/jr-core`: the theory, with its tests.
- `crates/jr-gui`: the Slint screens, `Ruler` (the window plus the code that fills it, which a host
  embeds) and the `jovee-ruler` binary, the standalone app. A host hands one guitar over with
  `Ruler::follow_song`, moves it on with `Ruler::at(seconds)`, hears a click on the phrase through
  `Ruler::on_seek` (the start of that moment, in seconds) and a pick of another guitar through
  `Ruler::on_guitar` (its index in `Song::guitars`). The scale and the shape picked stay across guitars.
- `harmonic-field/`, `scale/`: the printed pieces, as today.

Where the mockup and the printed art disagree, the screens follow the printed art: roman numerals
always uppercase, and the shapes numbered 1–5 with the 3- and 4-fret boxes of the slider.

## Order

1. Mockup of the screens, approved before any Slint: [`mockup/index.html`](mockup/index.html). Restyled
   in JoVee Rig's visual language (its theme tokens, fonts, tool-window bar, light and dark):
   [`mockup/rig.html`](mockup/rig.html), superseded by the three JoVee looks:
   [`mockup/themes.html`](mockup/themes.html), approved and applied to the screens.
2. `jr-core` with tests, checked against the printed art (same notes, same chords, same shapes).
3. `jr-gui` + the standalone app. Done; the song page is in place for step 5.
4. JoVee Rig imports it.
5. Embedded in JoVee Jam: one guitar of the song on the neck, its scale, its shape and its notes as they play.
6. JoVee Studio imports it; the tuner link.
