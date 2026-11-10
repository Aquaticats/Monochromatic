//! What:
//!  A true `git merge` run through the built executable in a checkout whose index holds another session's staged
//!  changes,
//! checked against native Git on a clean copy of the same repository.
//! Why:
//!  Native Git refuses every merge that creates a commit while the index differs from `HEAD`
//!  (`doc/planning/cli-git-merge-with-staged-index.md`).
//!  The wrapper completes the merge without disturbing the staged or unstaged work,
//! so the merge commit must equal
//!  what native Git writes on a clean checkout,
//! every entry and file that is not the merge's must stay byte for byte,
//!  and a merge that would write over someone's work,
//! or that conflicts,
//! must change nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ours = await wrapped(['merge', 'feature'], dirtyCopy); const native = await direct(['merge', 'feature'], cleanCopy);
//! expect(await commitShape(ours)).toEqual(await commitShape(native));
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::support::{
    Fixture, Observed, WAIT_NOTICE_PREFIX, direct, executable, fixture, git, observe, pseudo_terminal_slave_name,
    repository, without_hint_fields, without_wait_notices, TIMED_OUT_SIGNAL,
};
use git_policy_cli::merge_isolation_journal::{MergeIntent, write_merge_intent};
use git_policy_cli::transaction_owner::TransactionOwner;
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use git_policy_cli::wait_ceilings::MERGE_LOCK_CEILING;
use std::ffi::OsStr;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// Paths of the fixture repository and what each side of the history does to them:
///  `feature` changes `f.txt` and `g.txt`,
/// deletes `gone.txt` and adds `dir/new.txt`;
/// `main` changes `h.txt`;
///  `u.txt`,
/// `v.txt` and `w.txt` are untouched by the merge.
pub(super) fn diverged(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    for file in ["u.txt", "v.txt", "w.txt", "f.txt", "g.txt", "h.txt", "gone.txt"] {
        std::fs::write(repo.join(file), format!("{file} base\n")).expect("file");
    }
    std::fs::create_dir(repo.join("sub")).expect("subdirectory");
    std::fs::write(repo.join("sub/s.txt"), b"s base\n").expect("file");
    git(
        fixture,
        repo.as_path(),
        &["add", "--", "u.txt", "v.txt", "w.txt", "f.txt", "g.txt", "h.txt", "gone.txt", "sub/s.txt"],
    );
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    std::fs::write(repo.join("g.txt"), b"g.txt feature\n").expect("file");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/new.txt"), b"new\n").expect("file");
    git(fixture, repo.as_path(), &["rm", "--quiet", "--", "gone.txt"]);
    git(fixture, repo.as_path(), &["add", "--", "f.txt", "g.txt", "dir/new.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=feature work"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    return repo;
}

/// A byte-for-byte copy of a repository,
/// for the clean checkout the oracle merges in.
pub(super) fn copy_of(fixture: &Fixture, from: &Path, name: &str) -> PathBuf {
    let to: PathBuf = fixture.root.join(name);
    let copied = Command::new("cp").arg("--archive").arg("--").arg(from).arg(&to).status().expect("cp starts");
    assert!(copied.success(), "the copy of {from:?} failed");
    return to;
}

/// Another session's staged work:
/// a modification,
/// an addition and a deletion of paths the merge does not touch.
pub(super) fn stage_unrelated_work(fixture: &Fixture, repo: &Path) {
    std::fs::write(repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    std::fs::write(repo.join("new.txt"), b"added by another session\n").expect("file");
    git(fixture, repo, &["add", "--", "u.txt", "new.txt"]);
    git(fixture, repo, &["rm", "--quiet", "--", "v.txt"]);
}

/// What another session would notice about a checkout.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct Snapshot {
    /// The branch tip.
    pub(super) head: String,
    /// The index file,
    /// every byte.
    pub(super) index: Vec<u8>,
    /// Every index entry with its stat data.
    pub(super) entries: String,
    /// Porcelain status including untracked files.
    pub(super) status: String,
}

/// Read the observable state of a checkout without changing it.
pub(super) fn snapshot(fixture: &Fixture, repo: &Path) -> Snapshot {
    return Snapshot {
        head: git_text(fixture, repo, &["rev-parse", "HEAD"]),
        index: std::fs::read(repo.join(".git/index")).expect("the index"),
        entries: git_text(fixture, repo, &["--no-optional-locks", "ls-files", "--stage", "--debug"]),
        status: git_text(fixture, repo, &["--no-optional-locks", "status", "--porcelain=v1", "--untracked-files=all"]),
    };
}

/// A wrapped command with the merge's bound.
fn wrapped_in(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Command {
    return wrapped_commit(fixture, repo, arguments);
}

/// Run a wrapped command to its end,
/// without the wait notices a slow step may print (`without_wait_notices`):
///  these controls compare the merge's diagnostics with native Git's words,
/// which a loaded host does not change.
fn run(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    let observed: Observed = observe(&mut wrapped_in(fixture, repo, arguments), b"");
    return Observed { stderr: without_wait_notices(observed.stderr.as_slice()), ..observed };
}

/// Wait notices go and nothing else does,
/// so the comparisons above still see every other diagnostic.
#[test]
fn only_wait_notice_lines_are_left_out_of_the_diagnostics() {
    let notice: String = format!(
        "{}git commit-tree (process 7) after 10 seconds; it is ended only after 300 seconds with no sign of work.\n",
        String::from_utf8_lossy(WAIT_NOTICE_PREFIX)
    );
    let other: &str = "monochromatic-git-policy-cli: copied 1 ignored filesystem entry\nHint: still waiting for nothing\n";
    let mixed: String = format!("{notice}{other}{notice}");
    assert_eq!(without_wait_notices(mixed.as_bytes()), other.as_bytes());
    assert_eq!(without_wait_notices(other.as_bytes()), other.as_bytes());
    assert_eq!(without_wait_notices(b"no newline at the end"), b"no newline at the end");
}

/// Run native Git directly,
/// whatever its exit status.
fn native(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    return observe(direct(fixture).current_dir(repo).args(arguments), b"");
}

/// What:
///  A merge that was saved on a side ref while another program's empty `index.lock` stood,
///  with nothing of it in the checkout:
///  the record the merge's own apply leaves when its wait reaches `MERGE_LOCK_CEILING`.
/// Why:
///  The state is the one two read-only commands then meet,
///  and building it directly keeps the control from spending the ceiling twice to make it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const saved = savedMerge(repo); // records a dead owner, a side ref and the merge commit
/// ```
fn saved_merge_record(fixture: &Fixture, repo: &Path) -> (String, String) {
    let old: String = git_text(fixture, repo, &["rev-parse", "main"]);
    let feature: String = git_text(fixture, repo, &["rev-parse", "feature"]);
    let printed: String = git_text(fixture, repo, &["merge-tree", "--write-tree", "main", "feature"]);
    let tree: String = printed.lines().next().expect("the merged tree").to_owned();
    let merge: String = git_text(
        fixture,
        repo,
        &["commit-tree", tree.as_str(), "-p", old.trim(), "-p", feature.trim(), "-m", "merge feature"],
    )
    .trim()
    .to_owned();
    git(fixture, repo, &["update-ref", "refs/cli-git/kept/1", merge.as_str()]);
    let mut ended: Child = Command::new("/bin/sh").arg("-c").arg("exit 0").spawn().expect("a process that ends");
    let gone: i64 = i64::from(ended.id());
    ended.wait().expect("it ends");
    let intent: MergeIntent = MergeIntent {
        owner: TransactionOwner {
            transaction_id: String::from("11111111-1111-4111-8111-111111111111"),
            owner_pid: gone,
            owner_identity: String::from("0"),
            created_at: String::from("2026-11-09T00:00:00Z"),
            owner_pid_namespace: None,
        },
        repository_root: repo.to_path_buf(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: old.trim().to_owned(),
        new_oid: merge.clone(),
        side_ref: Some(String::from("refs/cli-git/kept/1")),
    };
    write_merge_intent(repo.join(".git").as_path(), &intent).expect("the merge record");
    return (old.trim().to_owned(), merge);
}

/// What:
///  A read-only command answers at once while a merge saved on a side ref waits for another program's `index.lock`,
///  names where that merge is,
///  and the next command finishes it once the lock is gone.
/// Why:
///  The recovery every command runs waited `MERGE_LOCK_CEILING` for the lock before it could report the saved merge,
///  so `git status` and `git log` each cost the person the whole ceiling for as long as the lock stood
///  (finding 3 of the merge-and-pull audit of 2026-11-09),
///  and the owner's rule is "nothing should ever wait forever or indefinitely".
///  Nothing of such a merge is in the checkout,
///  so no command has anything to wait for:
///  it reads the state as it is and says where the merge is saved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// savedMergeRecord(); writeFileSync(arrayLock, ''); expect(timed(['status', '--short']).ms).toBeLessThan(MERGE_LOCK_CEILING * 3 / 4);
/// ```
#[test]
fn a_read_only_command_over_a_saved_merge_answers_without_waiting_for_the_index_lock() {
    let fixture: Fixture = fixture("merge-saved-read-only");
    let repo: PathBuf = diverged(&fixture, "repo");
    let (old, merge): (String, String) = saved_merge_record(&fixture, repo.as_path());
    // A Git killed with the index lock leaves an empty file and no PID file, so nothing proves its holder dead or alive,
    //  which is the shape a wait runs to its ceiling for.
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), b"").expect("a lock a killed Git left");
    let records: PathBuf = repo.join(".git/monochromatic-git-policy-cli-merges");
    // The wait lasts the whole ceiling and the honest answer is a few looks at the repository, so three quarters of it
    //  separates the two even on a loaded host.
    let bound: Duration = MERGE_LOCK_CEILING * 3 / 4;
    for arguments in [&["status", "--short"][..], &["log", "--oneline", "-1"][..]] {
        let began: Instant = Instant::now();
        let observed: Observed = observe(&mut wrapped_in(&fixture, repo.as_path(), arguments), b"");
        let took: Duration = began.elapsed();
        let said: String = text(&observed.stderr);
        assert_eq!(observed.code, Some(0), "git {}: {said}", arguments.join(" "));
        assert!(
            took < bound,
            "git {} took {took:?}, and the wait for the lock this command used to make lasts {MERGE_LOCK_CEILING:?}: {said}",
            arguments.join(" ")
        );
        assert!(
            !said.contains("has changed nothing yet and waits for the index lock"),
            "a read-only command must not wait for the lock: git {}: {said}",
            arguments.join(" ")
        );
        assert!(said.contains("refs/cli-git/kept/1"), "it names the merge saved on its side ref: {said}");
        assert!(said.contains("nothing of it is in the checkout yet"), "it says why it need not wait: {said}");
        assert!(said.contains("goes on without waiting for it"), "it says this command goes on: {said}");
    }
    assert!(lock.exists(), "the lock is not the command's to remove");
    assert_eq!(records_in(records.as_path()).len(), 1, "the record is kept for the next command");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "main"]).trim(), old, "the branch did not move");
    // The lock goes, and the next command finishes what the saved merge left undone and says so: the notice the route
    //  prints for a command that lands a merge someone else's process started.
    std::fs::remove_file(lock.as_path()).expect("the other program lets go");
    let finished: Observed = observe(&mut wrapped_in(&fixture, repo.as_path(), &["status", "--short"]), b"");
    let said: String = text(&finished.stderr);
    assert_eq!(finished.code, Some(0), "{said}");
    assert!(said.contains("finished an interrupted merge"), "the command that lands the merge says so: {said}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "main"]).trim(), merge, "the saved merge landed");
    assert_eq!(records_in(records.as_path()).len(), 0, "the record goes with the merge it named");
    remove_settled(&fixture);
}

/// The parts of a merge commit that must equal native Git's:
/// tree,
/// both parents,
/// message,
/// author and committer names.
fn commit_shape(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return vec![
        git_text(fixture, repo, &["rev-parse", "HEAD^{tree}"]),
        git_text(fixture, repo, &["rev-parse", "HEAD^1"]),
        git_text(fixture, repo, &["rev-parse", "HEAD^2"]),
        git_text(fixture, repo, &["log", "-1", "--format=%an <%ae>|%cn <%ce>|%B"]),
    ];
}

/// The `ls-files --stage --debug` blocks (the entry line and the stat lines under it) of the named paths.
fn entries_of(listing: &str, names: &[&str]) -> Vec<String> {
    let mut blocks: Vec<String> = Vec::new();
    let mut keep: bool = false;
    for line in listing.lines() {
        if !line.starts_with(' ') {
            let path: &str = line.split('\t').nth(1).unwrap_or_default();
            keep = names.contains(&path);
        }
        if keep {
            blocks.push(line.to_owned());
        }
    }
    return blocks;
}

/// The text of a stream.
fn text(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).into_owned();
}

/// For each way of asking for a merge,
/// the wrapper's merge over another session's staged work has the tree,
/// the
/// parents,
/// the message and the output that native Git's merge has on a clean copy.
#[test]
fn the_merge_equals_what_native_git_writes_on_a_clean_checkout() {
    let cases: [(&str, &[&str]); 9] = [
        ("default", &["feature"]),
        ("no-edit", &["--no-edit", "feature"]),
        ("message", &["-m", "custom message", "feature"]),
        ("messages", &["-m", "first paragraph", "-m", "second paragraph", "feature"]),
        ("untidy", &["-m", "  spaced   \n\n\n\ntail  ", "feature"]),
        ("log", &["--log", "feature"]),
        ("no-log", &["--no-log", "feature"]),
        ("quiet", &["--quiet", "feature"]),
        ("no-stat", &["--no-stat", "feature"]),
    ];
    for (name, arguments) in cases {
        let fixture: Fixture = fixture(format!("merge-oracle-{name}").as_str());
        let dirty: PathBuf = diverged(&fixture, "dirty");
        let clean: PathBuf = copy_of(&fixture, dirty.as_path(), "clean");
        stage_unrelated_work(&fixture, dirty.as_path());
        let mut merge: Vec<&str> = vec!["merge"];
        merge.extend_from_slice(arguments);
        let ours: Observed = run(&fixture, dirty.as_path(), merge.as_slice());
        let theirs: Observed = native(&fixture, clean.as_path(), merge.as_slice());
        assert_eq!(theirs.code, Some(0), "{name}: native Git merges the clean copy: {}", text(&theirs.stderr));
        assert_eq!(ours.code, Some(0), "{name}: {}", text(&ours.stderr));
        assert_eq!(text(&ours.stdout), text(&theirs.stdout), "{name}: output");
        assert_eq!(text(&ours.stderr), text(&theirs.stderr), "{name}: diagnostics");
        assert_eq!(commit_shape(&fixture, dirty.as_path()), commit_shape(&fixture, clean.as_path()), "{name}");
        remove_settled(&fixture);
    }
}

/// A tag,
/// a remote-tracking branch and an abbreviated commit name are described the way native Git describes them.
#[test]
fn every_kind_of_merged_name_is_worded_as_native_git_words_it() {
    for kind in ["tag", "remote-tracking", "abbreviated-commit"] {
        let fixture: Fixture = fixture(format!("merge-oracle-name-{kind}").as_str());
        let dirty: PathBuf = diverged(&fixture, "dirty");
        let tip: String = git_text(&fixture, dirty.as_path(), &["rev-parse", "feature"]);
        let name: String = match kind {
            "tag" => {
                git(&fixture, dirty.as_path(), &["tag", "release-1", "feature"]);
                String::from("release-1")
            }
            "remote-tracking" => {
                git(&fixture, dirty.as_path(), &["update-ref", "refs/remotes/origin/work", tip.trim()]);
                String::from("origin/work")
            }
            _ => String::from(&tip.trim()[..10]),
        };
        let clean: PathBuf = copy_of(&fixture, dirty.as_path(), "clean");
        stage_unrelated_work(&fixture, dirty.as_path());
        let ours: Observed = run(&fixture, dirty.as_path(), &["merge", "--no-edit", name.as_str()]);
        let theirs: Observed = native(&fixture, clean.as_path(), &["merge", "--no-edit", name.as_str()]);
        assert_eq!((ours.code, theirs.code), (Some(0), Some(0)), "{kind}: {}", text(&ours.stderr));
        assert_eq!(commit_shape(&fixture, dirty.as_path()), commit_shape(&fixture, clean.as_path()), "{kind}");
        remove_settled(&fixture);
    }
}

/// The staged entries,
/// the unstaged edits and the untracked files of the other session are exactly as they were:
/// the same index entries with the same stat data,
/// the same bytes on disk.
#[test]
fn other_sessions_work_is_byte_identical_afterwards() {
    let fixture: Fixture = fixture("merge-keeps-other-work");
    let repo: PathBuf = diverged(&fixture, "repo");
    stage_unrelated_work(&fixture, repo.as_path());
    std::fs::write(repo.join("w.txt"), b"w.txt edited and not staged\n").expect("unstaged edit");
    std::fs::write(repo.join("untracked.txt"), b"untracked\n").expect("untracked file");
    let before: Snapshot = snapshot(&fixture, repo.as_path());
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", text(&observed.stderr));
    let after: Snapshot = snapshot(&fixture, repo.as_path());
    assert_ne!(after.head, before.head, "the branch moved");
    let untouched = |snapshot: &Snapshot| -> Vec<String> {
        return entries_of(snapshot.entries.as_str(), &["u.txt", "w.txt", "h.txt", "new.txt"]);
    };
    assert_eq!(untouched(&after), untouched(&before), "the entries of paths the merge does not change");
    assert!(untouched(&before).iter().filter(|line| { return !line.starts_with(' '); }).count() == 4);
    assert_eq!(
        after.status.lines().filter(|line| { return !line.contains("dir/") && !line.contains("f.txt"); }).collect::<Vec<&str>>(),
        before.status.lines().collect::<Vec<&str>>(),
        "the staged and unstaged work reads as it did"
    );
    assert_eq!(std::fs::read(repo.join("w.txt")).expect("w"), b"w.txt edited and not staged\n");
    assert_eq!(std::fs::read(repo.join("u.txt")).expect("u"), b"u.txt staged by another session\n");
    assert_eq!(std::fs::read(repo.join("untracked.txt")).expect("untracked"), b"untracked\n");
    assert_eq!(std::fs::read(repo.join("f.txt")).expect("f"), b"f.txt feature\n");
    assert!(!repo.join("gone.txt").exists() && repo.join("dir/new.txt").exists());
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1\n");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

/// The one JSON line a refusal writes.
fn finding_of(observed: &Observed) -> serde_json::Value {
    let stderr: String = text(&observed.stderr);
    assert_eq!(stderr.matches('\n').count(), 1, "one event line: {stderr}");
    return serde_json::from_str(stderr.as_str()).expect("a JSON event");
}

/// A merge that changes a path the other session staged is refused with the path and the ways forward named,
/// and
/// nothing is changed:
/// not the branch,
/// not one byte of the index,
/// not a file.
#[test]
fn a_merge_over_a_staged_path_is_refused_and_changes_nothing() {
    let fixture: Fixture = fixture("merge-refuses-staged-overlap");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("g.txt"), b"g.txt staged by another session\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "g.txt"]);
    let before: Snapshot = snapshot(&fixture, repo.as_path());
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(2));
    assert_eq!(text(&observed.stdout), "");
    let finding = finding_of(&observed);
    assert_eq!(finding["code"], "merge-isolation/paths-in-use");
    let message: &str = finding["message"].as_str().expect("message");
    assert!(message.contains("g.txt") && !message.contains("u.txt"), "{message}");
    for way in ["wait until", "clean linked worktree", "git stash push"] {
        assert!(message.contains(way), "{way}: {message}");
    }
    assert_eq!(snapshot(&fixture, repo.as_path()), before);
    assert!(!repo.join(".git/MERGE_HEAD").exists());
    assert!(!repo.join(".git/monochromatic-git-policy-cli-merges").exists(), "no record is written before the check passes");
    assert_eq!(std::fs::read(repo.join("g.txt")).expect("g"), b"g.txt staged by another session\n");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

/// A staged deletion of a path the merge changes and an unstaged edit of one are the same kind of overlap.
#[test]
fn other_kinds_of_overlap_are_refused_the_same_way() {
    for kind in ["staged-deletion", "unstaged-edit", "staged-rename-source"] {
        let fixture: Fixture = fixture(format!("merge-refuses-{kind}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        match kind {
            "staged-deletion" => {
                git(&fixture, repo.as_path(), &["rm", "--quiet", "--", "f.txt"]);
                std::fs::write(repo.join("u.txt"), b"another change\n").expect("file");
                git(&fixture, repo.as_path(), &["add", "--", "u.txt"]);
            }
            "unstaged-edit" => {
                std::fs::write(repo.join("f.txt"), b"f.txt edited, not staged\n").expect("file");
                std::fs::write(repo.join("u.txt"), b"another change\n").expect("file");
                git(&fixture, repo.as_path(), &["add", "--", "u.txt"]);
            }
            _ => {
                git(&fixture, repo.as_path(), &["mv", "g.txt", "g-renamed.txt"]);
            }
        }
        let before: Snapshot = snapshot(&fixture, repo.as_path());
        let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
        assert_eq!(observed.code, Some(2), "{kind}: {}", text(&observed.stderr));
        let finding = finding_of(&observed);
        assert_eq!(finding["code"], "merge-isolation/paths-in-use", "{kind}");
        assert_eq!(snapshot(&fixture, repo.as_path()), before, "{kind}");
        remove_settled(&fixture);
    }
}

/// A merge that conflicts textually changes nothing and names the conflicting paths and the remedies.
#[test]
fn a_conflicting_merge_changes_nothing_and_names_the_paths() {
    let fixture: Fixture = fixture("merge-refuses-conflict");
    let repo: PathBuf = diverged(&fixture, "repo");
    std::fs::write(repo.join("f.txt"), b"f.txt main edit\n").expect("file");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main edits f"]);
    stage_unrelated_work(&fixture, repo.as_path());
    let before: Snapshot = snapshot(&fixture, repo.as_path());
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(2), "{}", text(&observed.stderr));
    let finding = finding_of(&observed);
    assert_eq!(finding["code"], "merge-isolation/merge-conflict");
    let message: &str = finding["message"].as_str().expect("message");
    assert!(message.contains("f.txt") && !message.contains("g.txt"), "{message}");
    for way in ["clean linked worktree", "clean checkout", "rebase"] {
        assert!(message.contains(way), "{way}: {message}");
    }
    assert_eq!(snapshot(&fixture, repo.as_path()), before);
    assert!(!repo.join(".git/MERGE_HEAD").exists());
    assert_eq!(std::fs::read(repo.join("f.txt")).expect("f"), b"f.txt main edit\n", "no conflict markers");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

/// An untracked file where the merge adds one is Git's own refusal at the last step:
/// the merge commit stays
/// unreachable,
/// the record is gone,
/// and the checkout is as it was.
#[test]
fn an_untracked_file_in_the_way_is_refused_by_git_and_changes_nothing() {
    let fixture: Fixture = fixture("merge-refuses-untracked");
    let repo: PathBuf = diverged(&fixture, "repo");
    stage_unrelated_work(&fixture, repo.as_path());
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/new.txt"), b"an untracked file in the way\n").expect("file");
    let before: Snapshot = snapshot(&fixture, repo.as_path());
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_ne!(observed.code, Some(0));
    let stderr: String = text(&observed.stderr);
    assert!(stderr.contains("would be overwritten") && stderr.contains("dir/new.txt"), "Git's own words: {stderr}");
    assert!(stderr.contains("merge-isolation/not-applied"), "the finding follows: {stderr}");
    assert_eq!(snapshot(&fixture, repo.as_path()), before);
    assert_eq!(std::fs::read(repo.join("dir/new.txt")).expect("file"), b"an untracked file in the way\n");
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "the record is removed");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

/// With nothing staged the wrapper completes the merge itself,
/// because native Git's merge stashes first and moves the
/// branch only after it changed the checkout:
/// the merge commit,
/// the output and the checkout equal what native Git
/// writes on a clean copy.
#[test]
fn a_clean_index_merge_is_completed_and_equals_what_native_git_writes() {
    let fixture: Fixture = fixture("merge-clean-index");
    let wrapped_repo: PathBuf = diverged(&fixture, "wrapped");
    let clean: PathBuf = copy_of(&fixture, wrapped_repo.as_path(), "native");
    let ours: Observed = run(&fixture, wrapped_repo.as_path(), &["merge", "--no-edit", "feature"]);
    let theirs: Observed = native(&fixture, clean.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!((ours.code, text(&ours.stdout), text(&ours.stderr)), (theirs.code, text(&theirs.stdout), text(&theirs.stderr)));
    assert_eq!(commit_shape(&fixture, wrapped_repo.as_path()), commit_shape(&fixture, clean.as_path()));
    assert_eq!(git_text(&fixture, wrapped_repo.as_path(), &["status", "--porcelain=v1"]), "");
    assert_eq!(records_in(wrapped_repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "no record is left");
    remove_settled(&fixture);
}

/// A conflict with nothing staged is native Git's:
/// it writes the markers for the person to resolve in the worktree.
#[test]
fn a_clean_index_conflict_is_native_gits() {
    let fixture: Fixture = fixture("merge-clean-index-conflict");
    let repo: PathBuf = diverged(&fixture, "wrapped");
    std::fs::write(repo.join("f.txt"), b"f.txt main\n").expect("file");
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main edits f"]);
    let clean: PathBuf = copy_of(&fixture, repo.as_path(), "native");
    let ours: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    let theirs: Observed = native(&fixture, clean.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!((ours.code, text(&ours.stdout)), (theirs.code, text(&theirs.stdout)));
    assert_eq!(theirs.code, Some(1));
    assert!(repo.join(".git/MERGE_HEAD").exists(), "the merge waits for the person's resolution, as in native Git");
    assert!(std::fs::read_to_string(repo.join("f.txt")).expect("file").contains("<<<<<<<"));
    remove_settled(&fixture);
}

/// A fast-forward with unrelated staged work is native Git's own two-way update,
/// which keeps the staged work.
#[test]
fn a_fast_forward_over_unrelated_staged_work_is_native_gits() {
    let fixture: Fixture = fixture("merge-fast-forward");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("u.txt"), b"u\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "u.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "f.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=feature"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("u.txt"), b"u staged\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "u.txt"]);
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", text(&observed.stderr));
    assert!(text(&observed.stdout).contains("Fast-forward"), "{}", text(&observed.stdout));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), git_text(&fixture, repo.as_path(), &["rev-parse", "feature"]));
    assert_eq!(git_text(&fixture, repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt\n");
    remove_settled(&fixture);
}

/// `--no-ff` over a fast-forward is a true merge,
/// which native Git refuses with staged work;
/// the wrapper completes it.
#[test]
fn no_ff_over_a_fast_forward_is_completed() {
    let fixture: Fixture = fixture("merge-no-ff");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::write(repo.join("u.txt"), b"u\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "u.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "f.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=feature"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    let clean: PathBuf = copy_of(&fixture, repo.as_path(), "clean");
    std::fs::write(repo.join("u.txt"), b"u staged\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "u.txt"]);
    let ours: Observed = run(&fixture, repo.as_path(), &["merge", "--no-ff", "--no-edit", "feature"]);
    let theirs: Observed = native(&fixture, clean.as_path(), &["merge", "--no-ff", "--no-edit", "feature"]);
    assert_eq!((ours.code, theirs.code), (Some(0), Some(0)), "{}", text(&ours.stderr));
    assert_eq!(commit_shape(&fixture, repo.as_path()), commit_shape(&fixture, clean.as_path()));
    assert_eq!(git_text(&fixture, repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt\n");
    remove_settled(&fixture);
}

/// Shapes whose result the wrapper does not reproduce reach native Git unchanged,
/// with its refusal.
#[test]
fn shapes_that_cannot_be_made_exact_fall_back_to_native_git() {
    let cases: [(&str, &[&str]); 7] = [
        ("squash", &["--squash", "feature"]),
        ("no-commit", &["--no-commit", "feature"]),
        ("ours", &["-s", "ours", "feature"]),
        ("strategy-option", &["-X", "theirs", "feature"]),
        ("ff-only", &["--ff-only", "feature"]),
        ("abbreviation", &["--no-e", "feature"]),
        ("editor", &["--edit", "feature"]),
    ];
    for (name, arguments) in cases {
        let fixture: Fixture = fixture(format!("merge-fallback-{name}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        let clean: PathBuf = copy_of(&fixture, repo.as_path(), "clean");
        stage_unrelated_work(&fixture, repo.as_path());
        stage_unrelated_work(&fixture, clean.as_path());
        let mut merge: Vec<&str> = vec!["merge"];
        merge.extend_from_slice(arguments);
        let ours: Observed = run(&fixture, repo.as_path(), merge.as_slice());
        let theirs: Observed = native(&fixture, clean.as_path(), merge.as_slice());
        assert_eq!((ours.code, text(&ours.stdout)), (theirs.code, text(&theirs.stdout)), "{name}");
        assert_eq!(text(&ours.stderr), text(&theirs.stderr), "{name}: native Git's words");
        assert!(!repo.join(".git/monochromatic-git-policy-cli-merges").exists(), "{name}");
        remove_settled(&fixture);
    }
}

/// Two branches at once is not one merge commit of two parents;
/// native Git refuses it and the wrapper does not guess.
#[test]
fn an_octopus_merge_falls_back_to_native_git() {
    let fixture: Fixture = fixture("merge-fallback-octopus");
    let repo: PathBuf = diverged(&fixture, "repo");
    git(&fixture, repo.as_path(), &["branch", "second", "feature"]);
    let clean: PathBuf = copy_of(&fixture, repo.as_path(), "clean");
    stage_unrelated_work(&fixture, repo.as_path());
    stage_unrelated_work(&fixture, clean.as_path());
    let ours: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature", "second"]);
    let theirs: Observed = native(&fixture, clean.as_path(), &["merge", "--no-edit", "feature", "second"]);
    assert_eq!((ours.code, text(&ours.stdout), text(&ours.stderr)), (theirs.code, text(&theirs.stdout), text(&theirs.stderr)));
    remove_settled(&fixture);
}

/// A branch that is already merged is native Git's answer,
/// whatever is staged.
#[test]
fn an_up_to_date_merge_is_native_gits() {
    let fixture: Fixture = fixture("merge-up-to-date");
    let repo: PathBuf = diverged(&fixture, "repo");
    git(&fixture, repo.as_path(), &["merge", "--quiet", "--no-edit", "feature"]);
    stage_unrelated_work(&fixture, repo.as_path());
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!((observed.code, text(&observed.stdout)), (Some(0), String::from("Already up to date.\n")));
    remove_settled(&fixture);
}

/// Configure a hook that appends its name to a marker file outside the worktree.
fn marker_hook(fixture: &Fixture, repo: &Path, event: &str) -> PathBuf {
    let marker: PathBuf = fixture.root.join(format!("{event}.marker"));
    let script: PathBuf = fixture.root.join(format!("{event}-hook"));
    executable(script.as_path(), format!("#!/bin/sh\necho ran >> '{}'\n", marker.display()).as_bytes());
    git(fixture, repo, &["config", format!("hook.control-{event}.event").as_str(), event]);
    git(fixture, repo, &["config", format!("hook.control-{event}.command").as_str(), script.to_str().expect("UTF-8")]);
    return marker;
}

/// The merge runs no configured hook at all,
/// `post-merge` included:
/// the wrapper supports no external hook program,
/// and
/// the merge commit is made from objects,
/// as the auto-push merge is.
#[test]
fn no_configured_hook_runs_on_the_isolated_merge() {
    let fixture: Fixture = fixture("merge-hooks");
    let repo: PathBuf = diverged(&fixture, "repo");
    let post_merge: PathBuf = marker_hook(&fixture, repo.as_path(), "post-merge");
    let pre_merge_commit: PathBuf = marker_hook(&fixture, repo.as_path(), "pre-merge-commit");
    let commit_msg: PathBuf = marker_hook(&fixture, repo.as_path(), "commit-msg");
    stage_unrelated_work(&fixture, repo.as_path());
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", text(&observed.stderr));
    assert!(
        !post_merge.exists() && !pre_merge_commit.exists() && !commit_msg.exists(),
        "no hook runs on the isolated merge"
    );
    remove_settled(&fixture);
}

/// A repository that signs commits gets a signed merge,
/// as `commit-tree` never reads `commit.gpgSign` by itself.
#[test]
fn a_signing_repository_gets_a_signed_merge() {
    let fixture: Fixture = fixture("merge-signed");
    let repo: PathBuf = diverged(&fixture, "repo");
    let program: PathBuf = fixture.root.join("stand-in-gpg");
    executable(
        program.as_path(),
        b"#!/bin/sh\ncat > /dev/null\necho '[GNUPG:] SIG_CREATED ' >&2\nprintf -- '-----BEGIN PGP SIGNATURE-----\\n\\nplaceholder\\n-----END PGP SIGNATURE-----\\n'\n",
    );
    git(&fixture, repo.as_path(), &["config", "gpg.program", program.to_str().expect("UTF-8")]);
    git(&fixture, repo.as_path(), &["config", "commit.gpgsign", "true"]);
    stage_unrelated_work(&fixture, repo.as_path());
    let observed: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", text(&observed.stderr));
    assert!(git_text(&fixture, repo.as_path(), &["cat-file", "commit", "HEAD"]).contains("\ngpgsig "), "signed");
    remove_settled(&fixture);
}

/// What:
///  Run `command` to its end like `observe`,
/// and create the file `release` as soon as a wait notice is read from its
///  standard error.
/// Why:
///  A child process held until its release file exists is still running when the wrapper's watcher prints,
/// whatever
///  the host's load,
/// so a control for the notice depends on no sleep and no schedule.
fn observe_releasing_on_notice(command: &mut Command, release: &Path) -> Observed {
    use std::io::{BufRead, Read};
    let mut child = command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("process starts");
    let mut standard_output = child.stdout.take().expect("piped stdout");
    // Read concurrently, so a full pipe never blocks the child while this thread waits on its standard error.
    let output_reader = std::thread::spawn(move || {
        let mut bytes: Vec<u8> = Vec::new();
        let _read = standard_output.read_to_end(&mut bytes);
        return bytes;
    });
    let mut errors = std::io::BufReader::new(child.stderr.take().expect("piped stderr"));
    let mut stderr: Vec<u8> = Vec::new();
    loop {
        let line_start: usize = stderr.len();
        if errors.read_until(b'\n', &mut stderr).expect("read standard error") == 0 {
            break;
        }
        if stderr[line_start..].starts_with(WAIT_NOTICE_PREFIX) {
            std::fs::write(release, b"").expect("release the held child");
        }
    }
    let status = child.wait().expect("process finishes");
    assert_ne!(
        status.signal(),
        Some(TIMED_OUT_SIGNAL),
        "the command hit its bound before it ended: {:?}",
        command.get_args().collect::<Vec<&OsStr>>()
    );
    return Observed {
        code: status.code(),
        stdout: output_reader.join().expect("standard output reader"),
        stderr: without_hint_fields(stderr.as_slice()),
    };
}

/// What:
///  A healthy `git commit-tree` that outlasts the notice grace makes the wrapper print its wait notice,
/// and the
///  controls that compare diagnostics leave that notice out.
/// Why:
///  A loaded host makes any step slow,
/// so the notice is correct behaviour and its absence cannot be asserted.
/// A signer
///  that merely sleeps longer than the grace proves nothing about a loaded host,
/// where the wrapper's watcher can resume
///  after the signer has ended and find nothing to announce.
/// The signer stand-in therefore waits for a release file,
///  and the control writes that file only when the notice has reached the wrapper's standard error,
/// so the step is
///  still running when the notice is due on every host.
#[test]
fn a_slow_commit_tree_prints_a_wait_notice_that_the_comparison_leaves_out() {
    /// The notice interval,
    /// in seconds,
    /// and so the grace of the first notice.
    const NOTICE_SECONDS: u64 = 1;
    /// How long the signer stand-in waits for its release at most;
    /// it ends sooner as soon as the release exists,
    /// and
    /// only a wrapper that never prints the notice makes it run out,
    /// so that the control fails instead of hanging.
    const HOLD_LIMIT_SECONDS: u64 = 60;
    /// The tenths of a second the signer stand-in sleeps between two looks for its release.
    const LOOK_TENTHS: u64 = 1;
    let fixture: Fixture = fixture("merge-slow-signer");
    let repo: PathBuf = diverged(&fixture, "repo");
    let program: PathBuf = fixture.root.join("slow-gpg");
    let release: PathBuf = fixture.root.join("slow-gpg-release");
    executable(
        program.as_path(),
        format!(
            "#!/bin/sh\ncat > /dev/null\nlooks=0\nwhile [ ! -e '{}' ] && [ \"$looks\" -lt {} ]; do\n  sleep 0.{LOOK_TENTHS}\n  looks=$((looks + 1))\ndone\necho '[GNUPG:] SIG_CREATED ' >&2\nprintf -- '-----BEGIN PGP SIGNATURE-----\\n\\nplaceholder\\n-----END PGP SIGNATURE-----\\n'\n",
            release.display(),
            HOLD_LIMIT_SECONDS * 10 / LOOK_TENTHS
        )
        .as_bytes(),
    );
    git(&fixture, repo.as_path(), &["config", "gpg.program", program.to_str().expect("UTF-8")]);
    git(&fixture, repo.as_path(), &["config", "commit.gpgsign", "true"]);
    stage_unrelated_work(&fixture, repo.as_path());
    let raw: Observed = observe_releasing_on_notice(
        wrapped_in(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"])
            .env("CLI_GIT_WAIT_NOTICE_SECONDS", NOTICE_SECONDS.to_string()),
        release.as_path(),
    );
    assert_eq!(raw.code, Some(0), "{}", text(&raw.stderr));
    assert!(
        text(&raw.stderr).contains("monochromatic-git-policy-cli: still waiting for git commit-tree"),
        "the slow step announces itself: {}",
        text(&raw.stderr)
    );
    assert_eq!(text(&without_wait_notices(raw.stderr.as_slice())), "", "nothing else was said");
    remove_settled(&fixture);
}

/// Global options before `merge` (`-c` settings and `-C` directories) reach every query the isolated merge makes.
#[test]
fn global_options_reach_every_query_of_the_merge() {
    let fixture: Fixture = fixture("merge-global-options");
    let repo: PathBuf = diverged(&fixture, "repo");
    stage_unrelated_work(&fixture, repo.as_path());
    std::fs::write(repo.join("sub/s.txt"), b"s staged by another session\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "sub/s.txt"]);
    let configured: Observed = run(&fixture, repo.as_path(), &["-c", "merge.stat=false", "merge", "--no-edit", "feature"]);
    assert_eq!(configured.code, Some(0), "{}", text(&configured.stderr));
    assert_eq!(text(&configured.stdout), "Merge made by the 'ort' strategy.\n", "merge.stat=false hides the diffstat");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1\n");
    git(&fixture, repo.as_path(), &["reset", "--hard", "--quiet", "HEAD^"]);
    stage_unrelated_work(&fixture, repo.as_path());
    let elsewhere: Observed = run(&fixture, fixture.root.as_path(), &["-C", repo.to_str().expect("UTF-8"), "merge", "--no-edit", "feature"]);
    assert_eq!(elsewhere.code, Some(0), "{}", text(&elsewhere.stderr));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1\n");
    let reflog: String = git_text(&fixture, repo.as_path(), &["reflog", "show", "--format=%gs", "-1", "HEAD"]);
    assert_eq!(reflog, "merge feature: Fast-forward\n");
    remove_settled(&fixture);
}

/// A merge started from a linked worktree whose index holds staged work is completed there.
#[test]
fn a_linked_worktree_with_staged_work_merges_too() {
    let fixture: Fixture = fixture("merge-linked-worktree");
    let repo: PathBuf = diverged(&fixture, "repo");
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "side", linked.to_str().expect("UTF-8"), "main"]);
    std::fs::write(linked.join("u.txt"), b"u.txt staged in the linked worktree\n").expect("file");
    git(&fixture, linked.as_path(), &["add", "--", "u.txt"]);
    let observed: Observed = run(&fixture, linked.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(observed.code, Some(0), "{}", text(&observed.stderr));
    assert_eq!(git_text(&fixture, linked.as_path(), &["status", "--porcelain=v1"]), "M  u.txt\n");
    assert_eq!(git_text(&fixture, linked.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "main"]), git_text(&fixture, linked.as_path(), &["rev-parse", "main"]));
    remove_settled(&fixture);
}

/// Start a merge that a phase marker kills,
/// and return what the killed process left.
fn killed_merge_at(fixture: &Fixture, repo: &Path, phase: &str) -> PathBuf {
    let markers: PathBuf = fixture.root.join(format!("markers-{phase}"));
    std::fs::create_dir(&markers).expect("marker directory");
    let signal: String = format!("{phase}:kill:{}", markers.display());
    let output = wrapped_commit(fixture, repo, &["merge", "--no-edit", "feature"])
        .env(PHASE_SIGNAL_VARIABLE, signal)
        .output()
        .expect("the merge starts");
    assert!(
        output.status.signal() == Some(9) || output.status.code() == Some(137),
        "the merge died of the marker: {:?} {}",
        output.status,
        text(&output.stderr)
    );
    assert!(markers.join(format!("{phase}.reached")).exists(), "{phase} was reached");
    return repo.join(".git/monochromatic-git-policy-cli-merges");
}

/// The files in a records directory.
pub(super) fn records_in(directory: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(directory)
        .map(|entries| { return entries.flatten().map(|entry| { return entry.path(); }).collect(); })
        .unwrap_or_default();
    files.sort();
    return files;
}

/// A process killed after it wrote its record and before Git touched the checkout:
/// the next command discards the
/// merge,
/// and the checkout is exactly as it was.
#[test]
fn a_kill_before_the_checkout_changes_is_discarded() {
    let fixture: Fixture = fixture("merge-kill-intent");
    let repo: PathBuf = diverged(&fixture, "repo");
    stage_unrelated_work(&fixture, repo.as_path());
    let before: Snapshot = snapshot(&fixture, repo.as_path());
    let records: PathBuf = killed_merge_at(&fixture, repo.as_path(), "merge-intent");
    assert_eq!(records_in(records.as_path()).len(), 1, "the dead merge left its record");
    let status: Observed = run(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(status.code, Some(0), "{}", text(&status.stderr));
    assert_eq!(records_in(records.as_path()), Vec::<PathBuf>::new(), "recovery removed the record");
    assert_eq!(snapshot(&fixture, repo.as_path()).head, before.head);
    assert_eq!(snapshot(&fixture, repo.as_path()).entries, before.entries);
    assert!(!repo.join(".git/index.lock").exists());
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    let again: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!(again.code, Some(0), "the merge can be run again: {}", text(&again.stderr));
    remove_settled(&fixture);
}

/// A process killed after Git moved the branch and before it removed its record:
/// the next command only removes it.
#[test]
fn a_kill_after_the_branch_moved_leaves_a_consistent_checkout() {
    let fixture: Fixture = fixture("merge-kill-applied");
    let repo: PathBuf = diverged(&fixture, "repo");
    stage_unrelated_work(&fixture, repo.as_path());
    let records: PathBuf = killed_merge_at(&fixture, repo.as_path(), "merge-applied");
    assert_eq!(records_in(records.as_path()).len(), 1);
    let status: Observed = run(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(status.code, Some(0), "{}", text(&status.stderr));
    assert_eq!(text(&status.stdout).lines().filter(|line| { return line.starts_with("M  u.txt") || line.starts_with("A  new.txt") || line.starts_with("D  v.txt"); }).count(), 3);
    assert_eq!(text(&status.stdout).lines().count(), 3, "only the other session's work is staged: {}", text(&status.stdout));
    assert_eq!(records_in(records.as_path()), Vec::<PathBuf>::new());
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1\n");
    git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
    remove_settled(&fixture);
}

/// Git updated the worktree and the index and the process died before the branch moved:
/// the next command
/// finishes the merge,
/// and another session's staged work is untouched.
#[test]
fn a_kill_inside_gits_update_is_completed_by_the_next_command() {
    for stage in ["checkout-and-index", "worktree-only"] {
        let fixture: Fixture = fixture(format!("merge-kill-inside-{stage}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        stage_unrelated_work(&fixture, repo.as_path());
        let staged: String = git_text(&fixture, repo.as_path(), &["--no-optional-locks", "ls-files", "--stage", "--debug", "--", "u.txt"]);
        let records: PathBuf = killed_merge_at(&fixture, repo.as_path(), "merge-intent");
        let record: String = std::fs::read_to_string(records_in(records.as_path())[0].as_path()).expect("record");
        let new_oid: String = serde_json::from_str::<serde_json::Value>(record.as_str()).expect("record JSON")["newOid"]
            .as_str()
            .expect("newOid")
            .to_owned();
        let old_oid: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
        if stage == "checkout-and-index" {
            git(&fixture, repo.as_path(), &["read-tree", "-m", "-u", old_oid.as_str(), new_oid.trim()]);
        } else {
            std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
            std::fs::write(repo.join("g.txt"), b"g.txt feature\n").expect("file");
            std::fs::remove_file(repo.join("gone.txt")).expect("removed by Git");
        }
        let status: Observed = run(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
        assert_eq!(status.code, Some(0), "{stage}: {}", text(&status.stderr));
        assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), new_oid.trim(), "{stage}: the branch moved");
        assert_eq!(text(&status.stdout).lines().count(), 3, "{stage}: only the other session's work is staged: {}", text(&status.stdout));
        assert_eq!(
            git_text(&fixture, repo.as_path(), &["--no-optional-locks", "ls-files", "--stage", "--debug", "--", "u.txt"]),
            staged,
            "{stage}: the staged entry is byte-identical"
        );
        assert_eq!(records_in(records.as_path()), Vec::<PathBuf>::new(), "{stage}");
        git(&fixture, repo.as_path(), &["fsck", "--strict", "--no-dangling"]);
        remove_settled(&fixture);
    }
}

/// A record damaged the way a power loss damages files:
/// emptied or cut in half,
/// recovery fails closed naming the
/// record and keeps it;
/// deleted,
/// there is nothing to recover and the checkout is whatever Git left.
#[test]
fn a_damaged_record_fails_closed_and_a_missing_one_recovers_nothing() {
    for damage in ["empty", "half", "delete"] {
        let fixture: Fixture = fixture(format!("merge-tear-{damage}").as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        stage_unrelated_work(&fixture, repo.as_path());
        let records: PathBuf = killed_merge_at(&fixture, repo.as_path(), "merge-intent");
        let record: PathBuf = records_in(records.as_path())[0].clone();
        let bytes: Vec<u8> = std::fs::read(record.as_path()).expect("record");
        match damage {
            "empty" => std::fs::write(record.as_path(), b"").expect("empty it"),
            "half" => std::fs::write(record.as_path(), &bytes[..bytes.len() / 2]).expect("cut it"),
            _ => std::fs::remove_file(record.as_path()).expect("delete it"),
        }
        let status: Observed = run(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
        if damage == "delete" {
            assert_eq!(status.code, Some(0), "{}", text(&status.stderr));
        } else {
            assert_eq!(status.code, Some(2), "{damage}: {}", text(&status.stderr));
            assert!(text(&status.stderr).contains("malformed") && text(&status.stderr).contains(record.to_string_lossy().as_ref()));
            assert!(record.exists(), "the damaged record is kept for inspection");
        }
        assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "0\n", "{damage}");
        remove_settled(&fixture);
    }
}

/// A merge that starts while another merge's record is live waits for the landing lock instead of recovering it;
/// here,
/// a record of a live process (this test) is left alone by a command that runs meanwhile.
#[test]
fn a_live_merges_record_is_left_alone() {
    let fixture: Fixture = fixture("merge-live-record");
    let repo: PathBuf = diverged(&fixture, "repo");
    let records: PathBuf = repo.join(".git/monochromatic-git-policy-cli-merges");
    std::fs::create_dir(&records).expect("directory");
    let owner = git_policy_cli::transaction_owner::current_transaction_owner(
        "0b6c2c1e-6f5b-4d0e-9a55-3f5d8e2f6a10",
        "2026-10-29T00:00:00.000Z",
    )
    .expect("owner");
    let tip: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let intent = git_policy_cli::merge_isolation_journal::MergeIntent {
        owner,
        repository_root: repo.to_path_buf(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: tip.clone(),
        new_oid: tip,
        side_ref: None,
    };
    let record: PathBuf = git_policy_cli::merge_isolation_journal::write_merge_intent(repo.join(".git").as_path(), &intent).expect("record");
    let status: Observed = run(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(status.code, Some(0), "{}", text(&status.stderr));
    assert!(record.exists(), "a running merge's record is not recovered");
    remove_settled(&fixture);
}

/// `merge.ff=only` refuses a merge that cannot fast-forward,
/// which is native Git's answer;
/// `--no-ff` and `--ff` on
/// the command line choose otherwise,
/// and the wrapper completes those over staged work as native Git does on a
/// clean copy.
#[test]
fn merge_ff_only_is_native_gits_unless_the_command_line_chooses() {
    let cases: [(&str, &[&str]); 3] =
        [("default", &["feature"]), ("no-ff", &["--no-ff", "feature"]), ("ff", &["--ff", "feature"])];
    for (name, arguments) in cases {
        let fixture: Fixture = fixture(format!("merge-ff-only-{name}").as_str());
        let dirty: PathBuf = diverged(&fixture, "dirty");
        git(&fixture, dirty.as_path(), &["config", "merge.ff", "only"]);
        let clean: PathBuf = copy_of(&fixture, dirty.as_path(), "clean");
        stage_unrelated_work(&fixture, dirty.as_path());
        let mut merge: Vec<&str> = vec!["merge", "--no-edit"];
        merge.extend_from_slice(arguments);
        let ours: Observed = run(&fixture, dirty.as_path(), merge.as_slice());
        let theirs: Observed = native(&fixture, clean.as_path(), merge.as_slice());
        assert_eq!(theirs.code == Some(0), name != "default", "{name}: native Git on the clean copy");
        assert_eq!((ours.code, text(&ours.stdout)), (theirs.code, text(&theirs.stdout)), "{name}: {}", text(&ours.stderr));
        if theirs.code == Some(0) {
            assert_eq!(commit_shape(&fixture, dirty.as_path()), commit_shape(&fixture, clean.as_path()), "{name}");
        } else {
            assert_eq!(text(&ours.stderr), text(&theirs.stderr), "{name}: native Git's words");
        }
        remove_settled(&fixture);
    }
}

/// The directory a global `-C` names,
/// relative to where the command runs,
/// reaches the queries of the merge but not
/// the recovery of a merge Git refused:
/// that one runs from the repository root,
/// where the same relative name would
/// name nothing.
#[test]
fn a_relative_directory_option_does_not_break_the_settling_of_a_refused_merge() {
    let fixture: Fixture = fixture("merge-refuses-untracked-relative");
    let repo: PathBuf = diverged(&fixture, "repo");
    stage_unrelated_work(&fixture, repo.as_path());
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/new.txt"), b"an untracked file in the way\n").expect("file");
    let before: Snapshot = snapshot(&fixture, repo.as_path());
    let observed: Observed = run(&fixture, fixture.root.as_path(), &["-C", "repo", "merge", "--no-edit", "feature"]);
    assert_ne!(observed.code, Some(0));
    let stderr: String = text(&observed.stderr);
    assert!(stderr.contains("merge-isolation/not-applied"), "the refusal is settled and reported: {stderr}");
    assert_eq!(snapshot(&fixture, repo.as_path()), before);
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "the record is removed");
    remove_settled(&fixture);
}

/// A merge that starts while a dead merge's record is waiting recovers it first,
/// before it looks at the staged work:
/// the half-applied checkout of the dead merge would otherwise read as another session's staged paths.
#[test]
fn a_merge_recovers_a_dead_merge_before_it_checks_the_staged_work() {
    let fixture: Fixture = fixture("merge-recovers-before-merging");
    let repo: PathBuf = diverged(&fixture, "repo");
    stage_unrelated_work(&fixture, repo.as_path());
    let staged: String = git_text(&fixture, repo.as_path(), &["--no-optional-locks", "ls-files", "--stage", "--debug", "--", "u.txt"]);
    let records: PathBuf = killed_merge_at(&fixture, repo.as_path(), "merge-intent");
    let record: String = std::fs::read_to_string(records_in(records.as_path())[0].as_path()).expect("record");
    let new_oid: String = serde_json::from_str::<serde_json::Value>(record.as_str()).expect("record JSON")["newOid"]
        .as_str()
        .expect("newOid")
        .to_owned();
    let old_oid: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    git(&fixture, repo.as_path(), &["read-tree", "-m", "-u", old_oid.as_str(), new_oid.as_str()]);
    let again: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]);
    assert_eq!((again.code, text(&again.stdout)), (Some(0), String::from("Already up to date.\n")), "{}", text(&again.stderr));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), new_oid, "the dead merge was finished");
    assert_eq!(records_in(records.as_path()), Vec::<PathBuf>::new());
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["--no-optional-locks", "ls-files", "--stage", "--debug", "--", "u.txt"]),
        staged,
        "the staged entry is byte-identical"
    );
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1\n");
    remove_settled(&fixture);
}

/// A pseudo-terminal:
/// the master end the test keeps open,
/// and the slave end a child can use as a terminal.
struct Terminal {
    /// Kept so the slave stays open for the child's whole run.
    _master: std::fs::File,
    /// What the child's standard streams are made from.
    slave: std::fs::File,
}

/// Open a new pseudo-terminal.
fn terminal() -> Terminal {
    use std::os::fd::FromRawFd;
    use std::os::unix::fs::OpenOptionsExt;
    // SAFETY: `posix_openpt` returns a new descriptor that nothing else owns, which `File` takes over at once; the
    // other calls only read or change that descriptor.
    let (master, descriptor): (std::fs::File, libc::c_int) = unsafe {
        let descriptor: libc::c_int = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY);
        assert!(descriptor >= 0, "a pseudo-terminal is opened");
        let master: std::fs::File = std::fs::File::from_raw_fd(descriptor);
        assert_eq!(libc::grantpt(descriptor), 0, "the slave is granted");
        assert_eq!(libc::unlockpt(descriptor), 0, "the slave is unlocked");
        (master, descriptor)
    };
    let name: std::ffi::CString = pseudo_terminal_slave_name(descriptor);
    let path: &str = name.to_str().expect("a UTF-8 terminal name");
    let slave: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOCTTY)
        .open(path)
        .expect("the slave opens");
    return Terminal { _master: master, slave };
}

/// Run a wrapped command with the given standard input and output,
/// standard error piped,
/// to its end.
fn run_on(
    fixture: &Fixture,
    repo: &Path,
    arguments: &[&str],
    streams: (std::process::Stdio, std::process::Stdio),
) -> Observed {
    let (input, output) = streams;
    let finished = wrapped_in(fixture, repo, arguments)
        .stdin(input)
        .stdout(output)
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("process starts")
        .wait_with_output()
        .expect("process finishes");
    return Observed { code: finished.status.code(), stdout: finished.stdout, stderr: finished.stderr };
}

/// Without `--no-edit` or `GIT_MERGE_AUTOEDIT`,
/// a merge opens an editor only when standard input and output are one
/// terminal;
/// then the wrapper hands it to native Git with the notice,
/// and in any other arrangement it completes it.
#[test]
fn the_editor_opens_only_when_input_and_output_are_one_terminal() {
    let one: Terminal = terminal();
    let other: Terminal = terminal();
    let same = || -> (std::process::Stdio, std::process::Stdio) {
        return (
            std::process::Stdio::from(one.slave.try_clone().expect("duplicate")),
            std::process::Stdio::from(one.slave.try_clone().expect("duplicate")),
        );
    };
    let two = || -> (std::process::Stdio, std::process::Stdio) {
        return (
            std::process::Stdio::from(one.slave.try_clone().expect("duplicate")),
            std::process::Stdio::from(other.slave.try_clone().expect("duplicate")),
        );
    };
    let input_only = || -> (std::process::Stdio, std::process::Stdio) {
        return (std::process::Stdio::from(one.slave.try_clone().expect("duplicate")), std::process::Stdio::piped());
    };
    let output_only = || -> (std::process::Stdio, std::process::Stdio) {
        return (std::process::Stdio::null(), std::process::Stdio::from(one.slave.try_clone().expect("duplicate")));
    };
    let neither = || -> (std::process::Stdio, std::process::Stdio) {
        return (std::process::Stdio::null(), std::process::Stdio::piped());
    };
    let cases: [(&str, &dyn Fn() -> (std::process::Stdio, std::process::Stdio), bool); 5] = [
        ("one terminal", &same, true),
        ("two terminals", &two, false),
        ("terminal input only", &input_only, false),
        ("terminal output only", &output_only, false),
        ("no terminal", &neither, false),
    ];
    for (name, streams, opens_editor) in cases {
        let fixture: Fixture = fixture(format!("merge-editor-{}", name.replace(' ', "-")).as_str());
        let repo: PathBuf = diverged(&fixture, "repo");
        stage_unrelated_work(&fixture, repo.as_path());
        let before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        let observed: Observed = run_on(&fixture, repo.as_path(), &["merge", "feature"], streams());
        let notice: bool = text(&observed.stderr).contains("this merge would open an editor");
        assert_eq!(notice, opens_editor, "{name}: {}", text(&observed.stderr));
        let merged: bool = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]) != before;
        assert_eq!(merged, !opens_editor, "{name}: {}", text(&observed.stderr));
        if !opens_editor {
            assert_eq!(observed.code, Some(0), "{name}: {}", text(&observed.stderr));
        }
        remove_settled(&fixture);
    }
}

/// What:
///  The Git starts of a wrapped true merge over staged work,
/// recorded by a stand-in that runs the real Git.
/// Why:
///   The owner's rule is that no Git start of the wrapper gets `--quiet` or `-q`.
/// The merge the wrapper completes ends in a
///  fast-forward of the checkout to the merge commit it built (Git 2.56.0 `builtin/merge.c` reads the option as less
///  output and nothing else),
/// and that start is looked at.
/// The stand-in stands where the wrapper looks for the real Git,
/// ahead
///  of the conventional locations,
/// so every start the wrapper makes passes through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await wrapped(['merge', 'feature'], dirty, { realGit: recordingGit }); expect(startsWith('--ff-only')).toHaveLength(1); expect(quietAmong(startsWith('--ff-only'))).toEqual([]);
/// ```
#[test]
fn the_fast_forward_to_the_merge_commit_starts_without_quiet() {
    let fixture: Fixture = fixture("merge-no-quiet");
    let dirty: PathBuf = diverged(&fixture, "dirty");
    stage_unrelated_work(&fixture, dirty.as_path());
    let log: PathBuf = fixture.root.join("git-starts.log");
    let spy_directory: PathBuf = fixture.root.join("spy");
    std::fs::create_dir(&spy_directory).expect("stand-in directory");
    let script: String = format!("#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec {} \"$@\"\n", log.display(), super::support::REAL_GIT);
    executable(spy_directory.join("git").as_path(), script.as_bytes());
    let mut path: std::ffi::OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(spy_directory.as_os_str());
    let observed: Observed = observe(wrapped_in(&fixture, dirty.as_path(), &["merge", "feature"]).env("PATH", path), b"");
    assert_eq!(observed.code, Some(0), "{}", text(&observed.stderr));
    let starts: Vec<String> = std::fs::read_to_string(&log).expect("the stand-in saw the starts").lines().map(String::from).collect();
    let forwards: Vec<&String> = starts.iter().filter(|start| { return start.split(' ').any(|word| { return word == "--ff-only"; }); }).collect();
    assert_eq!(forwards.len(), 1, "one fast-forward of the checkout: {starts:?}");
    for start in forwards {
        assert!(!start.split(' ').any(|word| { return word == "--quiet" || word == "-q"; }), "the fast-forward carries neither --quiet nor -q: {start}");
    }
    remove_settled(&fixture);
}

/// What:
///  What Git says of the wrapper's own fast-forward reaches the journal,
/// and the person's merge prints what native Git's does.
/// Why:
///   The owner dropped `--quiet` from every run the wrapper starts,
/// so the fast-forward says "Updating a..b" and "Fast-forward";
///  those words are about the wrapper's step,
/// not the merge the person asked for,
/// whose output is compared with native Git's
///  in `the_merge_equals_what_native_git_writes_on_a_clean_checkout`.
/// They go to the journal,
/// as one entry that names the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stdout } = await wrapped(['merge', 'feature'], dirty, { journal }); expect(stdout).not.toContain('Fast-forward'); expect(await journal.next()).toContain('Fast-forward');
/// ```
#[test]
fn what_git_says_of_the_fast_forward_reaches_the_journal_and_not_the_person() {
    use std::os::linux::net::SocketAddrExt;
    let fixture: Fixture = fixture("merge-words");
    let dirty: PathBuf = diverged(&fixture, "dirty");
    stage_unrelated_work(&fixture, dirty.as_path());
    let name: String = format!("cli-git-test-journal-merge-{}", std::process::id());
    let address = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("an abstract address");
    let receiver = std::os::unix::net::UnixDatagram::bind_addr(&address).expect("bind the journal stand-in");
    receiver.set_read_timeout(Some(std::time::Duration::from_secs(5))).expect("a read timeout");
    let observed: Observed = observe(
        wrapped_in(&fixture, dirty.as_path(), &["merge", "feature"]).env(git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@{name}")),
        b"",
    );
    assert_eq!(observed.code, Some(0), "{}", text(&observed.stderr));
    assert!(!text(&observed.stdout).contains("Fast-forward"), "{}", text(&observed.stdout));
    assert!(text(&observed.stdout).contains("Merge made by the 'ort' strategy."), "{}", text(&observed.stdout));
    let mut said: Option<String> = None;
    let mut buffer: Vec<u8> = vec![0; 65_536];
    while said.is_none() {
        let length: usize = receiver.recv(buffer.as_mut_slice()).expect("the entry about the fast-forward reached the journal stand-in");
        let entry: String = String::from_utf8_lossy(&buffer[..length]).into_owned();
        if entry.contains("git merge --ff-only --no-edit") && entry.contains(" said: ") {
            said = Some(entry);
        }
    }
    assert!(said.is_some_and(|entry| { return entry.contains("Fast-forward"); }));
    remove_settled(&fixture);
}

/// What:
///  `--no-summary` hides the end-of-merge diffstat,
///  and `--summary` prints it even with `merge.stat=false`,
///  as native Git 2.56.0 does (measured:
/// `git merge -h` documents `--[no-]summary` as a
///  synonym to `--stat`).
/// Why:
///  The isolated merge prints the diffstat this command line asks for;
///  reading these two switches as no-ops printed the diffstat native Git suppresses,
///  and dropped the one native Git prints.
#[test]
fn the_summary_switches_decide_the_printed_diffstat() {
    let fixture: Fixture = fixture("merge-summary-switches");
    let repo: PathBuf = diverged(&fixture, "repo");
    stage_unrelated_work(&fixture, repo.as_path());
    let hidden: Observed = run(&fixture, repo.as_path(), &["merge", "--no-edit", "--no-summary", "feature"]);
    assert_eq!(hidden.code, Some(0), "{}", text(&hidden.stderr));
    assert_eq!(
        text(&hidden.stdout),
        "Merge made by the 'ort' strategy.\n",
        "--no-summary hides the diffstat, as native Git does"
    );
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "--merges", "HEAD"]), "1\n");
    git(&fixture, repo.as_path(), &["reset", "--hard", "--quiet", "HEAD^"]);
    stage_unrelated_work(&fixture, repo.as_path());
    let shown: Observed = run(&fixture, repo.as_path(), &["-c", "merge.stat=false", "merge", "--no-edit", "--summary", "feature"]);
    assert_eq!(shown.code, Some(0), "{}", text(&shown.stderr));
    assert!(
        text(&shown.stdout).starts_with("Merge made by the 'ort' strategy.\n") && text(&shown.stdout).contains(" | "),
        "--summary prints the diffstat even with merge.stat=false, as native Git does: {}",
        text(&shown.stdout)
    );
    remove_settled(&fixture);
}
