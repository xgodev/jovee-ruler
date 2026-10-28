//! The theory behind JoVee Ruler, with no UI and no host types.
//!
//! The same facts the printed pieces carry: the harmonic field wheel (`field`) and the
//! scale ruler (`neck`). The numbers follow the printed art, so a player who knows the
//! physical ruler reads the digital one the same way.

pub mod field;
pub mod neck;
pub mod note;

pub use field::{harmonic_field, identify, outside_scale, Chord, Function, Quality};
pub use neck::{
    mark, note_at, outside_pentatonic, pentatonic, scale, shape_near, shapes, tonic_fret, Mark,
    Shape, STANDARD,
};
pub use note::Pc;
