//! Publication checks on real filesystem trees:
//! what exists at the destination name while the file is filled,
//! the mode it carries from its first look,
//! and the private name the fallback fills and leaves nothing of.

use super::*;
use crate::test_support::{fixture, remove};
use std::fs::OpenOptions;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::sync::mpsc::channel;

/// A name one copy uses for one entry.
fn temporary() -> OsString {
    return temporary_name_of(".cli-git-worktree-copy-fixture", "secrets/key.pem");
}

/// A named pipe at `path`,
/// so a control can hold a fill open between two of its reads.
fn make_pipe(path: &Path) {
    let text: CString = CString::new(path.as_os_str().as_bytes()).expect("path");
    // SAFETY: the string is NUL-terminated and outlives the call, and a new path is made at that name.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "making the pipe: {}", std::io::Error::last_os_error());
}

/// Whether this process holds a file inside `directory`,
/// which is the proof that a copy has made the file it is filling.
fn filling_a_file_in(directory: &Path) -> bool {
    let Ok(listing) = std::fs::read_dir("/proc/self/fd") else { return false };
    for listed in listing.flatten() {
        let Ok(target) = std::fs::read_link(listed.path()) else { continue };
        if target.parent() == Some(directory) {
            return true;
        }
    }
    return false;
}

/// Wait until this process holds a file inside `directory`,
/// or until the deadline passes.
fn waiting_for_the_fill_of(directory: &Path) -> bool {
    let deadline: std::time::Instant = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::time::Instant::now() < deadline {
        if filling_a_file_in(directory) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    return false;
}

/// A fill held open by a piped source:
/// the writer,
/// the join of the publishing thread,
/// and what it answered.
struct HeldFill {
    /// The pipe's writing end,
    /// which ends the copy when it closes.
    writer: std::fs::File,
    /// The publishing thread.
    publishing: std::thread::JoinHandle<Result<(), WorktreeCopyError>>,
}

/// Start a publication whose source is a pipe at `source`,
/// with the request that makes the file being filled chosen by the control.
fn start_a_held_fill(
    source: &Path,
    destination: &Path,
    mode: u32,
    unnamed: fn(&std::fs::File, u32) -> std::io::Result<std::fs::File>,
) -> HeldFill {
    let (told, heard) = channel::<()>();
    let stage: PathBuf = source.to_path_buf();
    let target: PathBuf = destination.to_path_buf();
    let name: OsString = temporary();
    let publishing: std::thread::JoinHandle<Result<(), WorktreeCopyError>> = std::thread::spawn(move || {
        let _told: Result<(), std::sync::mpsc::SendError<()>> = told.send(());
        return publish_copy_of_file_with(stage.as_path(), target.as_path(), mode, name.as_os_str(), unnamed);
    });
    heard.recv().expect("the publishing thread started");
    let writer: std::fs::File = OpenOptions::new()
        .write(true)
        .open(source)
        .expect("the pipe's writing end");
    return HeldFill { writer, publishing };
}

/// While a staged file is being filled,
/// nothing stands at the destination name,
/// and the file it is filled into is the copy's own,
/// so a kill there leaves nothing for a resumed copy to refuse.
#[test]
fn a_file_being_filled_is_never_readable_under_its_destination_name() {
    let root: PathBuf = fixture("publish-fill-window");
    let stage: PathBuf = root.join("stage.key");
    make_pipe(stage.as_path());
    let destination: PathBuf = root.join("key.pem");
    let mut held: HeldFill = start_a_held_fill(stage.as_path(), destination.as_path(), 0o600, open_unnamed);
    assert!(
        waiting_for_the_fill_of(root.as_path()),
        "the copy must have made the file it fills before this control looks"
    );
    let seen: std::io::Result<std::fs::Metadata> = std::fs::symlink_metadata(destination.as_path());
    held.writer.write_all(b"KEY=credential\n").expect("bytes");
    drop(held.writer);
    let published: Result<(), WorktreeCopyError> = held.publishing.join().expect("the publishing thread");
    assert!(seen.is_err(), "the destination name held a file while the copy was still filling it");
    published.expect("published");
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(destination.as_path()).expect("published");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600, "the entry's own mode, at the first look");
    assert_eq!(std::fs::read(destination.as_path()).expect("bytes"), b"KEY=credential\n");
    assert!(
        !root.join(temporary()).exists(),
        "no file of the copy's own name stays beside the entry"
    );
    remove(root.as_path());
}

/// The private name the fallback fills is never at the destination name either:
/// the same window holds where the filesystem has no unnamed files.
#[test]
fn the_fallback_also_fills_under_a_name_of_its_own() {
    let root: PathBuf = fixture("publish-fallback-window");
    let stage: PathBuf = root.join("stage.key");
    make_pipe(stage.as_path());
    let destination: PathBuf = root.join("key.pem");
    let mut held: HeldFill = start_a_held_fill(stage.as_path(), destination.as_path(), 0o640, unnamed_is_refused);
    assert!(
        waiting_for_the_fill_of(root.as_path()),
        "the copy must have made the file it fills before this control looks"
    );
    let seen: std::io::Result<std::fs::Metadata> = std::fs::symlink_metadata(destination.as_path());
    assert!(
        root.join(temporary()).exists(),
        "the fallback fills the private name the control computed"
    );
    held.writer.write_all(b"KEY=credential\n").expect("bytes");
    drop(held.writer);
    let published: Result<(), WorktreeCopyError> = held.publishing.join().expect("the publishing thread");
    assert!(seen.is_err(), "the destination name held a file while the copy was still filling it");
    published.expect("published");
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(destination.as_path()).expect("published");
    assert_eq!(metadata.permissions().mode() & 0o777, 0o640);
    assert_eq!(std::fs::read(destination.as_path()).expect("bytes"), b"KEY=credential\n");
    assert!(!root.join(temporary()).exists(), "the private name goes with the publication");
    remove(root.as_path());
}

/// A filesystem that refuses the unnamed file,
/// as the fallback's systems do.
fn unnamed_is_refused(_directory: &std::fs::File, _mode: u32) -> std::io::Result<std::fs::File> {
    return Err(std::io::Error::from_raw_os_error(libc::EOPNOTSUPP));
}

/// A temporary a killed run left at the copy's own name is removed before the name is made again,
/// so the resumed publication needs no person to clear it.
#[test]
fn a_temporary_left_by_a_killed_run_is_removed_and_never_published() {
    let root: PathBuf = fixture("publish-fallback-leftover");
    let stage: PathBuf = root.join("stage.key");
    std::fs::write(stage.as_path(), b"KEY=whole\n").expect("stage");
    let left: PathBuf = root.join(temporary());
    std::fs::write(left.as_path(), b"half").expect("leftover");
    let destination: PathBuf = root.join("key.pem");
    publish_copy_of_file_with(stage.as_path(), destination.as_path(), 0o600, temporary().as_os_str(), unnamed_is_refused).expect("published");
    assert_eq!(std::fs::read(destination.as_path()).expect("bytes"), b"KEY=whole\n");
    assert!(!left.exists(), "the name the killed run left is gone");
    remove(root.as_path());
}

/// A publication never replaces an entry that is already there,
/// by the unnamed link or by the fallback's rename that does not replace.
#[test]
fn an_entry_already_at_the_destination_name_is_kept() {
    for (label, unnamed) in [("unnamed", open_unnamed as fn(&std::fs::File, u32) -> std::io::Result<std::fs::File>), ("fallback", unnamed_is_refused)] {
        let root: PathBuf = fixture(format!("publish-existing-{label}").as_str());
        let stage: PathBuf = root.join("stage.key");
        std::fs::write(stage.as_path(), b"KEY=staged\n").expect("stage");
        let destination: PathBuf = root.join("key.pem");
        std::fs::write(destination.as_path(), b"KEY=theirs\n").expect("existing entry");
        let published: Result<(), WorktreeCopyError> = publish_copy_of_file_with(stage.as_path(), destination.as_path(), 0o600, temporary().as_os_str(), unnamed);
        assert!(published.is_err(), "{label}: a publication must not replace an entry");
        assert_eq!(std::fs::read(destination.as_path()).expect("kept"), b"KEY=theirs\n", "{label}");
        assert_eq!(
            std::fs::symlink_metadata(destination.as_path()).expect("kept").permissions().mode() & 0o777,
            0o644,
            "{label}: the existing entry keeps its mode"
        );
        remove(root.as_path());
    }
}

/// A control that has no unnamed file leaves nothing of the copy behind when the publication is refused.
#[test]
fn a_refused_fallback_publication_leaves_no_private_name() {
    let root: PathBuf = fixture("publish-fallback-refused");
    let stage: PathBuf = root.join("stage.key");
    std::fs::write(stage.as_path(), b"KEY=staged\n").expect("stage");
    let destination: PathBuf = root.join("key.pem");
    std::fs::write(destination.as_path(), b"KEY=theirs\n").expect("existing entry");
    let _ = publish_copy_of_file_with(stage.as_path(), destination.as_path(), 0o600, temporary().as_os_str(), unnamed_is_refused);
    assert!(!root.join(temporary()).exists(), "the temporary goes with the refused publication");
    remove(root.as_path());
}

/// A missing staged file fails the publication and leaves no destination entry.
#[test]
fn a_missing_staged_file_leaves_nothing_at_the_destination_name() {
    let root: PathBuf = fixture("publish-missing-stage");
    let destination: PathBuf = root.join("key.pem");
    let published: Result<(), WorktreeCopyError> = publish_copy_of_file(root.join("absent").as_path(), destination.as_path(), 0o600, temporary().as_os_str());
    assert!(published.is_err());
    assert!(std::fs::symlink_metadata(destination.as_path()).is_err());
    assert!(!root.join(temporary()).exists());
    remove(root.as_path());
}

/// The private name names the stage and the entry together,
/// is the same for them on every run,
/// and differs for another stage or another entry.
#[test]
fn the_private_name_names_the_stage_and_the_entry() {
    let first: OsString = temporary_name_of(".cli-git-worktree-copy-one", "cache/a");
    let again: OsString = temporary_name_of(".cli-git-worktree-copy-one", "cache/a");
    let other_entry: OsString = temporary_name_of(".cli-git-worktree-copy-one", "cache/b");
    let other_stage: OsString = temporary_name_of(".cli-git-worktree-copy-two", "cache/a");
    assert_eq!(first, again, "a resumed copy computes the same name");
    assert_ne!(first, other_entry);
    assert_ne!(first, other_stage);
    let text: &str = first.to_str().expect("a name of text");
    assert!(text.starts_with(TEMPORARY_PREFIX));
    assert!(!text.contains('/'), "the name is one component");
    assert!(text.len() <= 255, "a name fits in NAME_MAX");
}

/// The name is the one this release computes for that stage and entry,
/// so a later run,
/// and a run of another release that computes it again,
/// removes the same name and no other file.
#[test]
fn the_private_name_is_the_one_this_release_computes() {
    assert_eq!(
        temporary_name_of(".cli-git-worktree-copy-one", "cache/a").to_str().expect("text"),
        ".monochromatic-git-policy-cli-partial-66f6f876576d64a7"
    );
}

/// A published file is its own object,
/// never a second name for the staged one.
#[test]
fn a_published_file_shares_no_storage_with_the_stage() {
    let root: PathBuf = fixture("publish-storage");
    let stage: PathBuf = root.join("stage.key");
    std::fs::write(stage.as_path(), b"KEY=staged\n").expect("stage");
    let destination: PathBuf = root.join("key.pem");
    publish_copy_of_file_with(stage.as_path(), destination.as_path(), 0o600, temporary().as_os_str(), unnamed_is_refused).expect("published");
    assert_ne!(
        std::fs::symlink_metadata(stage.as_path()).expect("stage").ino(),
        std::fs::symlink_metadata(destination.as_path()).expect("published").ino(),
        "the copy is its own object"
    );
    remove(root.as_path());
}
