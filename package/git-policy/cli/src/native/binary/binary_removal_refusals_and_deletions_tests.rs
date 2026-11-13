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
use super::support::{
    Fixture, Observed, fixture, git, hermetic, remove, repository, run_wrapped, settle_service_work, spawn_owned, stderr_of, wrapper_first_path,
};
use std::path::{Path, PathBuf};
use std::process::Command;

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
    assert!(
        std::fs::read_dir(common.join("monochromatic-git-policy-cli/kept-landed-worktrees")).map_or(false, |listing| return listing.count() > 0),
        "the kept worktree is remembered for the next sweep"
    );
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
    assert!(
        std::fs::read_dir(common.join("monochromatic-git-policy-cli/kept-landed-worktrees")).map_or(true, |listing| return listing.count() == 0),
        "and its record is gone with it"
    );
    remove(&fixture);
}
