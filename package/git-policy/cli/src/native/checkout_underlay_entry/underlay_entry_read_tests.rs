//! What:
//!  The live read's controls:
//!  a fixture directory reads as the underlay entry of its parent,
//!  a file at the path is no directory,
//!  the root has no parent,
//!  a missing path is a refusal naming the read,
//!  and a path that is not text is refused before any call.
//! Why:
//!  the verdict is only as good as this read,
//!  and the three shapes a boot meets (a folder,
//! something else,
//! nothing) each
//!  answer a branch a person acts on.
//!  Every control takes its folder from `fixture`,
//! so two tests share none and
//!  each folder goes when its test thread ends.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(new LiveEntryReader().read(fixture('underlay-entry-live')).directory).toBe(true);
//! ```

use crate::checkout_underlay_entry::underlay_entry_read::{CoveredEntryFacts, EntryReadRefusal, EntryReader, LiveEntryReader};
use crate::test_support::fixture;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[test]
fn a_fixture_folder_reads_as_the_entry_on_its_parent_s_device() {
    let root: PathBuf = fixture("underlay-entry-live-folder");
    let facts: CoveredEntryFacts = LiveEntryReader.read(root.as_path()).expect("the fixture folder reads");
    assert!(facts.directory, "a fixture folder is a directory");
    assert!(facts.same_device_as_parent, "the fixture is not a mount of its own");
    assert_eq!(facts.owner, rustix::process::geteuid().as_raw(), "the test account owns its own fixture");
    assert_eq!(facts.mode & libc::S_IFMT, libc::S_IFDIR, "the mode says directory too");
    assert!(facts.inode != 0, "the kernel reports an inode");
    assert!(
        facts.attributes_reported,
        "the build volume's filesystem reports the immutable attribute, which is the proof this check needs; a filesystem that does not is unproven, which the guest arm covers"
    );
}

#[test]
fn a_file_at_the_path_is_no_directory() {
    let root: PathBuf = fixture("underlay-entry-live-file");
    let file: PathBuf = root.join("a-file");
    std::fs::write(file.as_path(), b"x").expect("the fixture file writes");
    let facts: CoveredEntryFacts = LiveEntryReader.read(file.as_path()).expect("the fixture file reads");
    assert!(!facts.directory, "a file is not the covered folder");
}

#[test]
fn a_symlink_at_the_path_is_no_directory_either() {
    let root: PathBuf = fixture("underlay-entry-live-link");
    let link: PathBuf = root.join("a-link");
    std::os::unix::fs::symlink(root.as_path(), link.as_path()).expect("the fixture link is made");
    let facts: CoveredEntryFacts = LiveEntryReader.read(link.as_path()).expect("the link itself reads");
    assert!(!facts.directory, "the read follows no link at the entry");
}

#[test]
fn the_root_has_no_parent_to_compare_against() {
    let refusal: EntryReadRefusal = LiveEntryReader.read(Path::new("/")).expect_err("a filesystem root has no parent");
    assert!(matches!(refusal, EntryReadRefusal::NoParent { .. }), "the refusal names the shape: {refusal:?}");
    assert!(refusal.message().contains("no parent"), "and says why: {}", refusal.message());
}

#[test]
fn a_path_that_is_not_there_is_a_refusal_naming_the_read() {
    let root: PathBuf = fixture("underlay-entry-live-absent");
    let absent: PathBuf = root.join("absent");
    let refusal: EntryReadRefusal = LiveEntryReader.read(absent.as_path()).expect_err("a missing path is no entry");
    assert!(matches!(refusal, EntryReadRefusal::Call { what: "the entry", .. }), "the refusal names the read: {refusal:?}");
    assert!(refusal.message().contains("unknown"), "an unreadable check is unknown: {}", refusal.message());
}

#[test]
fn a_path_that_is_not_text_is_refused_before_any_call() {
    let refusal: EntryReadRefusal = LiveEntryReader.read(Path::new(OsStr::from_bytes(b"/a\0b"))).expect_err("a NUL ends a C string");
    assert!(matches!(refusal, EntryReadRefusal::PathNotText { .. }));
    assert!(refusal.message().contains("Nothing was attached"), "the refusal says nothing happened: {}", refusal.message());
}
