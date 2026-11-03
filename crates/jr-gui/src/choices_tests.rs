use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A folder of its own under the system temp dir, empty, for one test.
pub(crate) fn scratch(name: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("jr-gui-{name}-{}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn nothing_saved_puts_the_highest_string_on_top() {
    assert!(!low_on_top(&scratch("nothing")));
}

#[test]
fn a_saved_flip_reads_back() {
    let root = scratch("flip");
    save(&root, true).unwrap();
    assert!(low_on_top(&root));
    save(&root, false).unwrap();
    assert!(!low_on_top(&root));
    assert_eq!(fs::read_to_string(root.join("ruler/config.yaml")).unwrap(), "top_string: high\n");
}

#[test]
fn saving_keeps_the_other_lines() {
    let root = scratch("others");
    fs::create_dir_all(root.join("ruler")).unwrap();
    fs::write(root.join("ruler/config.yaml"), "zoom: 2\ntop_string: high # the tab way\n").unwrap();
    save(&root, true).unwrap();
    assert_eq!(fs::read_to_string(root.join("ruler/config.yaml")).unwrap(), "zoom: 2\ntop_string: low\n");
}

#[test]
fn a_word_this_version_does_not_know_keeps_the_highest_on_top() {
    let root = scratch("unknown");
    fs::create_dir_all(root.join("ruler")).unwrap();
    fs::write(root.join("ruler/config.yaml"), "top_string: sideways\n").unwrap();
    assert!(!low_on_top(&root));
}
