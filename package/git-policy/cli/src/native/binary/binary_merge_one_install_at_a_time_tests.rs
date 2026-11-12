//! What:
//!  One merge installs in a worktree at a time:
//!  a second merge that arrives while the first is part-way waits for it and then merges on what it left,
//!  a merge that is partly installed and cannot be finished names the state to a command that only reads,
//!  a merge whose checkout never gets its content ends and puts back what it held,
//!  and a merge in another worktree of the same repository is not held up.
//! Why:
//!  Two installs beside each other left an index that lacks two files the merge commit holds,
//! or wedged the worktree so every command,
//!  `git status` included,
//!  exited 2 (`F12` of the alpha.12 audit);
//! and a checkout whose `git-lfs` download never answered held the Git and its
//!  filter process running with `index.lock` for good (`F13` of the same audit).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const second = spawn(wrapped(['merge', '--no-edit', 'feature'])); expect(firstAndSecond()).toEqual(theSingleMerge());
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::merge_isolation::records_in;
use super::support::{Fixture, Observed, executable, fixture, git, observe, stderr_of};
use git_policy_cli::transaction_phase::PHASE_SIGNAL_VARIABLE;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

/// How long a control waits for a paused merge to reach its marker.
const MARKER_PATIENCE: Duration = Duration::from_secs(120);

/// The status one merge of `feature` into this shape leaves:
/// the unstaged edit,
/// the staged edit,
/// the untracked file and the ignored file,
/// each carried the way a
///  single merge carries it.
const THE_SINGLE_MERGE_STATUS: &str = " M a.txt\nM  b.txt\n M c.txt\n M d.txt\n";

/// What:
///  The shape both findings were measured in:
/// `feature` changes `a.txt` and `b.txt` and adds `c.txt` and `d.txt`,
/// the last two
///  force-added although `d.txt` is ignored,
/// and the person's own work stands at all four paths.
/// Why:
///   The merge writes every path the person has work in,
/// so one install carries and the other clears the same four
///  paths:
/// the overlap is what left the index short of two files.
fn world(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = fixture.root.join(name);
    std::fs::create_dir(&repo).expect("repository directory");
    git(fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), b"l1\nl2\nl3\nl4\nl5\nl6\nl7\nl8\n").expect("file");
    std::fs::write(repo.join("b.txt"), b"b1\nb2\nb3\nb4\nb5\nb6\nb7\nb8\n").expect("file");
    std::fs::write(repo.join(".gitignore"), b"d.txt\n").expect("file");
    std::fs::write(repo.join("other.txt"), b"o\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("a.txt"), b"l1\nl2\nl3\nl4\nl5\nl6\nl7\nFEATURE-A\n").expect("file");
    std::fs::write(repo.join("b.txt"), b"b1\nb2\nb3\nb4\nb5\nb6\nb7\nFEATURE-B\n").expect("file");
    std::fs::write(repo.join("c.txt"), b"feature c\n").expect("file");
    std::fs::write(repo.join("d.txt"), b"feature d\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--force", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=feature"]);
    git(fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("other.txt"), b"o2\n").expect("file");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    persons_work(fixture, repo.as_path());
    return repo;
}

/// Another session's staged work in a path the merge does not change,
/// which is what makes a merge one the wrapper completes itself.
fn an_unrelated_staged_file(fixture: &Fixture, repo: &Path) {
    std::fs::write(repo.join("unrelated.txt"), b"staged by another session\n").expect("file");
    git(fixture, repo, &["add", "--", "unrelated.txt"]);
}

/// The person's own work at every path the merge writes:
/// one unstaged edit,
/// one staged edit,
/// one untracked file and one ignored file.
fn persons_work(fixture: &Fixture, repo: &Path) {
    std::fs::write(repo.join("a.txt"), b"MARK-A-UNSTAGED\nl2\nl3\nl4\nl5\nl6\nl7\nl8\n").expect("file");
    std::fs::write(repo.join("b.txt"), b"MARK-B-STAGED\nb2\nb3\nb4\nb5\nb6\nb7\nb8\n").expect("file");
    git(fixture, repo, &["add", "--", "b.txt"]);
    std::fs::write(repo.join("c.txt"), b"MARK-C-UNTRACKED\n").expect("file");
    std::fs::write(repo.join("d.txt"), b"MARK-D-IGNORED\n").expect("file");
}

/// What:
///  A first merge held at one of its own phases,
///  and the directory its markers are in.
/// Why:
///   A slow disk and a stopped process are both a merge that is part-way,
/// and the marker holds one there
///  without a tracer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdTheMergeAt(fixture, repo, 'merge-cleared');
/// ```
struct Held {
    /// The merge that is part-way.
    child: Child,
    /// The directory its marker files are in.
    markers: PathBuf,
    /// The phase it is held at.
    phase: String,
}

impl Held {
    /// Release the merge and wait for it,
    /// answering its exit status.
    fn release(mut self) -> Option<i32> {
        for stem in [self.phase.clone(), format!("{}-1", self.phase), format!("{}-2", self.phase)] {
            std::fs::write(self.markers.join(format!("{stem}.release")), b"").expect("the release");
        }
        return self.child.wait().ok().and_then(|status| status.code());
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        let _ended = self.child.kill();
        let _waited = self.child.wait();
    }
}

/// Hold a merge of `feature` at `phase`,
/// and answer once it is there.
fn hold_the_merge_at(fixture: &Fixture, repo: &Path, phase: &str) -> Held {
    let markers: PathBuf = fixture.root.join(format!("markers-{phase}-{}", std::process::id()));
    std::fs::create_dir_all(markers.as_path()).expect("the marker directory");
    let child: Child = wrapped_commit(fixture, repo, &["merge", "--no-edit", "feature"])
        .env(PHASE_SIGNAL_VARIABLE, format!("{phase}:pause:{}", markers.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the merge starts");
    let began: Instant = Instant::now();
    while began.elapsed() < MARKER_PATIENCE {
        let reached: bool = std::fs::read_dir(markers.as_path()).is_ok_and(|mut entries| {
            return entries.any(|entry| {
                let Ok(found) = entry else { return false };
                return found.file_name().to_string_lossy().ends_with(".reached");
            });
        });
        if reached {
            return Held { child, markers, phase: String::from(phase) };
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("the merge never reached {phase}");
}

/// Every path the commit at `HEAD` holds that the index does not:
/// the index of a whole result lacks none of them.
fn paths_the_index_lacks(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let committed: Vec<String> =
        git_text(fixture, repo, &["ls-tree", "-r", "--name-only", "HEAD"]).lines().map(String::from).collect();
    let indexed: String = git_text(fixture, repo, &["ls-files", "--cached"]);
    let mut lacking: Vec<String> = Vec::new();
    for path in committed {
        if !indexed.lines().any(|listed| { return listed == path; }) {
            lacking.push(path);
        }
    }
    return lacking;
}

/// What:
///  A second merge that arrives while the first is part-way waits for it,
/// and the worktree ends in the state one merge leaves.
/// Why:
///   This is `F12` of the alpha.12 audit:
///  held at `merge-cleared`,
/// both merges exited 0 and the index lacked two files the merge commit held,
/// so the next
///  commit would have deleted them from the branch without anyone asking.
///  The second merge may not install beside the first,
/// and it may not fail either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = holdTheMergeAt(fixture, repo, 'merge-cleared'); const second = run(['merge', '--no-edit', 'feature']);
/// expect(held.release()).toBe(0); expect(second.code).toBe(0); expect(status()).toBe(THE_SINGLE_MERGE_STATUS);
/// ```
#[test]
fn a_second_merge_waits_for_the_install_and_the_worktree_ends_in_one_whole_state() {
    let fixture: Fixture = fixture("merge-one-install-at-a-time");
    let repo: PathBuf = world(&fixture, "repo");
    let held: Held = hold_the_merge_at(&fixture, repo.as_path(), "merge-cleared");
    // The second merge starts while the first is held, and the first is released once the second has begun to wait: the
    //  ordinary shape, where the merge that installs finishes in a moment.
    let second: Child = wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the second merge starts");
    std::thread::sleep(Duration::from_secs(2));
    let first: Option<i32> = held.release();
    let second: std::process::Output = second.wait_with_output().expect("the second merge ends");
    let said: String = String::from_utf8_lossy(&second.stderr).into_owned();
    assert_eq!(second.status.code(), Some(0), "the second merge ends with 0: {said}");
    assert_eq!(first, Some(0), "the first merge ends with 0");
    assert!(
        said.contains("still waiting for the merge that is installing its result in this worktree")
            || said.contains("saves itself on a ref of its own"),
        "the second merge says what it waited for or where it went: {said}"
    );
    let status: String = git_text(&fixture, repo.as_path(), &["--no-optional-locks", "status", "--porcelain"]);
    assert_eq!(status, THE_SINGLE_MERGE_STATUS, "the worktree ends as one merge leaves it");
    assert_eq!(paths_the_index_lacks(&fixture, repo.as_path()), Vec::<String>::new(), "the index holds every path the merge commit holds");
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "no merge record is left");
    assert!(!repo.join(".git/monochromatic-git-policy-cli-merges/.installing").exists(), "the mark of the install went with it");
    let later: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["status", "--porcelain"]), b"");
    assert_eq!(later.code, Some(0), "the next command reads the worktree: {}", stderr_of(&later));
    remove_settled(&fixture);
}

/// What:
///  A merge that is partly installed and cannot be finished says so to a command that only reads,
///  and stops a command that would write.
/// Why:
///  `F12` wedged a worktree:
/// after the second merge,
/// a record was kept and every later command,
/// `git status` included,
///  exited 2,
/// and the way out the refusal named was itself refused.
/// Reading the checkout cannot make such a state
///  worse,
/// and it is what a person does first;
/// a commit or a merge that read it would take the half-applied state for the worktree's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordAMergeThatLandedOnAnotherBranch();
/// expect(run(['status']).code).toBe(0); expect(run(['merge','feature']).code).toBe(2);
/// ```
#[test]
fn a_partly_installed_merge_names_its_state_to_a_read_and_stops_a_write() {
    let fixture: Fixture = fixture("merge-partly-installed-read");
    let repo: PathBuf = world(&fixture, "repo");
    // A merge whose commit is the tip of a branch this worktree does not have checked out: nothing of it can be
    //  finished here, and its index entries would be put into another branch's checkout.
    let tip: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    git(&fixture, repo.as_path(), &["branch", "landed", tip.trim()]);
    let record: PathBuf = git_policy_cli::merge_isolation_journal::write_merge_intent(
        repo.join(".git").as_path(),
        &git_policy_cli::merge_isolation_journal::MergeIntent {
            owner: a_gone_owner("88888888-0000-4000-8000-000000000008"),
            repository_root: repo.clone(),
            target_ref: Some(String::from("refs/heads/landed")),
            old_oid: git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD~1"]).trim().to_owned(),
            new_oid: tip.trim().to_owned(),
            side_ref: None,
            carried: None,
        },
    )
    .expect("a merge record");
    let status: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["status", "--porcelain"]), b"");
    let said: String = stderr_of(&status);
    assert_eq!(status.code, Some(0), "a command that only reads the checkout goes on and is told the state: {said}");
    assert!(said.contains("is partly installed in this worktree"), "the line names the state: {said}");
    assert!(said.contains("check out refs/heads/landed"), "the line says the way out: {said}");
    assert!(record.exists(), "the record is kept for a person to see");
    let merge: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]), b"");
    assert_eq!(merge.code, Some(2), "a command that would write stops: {}", stderr_of(&merge));
    assert!(stderr_of(&merge).contains("is partly installed in this worktree"), "the stop names the state: {}", stderr_of(&merge));
    remove_settled(&fixture);
}

/// An owner record naming a process that ended,
/// which is what a killed merge leaves.
fn a_gone_owner(transaction_id: &str) -> git_policy_cli::transaction_owner::TransactionOwner {
    let mut ended = std::process::Command::new("/bin/sh").arg("-c").arg("exit 0").spawn().expect("a process that ends");
    let pid: i64 = i64::from(ended.id());
    ended.wait().expect("it ends");
    return git_policy_cli::transaction_owner::TransactionOwner {
        transaction_id: String::from(transaction_id),
        owner_pid: pid,
        owner_identity: String::from("0"),
        created_at: String::from("2026-11-12T00:00:00.000Z"),
        owner_pid_namespace: None,
    };
}

/// Processes whose current directory is inside this fixture,
/// which is what a step that was left running looks like from here.
fn processes_in(fixture: &Fixture) -> Vec<String> {
    let root: String = fixture.root.to_string_lossy().into_owned();
    let mut found: Vec<String> = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else { return found };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.bytes().all(|byte| { return byte.is_ascii_digit(); }) {
            continue;
        }
        let Ok(cwd) = std::fs::read_link(format!("/proc/{name}/cwd")) else { continue };
        if cwd.to_string_lossy().starts_with(root.as_str()) {
            let command: String = std::fs::read(format!("/proc/{name}/cmdline"))
                .map(|bytes| { return String::from_utf8_lossy(&bytes).replace('\0', " "); })
                .unwrap_or_default();
            found.push(format!("{name}: {command}"));
        }
    }
    return found;
}

/// A filter program that never answers,
/// its clean side passing the bytes through,
/// as a stand-in for a `git-lfs` download from a server that accepts the request and never answers.
/// Why:
///   The control must not need a network,
/// and the shape is the same:
/// the checkout writes a path whose content a filter must
///  produce,
/// and that filter never returns.
const NEVER_ANSWERS: &str = "#!/bin/sh\ncase \"$1\" in\nclean) cat ;;\n*) sleep 3000 ;;\nesac\n";

/// What:
///  A merge whose checkout step never gets its content ends at its bound,
///  puts back the paths it had begun to write,
///  and leaves no program and no lock behind.
/// Why:
///  This is `F13` of the alpha.12 audit:
/// a silent `git-lfs` server held `git merge --ff-only` and `git-lfs` running for good with
///  `index.lock` held,
/// the wrapper exited 2 after 607 seconds,
/// and the next merge waited a minute for that lock.
///  The owner's rule is that nothing waits forever and nothing the wrapper made is left for a person to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = run(['merge', '--no-edit', 'feature'], { CLI_GIT_CHILD_WAIT_SECONDS: '5' });
/// expect(ran.code).toBe(2); expect(indexLock()).toBe(false); expect(records()).toEqual([]); expect(status()).toBe(before);
/// ```
#[test]
fn a_merge_whose_checkout_never_answers_ends_and_puts_back_what_it_held() {
    let fixture: Fixture = fixture("merge-checkout-never-answers");
    let repo: PathBuf = fixture.root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(&fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let filter: PathBuf = fixture.root.join("bin/never-answers");
    executable(filter.as_path(), NEVER_ANSWERS.as_bytes());
    git(&fixture, repo.as_path(), &["config", "filter.hang.clean", format!("{} clean", filter.display()).as_str()]);
    git(&fixture, repo.as_path(), &["config", "filter.hang.smudge", format!("{} smudge", filter.display()).as_str()]);
    git(&fixture, repo.as_path(), &["config", "filter.hang.required", "true"]);
    std::fs::write(repo.join(".gitattributes"), b"*.bin filter=hang -text\n").expect("file");
    std::fs::write(repo.join("keep.txt"), b"k base\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("big.bin"), b"a file whose content a filter must produce\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "big.bin"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=feature adds a filtered file"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "main"]);
    // Main moves after the feature, so the merge is a true merge and the wrapper completes it itself.
    std::fs::write(repo.join("note.txt"), b"main work\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "note.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=main work"]);
    // The person's staged edit makes the merge one the wrapper completes itself.
    std::fs::write(repo.join("keep.txt"), b"STAGED\n").expect("file");
    git(&fixture, repo.as_path(), &["add", "--", "keep.txt"]);
    let before: String = git_text(&fixture, repo.as_path(), &["--no-optional-locks", "status", "--porcelain"]);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);

    let began: Instant = Instant::now();
    let ran: Observed = observe(
        wrapped_commit(&fixture, repo.as_path(), &["merge", "--no-edit", "feature"]).env("CLI_GIT_CHILD_WAIT_SECONDS", "5"),
        b"",
    );
    let took: Duration = began.elapsed();
    let said: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(2), "the merge ends instead of waiting forever: {said}");
    assert!(took < Duration::from_secs(240), "the merge ends within its bound, and took {took:?}");
    assert!(said.contains("showed no sign of work") || said.contains("ran to its bound"), "the words name the ending: {said}");
    assert!(said.contains("the merge was not made"), "the words say what became of it: {said}");
    assert!(!repo.join(".git/index.lock").exists(), "the ended Git's lock is gone");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head, "the branch did not move");
    assert_eq!(records_in(repo.join(".git/monochromatic-git-policy-cli-merges").as_path()), Vec::<PathBuf>::new(), "no record is left");
    assert!(!repo.join(".git/monochromatic-git-policy-cli-merges/.installing").exists(), "no mark of an install is left");
    let after: String = git_text(&fixture, repo.as_path(), &["--no-optional-locks", "status", "--porcelain"]);
    assert_eq!(after, before, "the paths the merge had begun to write are back where they were");
    let left: Vec<String> = processes_in(&fixture);
    assert_eq!(left, Vec::<String>::new(), "no program of the ended step is left running");
    let later: Observed = observe(&mut wrapped_commit(&fixture, repo.as_path(), &["status", "--porcelain"]), b"");
    assert_eq!(later.code, Some(0), "the next command reads the worktree: {}", stderr_of(&later));
    remove_settled(&fixture);
}

/// What:
///  The mark of an install lives in the worktree's own record directory,
///  so a merge in another worktree of the same repository is not held by it.
/// Why:
///   The owner's rule is that no command of one agent waits on another agent's command,
/// and a mark that covered a whole
///  repository would make every merge of one worktree wait for every merge of the others.
#[test]
fn a_merge_in_another_worktree_of_the_same_repository_installs_beside_this_one() {
    let fixture: Fixture = fixture("merge-one-install-per-worktree");
    let repo: PathBuf = world(&fixture, "repo");
    an_unrelated_staged_file(&fixture, repo.as_path());
    let other: PathBuf = fixture.root.join("other");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "elsewhere", other.to_str().expect("UTF-8")]);
    std::fs::write(other.join("other.txt"), b"o3\n").expect("file");
    git(&fixture, other.as_path(), &["commit", "--quiet", "--all", "--message=elsewhere work"]);
    an_unrelated_staged_file(&fixture, other.as_path());
    let held: Held = hold_the_merge_at(&fixture, repo.as_path(), "merge-cleared");
    let beside: Observed = observe(&mut wrapped_commit(&fixture, other.as_path(), &["merge", "--no-edit", "feature"]), b"");
    let said: String = stderr_of(&beside);
    assert_eq!(beside.code, Some(0), "a merge in the other worktree installs while this one is held: {said}");
    assert!(!said.contains("is installing its result"), "the other worktree does not wait for this install: {said}");
    let first: Option<i32> = held.release();
    assert_eq!(first, Some(0), "the merge in this worktree ends");
    remove_settled(&fixture);
}
