//! The conversion of a linked worktree into a subvolume on the stand-in platform (any filesystem):
//! the whole sequence,
//! every
//! refusal,
//! a kill at every phase followed by a rerun,
//! the abandon,
//! and the guarantee that nothing foreign is deleted.

use super::*;
use crate::subvolume_test_support::{Hooked, PlainPlatform, backdate, checkout_moved_behind_a_link, checkout_with_worktree, index_inode};
use crate::test_support::{git, git_text, remove};
use std::os::unix::fs::MetadataExt;

/// The options of a real run.
const RUN: ConvertOptions = ConvertOptions { dry_run: false, abandon: false, command_line_attributes: false };

/// A checkout with one linked worktree whose tracked files are old and whose index is fresh,
/// so it is quiet.
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

/// Nothing of the conversion is left:
/// no converting directory,
/// no journal,
/// no lock of its token.
fn assert_nothing_left(worktree: &Path) {
    assert!(std::fs::symlink_metadata(converting_path(worktree)).is_err(), "the converting directory is gone");
    let admin: PathBuf = admin_of(worktree);
    assert!(!admin.join(JOURNAL_NAME).exists(), "the journal is gone");
    assert!(!admin.join("index.lock").exists(), "the lock is released");
}

/// The state every successful conversion ends in:
/// a subvolume at the same path with the same bytes,
/// a clean quick status,
/// a
///  registration Git still lists,
/// and an index that describes the new files.
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

/// A worktree is converted from start to end,
/// and everything about it survives.
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

/// A dry run changes nothing at all,
/// and says it would convert.
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

/// A registration that spells the worktree through a symbolic link names the same worktree,
/// so the conversion runs:
/// after a repository has moved behind a link,
/// every registration still spells the old name.
#[test]
fn a_registration_spelled_through_a_link_is_the_same_worktree() {
    let (parent, _root, worktree) = checkout_moved_behind_a_link("convert-link-registration");
    let platform: PlainPlatform = PlainPlatform::new();
    let candidate: Candidate = inspect(worktree.as_path()).expect("the two spellings are one file");
    assert_eq!(candidate.path, worktree);
    let (verdict, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(verdict.expect("converted"), Verdict::Converted, "{lines:#?}");
    assert!(platform.is_subvolume_root(worktree.as_path()).expect("examined"));
    assert_eq!(std::fs::read(worktree.join("README.md")).expect("readme"), b"readme\n");
    assert_eq!(git_text(worktree.as_path(), &["status", "--porcelain"]), "", "Git still sees a clean worktree");
    assert!(!admin_of(worktree.as_path()).join(JOURNAL_NAME).exists(), "the journal is gone");
    remove(parent.as_path());
}

/// A registration that names another worktree's `.git` file is still refused,
/// because the two names are not one file.
#[test]
fn a_registration_that_names_another_worktree_is_refused() {
    let (parent, root) = checkout_with_worktree("convert-elsewhere");
    git(root.as_path(), &["worktree", "add", "--quiet", "--detach", "second"]);
    let admin: PathBuf = root.join(".git/worktrees/wt");
    let elsewhere: String = format!("{}/.git\n", root.join("second").display());
    std::fs::write(admin.join("gitdir"), elsewhere.as_str()).expect("a registration of another worktree");
    let reason: String = inspect(root.join("wt").as_path()).expect_err("the registration names another worktree");
    assert!(reason.contains("does not name") && reason.contains("points elsewhere"), "{reason}");
    let platform: PlainPlatform = PlainPlatform::new();
    skipped_with(run(&platform, root.join("wt").as_path(), RUN, &Interrupt::never()).0, "points elsewhere");
    assert!(!platform.is_subvolume_root(root.join("wt").as_path()).expect("examined"), "nothing was converted");
    remove(parent.as_path());
}

/// A registration whose `.git` file does not exist is refused as dangling,
/// and nothing is converted.
#[test]
fn a_dangling_registration_is_refused() {
    let (parent, root) = checkout_with_worktree("convert-dangling");
    let admin: PathBuf = root.join(".git/worktrees/wt");
    let dangling: String = format!("{}/.git\n", root.join("gone").display());
    std::fs::write(admin.join("gitdir"), dangling.as_str()).expect("a dangling registration");
    let reason: String = inspect(root.join("wt").as_path()).expect_err("the registration is dangling");
    assert!(reason.contains("does not name") && reason.contains("points elsewhere"), "{reason}");
    let platform: PlainPlatform = PlainPlatform::new();
    skipped_with(run(&platform, root.join("wt").as_path(), RUN, &Interrupt::never()).0, "points elsewhere");
    assert!(!platform.is_subvolume_root(root.join("wt").as_path()).expect("examined"), "nothing was converted");
    remove(parent.as_path());
}

/// The reasons the verdict lists when something stands in the way,
/// and the worktree is untouched.
fn skipped_with(verdict: Result<Verdict, MigrateError>, fragment: &str) {
    match verdict.expect("no failure") {
        Verdict::Skipped(reasons) => assert!(reasons.iter().any(|reason: &String| { return reason.contains(fragment); }), "{reasons:?} should mention {fragment}"),
        other => panic!("expected a skip mentioning {fragment}, got {other:?}"),
    }
}

/// Every refusal is reported,
/// with the worktree untouched:
/// locked,
/// modified,
/// an index lock,
/// a foreign converting directory,
/// a
///  process inside,
/// and a worktree registered inside.
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

/// A kill between creating the converting subvolume and recording it leaves an empty one,
/// which the next run adopts instead of leaving the worktree blocked by the conversion's own leftover:
/// the journal was at the phase that creates it and holds nothing yet,
/// so the object at that name is the one this conversion made.
#[test]
fn a_stage_created_but_not_recorded_is_adopted_by_the_next_run() {
    let (parent, root, worktree) = quiet_worktree("convert-stage-unrecorded");
    let platform: PlainPlatform = PlainPlatform::new();
    let git_file: Vec<u8> = std::fs::read(worktree.join(".git")).expect(".git");
    let killed = run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("stage-unrecorded", 0));
    assert!(killed.is_err(), "the run was killed between creating the stage and recording it");
    let converting: PathBuf = converting_path(worktree.as_path());
    assert_eq!(std::fs::read_dir(converting.as_path()).expect("listing").count(), 0, "the stage the kill left holds nothing");
    let (verdict, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(verdict.expect("resumed"), Verdict::Resumed, "{lines:#?}");
    assert_converted(&platform, root.as_path(), worktree.as_path(), git_file.as_slice());
    remove(parent.as_path());
}

/// The abandon of that state destroys the empty stage,
/// so the person is not left with a directory only a hand can remove.
#[test]
fn abandon_destroys_the_unrecorded_empty_stage() {
    let (parent, _root, worktree) = quiet_worktree("convert-stage-unrecorded-abandon");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("stage-unrecorded", 0)).is_err());
    let converting: PathBuf = converting_path(worktree.as_path());
    let undone = run_quiet(&platform, worktree.as_path(), ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false }, &Interrupt::never());
    assert_eq!(undone.expect("abandoned"), Verdict::Abandoned);
    assert!(std::fs::symlink_metadata(converting.as_path()).is_err(), "the empty stage this conversion made is gone");
    assert_eq!(std::fs::read(worktree.join("README.md")).expect("original intact"), b"readme\n");
    remove(parent.as_path());
}

/// A kill at every phase is finished by running the command again,
/// with the same final state and nothing foreign deleted.
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

/// Abandoning before the swap destroys only the subvolume this command created and leaves the original whole;
/// after the swap
///  there is nothing to undo,
/// and a rerun finishes.
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

/// What:
///  A process that enters the stage after the first holder scan keeps the stage:
///  the scan immediately before the destroy
///  sees it,
///  so the window between the check and the kernel call is one check wide.
/// Why:
///   That window cannot be closed without a lock,
/// so the command takes the last look it can and the person is told
///  which process entered.
#[test]
fn a_process_that_enters_the_stage_after_the_scan_keeps_it() {
    let (parent, _root, worktree) = quiet_worktree("convert-abandon-late-holder");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("verified", 0)).is_err());
    let converting: PathBuf = converting_path(worktree.as_path());
    let candidate: Candidate = inspect(worktree.as_path()).expect("candidate");
    let journal: Journal = Journal::load(candidate.admin.as_path()).expect("the journal loads").expect("a journal of the interrupted conversion");
    let mut say: Vec<String> = Vec::new();
    let late: std::cell::RefCell<Option<Sleeper>> = std::cell::RefCell::new(None);
    let mut enter = || {
        *late.borrow_mut() = Some(Sleeper::start(converting.as_path()));
    };
    let verdict = abandon_seamed(&platform, &candidate, journal, &mut |text: &str| { say.push(String::from(text)); }, &mut enter);
    let pid: i64 = late.borrow().as_ref().expect("the sleeper entered the stage").pid();
    let Verdict::Skipped(reasons) = verdict.expect("no failure") else { panic!("the abandon keeps the stage a process entered: {say:?}") };
    let named: String = reasons.join("; ");
    assert!(named.contains(format!("process {pid}").as_str()), "the refusal names the process that entered: {named}");
    assert!(std::fs::symlink_metadata(converting.as_path()).is_ok(), "the stage is kept");
    drop(late);
    remove(parent.as_path());
}

/// What:
///  A stage renamed aside after the process scan and replaced keeps its replacement:
///  the last look at the identity,
/// immediately
///  before the destroy,
/// refuses it.
/// Why:
///   Binding the destroy to the checked object needs a destroy that takes a descriptor,
/// which the platform does not
///  offer,
/// so the look is placed as late as it can be and the replacement is kept.
#[test]
fn a_replacement_that_arrives_after_the_scan_of_a_leftover_stage_is_kept() {
    let (parent, _root, worktree) = quiet_worktree("convert-leftover-late-replacement");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).is_err());
    let converting: PathBuf = converting_path(worktree.as_path());
    let admin: PathBuf = admin_of(worktree.as_path());
    remove(worktree.as_path());
    remove(admin.as_path());
    let moved: PathBuf = parent.join("stage.kept");
    let (at, kept) = (converting.clone(), moved.clone());
    let mut say: Vec<String> = Vec::new();
    let replace = &mut || {
        std::fs::rename(at.as_path(), kept.as_path()).expect("move the recorded stage aside");
        std::fs::create_dir_all(at.join("d")).expect("a directory at the stage's name");
        std::fs::write(at.join("d/precious"), b"somebody else's file\n").expect("a file of theirs");
    };
    let reason: &str = "the worktree is gone";
    let verdict = abandon_leftover_stage_seamed(&platform, worktree.as_path(), reason, &mut |text: &str| { say.push(String::from(text)); }, replace);
    let Some(Verdict::Skipped(reasons)) = verdict.expect("no failure") else { panic!("the replacement is kept: {say:?}") };
    assert!(reasons.join("; ").contains("changed while the command was about to destroy it"), "{reasons:?}");
    assert_eq!(std::fs::read(converting.join("d/precious")).expect("the replacement is kept"), b"somebody else's file\n");
    assert!(moved.exists(), "the recorded stage is still where the other process moved it");
    remove(parent.as_path());
}

/// What:
///  `--abandon` destroys a leftover stage only while the object at that name is the one this conversion recorded,
///  so a directory someone put there instead is kept and the reason is said.
/// Why:
///   Without that proof the suffix alone would let the command delete a directory of the person's own files,
/// and a
///  stage moved aside and replaced is exactly how a person changes their mind about a leftover copy.
#[test]
fn an_abandon_keeps_a_replacement_that_took_the_leftover_stage_name() {
    let (parent, _root, worktree) = quiet_worktree("convert-abandon-replaced-stage");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).is_err());
    let converting: PathBuf = converting_path(worktree.as_path());
    let admin: PathBuf = admin_of(worktree.as_path());
    // The worktree and its registration go, as a removal leaves them, and the stage stays beside them.
    remove(worktree.as_path());
    remove(admin.as_path());
    let moved: PathBuf = parent.join("stage.kept");
    std::fs::rename(converting.as_path(), moved.as_path()).expect("move the stage aside");
    std::fs::create_dir_all(converting.join("d")).expect("a directory at the stage's name");
    std::fs::write(converting.join("d/precious"), b"somebody else's file\n").expect("a file of theirs");
    let abandon: ConvertOptions = ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false };
    let (verdict, lines) = run(&platform, worktree.as_path(), abandon, &Interrupt::never());
    let Verdict::Skipped(reasons) = verdict.expect("no failure") else { panic!("the abandon keeps what it cannot prove: {lines:?}") };
    let named: String = reasons.join("; ");
    assert!(named.contains("not the stage this conversion recorded"), "the refusal says why: {named}");
    assert_eq!(std::fs::read(converting.join("d/precious")).expect("the replacement is kept"), b"somebody else's file\n");
    assert!(moved.exists(), "the stage itself is still where the person moved it");
    remove(parent.as_path());
}

/// A sleeping process whose working directory is a chosen directory,
/// killed and reaped when it goes out of scope.
struct Sleeper(std::process::Child);

impl Sleeper {
    /// Start `sleep` inside `directory`.
    fn start(directory: &Path) -> Sleeper {
        let mut command: std::process::Command = std::process::Command::new("sleep");
        command.arg("120").current_dir(directory).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        return Sleeper(command.spawn().expect("the sleeper starts"));
    }

    /// The sleeping process's number.
    fn pid(&self) -> i64 {
        return i64::from(self.0.id());
    }
}

impl Drop for Sleeper {
    /// Kill and reap the process.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// What:
///  The abandon destroys the stage only while no process is working inside it,
///  as the removal phase of the same conversion refuses,
///  because the stage is a directory a person may have a shell or an editor in.
#[test]
fn an_abandon_keeps_the_stage_while_a_process_is_inside_it() {
    let (parent, _root, worktree) = quiet_worktree("convert-abandon-holder");
    let platform: PlainPlatform = PlainPlatform::new();
    assert!(run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("verified", 0)).is_err());
    let converting: PathBuf = converting_path(worktree.as_path());
    let abandon: ConvertOptions = ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false };
    let holder: Sleeper = Sleeper::start(converting.as_path());
    let (refused, lines) = run(&platform, worktree.as_path(), abandon, &Interrupt::never());
    let Verdict::Skipped(reasons) = refused.expect("no failure") else { panic!("the abandon refuses while a process is inside the stage: {lines:?}") };
    let named: String = reasons.join("; ");
    assert!(named.contains(format!("process {}", holder.pid()).as_str()), "the refusal names the process: {named} {lines:?}");
    assert!(std::fs::symlink_metadata(converting.as_path()).is_ok(), "the stage is kept while a process is in it");
    drop(holder);
    let (undone, lines2) = run(&platform, worktree.as_path(), abandon, &Interrupt::never());
    assert_eq!(undone.expect("abandoned"), Verdict::Abandoned, "{lines2:?}");
    assert!(std::fs::symlink_metadata(converting.as_path()).is_err(), "the stage goes once nothing is inside it");
    remove(parent.as_path());
}

/// What:
///  A conversion interrupted past its swap keeps the original tree at the converting name on purpose,
///  so a worktree removed afterwards leaves that full copy beside where it was,
///  and the journal that alone records it goes with the registration.
/// `--abandon` must reach it:
///  every other mode refuses a path without a `.git` before it looks at anything.
#[test]
fn an_abandon_reaches_the_stage_that_a_removal_left_behind() {
    let (parent, _root, worktree) = quiet_worktree("convert-abandon-leftover");
    let platform: PlainPlatform = PlainPlatform::new();
    // Past the swap: the checkout is the new subvolume and the original tree waits at the converting name.
    assert!(run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).is_err());
    let converting: PathBuf = converting_path(worktree.as_path());
    let admin: PathBuf = admin_of(worktree.as_path());
    // What the removal of the worktree does: its directory and its registration go, and the journal goes with them.
    remove(worktree.as_path());
    remove(admin.as_path());
    assert!(std::fs::symlink_metadata(converting.as_path()).is_ok(), "the full copy stays beside where the worktree was");
    let abandon: ConvertOptions = ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false };
    let (verdict, lines) = run(&platform, worktree.as_path(), abandon, &Interrupt::never());
    assert_eq!(verdict.expect("abandoned"), Verdict::Abandoned, "{lines:#?}");
    assert!(std::fs::symlink_metadata(converting.as_path()).is_err(), "the stage is gone after the abandon");
    assert!(lines.iter().any(|line: &String| { return line.contains("leftover"); }), "the abandon says what it destroyed: {lines:#?}");
    remove(parent.as_path());
}

/// The final removal deletes the directory the journal recorded and nothing else:
/// a different directory at the converting
///  name is left alone,
/// the run says it is not finished,
/// and the journal stays so that the next run can finish it.
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

/// A process that appears after the copy was made stops the swap on the rerun,
/// and nothing is swapped.
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

/// A file that changes while the copy is made is caught by the proving pass,
/// and nothing is swapped.
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

/// What:
///  Bind a socket named `name` inside `directory` through a short path,
///  and hand it back so the caller can drop it.
/// Why:
///   A socket path is limited to about a hundred bytes,
/// and a fixture under the build directory is longer than that,
///  so the bind goes through the open directory's descriptor.
/// Dropping the listener leaves the socket file where it is,
///  as a stopped development server does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listener = bindSocketIn(directory, 'agent.sock');
/// ```
fn bind_socket_in(directory: &Path, name: &str) -> std::os::unix::net::UnixListener {
    use std::os::fd::AsRawFd as _;
    use std::os::unix::fs::FileTypeExt as _;
    let handle: std::fs::File = std::fs::File::open(directory).expect("directory handle");
    let short: PathBuf = PathBuf::from(format!("/proc/self/fd/{}/{name}", handle.as_raw_fd()));
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind(short).expect("socket");
    assert!(std::fs::symlink_metadata(directory.join(name)).expect("the socket file").file_type().is_socket());
    return listener;
}

/// A copy that leaves an entry out of the stage says which entry and why,
/// so the silence cannot hide what
///  the conversion does not carry.
#[test]
fn a_copy_that_leaves_a_socket_out_says_so() {
    let (parent, _root, worktree) = quiet_worktree("convert-socket");
    let listener: std::os::unix::net::UnixListener = bind_socket_in(worktree.as_path(), "agent.sock");
    drop(listener);
    let platform: PlainPlatform = PlainPlatform::new();
    let (_verdict, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    let said: String = lines.join("\n");
    assert!(said.contains("not recreated (a socket or device node): agent.sock"), "{said}");
    remove(parent.as_path());
}

/// A socket in the worktree stops the conversion before the swap,
/// naming the entry,
/// and a person who deletes
///  it finishes the conversion by running the command again.
#[test]
fn a_socket_stops_a_conversion_before_the_swap_and_deleting_it_finishes() {
    let (parent, _root, worktree) = quiet_worktree("convert-socket-stop");
    let listener: std::os::unix::net::UnixListener = bind_socket_in(worktree.as_path(), "agent.sock");
    drop(listener);
    let platform: PlainPlatform = PlainPlatform::new();
    let message: String = run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0.expect_err("the conversion refuses").to_string();
    assert!(message.contains("agent.sock") && message.contains("does not recreate"), "{message}");
    assert!(message.contains("nothing was swapped"), "{message}");
    assert!(!platform.is_subvolume_root(worktree.as_path()).expect("examined"), "nothing was swapped");
    assert!(worktree.join("agent.sock").exists(), "the socket is still in the worktree");
    std::fs::remove_file(worktree.join("agent.sock")).expect("the person removes the socket");
    let (finished, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(finished.expect("converted"), Verdict::Resumed, "{lines:#?}");
    assert!(platform.is_subvolume_root(worktree.as_path()).expect("examined"), "the conversion finished");
    remove(parent.as_path());
}

/// What:
///  The refusal a socket causes is a state `--abandon` ends:
/// the stage is a copy that was never exchanged,
/// so abandoning it removes the stage and its
///  journal and leaves the worktree,
/// with its socket,
/// exactly as it was.
/// Why:
///   The refusal is worth nothing to a person if the only copy of their worktree cannot be
/// let go of;
/// this
///  proves the abandon works after it rather than leaving the claim and the stage behind.
#[test]
fn a_conversion_refused_over_a_socket_can_still_be_abandoned() {
    let (parent, _root, worktree) = quiet_worktree("convert-socket-abandon");
    let listener: std::os::unix::net::UnixListener = bind_socket_in(worktree.as_path(), "agent.sock");
    drop(listener);
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    let message: String = run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0.expect_err("the conversion refuses").to_string();
    assert!(message.contains("nothing was swapped"), "{message}");
    assert!(converting.exists(), "the refusal leaves the stage it copied");
    let abandon: ConvertOptions = ConvertOptions { dry_run: false, abandon: true, command_line_attributes: false };
    let (abandoned, lines) = run(&platform, worktree.as_path(), abandon, &Interrupt::never());
    assert_eq!(abandoned.expect("abandoned"), Verdict::Abandoned, "{lines:#?}");
    assert!(std::fs::symlink_metadata(&converting).is_err(), "the stage is gone");
    assert!(!admin_of(worktree.as_path()).join(JOURNAL_NAME).exists(), "the journal is gone");
    assert!(worktree.join("agent.sock").exists(), "the worktree keeps its socket");
    assert!(!platform.is_subvolume_root(worktree.as_path()).expect("examined"), "the worktree is not a subvolume");
    remove(parent.as_path());
}

/// A socket that reaches the old tree after the swap names the entry to delete,
/// and the conversion finishes once it
///  is gone;
/// nothing else is asked of the person.
#[test]
fn a_socket_left_in_the_old_tree_names_the_entry_to_delete() {
    let (parent, _root, worktree) = quiet_worktree("convert-socket-past");
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    assert!(run(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).0.is_err());
    let listener: std::os::unix::net::UnixListener = bind_socket_in(converting.as_path(), "agent.sock");
    drop(listener);
    let message: String = run(&platform, worktree.as_path(), RUN, &Interrupt::never()).0.expect_err("the conversion refuses").to_string();
    assert!(message.contains("agent.sock") && message.contains("delete it from"), "{message}");
    assert!(converting.join("agent.sock").exists(), "the old tree is kept whole");
    std::fs::remove_file(converting.join("agent.sock")).expect("the person deletes the socket");
    let (finished, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(finished.expect("converted"), Verdict::Resumed, "{lines:#?}");
    assert!(std::fs::symlink_metadata(&converting).is_err(), "the old tree is gone");
    remove(parent.as_path());
}

/// What:
///  A stage container that reaches the old tree after the swap does not keep it.
/// Why:
///   Another command filling a stage inside the worktree can finish after the proving pass,
/// so the old tree holds a
///  container the installed tree never had;
/// the comparison demanded it and kept the old tree on every run,
/// asking the
///  person to settle a version that exists on one side only.
#[test]
fn a_stage_container_left_in_the_old_tree_does_not_keep_it() {
    let (parent, _root, worktree) = quiet_worktree("convert-stage-past");
    let platform: PlainPlatform = PlainPlatform::new();
    let converting: PathBuf = converting_path(worktree.as_path());
    assert!(run_quiet(&platform, worktree.as_path(), RUN, &Interrupt::at("exchanged", 0)).is_err());
    let container: PathBuf = converting.join(".claude/worktrees/.cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543/payload");
    std::fs::create_dir_all(&container).expect("the stage container");
    std::fs::write(container.join("credentials"), b"token\n").expect("its payload");
    let (finished, lines) = run(&platform, worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(finished.expect("finished"), Verdict::Resumed, "{lines:#?}");
    assert!(std::fs::symlink_metadata(&converting).is_err(), "the old tree is gone");
    assert!(!worktree.join(".claude").exists(), "the container is in neither tree");
    remove(parent.as_path());
}

/// On a real btrfs filesystem the same conversion creates a real subvolume (root inode 256),
/// clones the file data,
/// keeps the
///  registration and the index fast;
/// elsewhere the control prints its one loud line and returns.
#[test]
fn btrfs_a_real_conversion_makes_a_subvolume_and_clones_file_data() {
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

/// What:
///  A real conversion of a worktree that holds another command's stage container finishes,
///  and no part of that container reaches the new subvolume.
/// Why:
///   The stage holds a whole worktree's ignored state,
/// credentials included,
/// and a conversion that carried it
///  would leave that state in a worktree on another branch with nothing naming it.
/// The container belongs to the
///  command filling it,
/// which removes it at its own path,
/// so it is left out and goes with the old tree.
#[test]
fn btrfs_a_stage_container_is_left_out_of_the_new_subvolume() {
    use crate::fs_btrfs_test_support::btrfs_fixture;
    use crate::subvolume_platform::LinuxPlatform;
    let Some(probe) = btrfs_fixture("convert-stage-container") else { return };
    remove(probe.as_path());
    let (parent, _root, worktree) = quiet_worktree("convert-stage-container");
    let container: PathBuf = worktree.join(".claude/worktrees/.cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543");
    std::fs::create_dir_all(container.join("payload")).expect("the stage container");
    std::fs::write(container.join("payload/credentials"), b"token\n").expect("its payload");
    std::fs::set_permissions(container.join("payload/credentials"), std::os::unix::fs::PermissionsExt::from_mode(0o600)).expect("a credential's mode");
    let (verdict, lines) = run(&LinuxPlatform::default(), worktree.as_path(), RUN, &Interrupt::never());
    assert_eq!(verdict.expect("converted"), Verdict::Converted, "{lines:#?}");
    assert!(!worktree.join(".claude").exists(), "no part of the stage container reaches the new subvolume");
    assert!(std::fs::symlink_metadata(converting_path(worktree.as_path())).is_err(), "the old tree is gone");
    assert_nothing_left(worktree.as_path());
    remove(parent.as_path());
}
