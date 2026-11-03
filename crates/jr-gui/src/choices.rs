//! Responsibility: keeps what only the ruler remembers in `<root>/ruler/config.yaml`: which string the song
//! page draws on top.

use std::fs;
use std::io;
use std::path::Path;

const FOLDER: &str = "ruler";
const FILE_NAME: &str = "config.yaml";
const KEY: &str = "top_string:";

/// Whether the song page draws the lowest string on top, as saved under `root` (normally
/// `jovee_theme::jovee_root()`); no, the highest on top, when nothing was saved yet or the file holds a
/// word this version does not know.
pub fn low_on_top(root: &Path) -> bool {
    fs::read_to_string(root.join(FOLDER).join(FILE_NAME))
        .is_ok_and(|text| text.lines().find_map(top_string) == Some("low"))
}

/// Saves which string is on top under `root`, creating the folder if needed. Only the `top_string` key
/// changes. The file is written aside and renamed over the old one, so a reader never sees half a file.
pub fn save(root: &Path, low_on_top: bool) -> io::Result<()> {
    let folder = root.join(FOLDER);
    fs::create_dir_all(&folder)?;
    let path = folder.join(FILE_NAME);
    let before = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error),
    };
    let aside = folder.join(format!(".{FILE_NAME}.{}.tmp", std::process::id()));
    fs::write(&aside, with_top_string(&before, low_on_top))?;
    fs::rename(&aside, path).inspect_err(|_| {
        let _ = fs::remove_file(&aside);
    })
}

/// The value of a top-level `top_string:` line, without quotes or a trailing comment.
fn top_string(line: &str) -> Option<&str> {
    let value = line.strip_prefix(KEY)?;
    let value = value.split('#').next().unwrap_or_default().trim();
    Some(value.trim_matches(|c| c == '"' || c == '\''))
}

fn with_top_string(text: &str, low_on_top: bool) -> String {
    let line = format!("{KEY} {}", if low_on_top { "low" } else { "high" });
    let mut found = false;
    let mut out = String::with_capacity(text.len() + line.len() + 1);
    for old in text.lines() {
        let is_key = top_string(old).is_some();
        out.push_str(if is_key { &line } else { old });
        out.push('\n');
        found |= is_key;
    }
    if !found {
        out.push_str(&line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
#[path = "choices_tests.rs"]
pub(crate) mod choices_tests;
