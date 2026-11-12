//! What:
//!  Guard for the owner's rule that nothing of the wrapper starts at system startup or at login.
//! The source this release ships has
//!  no `[Install]` section,
//! no `WantedBy=`,
//! `RequiredBy=`,
//! `UpheldBy=`,
//! `Also=`,
//! `Alias=` or `DefaultInstance=` line in a unit it
//!  writes,
//! never asks the user manager to enable,
//! link,
//! preset or default anything,
//! and speaks to the manager only with the
//!  members on the list below.
//! Why:
//!  Owner,
//! 2026-11-07:
//! "Were you implying our wrapper or something in it auto starts itself on system startup or user login?
//! It
//!  should never do that!
//! Follow Bazel."
//! Then:
//! "Systemd units are fine."
//! The commit service is started by the first command that
//!  needs it (socket activation of a runtime unit),
//! and exits after three idle hours,
//! like Bazel's server.
//! A unit with an
//!  `[Install]` section is what `systemctl --user enable` links into a `*.wants` folder that the login session starts,
//! so the
//!  guard keeps that out of the texts,
//! and the call that would enable one out of the code.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of releaseSources()) expect(codeLines(file).filter(forbidden)).toEqual([]); expect(membersCalled()).toSubsetOf(ALLOWED);
//! ```

use std::path::{Path, PathBuf};

/// Texts that no release code line may carry:
/// the manager's calls that make a unit start at startup or login,
/// and the program
///  that makes them from a shell.
const FORBIDDEN_IN_CODE: &[&str] = &[
    "EnableUnitFiles",
    "LinkUnitFiles",
    "AddDependencyUnitFiles",
    "PresetUnitFiles",
    "SetDefaultTarget",
    "MaskUnitFiles",
    "Command::new(\"systemctl",
    "WantedBy=",
    "RequiredBy=",
    "[Install]",
];


/// The lines of a unit's text that would make it start at startup or at login (`systemd.unit(5)`,
/// "[Install] Section Options"):
///  the section header and the options that only an install reads.
/// Comment lines are skipped.
pub(crate) fn lines_that_start_at_login(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        let lower: String = trimmed.to_ascii_lowercase();
        let install: bool = lower == "[install]"
            || ["wantedby=", "requiredby=", "upheldby=", "also=", "alias=", "defaultinstance="].iter().any(|key| { return lower.starts_with(key); });
        if install {
            found.push(trimmed.to_string());
        }
    }
    return found;
}

/// Every file below `root` as a path relative to it,
/// found with a work stack,
/// keeping those whose extension is in `extensions`.
fn files_below(root: &Path, extensions: &[&str]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("a readable source directory").flatten() {
            let path: PathBuf = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let extension: String = path.extension().map(|text| { return text.to_string_lossy().into_owned(); }).unwrap_or_default();
            if extensions.contains(&extension.as_str()) {
                let relative: &Path = path.strip_prefix(root).expect("a path below the root");
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    found.sort();
    return found;
}

/// Whether a file is test code by its name,
/// which is how the release sources are told apart here.
fn is_test_file(relative: &str) -> bool {
    let name: &str = relative.rsplit('/').next().unwrap_or(relative);
    return name.ends_with("_tests.rs") || name.contains("test_support") || relative.starts_with("testing/") || relative.starts_with("binary/");
}

/// The source root of this package,
/// `src/native`.
fn native_root() -> PathBuf {
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
}

/// The code lines of a text:
/// not blank,
/// not a comment.
fn code_lines(text: &str) -> Vec<(usize, &str)> {
    return text
        .lines()
        .enumerate()
        .filter(|(_, line)| { let trimmed: &str = line.trim_start(); return !trimmed.is_empty() && !trimmed.starts_with("//"); })
        .map(|(index, line)| { return (index + 1, line); })
        .collect();
}

/// The detector finds each forbidden line in a unit text,
/// so the guards above and in the provisioning controls cannot pass for
///  want of looking.
#[test]
fn the_detector_finds_every_install_line_and_skips_comments() {
    for bad in ["[Install]", "WantedBy=default.target", "RequiredBy=sockets.target", "UpheldBy=x.target", "Also=y.socket", "Alias=z.service", "DefaultInstance=a", "  wantedby=sockets.target"] {
        let text: String = format!("[Unit]\nDescription=x\n{bad}\n");
        assert_eq!(lines_that_start_at_login(text.as_str()).len(), 1, "{bad}");
    }
    assert!(lines_that_start_at_login("[Unit]\n# WantedBy=default.target is not written\n; [Install]\n[Socket]\nListenStream=%t/x.sock\n").is_empty());
}

/// What:
///  No release code writes a unit file,
///  links one,
///  starts `systemctl`,
///  or writes an install line.
/// Why:
///   Nothing of the wrapper starts at startup or at login,
/// and this release writes no unit at all:
/// the wrapper starts its own server
/// (`doc/planning/cli-git-commit-service-without-a-service-manager.md`).
#[test]
fn no_release_code_writes_a_unit_or_starts_a_service_manager() {
    let root: PathBuf = native_root();
    let mut found: Vec<String> = Vec::new();
    for relative in files_below(root.as_path(), &["rs"]) {
        if is_test_file(relative.as_str()) {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(relative.as_str())).expect("a source file");
        for (number, line) in code_lines(text.as_str()) {
            for forbidden in FORBIDDEN_IN_CODE {
                if line.contains(forbidden) {
                    found.push(format!("{relative}:{number}: {forbidden}"));
                }
            }
            if line.contains(".wants") || line.contains(".requires") || line.contains("systemd/user") {
                found.push(format!("{relative}:{number}: a unit path"));
            }
        }
    }
    assert_eq!(found, Vec::<String>::new());
    assert!(!root.join("service").join("service_units").exists(), "the unit templates are gone with the units");
}

/// What:
///  No release code names a member of the user manager's interface at all.
/// Why:
///  This release asks a service manager for nothing:
/// the wrapper starts, finds, replaces and stops its own server,
/// so a call that came back would be a design regression and not a documented exception.
#[test]
fn no_release_code_names_a_manager_member() {
    let root: PathBuf = native_root();
    let mut called: Vec<String> = Vec::new();
    for relative in files_below(root.as_path(), &["rs"]) {
        if is_test_file(relative.as_str()) {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(relative.as_str())).expect("a source file");
        for (number, line) in code_lines(text.as_str()) {
            if line.contains("org.freedesktop.systemd1") || line.contains("StartUnit") || line.contains("ResetFailedUnit") || line.contains("ActiveState") {
                called.push(format!("{relative}:{number}"));
            }
        }
    }
    assert!(called.is_empty(), "no release code names a manager member: {called:?}");
}
