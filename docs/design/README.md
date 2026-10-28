# JoVee Ruler — design

JoVee Ruler is the two music-theory tools of this repo — the **harmonic field wheel** and the
**scale ruler** — in two forms:

- **Printed**: the 3D-printed pieces with paper art (`harmonic-field/`, `scale/`). Unchanged.
- **Digital**: the same tools as interactive screens. One codebase used in three ways.

## Three ways to run the digital ruler

| where | how |
|---|---|
| **Standalone** (`jovee-ruler` app) | Its own window. Needs nothing else running. |
| **Inside JoVee Rig** | JoVee Rig imports the JoVee Ruler crates from GitHub and opens the ruler from its main screen and from the Jam screen. Those buttons are JoVee Rig's work; this repo only provides the screens. |
| **Inside JoVee Studio** | Same as inside JoVee Rig: imported as a library. |

Same pattern as JoVee Jam and JoVee Tone (library inside the host, its own app outside).

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

## Inside JoVee Jam: the harmonic field of the song

The ruler is embedded in JoVee Jam's song screen. Jam gives it two things: the **key** of the song
and the **notes** being played (from the tab/score, as the song advances). With those, the ruler says,
in real time:

- **Which chord of the harmonic field this is**: the notes of the current moment are matched against
  the seven chords of the key, and the ruler shows the degree (e.g. `IV` in C = F), its function
  (rest, transition, tension) and lights it on the wheel. A chord that is not in the field (borrowed,
  secondary dominant) is shown as such, not forced onto the nearest degree.
- **Which pentatonic fits**: the same input places the pentatonic of the key on the neck, with the
  CAGED shape nearest to where the song is being played, and marks which notes of the moment are
  inside it.

Example: song in G, the tab plays G–B–D, then C–E–G, then D–F#–A: the wheel lights `I`, `IV`, `V`
in turn, and the neck keeps the G major pentatonic in the shape around the 3rd fret.

The matching (notes + key → degree, notes + key → pentatonic shape) lives in `jr-core`, so it is the
same in the standalone app (where the user types the key and plays the notes) and inside Jam.

## Boundary

- One core crate holds the theory: notes, intervals, scales, harmonic field, CAGED shapes, tunings.
  It has no UI and no host types. The printed art can later be generated from it too, which ends the
  duplicated `CW`/`RS`/`TUNING` between `scale/scale.scad` and `scale/art_scale.py`.
- Screens are Slint, with the JoVee theme, fonts and layout, so they look native inside JoVee Rig and
  JoVee Studio.
- Talking to a host (song key, tuner note, playing a note) goes through one trait. Standalone has a
  version that does nothing but play notes on its own output; nothing in the ruler depends on a host
  being there.

## Layout

- `crates/jr-core`: the theory, with its tests.
- `crates/jr-gui`: the Slint screens, `Ruler` (the window plus the code that fills it, which a host
  embeds) and the `jovee-ruler` binary, the standalone app. A host follows a song with
  `Ruler::follow_song` and moves it on with `Ruler::at`.
- `harmonic-field/`, `scale/`: the printed pieces, as today.

Where the mockup and the printed art disagree, the screens follow the printed art: roman numerals
always uppercase, and the shapes numbered 1–5 with the 3- and 4-fret boxes of the slider.

## Order

1. Mockup of the screens, approved before any Slint: [`mockup/index.html`](mockup/index.html).
2. `jr-core` with tests, checked against the printed art (same notes, same chords, same shapes).
3. `jr-gui` + the standalone app. Done; the song page is in place for step 5.
4. JoVee Rig imports it.
5. Embedded in JoVee Jam: key + notes of the song → degree of the harmonic field and pentatonic.
6. JoVee Studio imports it; the tuner link.
