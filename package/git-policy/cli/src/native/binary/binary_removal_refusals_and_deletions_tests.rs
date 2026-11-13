//! What:
//!  What the built executable does when a worktree is removed:
//!  a process working in it,
//!  the ignored files a removal deletes,
//!  a worktree nested inside it,
//!  and the automatic removal of a landed one.
//! Why:
//!  These are promises about the program a caller runs as `git`,
//!  so each is checked through it against real Git:
//!  a refusal that names the process and the one flag that runs the removal anyway,
//!  and a record of every file a removal deleted,
//!  which is the only thing left of those files afterwards.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(runWrapped(repo, ['worktree', 'remove', '../topic'])).toMatchObject({ code: 1 });
//! ```

/// The shared fixtures and bounded process helpers.
use super::fault_run::{run_watched, unbounded_wrapper};
use super::support::{
    Fixture, Observed, fixture, git, hermetic, remove, repository, run_wrapped, settle_service_work, spawn_owned, stderr_of, wrapper_first_path,
};
use std::path::{Path, PathBuf};
use std::os::unix::process::ExitStatusExt as _;
use std::process::{Command, Output};
/// The test-only kill point of a removal,
/// which is inert unless a control arms it.
use git_policy_cli::worktree_removal_deletions::{AFTER_DELETION, FAULT_VARIABLE};

/// The record a removal writes of the files it deleted,
/// in the repository's common Git directory.
const DELETIONS_LOG: &str = "monochromatic-git-policy-cli-worktree-deletions.jsonl";

/// What:
///  A repository with one commit and one worktree added through the wrapper,
///  whose first commit also carries the ignore rules a control writes.
/// Why:
///  A removal names a registered worktree the wrapper made,
///  and the ignored files are the ones the record is about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { fixture, repo } = aRepositoryWithAWorktree('removal-1', 'build/\n*.log\n');
/// ```
fn a_repository_with_a_worktree(name: &str, rules: &str) -> (Fixture, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("README.md"), b"# fixture\n").expect("a tracked file");
    let rules_path: PathBuf = repo.join(".gitignore");
    std::fs::write(rules_path.as_path(), rules.as_bytes()).expect("the ignore rules");
    git(&fixture, repo.as_path(), &["add", "--", "README.md", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=a tracked file"]);
    let topic: PathBuf = std::fs::canonicalize(fixture.root.as_path()).expect("the fixture root").join("topic");
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", "../topic"]);
    assert_eq!(created.code, Some(0), "the worktree is added: {}", stderr_of(&created));
    return (fixture, repo, topic);
}

/// What:
///  A helper process whose working folder is the worktree,
///  as a person's shell or a build would be.
/// Why:
///  The refusal is about a live process,
///  and the reading that finds it reads the working folder the kernel records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const child = aProcessWorkingIn(fixture, topic);
/// ```
fn a_process_working_in(fixture: &Fixture, directory: &Path) -> std::process::Child {
    let mut command: Command = Command::new("/usr/bin/sleep");
    command.arg("120");
    // The hermetic surroundings set the working folder to the fixture root last, so the worktree is set after them.
    hermetic(&mut command, fixture, wrapper_first_path(fixture).as_os_str());
    command.current_dir(directory);
    let child: std::process::Child = spawn_owned(&mut command).expect("a helper working in the worktree");
    // The kernel's own record is what the wrapper reads, so the control reads the same one.
    let start: std::time::Instant = std::time::Instant::now();
    while std::fs::read_link(format!("/proc/{}/cwd", child.id())).ok().as_deref() != Some(directory) {
        assert!(start.elapsed() < std::time::Duration::from_secs(10), "the helper did not enter the worktree");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    return child;
}

/// The record of one removal,
/// read out of the repository's common Git directory,
/// or nothing when no removal wrote one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deletionRecord(repo) // the log's text | undefined
/// ```
fn deletion_record(repo: &Path) -> Option<String> {
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    return std::fs::read_to_string(common.join(DELETIONS_LOG)).ok();
}

/// The number of kept-worktree records one repository holds,
/// and zero when the folder is not there at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptRecords(commonDir) // the count of files in kept-landed-worktrees/
/// ```
fn kept_records(common: &Path) -> usize {
    return std::fs::read_dir(common.join("monochromatic-git-policy-cli/kept-landed-worktrees")).map_or(0, |listing| return listing.count());
}

/// What:
///  A worktree a process works in is refused,
///  the words name the process and the one flag that runs the removal anyway,
///  and the worktree is still there.
/// Why:
///  This is the owner's decision of 2026-11-12 ("Delete ignored, spare busy"):
///  a removal that pulls the folder out from under a running program is refused,
///  and the refusal is worth nothing unless it says what to stop and what to type when the person means it.
#[test]
fn a_removal_a_process_works_in_is_refused_and_the_escape_runs_it() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-busy", "build/\n");
    let mut child: std::process::Child = a_process_working_in(&fixture, topic.as_path());
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_ne!(refused.code, Some(0), "a worktree a process works in is not removed: {}", stderr_of(&refused));
    let said: String = stderr_of(&refused);
    assert!(said.contains("refusing to remove the worktree"), "{said}");
    assert!(said.contains(format!("process {}", child.id()).as_str()), "the refusal names the process: {said}");
    assert!(said.contains("`sleep`"), "and the program it runs: {said}");
    assert!(said.contains("--no-enforce-worktree"), "and the escape: {said}");
    assert!(topic.is_dir(), "the worktree is still there");
    assert!(topic.join("README.md").is_file(), "with its files");
    // The escape is the person's own decision, so the removal runs exactly as real Git runs it.
    let escaped: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "--no-enforce-worktree", "remove", "../topic"]);
    let _ended = child.kill();
    let _waited = child.wait();
    assert_eq!(escaped.code, Some(0), "the escape runs the removal: {}", stderr_of(&escaped));
    assert!(!topic.exists(), "the worktree is gone");
    assert!(!String::from_utf8_lossy(&run_wrapped(&fixture, repo.as_path(), &["worktree", "list"]).stdout).contains(topic.display().to_string().as_str()), "and its registration");
    remove(&fixture);
}

/// What:
///  A removal a process works in takes no copy of anything
///  and writes no deletion record,
///  because nothing was removed.
/// Why:
///  A record of a deletion that did not happen is worse than none:
///  a person would read that their files went, and look for them.
#[test]
fn a_refused_removal_writes_no_record() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-busy-no-record", "build/\n");
    std::fs::create_dir_all(topic.join("build").as_path()).expect("an ignored directory");
    std::fs::write(topic.join("build/only-copy.txt"), b"the only copy\n").expect("an ignored file");
    let mut child: std::process::Child = a_process_working_in(&fixture, topic.as_path());
    let refused: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    let _ended = child.kill();
    let _waited = child.wait();
    assert_ne!(refused.code, Some(0), "{}", stderr_of(&refused));
    assert_eq!(deletion_record(repo.as_path()), None, "a removal that did not happen records nothing");
    assert_eq!(std::fs::read(topic.join("build/only-copy.txt")).expect("the ignored file is still there"), b"the only copy\n");
    remove(&fixture);
}

/// What:
///  An unforced removal deletes the files Git ignores,
///  and one record says what went,
///  from which worktree,
///  when,
///  and by which command.
/// Why:
///  The owner's decision takes those files and requires each deletion to be recorded:
///  after the removal the record is the only thing that says the file was ever there.
#[test]
fn a_removal_deletes_the_ignored_files_and_the_record_names_them() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-record", "build/\n*.log\n");
    std::fs::create_dir_all(topic.join("build").as_path()).expect("an ignored directory");
    std::fs::write(topic.join("build/only-copy.txt"), b"the only copy\n").expect("a file in the ignored directory");
    std::fs::write(topic.join("long.log"), b"a log nobody else has\n").expect("an ignored file");
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    assert!(!topic.exists(), "the worktree is gone");
    let record: String = deletion_record(repo.as_path()).expect("the record of what the removal deleted");
    assert!(record.contains("\"kind\":\"worktree-removal\""), "{record}");
    assert!(record.contains("\"build/\""), "the whole ignored directory is one entry: {record}");
    assert!(record.contains("\"long.log\""), "the ignored file is named: {record}");
    assert!(record.contains(topic.display().to_string().as_str()), "the worktree it went from is named: {record}");
    assert!(record.contains("\"command\":[\"worktree\",\"remove\""), "and the command that did it: {record}");
    assert!(record.contains("\"time\":\""), "and when: {record}");
    remove(&fixture);
}

/// What:
///  A worktree with a worktree nested inside it goes,
///  and the record names the nested worktree and the files it held.
/// Why:
///  The nested checkout sits under a name the outer worktree ignores,
///  so real Git deletes the whole of it without asking the nested worktree's own rules:
///  the decision of 2026-11-12 lets it go and requires every file of it to be recorded.
#[test]
fn a_removal_records_a_nested_worktrees_own_files() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-nested", "nested/\n");
    let nested: PathBuf = topic.join("nested");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "nested", nested.to_str().expect("a UTF-8 fixture path")]);
    std::fs::write(nested.join("personal.txt"), b"untracked personal work\n").expect("a file in the nested worktree");
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    assert!(!topic.exists(), "the outer worktree is gone");
    let record: String = deletion_record(repo.as_path()).expect("the record of what the removal deleted");
    assert!(record.contains(nested.display().to_string().as_str()), "the nested worktree is named: {record}");
    assert!(record.contains("\"personal.txt\""), "and the file it held: {record}");
    remove(&fixture);
}

/// What:
///  An ordinary clean worktree with nothing in it that Git ignores is removed,
///  and no record is written for it.
/// Why:
///   A record of nothing would fill the log with a line per ordinary removal,
///  and every line says a removal deleted something nobody else has.
#[test]
fn an_ordinary_clean_removal_writes_no_record() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-clean", "build/\n");
    let began: std::time::Instant = std::time::Instant::now();
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    let took: std::time::Duration = began.elapsed();
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    assert!(!topic.exists(), "the worktree is gone");
    assert_eq!(deletion_record(repo.as_path()), None, "nothing was deleted that the record has to explain");
    // The reading this removal added is one listing of a worktree with nothing ignored, and the removal of a worktree this
    //  size stays far under the bound every wrapped command of this suite runs under.
    eprintln!("an ordinary clean removal took {took:?}");
    assert!(took < std::time::Duration::from_secs(5), "an ordinary removal stays under the suite's own bound: {took:?}");
    remove(&fixture);
}

/// What:
///  A landed sweep leaves a worktree a process works in,
///  says why,
///  and a later sweep removes it once the process is gone.
/// Why:
///  The merge has nobody to refuse to,
///  so the worktree stays with its reason and the next sweep is the pass that takes it:
///  without that pass the snapshot's space never comes back and nothing ever looks at the worktree again.
#[test]
fn a_landed_sweep_leaves_a_worktree_a_process_works_in_and_a_later_sweep_removes_it() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-landed-busy", "build/\n");
    git(&fixture, topic.as_path(), &["commit", "--quiet", "--allow-empty", "--message=feature work"]);
    let mut child: std::process::Child = a_process_working_in(&fixture, topic.as_path());
    let landed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-ff", "--message=land feature", "feature"]);
    assert_eq!(landed.code, Some(0), "the merge keeps going: {}", stderr_of(&landed));
    let said: String = stderr_of(&landed);
    assert!(topic.is_dir(), "the worktree stays while the process works in it: {said}");
    assert!(said.contains(topic.display().to_string().as_str()), "the merge says which worktree it left: {said}");
    assert!(said.contains(format!("process {}", child.id()).as_str()), "and names the process: {said}");
    // The record of the kept worktree is what the later pass reads, and it waits in the common Git directory.
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    assert!(kept_records(common.as_path()) > 0, "the kept worktree is remembered for the next sweep");
    let _ended = child.kill();
    let _waited = child.wait();
    // The later pass: another landing of this repository runs the sweep again, and nothing asks it to look at this
    //  worktree. The branch it lands is a second worktree so the merge really moves the branch.
    let second: PathBuf = std::fs::canonicalize(fixture.root.as_path()).expect("the fixture root").join("second");
    let created: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "second", "../second"]);
    assert_eq!(created.code, Some(0), "the second worktree is added: {}", stderr_of(&created));
    git(&fixture, second.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second work"]);
    let later: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-ff", "--message=a later merge", "second"]);
    assert_eq!(later.code, Some(0), "the later merge keeps going: {}", stderr_of(&later));
    settle_service_work(repo.as_path());
    assert!(!topic.exists(), "the later sweep removed the worktree nobody works in any more: {}", stderr_of(&later));
    assert_eq!(kept_records(common.as_path()), 0, "and its record is gone with it");
    remove(&fixture);
}

/// What:
///  A removal killed between taking the files and confirming it leaves the line it wrote before the deletion,
///  naming what went,
///  and no line saying it finished.
/// Why:
///   The log is the only thing left of those files,
///  so a kill in that window must leave a reader the list rather than silence.
///  A line written only after the deletion would be lost by exactly this kill.
///  The kill point is the wrapper's own test-only one,
///  inert unless this control arms it,
///  and the same on all three routes: the destroy,
///  the service's `git worktree remove` and Git's own removal of a typed one.
#[test]
fn a_removal_killed_between_the_deletion_and_its_record_leaves_the_list_it_wrote() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-killed-before-record", "build/\n");
    std::fs::create_dir_all(topic.join("build").as_path()).expect("an ignored directory");
    std::fs::write(topic.join("build/only-copy.txt"), b"the only copy\n").expect("a file in the ignored directory");
    // No `timeout` in front of the wrapper: the kill this control reads is the wrapper's own, and its status must be the
    //  signal's rather than a bound's.
    let mut armed: Command = unbounded_wrapper(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    armed.env(FAULT_VARIABLE, AFTER_DELETION);
    let killed: Output = run_watched(&fixture, &mut armed);
    assert_eq!(killed.status.signal(), Some(9), "the control kills the removal: {}", String::from_utf8_lossy(killed.stderr.as_slice()));
    assert!(!topic.exists(), "the files went before the kill");
    let record: String = deletion_record(repo.as_path()).expect("the line written before the deletion outlives the kill");
    let lines: Vec<&str> = record.lines().filter(|line: &&str| return !line.is_empty()).collect();
    assert_eq!(lines.len(), 1, "the removal never confirmed what it deleted: {record}");
    assert!(lines[0].contains("\"outcome\":\"begun\""), "the line says the removal set out to delete: {record}");
    assert!(lines[0].contains("\"build/\""), "and names what it was about to delete: {record}");
    assert!(!record.contains("\"outcome\":\"deleted\""), "a kill before the confirming line leaves no confirmation: {record}");
    remove(&fixture);
}

/// What:
///  A worktree added again at the path of one a sweep kept,
///  on a branch that has not landed,
///  is not removed by a later sweep of an unrelated branch,
///  and the record of the worktree that went no longer answers for it.
/// Why:
///  The record names a path,
///  and a path names whatever stands there when a later sweep reads it:
///  a merge of an unrelated branch removed a worktree whose branch had not landed at all
///  (a review of this change, finding 1).
#[test]
fn a_later_sweep_does_not_remove_a_worktree_added_again_at_a_kept_worktrees_path_on_another_branch() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-kept-then-readded-elsewhere", "build/\n");
    git(&fixture, topic.as_path(), &["commit", "--quiet", "--allow-empty", "--message=feature work"]);
    let mut child: std::process::Child = a_process_working_in(&fixture, topic.as_path());
    let landed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-ff", "--message=land feature", "feature"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert!(topic.is_dir(), "the worktree is kept while the process works in it");
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    assert_eq!(kept_records(common.as_path()), 1, "the kept worktree is remembered");
    let _ended = child.kill();
    let _waited = child.wait();
    // The person removes that worktree by hand, which forgets nothing, and adds another one at the same path for other
    //  work whose branch has not landed.
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    let readded: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "second", "../topic"]);
    assert_eq!(readded.code, Some(0), "the second worktree is added: {}", stderr_of(&readded));
    git(&fixture, topic.as_path(), &["commit", "--quiet", "--allow-empty", "--message=second work"]);
    let (_other, unrelated): (PathBuf, Observed) = a_merge_of_an_unrelated_branch(&fixture, &repo, "third");
    assert_eq!(unrelated.code, Some(0), "{}", stderr_of(&unrelated));
    settle_service_work(repo.as_path());
    assert!(topic.is_dir(), "the worktree whose branch has not landed stays: {}", stderr_of(&unrelated));
    assert!(topic.join("README.md").is_file(), "with its files");
    let listing: String = String::from_utf8_lossy(&run_wrapped(&fixture, repo.as_path(), &["worktree", "list"]).stdout).into_owned();
    assert!(listing.contains(topic.display().to_string().as_str()), "and its registration: {listing}");
    assert_eq!(kept_records(common.as_path()), 0, "the record of the worktree that went no longer answers for this one");
    remove(&fixture);
}

/// What:
///  A worktree added again at the path of one a sweep kept,
///  on the branch that already landed,
///  is not removed by a later sweep either.
/// Why:
///  This is the case a landed branch alone cannot tell from the worktree the record was written for:
///  the branch is on the landed branch either way,
///  so only the identity of the two directories says which worktree this is,
///  and without it a person who re-made a worktree to go on with that branch lost it,
///  with what they had put in it.
#[test]
fn a_later_sweep_does_not_remove_a_worktree_added_again_on_the_landed_branch_at_a_kept_worktrees_path() {
    let (fixture, repo, topic): (Fixture, PathBuf, PathBuf) = a_repository_with_a_worktree("removal-kept-then-readded-same-branch", "build/\n");
    git(&fixture, topic.as_path(), &["commit", "--quiet", "--allow-empty", "--message=feature work"]);
    let mut child: std::process::Child = a_process_working_in(&fixture, topic.as_path());
    let landed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-ff", "--message=land feature", "feature"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    let common: PathBuf = std::fs::canonicalize(repo.join(".git")).expect("the common Git directory");
    assert_eq!(kept_records(common.as_path()), 1, "the kept worktree is remembered");
    let _ended = child.kill();
    let _waited = child.wait();
    let removed: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "remove", "../topic"]);
    assert_eq!(removed.code, Some(0), "{}", stderr_of(&removed));
    let readded: Observed = run_wrapped(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "../topic", "feature"]);
    assert_eq!(readded.code, Some(0), "the worktree is added again on the landed branch: {}", stderr_of(&readded));
    std::fs::create_dir(topic.join("build").as_path()).expect("an ignored directory in the new worktree");
    std::fs::write(topic.join("build/person.txt"), b"what the person is working on\n").expect("a file in the new worktree");
    let (_other, unrelated): (PathBuf, Observed) = a_merge_of_an_unrelated_branch(&fixture, &repo, "fourth");
    assert_eq!(unrelated.code, Some(0), "{}", stderr_of(&unrelated));
    settle_service_work(repo.as_path());
    assert!(topic.is_dir(), "a worktree added again on the landed branch stays: {}", stderr_of(&unrelated));
    assert_eq!(std::fs::read(topic.join("build/person.txt")).expect("the file is still there"), b"what the person is working on\n");
    assert_eq!(kept_records(common.as_path()), 0, "and the record of the worktree that went is dropped");
    remove(&fixture);
}

/// What:
///  A branch of its own with one commit,
///  in a worktree of its own,
///  merged into the repository's checked-out branch.
/// Why:
///  A sweep only runs for a merge that moves a branch,
///  so a control that needs a sweep with nothing else landing builds one.
///  The merge then removes that branch's own worktree,
///  as a sweep does for every branch it landed,
///  which is why the control reads only the merge's own outcome.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const other = worktreeAdd('-b', 'third', '../third'); git(other, ['commit', '--allow-empty', '-m', 'third work']); merge('third')
/// ```
fn a_merge_of_an_unrelated_branch(fixture: &Fixture, repo: &Path, branch: &str) -> (PathBuf, Observed) {
    let other: PathBuf = std::fs::canonicalize(fixture.root.as_path()).expect("the fixture root").join(branch);
    let added: Observed = run_wrapped(fixture, repo, &["worktree", "add", "--quiet", "-b", branch, format!("../{branch}").as_str()]);
    assert_eq!(added.code, Some(0), "the branch's worktree is added: {}", stderr_of(&added));
    git(fixture, other.as_path(), &["commit", "--quiet", "--allow-empty", "--message=the branch's own work"]);
    let merged: Observed = run_wrapped(fixture, repo, &["merge", "--no-ff", "--message=land it", branch]);
    return (other, merged);
}
