//! Entry comparison and source-stability checks on real filesystem trees.

use super::*;
use crate::test_support::{fixture, remove};

/// A file length that spans two comparison chunks with a short last chunk.
const LARGE_LENGTH: usize = 64 * 1_024 + 17;

/// One manifest entry for a path and kind,
/// with the mode the path currently has.
fn entry_of(root: &Path, kind: EntryKind, relative_path: &[u8]) -> WorktreeCopyEntry {
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(
        root.join(crate::path_bytes::os_from_path_bytes(relative_path)),
    )
    .expect("metadata");
    return WorktreeCopyEntry {
        kind,
        mode: retained_mode(&metadata),
        relative_path: relative_path.to_vec(),
    };
}

/// A missing path reads as absent and a present one reads as its metadata.
#[test]
fn lstat_distinguishes_absence_from_presence() {
    let root: PathBuf = fixture("compare-lstat");
    assert!(
        lstat_or_absent(root.join("missing").as_path())
            .expect("absent")
            .is_none()
    );
    std::fs::write(root.join("present"), b"x").expect("present");
    assert!(
        lstat_or_absent(root.join("present").as_path())
            .expect("present")
            .is_some()
    );
    remove(root.as_path());
}

/// An absent destination never matches,
/// whatever the expected entry.
#[test]
fn absent_entry_never_matches() {
    let root: PathBuf = fixture("compare-absent");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir(stage.as_path()).expect("stage");
    std::fs::write(stage.join("file"), b"x").expect("staged");
    let target: PathBuf = root.join("target");
    std::fs::create_dir(target.as_path()).expect("target");
    let staged: WorktreeCopyEntry = entry_of(stage.as_path(), EntryKind::File, b"file");
    assert!(!entry_matches(stage.as_path(), target.as_path(), &staged).expect("compare"));
    remove(root.as_path());
}

/// A directory matches a directory with the same mode and nothing else;
/// a file never matches a directory.
#[test]
fn directory_entries_compare_kind_and_mode() {
    let root: PathBuf = fixture("compare-directory");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("dir")).expect("stage dir");
    let target: PathBuf = root.join("target");
    std::fs::create_dir_all(target.join("dir")).expect("target dir");
    let staged: WorktreeCopyEntry = entry_of(stage.as_path(), EntryKind::Directory, b"dir");
    assert!(entry_matches(stage.as_path(), target.as_path(), &staged).expect("same"));
    std::fs::remove_dir(target.join("dir")).expect("replace");
    std::fs::write(target.join("dir"), b"file").expect("file in place of directory");
    assert!(!entry_matches(stage.as_path(), target.as_path(), &staged).expect("kind"));
    remove(root.as_path());
}

/// A file matches only identical bytes with the same mode;
/// a different size,
/// a different byte,
/// or a different mode does not.
#[cfg(unix)]
#[test]
fn file_entries_compare_bytes_and_mode() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("compare-file");
    let stage: PathBuf = root.join("stage");
    let target: PathBuf = root.join("target");
    std::fs::create_dir_all(stage.as_path()).expect("stage");
    std::fs::create_dir_all(target.as_path()).expect("target");
    std::fs::write(stage.join("file"), b"same bytes").expect("staged");
    std::fs::write(target.join("file"), b"same bytes").expect("target");
    let staged: WorktreeCopyEntry = entry_of(stage.as_path(), EntryKind::File, b"file");
    assert!(entry_matches(stage.as_path(), target.as_path(), &staged).expect("equal"));
    std::fs::write(target.join("file"), b"same bytez").expect("one byte differs");
    assert!(!entry_matches(stage.as_path(), target.as_path(), &staged).expect("byte"));
    std::fs::write(target.join("file"), b"longer than staged").expect("size differs");
    assert!(!entry_matches(stage.as_path(), target.as_path(), &staged).expect("size"));
    std::fs::write(target.join("file"), b"same bytes").expect("restore bytes");
    std::fs::set_permissions(target.join("file"), std::fs::Permissions::from_mode(0o600))
        .expect("mode");
    std::fs::set_permissions(stage.join("file"), std::fs::Permissions::from_mode(0o644))
        .expect("staged mode");
    let staged_with_mode: WorktreeCopyEntry = entry_of(stage.as_path(), EntryKind::File, b"file");
    assert!(!entry_matches(stage.as_path(), target.as_path(), &staged_with_mode).expect("mode"));
    remove(root.as_path());
}

/// Files longer than one comparison chunk are compared chunk by chunk,
/// and a difference in the last chunk is found.
#[test]
fn large_files_are_compared_across_chunks() {
    let root: PathBuf = fixture("compare-large");
    let stage: PathBuf = root.join("stage");
    let target: PathBuf = root.join("target");
    std::fs::create_dir_all(stage.as_path()).expect("stage");
    std::fs::create_dir_all(target.as_path()).expect("target");
    let mut bytes: Vec<u8> = vec![7; LARGE_LENGTH];
    std::fs::write(stage.join("big"), bytes.as_slice()).expect("staged");
    std::fs::write(target.join("big"), bytes.as_slice()).expect("target");
    let staged: WorktreeCopyEntry = entry_of(stage.as_path(), EntryKind::File, b"big");
    assert!(entry_matches(stage.as_path(), target.as_path(), &staged).expect("equal"));
    let last: usize = bytes.len() - 1;
    bytes[last] = 8;
    std::fs::write(target.join("big"), bytes.as_slice()).expect("last byte");
    assert!(!entry_matches(stage.as_path(), target.as_path(), &staged).expect("last chunk"));
    remove(root.as_path());
}

/// Symbolic links match on exact target text,
/// not on what they resolve to.
#[cfg(unix)]
#[test]
fn symbolic_links_compare_target_text() {
    let root: PathBuf = fixture("compare-link");
    let stage: PathBuf = root.join("stage");
    let target: PathBuf = root.join("target");
    std::fs::create_dir_all(stage.as_path()).expect("stage");
    std::fs::create_dir_all(target.as_path()).expect("target");
    std::os::unix::fs::symlink("../shared", stage.join("link")).expect("staged link");
    std::os::unix::fs::symlink("../shared", target.join("link")).expect("same link");
    let staged: WorktreeCopyEntry = entry_of(stage.as_path(), EntryKind::Symlink, b"link");
    assert!(entry_matches(stage.as_path(), target.as_path(), &staged).expect("same target"));
    std::fs::remove_file(target.join("link")).expect("remove");
    std::os::unix::fs::symlink("../other", target.join("link")).expect("other link");
    assert!(!entry_matches(stage.as_path(), target.as_path(), &staged).expect("other target"));
    std::fs::remove_file(target.join("link")).expect("remove again");
    std::fs::write(target.join("link"), b"plain").expect("plain file");
    assert!(!entry_matches(stage.as_path(), target.as_path(), &staged).expect("not a link"));
    remove(root.as_path());
}

/// A source that still matches the stage passes the final check.
#[test]
fn unchanged_source_passes_final_check() {
    let root: PathBuf = fixture("compare-final-same");
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    std::fs::write(source.join("cache/file"), b"x").expect("file");
    let selected: Vec<Vec<u8>> = vec![b"cache".to_vec()];
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(source.as_path(), selected.as_slice(), &[]).expect("manifest");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("cache")).expect("stage cache");
    std::fs::write(stage.join("cache/file"), b"x").expect("stage file");
    assert_final_source_equivalence(
        source.as_path(),
        stage.as_path(),
        selected.as_slice(),
        &[],
        entries.as_slice(),
    )
    .expect("unchanged");
    remove(root.as_path());
}

/// A source file that changed after the manifest blocks the destination with the changed entry named.
#[test]
fn changed_source_entry_fails_final_check() {
    let root: PathBuf = fixture("compare-final-changed");
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    std::fs::write(source.join("cache/file"), b"x").expect("file");
    let selected: Vec<Vec<u8>> = vec![b"cache".to_vec()];
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(source.as_path(), selected.as_slice(), &[]).expect("manifest");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("cache")).expect("stage cache");
    std::fs::write(stage.join("cache/file"), b"x").expect("stage file");
    std::fs::write(source.join("cache/file"), b"y").expect("source changed");
    let failure: WorktreeCopyError = assert_final_source_equivalence(
        source.as_path(),
        stage.as_path(),
        selected.as_slice(),
        &[],
        entries.as_slice(),
    )
    .expect_err("changed");
    assert_eq!(
        failure.0,
        "monochromatic-git-policy-cli: ignored source entry changed while copying: \"cache/file\"."
    );
    remove(root.as_path());
}

/// A source that gained an entry after the manifest blocks the destination by count.
#[test]
fn added_source_entry_fails_final_check_by_count() {
    let root: PathBuf = fixture("compare-final-count");
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    std::fs::write(source.join("cache/file"), b"x").expect("file");
    let selected: Vec<Vec<u8>> = vec![b"cache".to_vec()];
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(source.as_path(), selected.as_slice(), &[]).expect("manifest");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("cache")).expect("stage cache");
    std::fs::write(stage.join("cache/file"), b"x").expect("stage file");
    std::fs::write(source.join("cache/added"), b"new").expect("added");
    let failure: WorktreeCopyError = assert_final_source_equivalence(
        source.as_path(),
        stage.as_path(),
        selected.as_slice(),
        &[],
        entries.as_slice(),
    )
    .expect_err("count");
    assert_eq!(
        failure.0,
        "monochromatic-git-policy-cli: ignored source paths changed while snapshot was staged."
    );
    remove(root.as_path());
}

/// The source and stage of one cached file,
/// the manifest of the source,
/// and the stamp of the source file.
#[cfg(unix)]
fn stamped_pair(name: &str, source_bytes: &[u8], stage_bytes: &[u8]) -> (PathBuf, PathBuf, PathBuf, Vec<WorktreeCopyEntry>, FileStamp) {
    let root: PathBuf = fixture(name);
    let source: PathBuf = root.join("source");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    std::fs::write(source.join("cache/file"), source_bytes).expect("file");
    let entries: Vec<WorktreeCopyEntry> =
        collect_entry_manifest(source.as_path(), &[b"cache".to_vec()], &[]).expect("manifest");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(stage.join("cache")).expect("stage cache");
    std::fs::write(stage.join("cache/file"), stage_bytes).expect("stage file");
    let stamp: FileStamp = stamp_of(&std::fs::symlink_metadata(source.join("cache/file")).expect("metadata")).expect("stamp");
    return (root, source, stage, entries, stamp);
}

/// The final check for a cached file,
/// given the stamps recorded around its clone.
#[cfg(unix)]
fn final_check(source: &Path, stage: &Path, entries: &[WorktreeCopyEntry], stamps: &[Option<FileStamp>]) -> Result<(), WorktreeCopyError> {
    return assert_final_source_equivalence_pruned(
        source,
        stage,
        &[b"cache".to_vec()],
        &[],
        entries,
        &FinalCheck { prune: Prune::Never, stamps },
    );
}

/// A file whose source stamp is unchanged is proved by the stamp:
/// its bytes are not read again,
/// so a stage that differs in
/// content (which a clone cannot) passes,
/// while the same pair without a stamp fails by bytes.
#[cfg(unix)]
#[test]
fn an_unchanged_stamp_stands_in_for_reading_the_bytes() {
    let (root, source, stage, entries, stamp) = stamped_pair("compare-stamp-trusted", b"same", b"diff");
    let stamps: Vec<Option<FileStamp>> = entries.iter().map(|entry| return if entry.kind == EntryKind::File { Some(stamp) } else { None }).collect();
    final_check(source.as_path(), stage.as_path(), entries.as_slice(), stamps.as_slice()).expect("the stamp is unchanged");
    let unstamped: Vec<Option<FileStamp>> = vec![None; entries.len()];
    let failure: WorktreeCopyError = final_check(source.as_path(), stage.as_path(), entries.as_slice(), unstamped.as_slice()).expect_err("bytes differ");
    assert_eq!(failure.0, "monochromatic-git-policy-cli: ignored source entry changed while copying: \"cache/file\".");
    remove(root.as_path());
}

/// A write to the source after the stamp was recorded fails the final check,
/// even when the stage still holds the old bytes.
#[cfg(unix)]
#[test]
fn a_source_written_after_the_stamp_fails_the_final_check() {
    let (root, source, stage, entries, stamp) = stamped_pair("compare-stamp-written", b"first", b"first");
    let stamps: Vec<Option<FileStamp>> = entries.iter().map(|entry| return if entry.kind == EntryKind::File { Some(stamp) } else { None }).collect();
    final_check(source.as_path(), stage.as_path(), entries.as_slice(), stamps.as_slice()).expect("unchanged before the write");
    // Same size, so only the times can tell; the change time moves on every write.
    std::fs::write(source.join("cache/file"), b"other").expect("write");
    let failure: WorktreeCopyError = final_check(source.as_path(), stage.as_path(), entries.as_slice(), stamps.as_slice()).expect_err("written");
    assert_eq!(failure.0, "monochromatic-git-policy-cli: ignored source entry changed while copying: \"cache/file\".");
    remove(root.as_path());
}

/// A source file replaced by a rename carries a different inode,
/// so its old stamp no longer matches.
#[cfg(unix)]
#[test]
fn a_replaced_source_file_fails_the_final_check() {
    let (root, source, stage, entries, stamp) = stamped_pair("compare-stamp-replaced", b"first", b"first");
    let stamps: Vec<Option<FileStamp>> = entries.iter().map(|entry| return if entry.kind == EntryKind::File { Some(stamp) } else { None }).collect();
    std::fs::write(source.join("cache/replacement"), b"first").expect("replacement");
    std::fs::rename(source.join("cache/replacement"), source.join("cache/file")).expect("rename over");
    assert!(final_check(source.as_path(), stage.as_path(), entries.as_slice(), stamps.as_slice()).is_err());
    remove(root.as_path());
}

/// Where the system starts no thread the two manifest walks run one after the other and the final check still passes.
/// The body runs in
/// a child that is refused every thread.
#[test]
fn the_final_check_where_no_thread_can_start_walks_in_turn() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "the_final_check_where_no_thread_can_start_walks_in_turn") {
        return;
    }
    unchanged_source_passes_final_check();
}
