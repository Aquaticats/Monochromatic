//! What:
//!  Controls for the measured staleness hint.
//! Why:
//!  The hint is only honest when a lockfile pin is measured,
//!  ordered,
//!  and newer than the running
//! binary;
//!  every other case must stay silent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('config pin', () => { /* extraction, ordering, hint boundaries */ });
//! ```

/// Import the probe under test and the running build's version.
use super::{older_than_pin, pinned_tool_version, stale_binary_hint_with};
use crate::run_finish::VERSION;
use std::path::{Path, PathBuf};

/// The lockfile table header as it appears in a real mise lockfile,
///  written out here so the
/// extraction controls pin the format rather than this module's constant.
const REAL_HEADER: &str = "[[tools.\"cargo:monochromatic-lint\"]]";

/// A pin strictly newer than the running version,
///  built from the manifest version.
fn newer_pin() -> String {
    let major: u64 = VERSION
        .split('.')
        .next()
        .expect("manifest version has a major component")
        .parse()
        .expect("major component is numeric");
    return format!("{}.0.0", major + 1);
}

/// A reader over a fixed set of lockfile paths,
///  standing in for the filesystem.
fn reader(entries: &[(&Path, &str)]) -> Box<dyn Fn(&Path) -> Option<String>> {
    let mut owned: Vec<(PathBuf, String)> = Vec::new();
    for (path, text) in entries {
        owned.push(((*path).to_path_buf(), String::from(*text)));
    }
    return Box::new(move |path: &Path| -> Option<String> {
        for (candidate, text) in &owned {
            if candidate == path {
                return Some(text.clone());
            }
        }
        return None;
    });
}

/// Only a plain version key inside this tool's exact lockfile table reads as a pin.
#[test]
fn the_pin_comes_only_from_the_tools_own_table() {
    let measured: String =
        format!("{REAL_HEADER}\nversion = \"0.3.0\"\nbackend = \"cargo:monochromatic-lint\"\n");
    assert_eq!(
        pinned_tool_version(measured.as_str()),
        Some(String::from("0.3.0"))
    );
    assert_eq!(pinned_tool_version(""), None);
    assert_eq!(
        pinned_tool_version("[[tools.\"cargo:fd-find\"]]\nversion = \"0.3.0\"\n"),
        None
    );
    let without_version: String = format!("{REAL_HEADER}\nspecifiers = [\"latest\"]\n");
    assert_eq!(pinned_tool_version(without_version.as_str()), None);
    let version_before_table: String = format!("version = \"0.3.0\"\n{REAL_HEADER}\n");
    assert_eq!(pinned_tool_version(version_before_table.as_str()), None);
    let unquoted: String = format!("{REAL_HEADER}\nversion = 0.3.0\n");
    assert_eq!(pinned_tool_version(unquoted.as_str()), None);
    let padded: String = format!("{REAL_HEADER}\nversion = \" 0.3.0 \"\n");
    assert_eq!(pinned_tool_version(padded.as_str()), None);
    let other_table: String =
        format!("{REAL_HEADER}\nname = \"x\"\n[[tools.\"other\"]]\nversion = \"0.3.0\"\n");
    assert_eq!(pinned_tool_version(other_table.as_str()), None);
}

/// Version ordering is numeric and directional,
///  and an unorderable version measures nothing.
#[test]
fn staleness_needs_a_measured_older_running_version() {
    assert!(older_than_pin("0.2.0", "0.3.0"));
    // Positive control: numeric ordering must not fall back to text ordering.
    assert!(older_than_pin("0.9.0", "0.10.0"));
    assert!(!older_than_pin("0.3.0", "0.3.0"));
    assert!(!older_than_pin("0.3.1", "0.3.0"));
    assert!(!older_than_pin("0.3.0", "0.3.0-alpha.1"));
    assert!(!older_than_pin("0.3.0-alpha.1", "0.4.0"));
    assert!(!older_than_pin("1.0", "2.0"));
}

/// The hint appears only for a measured newer pin in the nearest lockfile,
///  and names the
/// measurement it stands on.
#[test]
fn the_hint_names_a_measured_newer_pin_and_nothing_else() {
    let root: &Path = Path::new("/repo");
    let lock: PathBuf = root.join("mise.lock");
    let newer: String = newer_pin();
    let stale_text: String = format!("{REAL_HEADER}\nversion = \"{newer}\"\n");
    let hint = stale_binary_hint_with(root, &reader(&[(lock.as_path(), stale_text.as_str())]))
        .expect("a measured newer pin produces a hint");
    assert!(hint.contains("may be stale"), "{hint}");
    assert!(hint.contains(newer.as_str()), "{hint}");
    assert!(hint.contains(lock.to_string_lossy().as_ref()), "{hint}");
    // The same pin in a nested configuration directory is still the repository's pin.
    let nested = stale_binary_hint_with(
        &root.join("src"),
        &reader(&[(lock.as_path(), stale_text.as_str())]),
    )
    .expect("an ancestor lockfile measures the pin");
    assert_eq!(nested, hint);
}

/// A pin that is not newer than the running binary measures no staleness,
///  and a nearest lockfile
/// this tool is absent from stays silent rather than borrowing a farther pin.
#[test]
fn unmeasured_pins_and_farther_lockfiles_produce_no_hint() {
    assert_ne!(
        VERSION, "0.0.0",
        "the older-pin control needs a strictly older pin"
    );
    let root: &Path = Path::new("/repo");
    let lock: PathBuf = root.join("mise.lock");
    let equal: String = format!("{REAL_HEADER}\nversion = \"{VERSION}\"\n");
    assert_eq!(
        stale_binary_hint_with(root, &reader(&[(lock.as_path(), equal.as_str())])),
        None
    );
    let older: String = format!("{REAL_HEADER}\nversion = \"0.0.0\"\n");
    assert_eq!(
        stale_binary_hint_with(root, &reader(&[(lock.as_path(), older.as_str())])),
        None
    );
    let unorderable: String = format!("{REAL_HEADER}\nversion = \"{VERSION}-alpha.1\"\n");
    assert_eq!(
        stale_binary_hint_with(root, &reader(&[(lock.as_path(), unorderable.as_str())])),
        None
    );
    let not_this_tool: String = String::from("[[tools.\"cargo:fd-find\"]]\nversion = \"9.9.9\"\n");
    assert_eq!(
        stale_binary_hint_with(root, &reader(&[(lock.as_path(), not_this_tool.as_str())])),
        None
    );
    // Positive control: the same probe hints when the nearest lockfile pins a newer version.
    let near: String = format!("{REAL_HEADER}\nversion = \"{}\"\n", newer_pin());
    assert!(stale_binary_hint_with(root, &reader(&[(lock.as_path(), near.as_str())])).is_some());
    // The nearest lockfile governs: this tool's absence there is measured silence, not a hint.
    let farther: PathBuf = Path::new("/").join("mise.lock");
    let no_tool: String = String::from("[[tools.\"cargo:fd-find\"]]\nversion = \"1.0.0\"\n");
    assert_eq!(
        stale_binary_hint_with(
            &root.join("src"),
            &reader(&[
                (lock.as_path(), no_tool.as_str()),
                (farther.as_path(), near.as_str()),
            ]),
        ),
        None
    );
    assert_eq!(
        stale_binary_hint_with(&root.join("src"), &reader(&[])),
        None
    );
}
