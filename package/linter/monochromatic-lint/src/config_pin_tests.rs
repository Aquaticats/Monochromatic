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
use std::path::{Path, PathBuf};

/// The lockfile table header as it appears in a real mise lockfile,
///  written out here so the
/// extraction controls pin the format rather than this module's constant.
const REAL_HEADER: &str = "[[tools.\"cargo:monochromatic-lint\"]]";

/// The running version every control compares against.
/// It is fixed here, not read from the manifest,
///  so that no control changes meaning when a release bumps the version;
///  the pre-release controls pass their own running version to the probe instead.
const RUNNING_VERSION: &str = "2.3.4";

/// The hint for one configuration directory as the running version `RUNNING_VERSION` would give it.
fn hint_for_running_version(
    directory: &Path,
    read: &dyn Fn(&Path) -> Option<String>,
) -> Option<String> {
    return stale_binary_hint_with(directory, RUNNING_VERSION, read);
}

/// A pin strictly newer than the running version,
///  built from `RUNNING_VERSION`.
fn newer_pin() -> String {
    let major: u64 = RUNNING_VERSION
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
    assert!(!older_than_pin("1.0", "2.0"));
    assert!(!older_than_pin("1.0.0", "2.0.0-"));
    assert!(!older_than_pin("1.0.0-", "2.0.0"));
    assert!(!older_than_pin("1.0.0-alpha..1", "2.0.0"));
}

/// A pre-release orders as semantic versioning says:
///  below its own release,
///  above the release before it,
///  by number for numeric identifiers,
///  numeric identifiers below text ones,
///  a shorter list below a longer one that starts the same,
///  and build metadata not at all.
#[test]
fn a_pre_release_orders_by_semantic_versioning() {
    assert!(older_than_pin("0.5.2-alpha.3", "0.5.2-alpha.4"));
    assert!(!older_than_pin("0.5.2-alpha.3", "0.5.2-alpha.2"));
    assert!(!older_than_pin("0.5.2-alpha.3", "0.5.2-alpha.3"));
    assert!(older_than_pin("0.5.2-alpha.3", "0.5.2"));
    assert!(!older_than_pin("0.5.2", "0.5.2-alpha.3"));
    assert!(older_than_pin("0.5.1", "0.5.2-alpha.1"));
    assert!(!older_than_pin("0.5.2-alpha.1", "0.5.1"));
    assert!(older_than_pin("0.5.2-alpha.3", "0.6.0"));
    assert!(older_than_pin("0.3.0-alpha.1", "0.4.0"));
    // Positive control: identifiers of digits order by number, not by text.
    assert!(older_than_pin("1.0.0-alpha.2", "1.0.0-alpha.10"));
    assert!(!older_than_pin("1.0.0-alpha.10", "1.0.0-alpha.2"));
    assert!(older_than_pin("1.0.0-alpha.1", "1.0.0-alpha.beta"));
    assert!(!older_than_pin("1.0.0-alpha.beta", "1.0.0-alpha.1"));
    assert!(older_than_pin("1.0.0-alpha", "1.0.0-alpha.1"));
    assert!(!older_than_pin("1.0.0-alpha.1", "1.0.0-alpha"));
    assert!(older_than_pin("1.0.0-alpha", "1.0.0-beta"));
    assert!(!older_than_pin("1.0.0-beta", "1.0.0-alpha"));
    assert!(!older_than_pin("1.0.0+first", "1.0.0+second"));
    assert!(older_than_pin("1.0.0-alpha+first", "1.0.0-alpha.1+second"));
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
    let hint = hint_for_running_version(root, &reader(&[(lock.as_path(), stale_text.as_str())]))
        .expect("a measured newer pin produces a hint");
    assert!(hint.contains("may be stale"), "{hint}");
    assert!(hint.contains(newer.as_str()), "{hint}");
    assert!(hint.contains(lock.to_string_lossy().as_ref()), "{hint}");
    // The same pin in a nested configuration directory is still the repository's pin.
    let nested = hint_for_running_version(
        &root.join("src"),
        &reader(&[(lock.as_path(), stale_text.as_str())]),
    )
    .expect("an ancestor lockfile measures the pin");
    assert_eq!(nested, hint);
    // A pre-release running version is measured against a pre-release or a release pin.
    for pin in ["0.5.2-alpha.4", "0.5.2", "0.6.0-alpha.1"] {
        let text: String = format!("{REAL_HEADER}\nversion = \"{pin}\"\n");
        let found = stale_binary_hint_with(
            root,
            "0.5.2-alpha.3",
            &reader(&[(lock.as_path(), text.as_str())]),
        )
        .expect("a newer pin produces a hint for a pre-release running version");
        assert!(found.contains(pin), "{found}");
    }
}

/// A pin that is not newer than the running binary measures no staleness,
///  and a nearest lockfile
/// this tool is absent from stays silent rather than borrowing a farther pin.
#[test]
fn unmeasured_pins_and_farther_lockfiles_produce_no_hint() {
    assert_ne!(
        RUNNING_VERSION, "0.0.0",
        "the older-pin control needs a strictly older pin"
    );
    let root: &Path = Path::new("/repo");
    let lock: PathBuf = root.join("mise.lock");
    let equal: String = format!("{REAL_HEADER}\nversion = \"{RUNNING_VERSION}\"\n");
    assert_eq!(
        hint_for_running_version(root, &reader(&[(lock.as_path(), equal.as_str())])),
        None
    );
    let older: String = format!("{REAL_HEADER}\nversion = \"0.0.0\"\n");
    assert_eq!(
        hint_for_running_version(root, &reader(&[(lock.as_path(), older.as_str())])),
        None
    );
    let earlier_pre_release: String =
        format!("{REAL_HEADER}\nversion = \"{RUNNING_VERSION}-alpha.1\"\n");
    assert_eq!(
        hint_for_running_version(
            root,
            &reader(&[(lock.as_path(), earlier_pre_release.as_str())])
        ),
        None
    );
    let unorderable: String = format!("{REAL_HEADER}\nversion = \"{RUNNING_VERSION}-\"\n");
    assert_eq!(
        hint_for_running_version(root, &reader(&[(lock.as_path(), unorderable.as_str())])),
        None
    );
    // A pre-release pin that is not newer than a pre-release running version measures no staleness.
    for pin in ["0.5.2-alpha.3", "0.5.2-alpha.2", "0.5.1"] {
        let text: String = format!("{REAL_HEADER}\nversion = \"{pin}\"\n");
        assert_eq!(
            stale_binary_hint_with(
                root,
                "0.5.2-alpha.3",
                &reader(&[(lock.as_path(), text.as_str())])
            ),
            None,
            "{pin}"
        );
    }
    let not_this_tool: String = String::from("[[tools.\"cargo:fd-find\"]]\nversion = \"9.9.9\"\n");
    assert_eq!(
        hint_for_running_version(root, &reader(&[(lock.as_path(), not_this_tool.as_str())])),
        None
    );
    // Positive control: the same probe hints when the nearest lockfile pins a newer version.
    let near: String = format!("{REAL_HEADER}\nversion = \"{}\"\n", newer_pin());
    assert!(hint_for_running_version(root, &reader(&[(lock.as_path(), near.as_str())])).is_some());
    // The nearest lockfile governs: this tool's absence there is measured silence, not a hint.
    let farther: PathBuf = Path::new("/").join("mise.lock");
    let no_tool: String = String::from("[[tools.\"cargo:fd-find\"]]\nversion = \"1.0.0\"\n");
    assert_eq!(
        hint_for_running_version(
            &root.join("src"),
            &reader(&[
                (lock.as_path(), no_tool.as_str()),
                (farther.as_path(), near.as_str()),
            ]),
        ),
        None
    );
    assert_eq!(
        hint_for_running_version(&root.join("src"), &reader(&[])),
        None
    );
}
