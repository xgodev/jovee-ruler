use super::*;

fn note(start: f64, string: u8, fret: u8) -> SongNote {
    SongNote { start, end: start + 0.4, string, fret, midi: 60 }
}

#[test]
fn notes_that_start_together_are_one_moment() {
    // given out of order, as a host may: the chord's notes a few milliseconds apart
    let m = moments(&[note(0.5, 1, 3), note(0.0, 6, 3), note(0.02, 5, 2), note(1.0, 2, 3)]);
    assert_eq!(m.iter().map(|m| m.notes.len()).collect::<Vec<_>>(), [2, 1, 1]);
    assert_eq!(m.iter().map(|m| m.start).collect::<Vec<_>>(), [0.0, 0.5, 1.0]);
    assert!(m[0].is_chord() && !m[1].is_chord());
}

#[test]
fn a_moment_is_measured_from_its_first_note() {
    // three notes 20 ms apart: the third is 40 ms after the first, so it starts the next moment
    let m = moments(&[note(0.0, 6, 3), note(0.02, 5, 2), note(0.04, 4, 0)]);
    assert_eq!(m.iter().map(|m| m.notes.len()).collect::<Vec<_>>(), [2, 1]);
}

#[test]
fn the_moment_sounding_is_the_last_one_started() {
    let m = moments(&[note(1.0, 1, 3), note(2.0, 1, 5), note(3.0, 1, 7)]);
    assert_eq!(moment_at(&m, 0.5), None, "nothing played yet");
    assert_eq!(moment_at(&m, 1.0), Some(0));
    assert_eq!(moment_at(&m, 2.9), Some(1), "a rest keeps the last note");
    assert_eq!(moment_at(&m, 99.0), Some(2));
    assert_eq!(moment_at(&[], 1.0), None);
}

#[test]
fn the_hand_is_where_the_fretted_notes_are() {
    let m = moments(&[note(0.0, 6, 3), note(0.0, 1, 5), note(1.0, 1, 0), note(2.0, 2, 8)]);
    assert_eq!(hand_fret(&m, Some(0)), Some(4.0), "the average of the frets held");
    assert_eq!(hand_fret(&m, Some(1)), Some(4.0), "an open string leaves the hand where it was");
    assert_eq!(hand_fret(&m, Some(2)), Some(8.0));
    assert_eq!(hand_fret(&m, None), Some(4.0), "before the first note the hand waits on it");
    let open = moments(&[note(0.0, 6, 0)]);
    assert_eq!(hand_fret(&open, Some(0)), None);
}

#[test]
fn the_song_says_how_many_frets_it_needs() {
    let song = |frets: &[u8]| Song {
        notes: frets.iter().enumerate().map(|(i, &f)| note(i as f64, 1, f)).collect(),
        ..Song::default()
    };
    assert_eq!(song(&[3, 5]).frets(), 15, "never fewer than the printed ruler");
    assert_eq!(song(&[3, 19]).frets(), 19);
    assert_eq!(song(&[30]).frets(), 24, "no guitar has more");
}

#[test]
fn the_tuning_reads_from_the_6th_string() {
    let standard = Song { tuning: vec![64, 59, 55, 50, 45, 40], ..Song::default() };
    assert_eq!(standard.open(), [4, 9, 2, 7, 11, 4]);
    let drop_d = Song { tuning: vec![64, 59, 55, 50, 45, 38], ..Song::default() };
    assert_eq!(drop_d.open(), [2, 9, 2, 7, 11, 4]);
    assert_eq!(Song::default().open(), jr_core::STANDARD, "a song with no tuning is in standard");
}
