//! Staging checks against real Git repositories:
//! what the private stage holds and how it is cleaned up.

use super::*;
use crate::parallel_budget::HostResources;
use crate::worktree_copy_manifest::{EntryKind, stamp_of};
use crate::worktree_copy_stage_leaf_copying::*;
use crate::test_support::{fixture, git, git_context, remove, repository};

/// A source with ignored files,
/// a symbolic link,
/// an executable and a nested private-looking name.
fn source_with_ignored_state(parent: &Path) -> PathBuf {
    let root: PathBuf = repository(parent, "source");
    std::fs::write(
        root.join(".gitignore"),
        b"cache/\nlinked\n.cli-git-worktree-copy-*\n",
    )
    .expect("gitignore");
    std::fs::create_dir_all(root.join("cache/deep")).expect("cache");
    std::fs::write(root.join("cache/deep/data"), b"payload").expect("data");
    std::fs::write(root.join("cache/run.sh"), b"#!/bin/sh\n").expect("script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            root.join("cache/run.sh"),
            std::fs::Permissions::from_mode(0o750),
        )
        .expect("mode");
        std::os::unix::fs::symlink("deep/data", root.join("linked")).expect("link");
    }
    std::fs::create_dir_all(root.join(".cli-git-worktree-copy-leftover")).expect("private name");
    std::fs::write(
        root.join(".cli-git-worktree-copy-leftover/file"),
        b"private",
    )
    .expect("private file");
    return root;
}

/// The staged container is private,
/// beside the destination,
/// and holds the payload that mirrors the source.
#[cfg(unix)]
#[test]
fn stage_mirrors_selected_entries_with_modes_and_links() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("stage-mirror");
    let source: PathBuf = source_with_ignored_state(root.as_path());
    let destination: PathBuf = root.join("worktrees").join("topic");
    std::fs::create_dir_all(destination.parent().expect("parent")).expect("worktrees");
    let staged: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(destination.as_path()).expect("container").as_path(),
        &[std::fs::canonicalize(source.as_path()).expect("canonical")],
        &CopySelection::Ignored,
    )
    .expect("staged");
    assert_eq!(staged.stage_container.parent(), destination.parent());
    assert!(
        staged
            .stage_container
            .file_name()
            .expect("name")
            .to_string_lossy()
            .starts_with(STAGE_PREFIX)
    );
    assert_eq!(
        std::fs::metadata(staged.stage_container.as_path())
            .expect("container")
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        std::fs::read(staged.stage_root.join("cache/deep/data")).expect("data"),
        b"payload"
    );
    assert_eq!(
        std::fs::metadata(staged.stage_root.join("cache/run.sh"))
            .expect("script")
            .permissions()
            .mode()
            & 0o777,
        0o750
    );
    assert_eq!(
        std::fs::read_link(staged.stage_root.join("linked")).expect("link"),
        PathBuf::from("deep/data")
    );
    assert!(
        !staged
            .stage_root
            .join(".cli-git-worktree-copy-leftover")
            .exists()
    );
    assert!(
        staged
            .selected_roots
            .iter()
            .all(|path| return !path.starts_with(b".cli-git-worktree-copy-"))
    );
    assert!(
        staged
            .entries
            .iter()
            .any(|entry| return entry.relative_path == b"linked")
    );
    remove(root.as_path());
}

/// A nested registered worktree under the source is excluded from the copy.
#[test]
fn nested_registered_worktree_is_not_copied() {
    let root: PathBuf = fixture("stage-nested");
    let source: PathBuf = repository(root.as_path(), "source");
    std::fs::write(source.join(".gitignore"), b"nested/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("nested")).expect("nested");
    git(
        source.as_path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "inner",
            source.join("nested/wt").to_str().expect("utf8"),
        ],
    );
    std::fs::write(source.join("nested/plain"), b"x").expect("plain");
    let nested_root: PathBuf =
        std::fs::canonicalize(source.join("nested/wt")).expect("canonical nested");
    let destination: PathBuf = root.join("topic");
    let staged: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        std::fs::canonicalize(source.as_path())
            .expect("canonical source")
            .as_path(),
        new_stage_container(destination.as_path()).expect("container").as_path(),
        &[nested_root],
        &CopySelection::Ignored,
    )
    .expect("staged");
    assert!(
        staged
            .entries
            .iter()
            .all(|entry| return !entry.relative_path.starts_with(b"nested/wt"))
    );
    assert!(
        staged
            .entries
            .iter()
            .any(|entry| return entry.relative_path == b"nested/plain")
    );
    remove(root.as_path());
}

/// A source with an ignored FIFO fails the copy,
/// and the private stage is removed so nothing is left beside the destination.
#[cfg(unix)]
#[test]
fn failed_copy_removes_its_stage() {
    let root: PathBuf = fixture("stage-failure");
    let source: PathBuf = repository(root.as_path(), "source");
    std::fs::write(source.join(".gitignore"), b"pipe\n").expect("gitignore");
    let status = std::process::Command::new("mkfifo")
        .arg(source.join("pipe"))
        .status()
        .expect("mkfifo");
    assert!(status.success());
    let destination: PathBuf = root.join("topic");
    let failure: WorktreeCopyError = stage_ignored_snapshot(
        &git_context(),
        std::fs::canonicalize(source.as_path())
            .expect("canonical source")
            .as_path(),
        new_stage_container(destination.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect_err("fifo");
    assert!(
        failure.0.ends_with(" has unsupported filesystem type."),
        "{}",
        failure.0
    );
    let leftovers: Vec<String> = std::fs::read_dir(root.as_path())
        .expect("list")
        .map(|entry| {
            return entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
        })
        .filter(|name| return name.starts_with(STAGE_PREFIX))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    remove(root.as_path());
}

/// Copying bytes yields identical content whether or not the filesystem clones.
#[test]
fn copied_file_has_identical_bytes() {
    let root: PathBuf = fixture("stage-bytes");
    std::fs::write(root.join("source"), vec![3_u8; 200_000]).expect("source");
    copy_file_exclusive(root.join("source").as_path(), root.join("copy").as_path()).expect("copy");
    assert_eq!(
        std::fs::read(root.join("copy")).expect("copy bytes"),
        vec![3_u8; 200_000]
    );
    assert!(
        copy_file_exclusive(root.join("source").as_path(), root.join("copy").as_path()).is_err()
    );
    remove(root.as_path());
}

/// A refused exclusive copy reports the operating system's own error,
///  so a caller acts on the kind instead of reading the message.
#[test]
fn a_refused_exclusive_copy_reports_the_operating_systems_error() {
    let root: PathBuf = fixture("stage-exclusive-kind");
    std::fs::write(root.join("source"), b"payload").expect("source");
    copy_file_exclusive(root.join("source").as_path(), root.join("copy").as_path()).expect("copy");
    let existing: std::io::Error = copy_file_exclusive(
        root.join("source").as_path(),
        root.join("copy").as_path(),
    )
    .expect_err("existing destination");
    assert_eq!(existing.kind(), std::io::ErrorKind::AlreadyExists);
    let missing: std::io::Error = copy_file_exclusive(
        root.join("absent").as_path(),
        root.join("other").as_path(),
    )
    .expect_err("missing source");
    assert_eq!(missing.kind(), std::io::ErrorKind::NotFound);
    remove(root.as_path());
}

/// A private container under a missing parent is never created,
/// and the failure names the cause.
#[test]
fn staging_under_a_missing_parent_fails_with_cause() {
    let root: PathBuf = fixture("stage-missing-parent");
    let source: PathBuf = repository(root.as_path(), "source");
    let destination: PathBuf = root.join("absent").join("topic");
    let failure: WorktreeCopyError = stage_ignored_snapshot(
        &git_context(),
        std::fs::canonicalize(source.as_path())
            .expect("canonical source")
            .as_path(),
        new_stage_container(destination.as_path()).expect("container").as_path(),
        &[],
        &CopySelection::Ignored,
    )
    .expect_err("missing parent");
    assert!(failure.0.contains("No such file or directory"));
    remove(root.as_path());
}

/// A clone request that the filesystem accepts leaves the destination as the clone made it,
///  so no bytes are copied over it.
fn clone_accepts_and_copies_nothing(_source: &std::fs::File, _destination: &std::fs::File) -> bool {
    return true;
}

/// A clone request that the filesystem refuses.
fn clone_refuses(_source: &std::fs::File, _destination: &std::fs::File) -> bool {
    return false;
}

/// The bytes are copied exactly when the clone is refused:
///  an accepted clone is trusted to have filled the file,
/// and a refused one never leaves it empty.
#[test]
fn bytes_are_copied_exactly_when_the_clone_is_refused() {
    let root: PathBuf = fixture("stage-clone-choice");
    std::fs::write(root.join("source"), b"payload").expect("source");
    copy_file_exclusive_with(
        root.join("source").as_path(),
        root.join("accepted").as_path(),
        clone_accepts_and_copies_nothing,
    )
    .expect("accepted");
    assert_eq!(std::fs::read(root.join("accepted")).expect("accepted"), b"");
    copy_file_exclusive_with(
        root.join("source").as_path(),
        root.join("refused").as_path(),
        clone_refuses,
    )
    .expect("refused");
    assert_eq!(std::fs::read(root.join("refused")).expect("refused"), b"payload");
    remove(root.as_path());
}

/// The real clone request says it cloned exactly when the destination holds the source's bytes,
///  on any filesystem:
/// a refusal leaves the new file empty and an acceptance fills it.
#[cfg(target_os = "linux")]
#[test]
fn the_clone_request_reports_what_it_did() {
    let root: PathBuf = fixture("stage-clone-report");
    std::fs::write(root.join("source"), vec![9_u8; 8192]).expect("source");
    let input: std::fs::File = std::fs::File::open(root.join("source")).expect("input");
    let output: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("clone"))
        .expect("output");
    let cloned: bool = try_clone(&input, &output);
    let held: Vec<u8> = std::fs::read(root.join("clone")).expect("clone bytes");
    if cloned {
        assert_eq!(held, vec![9_u8; 8192], "an accepted clone fills the destination");
    } else {
        assert!(held.is_empty(), "a refused clone leaves the destination empty");
    }
    remove(root.as_path());
}

/// Where the filesystem accepts a clone,
/// the clone request accepts it too.
///  The probe asks the kernel directly on a separate pair of files,
/// so the answer does not depend on `try_clone`;
///  on a filesystem without clones (tmpfs,
/// overlay) both sides are refusals and the test shows nothing.
#[cfg(target_os = "linux")]
#[test]
fn the_clone_request_succeeds_where_the_filesystem_clones() {
    use std::os::unix::io::AsRawFd;
    let root: PathBuf = fixture("stage-clone-supported");
    std::fs::write(root.join("source"), vec![5_u8; 8192]).expect("source");
    let pair = |name: &str| -> (std::fs::File, std::fs::File) {
        let input: std::fs::File = std::fs::File::open(root.join("source")).expect("input");
        let output: std::fs::File = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(name))
            .expect("output");
        return (input, output);
    };
    let (probe_input, probe_output) = pair("probe");
    // SAFETY: both descriptors are open for the call, and FICLONE takes the source descriptor as its argument.
    let supported: bool = unsafe {
        libc::ioctl(probe_output.as_raw_fd(), libc::FICLONE, probe_input.as_raw_fd())
    } == 0;
    let (input, output) = pair("clone");
    assert_eq!(try_clone(&input, &output), supported);
    remove(root.as_path());
}

/// Only the roots strictly inside the source are nested,
/// and the source itself and its neighbours are not.
#[test]
fn only_strictly_nested_roots_are_nested() {
    let source: PathBuf = PathBuf::from("/work/source");
    let roots: Vec<PathBuf> = vec![
        PathBuf::from("/work/source"),
        PathBuf::from("/work/source/inner"),
        PathBuf::from("/work/source-neighbour"),
        PathBuf::from("/work/other"),
    ];
    assert_eq!(
        nested_under(source.as_path(), roots.as_slice()),
        vec![PathBuf::from("/work/source/inner")]
    );
}

/// A clone request that the filesystem accepts and that fills the file,
/// so the stamp logic sees a real clone.
fn clone_accepts_and_copies(source: &std::fs::File, destination: &std::fs::File) -> bool {
    use std::io::{Seek, SeekFrom};
    let mut from: &std::fs::File = source;
    let mut to: &std::fs::File = destination;
    from.seek(SeekFrom::Start(0)).expect("rewind");
    std::io::copy(&mut from, &mut to).expect("fill");
    return true;
}

/// A refused clone proves nothing and leaves no stamp;
/// an accepted clone of a settled file returns the source's stamp;
///  an accepted clone of a file changed within the timestamp granularity returns none,
/// so its bytes are compared later.
/// The clock is the control's,
/// pinned to an instant stated against the source's own change time,
/// so the question is answered by the order
///  (change time to clock) and never by how long the host took between the write and the look:
/// a run of this
///  control failed once with `left: Some(FileStamp { .. })` because that gap was more than the 20 ms
///  granularity.
#[cfg(unix)]
#[test]
fn a_stamp_comes_only_from_a_clone_of_a_settled_file() {
    let root: PathBuf = fixture("stage-stamp");
    std::fs::write(root.join("source"), b"payload").expect("source");
    let written: FileStamp = stamp_of(&std::fs::symlink_metadata(root.join("source")).expect("metadata")).expect("a stamp of the source");
    let changed: i128 = stamp_nanos(written.changed);
    // The clock stands at the very instant of the change: nothing about it can prove a write does not follow.
    let at_the_change: ClockPin = clock_reads(changed);
    let recent: Option<FileStamp> =
        copy_file_exclusive_with(root.join("source").as_path(), root.join("recent").as_path(), clone_accepts_and_copies)
            .expect("recent");
    assert_eq!(recent, None, "a file whose change time is the clock's own instant is within the timestamp granularity");
    drop(at_the_change);
    // One nanosecond short of the granularity is still too close.
    let just_short: ClockPin = clock_reads(changed + RECENT_CHANGE_NANOS - 1);
    let still_recent: Option<FileStamp> =
        copy_file_exclusive_with(root.join("source").as_path(), root.join("still-recent").as_path(), clone_accepts_and_copies)
            .expect("still recent");
    assert_eq!(still_recent, None, "a change time one nanosecond inside the granularity proves nothing either");
    drop(just_short);
    // A whole granularity later the stamp proves the bytes.
    let settled_at: ClockPin = clock_reads(changed + RECENT_CHANGE_NANOS);
    let settled: Option<FileStamp> =
        copy_file_exclusive_with(root.join("source").as_path(), root.join("settled").as_path(), clone_accepts_and_copies)
            .expect("settled");
    let source_stamp: Option<FileStamp> = stamp_of(&std::fs::symlink_metadata(root.join("source")).expect("metadata"));
    assert!(settled.is_some());
    assert_eq!(settled, source_stamp);
    drop(settled_at);
    let refused: Option<FileStamp> =
        copy_file_exclusive_with(root.join("source").as_path(), root.join("refused").as_path(), clone_refuses).expect("refused");
    assert_eq!(refused, None);
    assert_eq!(std::fs::read(root.join("refused")).expect("bytes"), b"payload");
    remove(root.as_path());
}

/// A manifest past the threshold is copied by several slices and arrives whole:
/// every file,
/// in every directory.
#[cfg(unix)]
#[test]
fn a_large_manifest_is_copied_in_slices_and_arrives_whole() {
    let root: PathBuf = fixture("stage-parallel");
    let source: PathBuf = repository(root.as_path(), "source");
    std::fs::write(source.join(".gitignore"), b"bulk/\n").expect("gitignore");
    let files: usize = PARALLEL_COPY_MIN_ENTRIES + 25;
    for index in 0..files {
        let directory: PathBuf = source.join(format!("bulk/d{}", index % 7));
        std::fs::create_dir_all(directory.as_path()).expect("directory");
        std::fs::write(directory.join(format!("f{index}")), format!("content {index}")).expect("file");
    }
    assert!(copy_worker_count(files) >= 1);
    let staged: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(root.join("destination").as_path()).expect("container").as_path(),
        &[],
        &CopySelection::MainCheckout,
    )
    .expect("staged");
    let staged_files: usize = staged.entries.iter().filter(|entry| return entry.kind == EntryKind::File).count();
    assert_eq!(staged_files, files);
    for index in [0, 1, files / 2, files - 1] {
        let path: PathBuf = staged.stage_root.join(format!("bulk/d{}/f{index}", index % 7));
        assert_eq!(std::fs::read_to_string(path).expect("staged file"), format!("content {index}"));
    }
    remove(root.as_path());
}

/// The stage records the unreadable entries it left out,
/// and a root that is another checkout is no root of the stage.
#[cfg(unix)]
#[test]
fn unreadable_entries_and_other_checkouts_are_left_out_of_a_main_stage() {
    use std::os::unix::fs::PermissionsExt;
    let root: PathBuf = fixture("stage-left-out");
    let source: PathBuf = repository(root.as_path(), "source");
    std::fs::write(source.join(".gitignore"), b"cache/\nclone/\n").expect("gitignore");
    std::fs::create_dir_all(source.join("cache")).expect("cache");
    std::fs::write(source.join("cache/kept"), b"kept").expect("kept");
    std::fs::write(source.join("cache/hidden"), b"hidden").expect("hidden");
    std::fs::set_permissions(source.join("cache/hidden"), std::fs::Permissions::from_mode(0o000)).expect("mode");
    std::fs::create_dir_all(source.join("clone/.git")).expect("clone");
    std::fs::write(source.join("clone/file"), b"another checkout").expect("clone file");
    // A process allowed to read anything cannot show the case.
    if std::fs::File::open(source.join("cache/hidden")).is_ok() {
        remove(root.as_path());
        return;
    }
    let staged: StagedSnapshot = stage_ignored_snapshot(
        &git_context(),
        source.as_path(),
        new_stage_container(root.join("destination").as_path()).expect("container").as_path(),
        &[],
        &CopySelection::MainCheckout,
    )
    .expect("staged");
    assert_eq!(staged.unreadable, vec![b"cache/hidden".to_vec()]);
    assert_eq!(staged.selected_roots, vec![b"cache".to_vec()], "the clone is no root of the stage");
    assert!(staged.stage_root.join("cache/kept").is_file());
    assert!(!staged.stage_root.join("cache/hidden").exists());
    assert!(!staged.stage_root.join("clone").exists());
    std::fs::set_permissions(source.join("cache/hidden"), std::fs::Permissions::from_mode(0o600)).expect("restore");
    remove(root.as_path());
}

/// A copy of few entries uses one thread;
/// a large one takes the host's budget up to the ceiling,
/// with the margin left free.
#[test]
fn the_copy_workers_follow_the_hosts_budget_up_to_the_ceiling() {
    let small_host: HostResources = HostResources { parallelism: PARALLEL_COPY_MAX_WORKERS, memory_available: None };
    let large_host: HostResources = HostResources { parallelism: PARALLEL_COPY_MAX_WORKERS * 8, memory_available: None };
    assert_eq!(copy_worker_count_on(&large_host, PARALLEL_COPY_MIN_ENTRIES - 1), 1);
    assert_eq!(copy_worker_count_on(&large_host, PARALLEL_COPY_MIN_ENTRIES), PARALLEL_COPY_MAX_WORKERS);
    assert_eq!(copy_worker_count_on(&small_host, PARALLEL_COPY_MIN_ENTRIES), PARALLEL_COPY_MAX_WORKERS - crate::parallel_budget::cpu_margin(PARALLEL_COPY_MAX_WORKERS));
}

/// A copy goes parallel from the measured break-even number of entries and not before.
#[test]
fn the_copy_threshold_is_the_measured_break_even() {
    assert_eq!(PARALLEL_COPY_MIN_ENTRIES, crate::parallel_budget::PARALLEL_BREAK_EVEN_ENTRIES);
}

/// The copy's slices are jobs of the process's blocking pool:
/// a slice asks for the one place of the process's bound before its files are copied,
/// which a slice on a thread of its own never does.
#[test]
fn the_copy_slices_take_places_of_the_processs_bound() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::RecvTimeoutError;
    let (given_back, wanted): (std::sync::Arc<AtomicBool>, std::sync::mpsc::Receiver<bool>) = crate::parallel_test_support::hold_the_only_place();
    let root: PathBuf = fixture("stage-copy-the-bound");
    let source: PathBuf = root.join("source");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(source.join("many")).expect("many");
    std::fs::create_dir_all(stage.join("many")).expect("stage many");
    for index in 0..PARALLEL_COPY_MIN_ENTRIES + 100 {
        std::fs::write(source.join(format!("many/file-{index:05}")), format!("{index}")).expect("file");
    }
    let entries = crate::worktree_copy_manifest::collect_entry_manifest(source.as_path(), &[b"many".to_vec()], &[]).expect("manifest");
    let copied = copy_manifest_entries(source.as_path(), stage.as_path(), entries.as_slice()).expect("every file is copied");
    assert_eq!(copied.stamps.len(), entries.len());
    // The holder gives its place back when a job of the work asks for one, so this answer is the proof that the
    //  copy's slices reached the process's bound instead of starting at once.
    match wanted.recv_timeout(crate::parallel_test_support::PATIENCE) {
        Ok(true) => {}
        Ok(false) => panic!("the job that holds the only place saw no ask, so no slice of the copy took a place of the process's bound"),
        Err(RecvTimeoutError::Timeout) => panic!("no slice of the copy asked for a place of the process's bound within the patience, so the slices did not run as jobs of the blocking pool"),
        Err(RecvTimeoutError::Disconnected) => panic!("the job that holds the only place ended without answering"),
    }
    assert!(given_back.load(Ordering::SeqCst), "the place of the process's bound was never given back to a slice of the copy");
    remove(root.as_path());
}

/// Where the system starts no thread the copy workers' slices are copied in turn and every file arrives with its bytes.
/// The body runs
/// in a child that is refused every thread.
#[test]
fn a_parallel_stage_copy_where_no_thread_can_start_copies_in_turn() {
    if crate::no_threads_child_test_support::ran_in_child(module_path!(), "a_parallel_stage_copy_where_no_thread_can_start_copies_in_turn") {
        return;
    }
    let root: PathBuf = fixture("stage-copy-no-threads");
    let source: PathBuf = root.join("source");
    let stage: PathBuf = root.join("stage");
    std::fs::create_dir_all(source.join("many")).expect("many");
    std::fs::create_dir_all(stage.join("many")).expect("stage many");
    for index in 0..PARALLEL_COPY_MIN_ENTRIES + 100 {
        std::fs::write(source.join(format!("many/file-{index:05}")), format!("{index}")).expect("file");
    }
    let entries = crate::worktree_copy_manifest::collect_entry_manifest(source.as_path(), &[b"many".to_vec()], &[]).expect("manifest");
    let copied = copy_manifest_entries(source.as_path(), stage.as_path(), entries.as_slice()).expect("every file is copied here");
    assert_eq!(copied.stamps.len(), entries.len());
    assert!(copied.gone.is_empty(), "no entry went away here");
    assert_eq!(std::fs::read(stage.join("many/file-00007")).expect("copied"), b"7");
    remove(&root);
}
