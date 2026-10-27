# JoVee Ruler — design

JoVee Ruler is the two music-theory tools of this repo — the **harmonic field wheel** and the
**scale ruler** — in two forms:

- **Printed**: the 3D-printed pieces with paper art (`harmonic-field/`, `scale/`). Unchanged.
- **Digital**: the same tools as interactive screens. One codebase used in three ways.

## Three ways to run the digital ruler

| where | how |
|---|---|
| **Standalone** (`jovee-ruler` app) | Its own window. Needs nothing else running. |
| **Inside JoVee Rig** | JoVee Rig imports the JoVee Ruler crates from GitHub and opens the ruler screens inside its own window. |
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
- Inside a host: follow the song. When JoVee Jam knows the key of the song that is playing, the ruler
  turns to it by itself; when the tuner hears a note, the ruler marks it on the neck.

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
- `crates/jr-gui`: the Slint screens.
- `apps/jovee-ruler`: the standalone app.
- `harmonic-field/`, `scale/`: the printed pieces, as today.

## Order

1. Mockup of both screens, approved before any Slint.
2. `jr-core` with tests, checked against the printed art (same notes, same chords, same shapes).
3. `jr-gui` + the standalone app.
4. JoVee Rig imports it.
5. JoVee Studio imports it; follow-the-song and the tuner link.
