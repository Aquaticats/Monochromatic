//! What:
//! Unit tests of the version and help announcements in `announce.rs`:
//! which argument lists count as a request,
//! and the exact added text.
//! Why:
//! The flag table is the part most likely to be wrong (`-h` means help for one tool and "human-readable" for
//! another),
//! so each tool's spelling is written out here independently of the table itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // announce.unit.test.ts
//! describe('announcementFor', () => { ... });
//! ```

/// Imports what is under test.
use crate::announce::{Announcement, announced_flags, announcement_for, help_section, version_line};

/// Imports the list of shadowed names,
/// to prove the flag table covers every one.
use crate::constants::SHADOWED_NAMES;

/// Imports the owned operating-system text type and the trait that builds one from raw bytes.
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

/// Imports the borrowed path type.
use std::path::Path;

/// What:
/// `fn texts(items: &[&str]) -> Vec<OsString>` declares a helper turning borrowed texts into owned
/// operating-system texts.
/// Why:
/// `announcement_for` takes arguments in the shape the process API gives them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = (items: string[]) => items;
/// ```
fn texts(items: &[&str]) -> Vec<OsString> {
    let mut owned: Vec<OsString> = Vec::new();
    for item in items {
        owned.push(OsString::from(*item));
    }
    return owned;
}

/// What:
/// `#[test]` marks the next function as a test case.
/// Why:
/// The runner finds the tests by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// it('every shadowed name has flags', () => { ... });
/// ```
#[test]
fn every_shadowed_name_has_flags_and_an_unshadowed_name_has_none() {
    for name in SHADOWED_NAMES {
        let maybe_flags = announced_flags(name);
        assert!(maybe_flags.is_some(), "{name} has no announced flags");
        let flags = maybe_flags.expect("checked above");
        assert!(flags.version.contains(&"--version"), "{name} must announce --version");
        assert!(flags.help.contains(&"--help"), "{name} must announce --help");
    }
    assert!(announced_flags("search-shadow").is_none());
    assert!(announced_flags("cat").is_none());
}

/// The expected spellings,
/// one row per tool,
/// read off each real tool's own CLI:
/// (name,
/// version flags,
/// help flags).
const EXPECTED: &[(&str, &[&str], &[&str])] = &[
    ("rg", &["--version", "-V"], &["--help", "-h"]),
    ("grep", &["--version", "-V"], &["--help"]),
    ("egrep", &["--version", "-V"], &["--help"]),
    ("fgrep", &["--version", "-V"], &["--help"]),
    ("ugrep", &["--version", "-V"], &["--help", "-?"]),
    ("ug", &["--version", "-V"], &["--help", "-?"]),
    ("ag", &["--version", "-V"], &["--help", "-h"]),
    ("ack", &["--version"], &["--help"]),
    ("find", &["--version", "-version"], &["--help", "-help"]),
    ("fd", &["--version", "-V"], &["--help", "-h"]),
    ("fdfind", &["--version", "-V"], &["--help", "-h"]),
    ("bfs", &["--version", "-version"], &["--help", "-help"]),
    ("tree", &["--version"], &["--help"]),
    ("du", &["--version"], &["--help"]),
    ("ncdu", &["--version", "-V"], &["--help", "-h", "-?"]),
    ("tokei", &["--version", "-V"], &["--help", "-h"]),
    ("cloc", &["--version"], &["--help"]),
];

/// Every tool recognizes exactly its own flags as a lone argument,
/// and the expected table covers every shadowed name.
#[test]
fn each_tool_recognizes_exactly_its_own_lone_version_and_help_flags() {
    for name in SHADOWED_NAMES {
        let mut listed: bool = false;
        for row in EXPECTED {
            if row.0 == *name {
                listed = true;
            }
        }
        assert!(listed, "{name} is missing from the expected table");
    }
    for (name, version_flags, help_flags) in EXPECTED {
        for flag in *version_flags {
            assert_eq!(announcement_for(name, &texts(&[flag])), Some(Announcement::Version), "{name} {flag}");
        }
        for flag in *help_flags {
            assert_eq!(announcement_for(name, &texts(&[flag])), Some(Announcement::Help), "{name} {flag}");
        }
        // Spellings that look like a request but are not one for this tool.
        for flag in ["-h", "-V", "-?", "-v", "-version", "-help", "--versions", "--HELP"] {
            let expected_version = version_flags.contains(&flag);
            let expected_help = help_flags.contains(&flag);
            if expected_version || expected_help {
                continue;
            }
            assert_eq!(announcement_for(name, &texts(&[flag])), None, "{name} {flag} must pass through untouched");
        }
    }
}

/// The short flags that mean something else must stay untouched:
/// `du -h` and `tree -h` are human-readable sizes,
/// `grep -h` and `ack -h` are no-filename,
/// `find -V` is an error.
#[test]
fn short_flags_with_another_meaning_are_not_requests() {
    assert_eq!(announcement_for("du", &texts(&["-h"])), None);
    assert_eq!(announcement_for("tree", &texts(&["-h"])), None);
    assert_eq!(announcement_for("grep", &texts(&["-h"])), None);
    assert_eq!(announcement_for("ack", &texts(&["-h"])), None);
    assert_eq!(announcement_for("ugrep", &texts(&["-h"])), None);
    assert_eq!(announcement_for("cloc", &texts(&["-h"])), None);
    assert_eq!(announcement_for("find", &texts(&["-V"])), None);
    assert_eq!(announcement_for("du", &texts(&["-V"])), None);
}

/// Only a lone flag counts:
/// extra arguments,
/// no arguments and a non-UTF-8 argument all pass through.
#[test]
fn only_a_lone_flag_is_a_top_level_request() {
    assert_eq!(announcement_for("rg", &texts(&["--version", "foo"])), None);
    assert_eq!(announcement_for("rg", &texts(&["foo", "--version"])), None);
    assert_eq!(announcement_for("rg", &texts(&["--no-config", "--help"])), None);
    assert_eq!(announcement_for("rg", &texts(&["--"])), None);
    assert_eq!(announcement_for("rg", &[]), None);
    assert_eq!(announcement_for("rg", &[OsString::from_vec(vec![0xff, 0x2d])]), None);
    assert_eq!(announcement_for("search-shadow", &texts(&["--version"])), None);
}

/// The version line has the exact documented shape.
#[test]
fn the_version_line_names_the_shadow_its_version_the_tool_and_the_real_path() {
    let line = version_line("rg", Path::new("/usr/bin/rg"));
    assert_eq!(line, format!("search-shadow {}, shadowing rg at /usr/bin/rg\n", env!("CARGO_PKG_VERSION")));
}

/// A relative real path is made absolute,
/// so the line means the same from any directory.
#[test]
fn a_relative_real_path_is_printed_absolute() {
    let line = version_line("fd", Path::new("bin/fd"));
    assert!(line.contains(" at /"), "{line}");
    assert!(line.ends_with("/bin/fd\n"), "{line}");
}

/// The help section begins with a blank line and the version line,
/// and names what is added per tool group.
#[test]
fn the_help_section_starts_with_the_version_line_and_describes_the_tools_policy() {
    let rg = help_section("rg", Path::new("/usr/bin/rg"));
    assert!(rg.starts_with(&format!("\n{}", version_line("rg", Path::new("/usr/bin/rg")))), "{rg}");
    assert!(rg.ends_with('\n'));
    assert!(rg.contains("-r") && rg.contains("--replace") && rg.contains("SEARCH_SHADOW_ALLOW_WIDE=1"), "{rg}");

    let grep = help_section("egrep", Path::new("/usr/bin/egrep"));
    assert!(grep.contains("recursive search") && grep.contains("SEARCH_SHADOW_ALLOW_WIDE=1"), "{grep}");

    let find = help_section("bfs", Path::new("/usr/bin/bfs"));
    assert!(find.contains("-maxdepth 3"), "{find}");

    let tree = help_section("tree", Path::new("/usr/bin/tree"));
    assert!(tree.contains("never refuses"), "{tree}");
    assert!(!tree.contains("--replace") && !tree.contains("-maxdepth"), "{tree}");
}
