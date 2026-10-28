//! What each screen shows, worked out from `jr-core`. Plain functions from the key (and the
//! notes sounding) to the rows the window draws, so they are tested without a window.

use jr_core::note::{fifths, name, name_in};
use jr_core::{
    harmonic_field, identify, mark, note_at, outside_pentatonic, roman_from, shape_near, shapes, triad, Function, Mark,
    Pc, Shape, STANDARD,
};

use crate::ui::{ChordCard, FieldCell, Moment, NoteMark, ShapeButton};

/// The neck drawn on screen: the nut to fret 15, as in the mockup.
pub const FRETS: i32 = 15;

const PT_SHARP: [&str; 12] = ["Dó", "Dó#", "Ré", "Ré#", "Mi", "Fá", "Fá#", "Sol", "Sol#", "Lá", "Lá#", "Si"];
const PT_FLAT: [&str; 12] = ["Dó", "Réb", "Ré", "Mib", "Mi", "Fá", "Solb", "Sol", "Láb", "Lá", "Sib", "Si"];

/// The note as it is said in Portuguese, spelled as in the key ("Sib" in F, "Fá#" in G).
pub fn spoken(key: Pc, p: Pc) -> &'static str {
    if name_in(key, p).contains('b') {
        PT_FLAT[(p % 12) as usize]
    } else {
        PT_SHARP[(p % 12) as usize]
    }
}

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

pub fn key_title(key: Pc) -> String {
    format!("{} maior", spoken(key, key))
}

pub fn key_subtitle(key: Pc) -> String {
    format!("Relativa: {} menor", spoken(key, relative(key)))
}

pub fn relative_short(key: Pc) -> String {
    format!("relativa {}", jr_core::note::minor_key_name(relative(key)))
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

/// One button per shape, 1 to 5 in the major key, each at its first box that fits whole on the
/// neck on screen; then the whole neck.
pub fn shape_buttons(key: Pc) -> Vec<ShapeButton> {
    let boxes = shapes(STANDARD[0], key, FRETS);
    let mut out: Vec<ShapeButton> = (1..=5)
        .map(|n| {
            let s = boxes
                .iter()
                .find(|s| s.major == n && s.first >= 0 && s.last <= FRETS)
                .expect("every shape fits on fifteen frets");
            ShapeButton { label: n.to_string().into(), first: s.first, last: s.last }
        })
        .collect();
    out.push(ShapeButton { label: "Braço todo".into(), first: -1, last: -1 });
    out
}

/// "Forma 1 · 2 na relativa menor · casas 3 a 5", or the whole neck.
pub fn shape_subtitle(key: Pc, button: &ShapeButton) -> String {
    let boxes = shapes(STANDARD[0], key, FRETS);
    match boxes.iter().find(|s| s.first == button.first && s.last == button.last) {
        Some(s) => format!("Forma {} · {} na relativa menor · casas {} a {}", s.major, s.minor, s.first, s.last),
        None => "Braço todo · as cinco formas".into(),
    }
}

/// The marks on the neck. Notes outside the scale are drawn only when they are sounding.
pub fn neck(key: Pc, window: Option<(i32, i32)>, sounding: &[Pc]) -> Vec<NoteMark> {
    let mut out = Vec::new();
    for (string, &open) in STANDARD.iter().enumerate() {
        for fret in 0..=FRETS {
            let p = note_at(open, fret as u8);
            let m = mark(key, p);
            let on = sounding.iter().any(|&n| n % 12 == p);
            if m == Mark::Outside && !on {
                continue;
            }
            let lit = window.is_none_or(|(a, b)| fret >= a && fret <= b);
            let in_pentatonic = matches!(m, Mark::Tonic | Mark::MinorTonic | Mark::Pentatonic);
            out.push(NoteMark {
                string: string as i32,
                fret,
                name: name_in(key, p).into(),
                mark: match m {
                    Mark::Tonic => 0,
                    Mark::MinorTonic => 1,
                    Mark::Pentatonic => 2,
                    Mark::Scale => 3,
                    Mark::Outside => 4,
                },
                lit,
                ring: match (on && lit, in_pentatonic) {
                    (false, _) => 0,
                    (true, true) => 1,
                    (true, false) => 2,
                },
            });
        }
    }
    out
}

/// One moment of a song inside JoVee Jam, as the window shows it.
pub struct JamMoment {
    pub moment: Moment,
    pub field: Vec<FieldCell>,
    pub notes: Vec<NoteMark>,
    pub window: Shape,
}

/// The chord sounding (`notes`), in the song's `key`, with the hand around `fret`. `chord` is the
/// name the song gives it; without one the triad names itself.
pub fn jam(key: Pc, chord: Option<&str>, notes: &[Pc], fret: f32) -> JamMoment {
    let found = identify(key, notes);
    let named = triad(notes);
    let chord = match (chord, named) {
        (Some(c), _) => c.to_string(),
        (None, Some((root, q))) => format!("{}{}", name_in(key, root), q.suffix()),
        (None, None) => notes.iter().map(|&n| name(n)).collect::<Vec<_>>().join(" "),
    };
    let roman = match (found, named) {
        (Some(c), _) => c.roman().to_string(),
        (None, Some((root, _))) => roman_from(key, root),
        (None, None) => "?".into(),
    };
    let window = shape_near(STANDARD[0], key, fret);
    let outside: Vec<&str> = outside_pentatonic(key, notes).iter().map(|&n| name_in(key, n)).collect();
    let moment = Moment {
        chord: chord.into(),
        roman: roman.into(),
        function: found.map_or(-1, |c| function(c.function)),
        lit: found.map_or(-1, |c| c.degree as i32 - 1),
        field_title: format!("Campo harmônico de {}", key_title(key)).into(),
        shape: format!(
            "Pentatônica de {}, forma {} (casas {} a {})",
            spoken(key, key),
            window.major,
            window.first.max(0),
            window.last
        )
        .into(),
        outside: outside.join(" e ").into(),
        outside_count: outside.len() as i32,
    };
    let field = harmonic_field(key)
        .iter()
        .map(|c| FieldCell { roman: c.roman().into(), name: c.name().into(), function: function(c.function) })
        .collect();
    JamMoment { moment, field, notes: neck(key, Some((window.first, window.last)), notes), window }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
