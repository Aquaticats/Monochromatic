//! The conversion of a linked worktree into a subvolume on the stand-in platform (any filesystem): the whole sequence, every
//! refusal, a kill at every phase followed by a rerun, the abandon, and the guarantee that nothing foreign is deleted.

use super::*;
use crate::subvolume_test_support::{Hooked, PlainPlatform, backdate, checkout_with_worktree, index_inode};
use crate::test_support::{git, git_text, remove};
use std::os::unix::fs::MetadataExt;

/// The options of a real run.
const RUN: ConvertOptions = ConvertOptions { dry_run: false, abandon: false, command_line_attributes: false };

/// A checkout with one linked worktree whose tracked files are old and whose index is fresh, so it is quiet.
fn quiet_worktree(name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let (parent, root) = checkout_with_worktree(name);
    let worktree: PathBuf = root.join("wt");
    backdate(worktree.as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    return (parent, root, std::fs::canonicalize(worktree).expect("canonical worktree"));
}

/// Run one conversion and collect its lines.
fn run(platform: &dyn Platform, worktree: &Path, options: ConvertOptions, interrupt: &Interrupt) -> (Result<Verdict, MigrateError>, Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let verdict = convert_worktree(platform, worktree, options, interrupt, &mut |text: &str| { lines.push(String::from(text)); });
    return (verdict, lines);
}

/// Run one conversion and ignore its lines.
fn run_quiet(platform: &dyn Platform, worktree: &Path, options: ConvertOptions, interrupt: &Interrupt) -> Result<Verdict, MigrateError> {
    return run(platform, worktree, options, interrupt).0;
}

/// The administrative directory of a worktree.
fn admin_of(worktree: &Path) -> PathBuf {
    return inspect(worktree).expect("registered").admin;
}

/// Nothing of the conversion is left: no converting directory, no journal, no lock of its token.
fn assert_nothing_left(worktree: &Path) {
    assert!(std::fs::symlink_metadata(converting_path(worktree)).is_err(), "the converting directory is gone");
    let admin: PathBuf = admin_of(worktree);
    assert!(!admin.join(JOURNAL_NAME).exists(), "the journal is gone");
    assert!(!admin.join("index.lock").exists(), "the lock is released");
}

/// The state every successful conversion ends in: a subvolume at the same path with the same bytes, a clean quick status, a
///  registration Git still lists, and an index that describes the new files.
fn assert_converted(platform: &PlainPlatform, root: &Path, worktree: &Path, git_file: &[u8]) {
    assert!(platform.is_subvolume_root(worktree).expect("examined"));
    assert_eq!(std::fs::read(worktree.join(".git")).expect(".git"), git_file, "the .git file is byte for byte the same");
    assert_eq!(std::fs::read(worktree.join("README.md")).expect("readme"), b"readme\n");
    assert_eq!(std::fs::read(worktree.join("src/deep/er/leaf.txt")).expect("leaf").len(), 70_000);
    assert_eq!(std::fs::read_link(worktree.join("link")).expect("link"), PathBuf::from("src/main.rs"));
    assert_eq!(std::fs::metadata(worktree.join("script.sh")).expect("script").mode() & 0o777, 0o755);
    assert_eq!(git_text(worktree, &["status", "--porcelain"]), "", "Git sees a clean worktree");
    assert!(git_text(root, &["worktree", "list", "--porcelain"]).contains(worktree.to_string_lossy().as_ref()), "still registered");
    assert_eq!(index_inode(worktree, "src/main.rs"), std::fs::metadata(worktree.join("src/main.rs")).expect("main").ino(), "the index describes the new file");
    assert_nothing_left(worktree);
}

/// A worktree is converted from start to end, and everything about it survives.
#[test]
fn a_worktree_is_converted_in_place() {
    let (parent, root, worktree) = quiet_worktree("convert-whole");
    let platform: PlainPlatform = PlainPlatform::new();
    let git_file: Vec<u8> = std::fs::read(worktree.join(".git")).expect(".git");
    let old_inode: u64 = std::fs::metadata(&worktree).expect("old").ino();
    std::fs::write(parent.join("foreign"), b"keep").expect("foreign file");
    let (verdict, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(verdict.expect("converted"), Verdict::Converted, "{lines:#?}");
    assert_converted(&platform, root.as_path(), worktree.as_path(), git_file.as_slice());
    assert_ne!(std::fs::metadata(&worktree).expect("new").ino(), old_inode, "the path names a new directory");
    assert_eq!(std::fs::read(parent.join("foreign")).expect("foreign"), b"keep", "nothing foreign was touched");
    // A second run finds the subvolume and does nothing.
    let again = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(again.expect("skipped"), Verdict::AlreadySubvolume);
    remove(parent.as_path());
}

/// A dry run changes nothing at all, and says it would convert.
#[test]
fn a_dry_run_changes_nothing() {
    let (parent, _root, worktree) = quiet_worktree("convert-dry");
    let platform: PlainPlatform = PlainPlatform::new();
    let verdict = run_quiet(&platform, worktree.as_path(), ConvertOptions { dry_run: true, abandon: false, command_line_attributes: false }, &Interrupt::never());
    assert_eq!(verdict.expect("dry"), Verdict::WouldConvert);
    assert!(!platform.is_subvolume_root(worktree.as_path()).expect("examined"));
    assert!(!admin_of(worktree.as_path()).join(JOURNAL_NAME).exists());
    assert!(!admin_of(worktree.as_path()).join("index.lock").exists());
    assert!(std::fs::symlink_metadata(converting_path(worktree.as_path())).is_err());
    remove(parent.as_path());
}

/// The reasons the verdict lists when something stands in the way, and the worktree is untouched.
fn skipped_with(verdict: Result<Verdict, MigrateError>, fragment: &str) {
    match verdict.expect("no failure") {
        Verdict::Skipped(reasons) => assert!(reasons.iter().any(|reason: &String| { return reason.contains(fragment); }), "{reasons:?} should mention {fragment}"),
        other => panic!("expected a skip mentioning {fragment}, got {other:?}"),
    }
}

/// Every refusal is reported, with the worktree untouched: locked, modified, an index lock, a foreign converting directory, a
///  process inside, and a worktree registered inside.
#[test]
fn each_refusal_leaves_the_worktree_untouched() {
    let (parent, root, worktree) = quiet_worktree("convert-refusals");
    let platform: PlainPlatform = PlainPlatform::new();
    let admin: PathBuf = admin_of(worktree.as_path());
    let untouched = |checked: &PlainPlatform| {
        assert!(!checked.is_subvolume_root(worktree.as_path()).expect("examined"));
        assert!(!admin.join(JOURNAL_NAME).exists(), "no journal was written");
    };
    // Locked.
    git(root.as_path(), &["worktree", "lock", worktree.to_str().expect("utf8")]);
    skipped_with(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0, "locked");
    git(root.as_path(), &["worktree", "unlock", worktree.to_str().expect("utf8")]);
    untouched(&platform);
    // A foreign index lock.
    std::fs::write(admin.join("index.lock"), b"another git").expect("lock");
    skipped_with(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0, "index.lock");
    assert_eq!(std::fs::read(admin.join("index.lock")).expect("lock"), b"another git", "the foreign lock is not removed");
    std::fs::remove_file(admin.join("index.lock")).expect("remove lock");
    // A foreign converting directory.
    std::fs::create_dir(converting_path(worktree.as_path())).expect("foreign directory");
    std::fs::write(converting_path(worktree.as_path()).join("theirs"), b"x").expect("theirs");
    skipped_with(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0, CONVERTING_SUFFIX);
    assert!(converting_path(worktree.as_path()).join("theirs").exists(), "the foreign directory is left alone");
    std::fs::remove_dir_all(converting_path(worktree.as_path())).expect("remove foreign");
    // A process with its working directory inside.
    let mut child: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("30").current_dir(&worktree)).expect("sleep");
    skipped_with(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0, "working directory or an open file");
    child.kill().expect("kill");
    child.wait().expect("wait");
    // A modified tracked file.
    std::fs::write(worktree.join("README.md"), b"changed\n").expect("modify");
    skipped_with(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0, "not quiet and unmodified");
    untouched(&platform);
    // A worktree registered inside it.
    std::fs::write(worktree.join("README.md"), b"readme\n").expect("restore");
    backdate(worktree.as_path());
    git(worktree.as_path(), &["update-index", "--refresh"]);
    git(worktree.as_path(), &["worktree", "add", "--quiet", "--detach", "inner"]);
    skipped_with(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0, "registered inside it");
    untouched(&platform);
    remove(parent.as_path());
}

/// A kill at every phase is finished by running the command again, with the same final state and nothing foreign deleted.
#[test]
fn a_kill_at_every_phase_is_finished_by_a_rerun() {
    for point in ["journaled", "created", "verified", "exchanged", "refreshed", "removed"] {
        let (parent, root, worktree) = quiet_worktree(format!("convert-kill-{point}").as_str());
        let platform: PlainPlatform = PlainPlatform::new();
        let git_file: Vec<u8> = std::fs::read(worktree.join(".git")).expect(".git");
        std::fs::write(parent.join("foreign"), b"keep").expect("foreign file");
        let killed = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at(point, 0));
        assert_eq!(killed.expect_err("killed"), MigrateError::Interrupted(String::from(point)), "{point}");
        let (resumed, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
        assert_eq!(resumed.expect("finished"), Verdict::Resumed, "{point}: {lines:#?}");
        assert_converted(&platform, root.as_path(), worktree.as_path(), git_file.as_slice());
        assert_eq!(std::fs::read(parent.join("foreign")).expect("foreign"), b"keep", "{point}");
        remove(parent.as_path());
    }
}

/// Abandoning before the swap destroys only the subvolume this command created and leaves the original whole; after the swap
///  there is nothing to undo, and a rerun finishes.
#[test]
fn abandon_undoes_before_the_swap_and_refuses_after() {
    let (parent, _root, worktree) = quiet_worktree("convert-abandon");
    let platform: PlainPlatform = PlainPlatform::new();
    let killed = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("verified", 0));
    assert!(killed.is_err());
    assert!(std::fs::symlink_metadata(converting_path(worktree.as_path())).is_ok(), "the copy exists");
    let undone = run_quiet(&platform, worktree.as_path(), ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false }, &Interrupt::never());
    assert_eq!(undone.expect("abandoned"), Verdict::Abandoned);
    assert_nothing_left(worktree.as_path());
    assert!(!platform.is_subvolume_root(worktree.as_path()).expect("examined"));
    assert_eq!(std::fs::read(worktree.join("README.md")).expect("original intact"), b"readme\n");
    // Nothing to abandon without a journal.
    let nothing = run_quiet(&platform, worktree.as_path(), ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false }, &Interrupt::never());
    skipped_with(nothing, "nothing to abandon");
    // Past the swap the abandon refuses and deletes nothing.
    let past = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0));
    assert!(past.is_err());
    let refused = run_quiet(&platform, worktree.as_path(), ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false }, &Interrupt::never());
    skipped_with(refused, "past the swap");
    assert!(std::fs::symlink_metadata(converting_path(worktree.as_path())).is_ok(), "the old tree is still there");
    let finished = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(finished.expect("finished"), Verdict::Resumed);
    remove(parent.as_path());
}

/// The final removal deletes the directory the journal recorded and nothing else: a different directory at the converting
///  name is left alone, the run says it is not finished, and the journal stays so that the next run can finish it.
#[test]
fn a_directory_that_is_not_the_recorded_old_tree_is_never_removed() {
    let (parent, _root, worktree) = quiet_worktree("convert-foreign-old");
    let platform: PlainPlatform = PlainPlatform::new();
    let killed = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("refreshed", 0));
    assert!(killed.is_err());
    let converting: PathBuf = converting_path(worktree.as_path());
    let kept: PathBuf = parent.join("old-tree-kept");
    std::fs::rename(&converting, &kept).expect("move the old tree away");
    std::fs::create_dir(&converting).expect("foreign directory at the name");
    std::fs::write(converting.join("theirs"), b"x").expect("theirs");
    let (finished, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    let message: String = finished.expect_err("the conversion is not finished").to_string();
    assert!(message.contains("not the old tree") && message.contains("journal"), "{message}: {lines:#?}");
    assert!(converting.join("theirs").exists(), "the foreign directory is untouched");
    assert!(kept.join("README.md").exists(), "the old tree is untouched");
    assert!(admin_of(worktree.as_path()).join(JOURNAL_NAME).exists(), "the journal stays");
    std::fs::remove_dir_all(&converting).expect("the person clears the name");
    std::fs::rename(&kept, &converting).expect("and puts the old tree back");
    let (completed, completed_lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(completed.expect("finished"), Verdict::Resumed, "{completed_lines:#?}");
    remove(parent.as_path());
}

/// A process that appears after the copy was made stops the swap on the rerun, and nothing is swapped.
#[test]
fn a_process_inside_stops_the_swap_of_a_resumed_conversion() {
    let (parent, _root, worktree) = quiet_worktree("convert-process");
    let platform: PlainPlatform = PlainPlatform::new();
    let killed = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("verified", 0));
    assert!(killed.is_err());
    let mut child: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("30").current_dir(&worktree)).expect("sleep");
    skipped_with(run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0, "working directory or an open file");
    assert!(!platform.is_subvolume_root(worktree.as_path()).expect("examined"), "nothing was swapped");
    child.kill().expect("kill");
    child.wait().expect("wait");
    let finished = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(finished.expect("finished"), Verdict::Resumed);
    remove(parent.as_path());
}

/// A file that changes while the copy is made is caught by the proving pass, and nothing is swapped.
#[test]
fn a_change_during_the_copy_is_caught_before_the_swap() {
    let (parent, _root, worktree) = quiet_worktree("convert-change");
    let watched: PathBuf = worktree.clone();
    let platform: Hooked = Hooked::new(Box::new(move |call: u32| {
        // After the first few files were copied, rewrite every tracked file of the source with different content.
        if call == 3 {
            for name in ["README.md", "src/main.rs", "script.sh", "empty"] {
                std::fs::write(watched.join(name), b"rewritten while copying\n").expect("rewrite");
            }
        }
    }));
    let verdict = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::never());
    let message: String = verdict.expect_err("the proving pass refuses").to_string();
    assert!(message.contains("changed while it was copied"), "{message}");
    assert!(!platform.is_subvolume_root(worktree.as_path()).expect("examined"), "nothing was swapped");
    remove(parent.as_path());
}

/// On a real btrfs filesystem the same conversion creates a real subvolume (root inode 256), clones the file data, keeps the
///  registration and the index fast; elsewhere the control prints its one loud line and returns.
#[test]
fn a_real_conversion_makes_a_subvolume_and_clones_file_data() {
    use crate::fs_btrfs_test_support::btrfs_fixture;
    use crate::subvolume_platform::LinuxPlatform;
    let Some(probe) = btrfs_fixture("convert-real") else { return };
    remove(probe.as_path());
    let (parent, root, worktree) = quiet_worktree("convert-real");
    let git_file: Vec<u8> = std::fs::read(worktree.join(".git")).expect(".git");
    let (verdict, lines) = run(&LinuxPlatform::default(), worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(verdict.expect("converted"), Verdict::Converted, "{lines:#?}");
    assert_eq!(std::fs::metadata(&worktree).expect("new").ino(), crate::fs_kind::BTRFS_SUBVOLUME_ROOT_INODE);
    let copied: &String = lines.iter().find(|line: &&String| { return line.contains("copied "); }).expect("the copy line");
    assert!(!copied.contains(", 0 cloned"), "file data was cloned: {copied}");
    assert_eq!(std::fs::read(worktree.join(".git")).expect(".git"), git_file);
    assert_eq!(git_text(worktree.as_path(), &["status", "--porcelain"]), "");
    assert!(git_text(root.as_path(), &["worktree", "list", "--porcelain"]).contains(worktree.to_string_lossy().as_ref()));
    assert_nothing_left(worktree.as_path());
    remove(parent.as_path());
}
