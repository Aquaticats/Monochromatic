//! Controls for publishing a staged record at its final name,
//!  each in its own disposable directory.
//! The ban on a link of the module's own reads each call by its name,
//! and it does not see a call a macro of another file expands to or a name a glob import (`use super::*;`)
//! brings in;
//! a reader checks each of those in a file that adds one.

use super::*;
#[cfg(unix)]
use crate::a_record_made_durable_with_a_sync::settle_directory_sync;
use crate::source_guard::{
    HoldingLine,
    MayHold,
    anchors_the_scope_no_longer_holds,
    lines_calling_one_of,
    lines_holding_one_of,
    lines_no_entry_excuses,
    release_sources_under,
};
use crate::test_support::{
    fixture,
    remove,
};
use std::path::PathBuf;

/// A staging file for `path` with `content` in it,
/// and the refusal a filesystem with neither `renameat2` with `RENAME_NOREPLACE`
///  nor hard links gives (`s3fs-fuse` answers `ENOTSUP` to the link),
/// as `persist_noclobber` reports it with the file handed back.
#[cfg(unix)]
fn refused_publish(
    path: &Path,
    content: &[u8],
) -> (PathBuf, Result<std::fs::File, tempfile::PersistError>) {
    let mut staged: tempfile::NamedTempFile = staging_beside(path).expect("staging file");
    staged
        .as_file_mut()
        .write_all(content)
        .expect("staged content");
    let staged_path: PathBuf = staged.path().to_path_buf();
    let refusal: tempfile::PersistError = tempfile::PersistError {
        error: std::io::Error::from_raw_os_error(libc::ENOTSUP),
        file: staged,
    };
    return (staged_path, Err(refusal));
}

/// The owner's answer on a filesystem that has neither (2026-11-07,
/// "Keep the fallback (Recommended)"):
/// the staged file is
///  published by the rename,
/// the staging name is gone,
/// and nothing is left to remove later.
#[cfg(unix)]
#[test]
fn a_filesystem_with_neither_no_replace_rename_nor_links_gets_the_checked_rename() {
    let root: PathBuf = fixture("publish-neither");
    let path: PathBuf = root.join("published");
    let (staged_path, outcome) = refused_publish(path.as_path(), b"whole");
    assert!(
        staged_path.exists(),
        "the control: the staged file exists before the publish"
    );
    assert!(!path.exists(), "the control: nothing is published yet");
    settle_exclusive_publish(path.as_path(), outcome)
        .expect("the refusal is answered by the rename");
    assert_eq!(std::fs::read(&path).expect("published content"), b"whole");
    assert!(!staged_path.exists(), "the rename moved the staged file");
    remove(root.as_path());
}

/// The fallback still refuses a taken name with `AlreadyExists`,
/// leaves the record that holds it as it was,
/// and leaves no
///  staging file behind.
#[cfg(unix)]
#[test]
fn the_checked_rename_refuses_a_taken_name_and_leaves_no_staging_file() {
    let root: PathBuf = fixture("publish-neither-taken");
    let path: PathBuf = root.join("record");
    std::fs::write(&path, b"old").expect("record");
    let (staged_path, outcome) = refused_publish(path.as_path(), b"new");
    let error: std::io::Error =
        settle_exclusive_publish(path.as_path(), outcome).expect_err("the name is taken");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(std::fs::read(&path).expect("record"), b"old");
    assert!(
        !staged_path.exists(),
        "the refused publish removed its staging file"
    );
    remove(root.as_path());
}

/// A taken name is reported with the operating system's own `EEXIST`,
/// straight from the publish,
/// the record that holds the name
///  is left as it was,
/// and the refused write leaves no staging file.
#[cfg(unix)]
#[test]
fn a_taken_name_is_refused_with_the_operating_systems_error() {
    let root: PathBuf = fixture("publish-taken");
    let path: PathBuf = root.join("record");
    std::fs::write(&path, b"old").expect("record");
    let error: std::io::Error =
        write_private_file(path.as_path(), b"new").expect_err("the name is taken");
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(
        error.raw_os_error(),
        Some(libc::EEXIST),
        "the publish's own refusal, not a rebuilt one"
    );
    assert_eq!(std::fs::read(&path).expect("record"), b"old");
    let names: Vec<String> = std::fs::read_dir(&root)
        .expect("listing")
        .flatten()
        .map(|entry| {
            return entry.file_name().to_string_lossy().into_owned();
        })
        .collect();
    assert_eq!(names, vec![String::from("record")], "no staging file stays");
    remove(root.as_path());
}

/// A publish that works leaves exactly the record,
/// mode `0600`,
/// and no staging file;
/// a failure between the write and the publish
///  leaves neither the record nor a staging file.
#[cfg(unix)]
#[test]
fn a_published_record_is_private_and_a_failed_publish_leaves_nothing() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("publish-private");
    let path: PathBuf = root.join("record");
    write_private_file_unsynced(path.as_path(), b"whole").expect("publish");
    assert_eq!(
        std::fs::metadata(&path)
            .expect("record")
            .permissions()
            .mode()
            & 0o777,
        PRIVATE_FILE_MODE
    );
    let other: PathBuf = root.join("other");
    AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| {
        *hook.borrow_mut() = Some(Box::new(|| {
            return Err(std::io::Error::other("the disk filled"));
        }));
    });
    let failed: std::io::Result<()> = write_private_file_unsynced(other.as_path(), b"cut");
    AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| {
        *hook.borrow_mut() = None;
    });
    failed.expect_err("the injected failure");
    let mut names: Vec<String> = std::fs::read_dir(&root)
        .expect("listing")
        .flatten()
        .map(|entry| {
            return entry.file_name().to_string_lossy().into_owned();
        })
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![String::from("record")],
        "the failed write left no record and no staging file"
    );
    remove(root.as_path());
}

/// A directory sync that failed in the flush itself stays an error with its own operating-system code;
/// no unprivileged
///  filesystem produces these,
/// so the sync result is fed in.
#[cfg(unix)]
#[test]
fn a_failed_directory_flush_stays_an_error() {
    for code in [libc::EIO, libc::ENOSPC, libc::EDQUOT] {
        let error: std::io::Error = settle_directory_sync(
            Path::new("/unused"),
            Err(std::io::Error::from_raw_os_error(code)),
        )
        .expect_err("a failed flush is not skipped");
        assert_eq!(error.raw_os_error(), Some(code));
    }
}

/// A filesystem that has no directory flush counts as synced,
/// whichever of the three codes it answers with,
/// and a
///  sync that worked stays a success.
#[cfg(unix)]
#[test]
fn a_filesystem_without_directory_sync_counts_as_synced() {
    for code in [libc::EINVAL, libc::ENOTSUP, libc::ENOSYS] {
        settle_directory_sync(
            Path::new("/unused"),
            Err(std::io::Error::from_raw_os_error(code)),
        )
        .expect("an unsupported directory sync is skipped");
    }
    settle_directory_sync(Path::new("/unused"), Ok(())).expect("a sync that worked");
}

/// What:
///  `THE_MODULE` names the release sources of the private-record module:
///  the publish itself,
///  the files `private_storage.rs` re-exports from,
///  the publish over a taken name,
///  and the folder whose entries a private read lists.
/// Why:
///  The ban below is about the module,
///  and the alpha.48 split spread that module over files of its own,
///  so the scope is stated as prefixes rather than as the one file it used to be:
///  a file a later split makes beside them is covered without anyone remembering it
///  (issue 6599).
///  A file of the module named without one of these prefixes is one this ban does not read,
///  and no folder closes that:
///  the folder they live in (`foundation/`) holds other files that link and name a random identifier for their own
///  work,
///  so the folder would be a ban on code that is no record writer;
///  `SCOPE_ANCHORS` is what fails when these prefixes stop reaching the module's publish,
///  and a reader checks the name of a new file of this module against these prefixes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_MODULE = ['foundation/private_storage', 'foundation/a_record_', 'foundation/entries_of_one_open_folder.rs'];
/// ```
const THE_MODULE: &[&str] = &[
    "foundation/private_storage",
    "foundation/a_record_",
    "foundation/entries_of_one_open_folder.rs",
];

/// What:
///  `THE_TEMPFILE_CRATES` is every name of the work the module does not do itself:
///  the link the exclusive publish is made through,
///  and the random name a staging file is given.
/// Why:
///  The owner's answer put them in the `tempfile` crate,
///  so a file of this module that grows its own link is the finding,
///  and a link is read by its name where a line spells it as a call,
///  as a path's last segment,
///  or with its types written out.
///  Another crate's name for the same system call is banned too (`rustix::fs::linkat`,
///  `libc::link`),
///  which is why the list names the calls and not one library's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_TEMPFILE_CRATES = ['hard_link', 'link', 'linkat', 'symlink', 'symlinkat', 'random_uuid'];
/// ```
const THE_TEMPFILE_CRATES: &[&str] = &[
    "hard_link",
    "link",
    "linkat",
    "symlink",
    "symlinkat",
    "random_uuid",
];

/// What:
///  `THE_STAGING_NAME` is how a file of this module would build a staging name of its own.
/// Why:
///  The staging file's name belongs to the `tempfile` crate as well,
///  and `fn staging_path` is how a file of the module would take it back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_STAGING_NAME = ['fn staging_path'];
/// ```
const THE_STAGING_NAME: &[&str] = &["fn staging_path"];

/// What:
///  `MAY_HOLD` is empty:
///  no file of the private-record module makes a link of its own or builds a staging name.
/// Why:
///  The ban reaches the whole module,
///  so a file added here fails until a reader decides whether the crate's own publish answers,
///  and that decision is what an entry in this list would record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAY_HOLD: MayHold[] = [];
/// ```
const MAY_HOLD: &[MayHold] = &[];

/// What:
///  `SCOPE_ANCHORS` is the text that proves the scope still reaches the publish this ban was written for.
/// Why:
///  A prefix that reached nothing would pass every tree,
///  and `persist_noclobber` is the one call that says the publish is still the crate's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SCOPE_ANCHORS = [['persist_noclobber', 'no file of the module publishes through the tempfile crate any more']];
/// ```
const SCOPE_ANCHORS: &[(&str, &str)] = &[(
    "persist_noclobber",
    "no release file of the private-record module publishes through the tempfile crate any more, so this ban reaches nothing",
)];

/// What:
///  `lines_the_module_may_not_hold(sources)` is every code line of those sources that spells one of
///  `THE_TEMPFILE_CRATES` or writes `THE_STAGING_NAME`.
/// Why:
///  The ban is one list of findings over one scope,
///  so the two readings are joined here rather than in each control that asks what the ban found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesTheModuleMayNotHold = (sources) => [...linesCallingOneOf(sources, THE_TEMPFILE_CRATES), ...linesHoldingOneOf(sources, THE_STAGING_NAME)];
/// ```
fn lines_the_module_may_not_hold(sources: &[(String, String)]) -> Vec<HoldingLine> {
    let mut holding: Vec<HoldingLine> = lines_calling_one_of(sources, THE_TEMPFILE_CRATES);
    holding.extend(lines_holding_one_of(sources, THE_STAGING_NAME));
    return holding;
}

/// The owner's rulings of 2026-11-07 ("safe-file-saving shouldn't be something we own at all",
/// and the answer "Keep the
///  fallback (Recommended)" on a filesystem that has neither `renameat2` with `RENAME_NOREPLACE` nor hard links) put the exclusive
///  publish in the `tempfile` crate (`persist_noclobber`);
/// the module keeps only the checked rename for that one case,
/// so it holds
///  no hard link of its own and builds no staging name of its own,
/// in every file the split made of it.
#[test]
fn the_exclusive_publish_is_the_tempfile_crates_and_the_module_links_nothing() {
    let sources: Vec<(String, String)> =
        release_sources_under(THE_MODULE).expect("the crate's release sources");
    let abandoned: Vec<String> =
        anchors_the_scope_no_longer_holds(sources.as_slice(), SCOPE_ANCHORS);
    assert!(
        abandoned.is_empty(),
        "the scope reached no file holding what this ban is about:\n{}",
        abandoned.join("\n")
    );
    let holding = lines_the_module_may_not_hold(sources.as_slice());
    let unexplained: Vec<String> = lines_no_entry_excuses(holding.as_slice(), MAY_HOLD);
    assert!(
        unexplained.is_empty(),
        "the private-record module keeps no hard link and builds no staging name of its own, because the publish belongs to the tempfile crate:\n{}",
        unexplained.join("\n")
    );
}

/// What:
///  Every spelling of a link a file of the module could write:
///  the standard library's calls,
///  the same system call
///  through another crate,
///  a path taken as a value,
///  and a rename's import.
/// Why:
///  The review of release 0.6.1-alpha.50 found that `rustix::fs::linkat` and `libc::link` are the same link through
///  another name and held none of the three texts the ban read (issue 6613).
const SPELLINGS_OF_A_LINK: &[&str] = &[
    "let _ = std::fs::hard_link(from, to);",
    "let _ = std::os::unix::fs::symlink(from, to);",
    "let _ = libc::link(from, to);",
    "let _ = libc::linkat(from, to, at);",
    "let _ = std::os::unix::fs::hard_link::<_, _>(from, to);",
    "let f = std::fs::hard_link;",
    "use std::fs::hard_link as make_link;",
    "use std::fs::{hard_link as make_link};",
    "let _ = rustix::fs::linkat(rustix::fs::CWD, from, rustix::fs::CWD, to, rustix::fs::AtFlags::empty());",
];

/// What:
///  The names that end in a link's name,
///  and the publish the module makes instead.
/// Why:
///  `private_storage.rs` holds `fn link_file`,
///  which names a link and makes none,
///  and the crate's own publish is `persist_noclobber`.
const READINGS_OF_A_NAME: &[&str] = &[
    "fn link_file(from: &Path, to: &Path) -> std::io::Result<()> {",
    "let _ = kind.is_symlink();",
    "let _ = std::fs::read_link(path);",
    "let _ = std::fs::symlink_metadata(path);",
    "let _ = staged.persist_noclobber(path);",
];

/// The scan flags a link a split carried into a receiver,
/// in every spelling of the call,
/// and reads the publish's own call as no finding.
#[test]
fn the_scan_flags_a_link_in_a_receiver_file() {
    for spelling in SPELLINGS_OF_A_LINK {
        let planted: Vec<(String, String)> = vec![(
            String::from("foundation/a_record_planted.rs"),
            format!(
                "fn planted(from: &std::path::Path, to: &std::path::Path) {{\n    {spelling}\n}}\n"
            ),
        )];
        let holding: Vec<HoldingLine> = lines_the_module_may_not_hold(planted.as_slice());
        let unexplained: Vec<String> = lines_no_entry_excuses(holding.as_slice(), MAY_HOLD);
        assert_eq!(
            unexplained.len(),
            1,
            "{spelling} makes one link and is {} finding(s): {unexplained:?}",
            unexplained.len()
        );
    }
    for reading in READINGS_OF_A_NAME {
        let planted: Vec<(String, String)> = vec![(
            String::from("foundation/private_storage.rs"),
            format!("fn planted() {{\n    {reading}\n}}\n"),
        )];
        assert!(
            lines_the_module_may_not_hold(planted.as_slice()).is_empty(),
            "{reading} makes no link"
        );
    }
    let publish: Vec<(String, String)> = vec![(
        String::from("foundation/private_storage.rs"),
        String::from(
            "fn published(staged: tempfile::NamedTempFile) { staged.persist_noclobber(path); }\n",
        ),
    )];
    assert!(
        lines_the_module_may_not_hold(publish.as_slice()).is_empty(),
        "the crate's own publish is no link and no staging name"
    );
}

/// A staging file that cannot be made reports the system's own error,
/// with its code and without the staging path the
///  `tempfile` crate adds,
/// so the callers' texts say exactly what they said when the standard library made the file.
#[cfg(unix)]
#[test]
fn a_staging_file_that_cannot_be_made_reports_the_plain_system_error() {
    let root: PathBuf = fixture("publish-no-directory");
    let path: PathBuf = root.join("absent-directory").join("record");
    let error: std::io::Error =
        write_private_file(path.as_path(), b"x").expect_err("the directory is missing");
    assert_eq!(error.raw_os_error(), Some(libc::ENOENT));
    assert_eq!(error.to_string(), "No such file or directory (os error 2)");
    remove(root.as_path());
}
