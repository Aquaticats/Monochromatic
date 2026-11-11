//! What:
//!  The commit states that used to be declined,
//! landed through the service and compared with native Git:
//! the conclusion of a
//!  merge,
//! a cherry-pick and a revert (with the state files it removes),
//! a detached head,
//! a branch that does not exist yet,
//!  an unmerged index,
//! nothing to commit,
//! a policy rewrite,
//! change lists that the embedded route cannot carry,
//! and a native Git that never answers.
//! Why:
//!  The landing of each is the service's compare-and-swap,
//! and the person must find the commit,
//! the head,
//! the index,
//! the
//!  state files and Git's own words as native Git leaves them.
//! Each conclusion state is its own control:
//! one control carrying all seven lands twenty-eight commits through the service,
//! took 284.9 s of them in one run on 2026-11-11 on a host other sessions load,
//! and another run of it reached the runner's five-minute stop,
//! which ended it naming no state.

use super::commit::remove_settled;
use super::service_commit::{Running, start};
use super::service_landing_selection::base_repository;
use super::service_landing_support::{Case, case, compare, copy_beside, finished, run_native, run_served, text_of_within};
use super::support::{Fixture, NATIVE_GIT_BOUND_SECONDS, executable, fixture, git, native_git_output};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn write(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("write");
}

/// Runs real Git and accepts any status (a conflicted merge exits 1).
/// The run is a control's fixture step and not its oracle,
/// but a Git waiting on a lock or a hook here would still hold the control,
/// so it carries the same bound as every other native Git command.
fn attempt(fixture: &Fixture, repo: &Path, arguments: &[&str]) {
    let mut command: Command = super::support::direct(fixture);
    command.args(arguments).env("GIT_EDITOR", "true");
    let _output = native_git_output(fixture, &mut command, repo, NATIVE_GIT_BOUND_SECONDS);
}

/// A side branch and the main branch both changed `b.txt`;
/// the main branch is checked out.
fn diverged(fixture: &Fixture, repo: &Path) -> (String, String) {
    git(fixture, repo, &["checkout", "--quiet", "-b", "side"]);
    write(repo, "b.txt", "side\n");
    write(repo, "s.txt", "side only\n");
    git(fixture, repo, &["add", "b.txt", "s.txt"]);
    git(fixture, repo, &["commit", "--quiet", "--message=side change"]);
    let side: String = text(fixture, repo, &["rev-parse", "HEAD"]);
    git(fixture, repo, &["checkout", "--quiet", "main"]);
    write(repo, "b.txt", "main\n");
    git(fixture, repo, &["add", "b.txt"]);
    git(fixture, repo, &["commit", "--quiet", "--message=main change"]);
    let main: String = text(fixture, repo, &["rev-parse", "HEAD"]);
    return (side, main);
}

fn text(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    return super::service_landing_support::text_of(fixture, repo, arguments).trim().to_string();
}

fn resolve(fixture: &Fixture, repo: &Path) {
    write(repo, "b.txt", "resolved\n");
    git(fixture, repo, &["add", "b.txt"]);
}

/// The bound the control below gives the read it makes,
/// in seconds,
/// and the delay its stand-in for a Git that never answers keeps.
/// Why these two numbers:
///  The stand-in delays longer than the bound and shorter than the control's own limit,
/// so a read that does answer the bound ends in about one second,
/// and one that lost the bound waits for the whole delay and fails with its own message.
const NEVER_ANSWERS_BOUND_SECONDS: &str = "1";

/// How long the stand-in for a Git that never answers delays,
/// in seconds.
const NEVER_ANSWERS_DELAY_SECONDS: u32 = 5;

/// The longest a control waits for a read that does not answer,
/// in seconds.
const CONTROL_LIMIT_SECONDS: u64 = 3;

/// What:
///  A native Git read that never answers is ended at the harness's bound,
/// and the failure says which command it was waiting for,
///  in which repository,
/// and for how long.
/// Why:
///  The oracle every landing control compares the service against is native Git,
/// and a Git waiting on something a control cannot see
/// (a lock another program holds,
/// a mount that stopped answering)
/// held the control instead until the runner's five-minute stop ended
///  the run without naming anything
/// (2026-11-11:
/// the conclusion control ran past that stop).
/// The stand-in is a real Git command whose fsmonitor hook never answers,
/// so what waits is Git and not the test:
/// the hook hands its own descriptors to `/dev/null` and execs the delay,
/// so nothing but Git holds the pipes the read has to drain.
#[test]
fn a_native_git_read_that_never_answers_ends_at_the_harness_bound_and_names_it() {
    let fixture: Fixture = fixture("landing-never-answers");
    let repo: PathBuf = base_repository(&fixture);
    let hook: PathBuf = fixture.root.join("never-answering-fsmonitor");
    executable(hook.as_path(), format!("#!/bin/sh\nexec sleep {NEVER_ANSWERS_DELAY_SECONDS} </dev/null >/dev/null 2>&1\n").as_bytes());
    let setting: String = format!("core.fsmonitor={}", hook.display());
    let arguments: [&str; 5] = ["-c", setting.as_str(), "status", "--short", "--untracked-files=all"];
    let began: Instant = Instant::now();
    // The panic a read that was ended at its bound raises is the control's subject, so its message is read, not printed.
    let printed = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let ended = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        return text_of_within(&fixture, &repo, arguments.as_slice(), NEVER_ANSWERS_BOUND_SECONDS);
    }));
    std::panic::set_hook(printed);
    let took: Duration = began.elapsed();
    let message: String = match ended {
        Ok(text) => panic!("a Git read that never answers must end at the harness's bound instead of answering after {took:?}: {text}"),
        Err(payload) => match payload.downcast_ref::<String>() {
            Some(text) => text.clone(),
            None => payload.downcast_ref::<&str>().map_or_else(|| { return String::from("(a panic with no text)"); }, |text| { return (*text).to_string(); }),
        },
    };
    assert!(message.contains("status"), "the failure names the command it waited for: {message}");
    assert!(message.contains(repo.display().to_string().as_str()), "the failure names the repository: {message}");
    assert!(message.contains(NEVER_ANSWERS_BOUND_SECONDS), "the failure names the bound: {message}");
    assert!(took < Duration::from_secs(CONTROL_LIMIT_SECONDS), "the read was ended at its bound, not when the delay ran out ({took:?} against a {NEVER_ANSWERS_DELAY_SECONDS} s delay): {message}");
    remove_settled(&fixture);
}

/// A repository in one state a conclusion can be in,
/// copied from `template`,
/// so each control of this file makes only the state it compares.
/// Why one state per control:
///  Each state is compared by four command lines,
/// and each comparison lands a commit through the service,
/// which writes and flushes the conclusion state it copies;
/// a host whose disk stalls makes one comparison take up to the bound of one commit,
/// and a control carrying every state took 284.9 s of such comparisons in one run (2026-11-11),
/// 300.0 s in another,
/// which the runner's five-minute stop then ended without naming a state.
fn conclusion_repository(fixture: &Fixture, template: &Path, name: &str, number: usize) -> PathBuf {
    let repo: PathBuf = copy_beside(fixture, template, name);
    let (side, main) = diverged(fixture, &repo);
    match number {
        0 => {
            attempt(fixture, &repo, &["merge", "side"]);
            resolve(fixture, &repo);
        }
        1 => {
            git(fixture, &repo, &["checkout", "--quiet", "-b", "clean-side", "HEAD~1"]);
            write(&repo, "n.txt", "new\n");
            git(fixture, &repo, &["add", "n.txt"]);
            git(fixture, &repo, &["commit", "--quiet", "--message=new file"]);
            git(fixture, &repo, &["checkout", "--quiet", "main"]);
            attempt(fixture, &repo, &["merge", "--no-commit", "--no-ff", "clean-side"]);
        }
        2 => {
            git(fixture, &repo, &["checkout", "--quiet", "-b", "clean-squash", "HEAD~1"]);
            write(&repo, "q.txt", "squashed\n");
            git(fixture, &repo, &["add", "q.txt"]);
            git(fixture, &repo, &["commit", "--quiet", "--message=for squash"]);
            git(fixture, &repo, &["checkout", "--quiet", "main"]);
            attempt(fixture, &repo, &["merge", "--squash", "clean-squash"]);
        }
        3 => {
            attempt(fixture, &repo, &["cherry-pick", side.as_str()]);
            resolve(fixture, &repo);
        }
        4 => {
            attempt(fixture, &repo, &["revert", main.as_str()]);
            write(&repo, "b.txt", "reverted by hand\n");
            git(fixture, &repo, &["add", "b.txt"]);
        }
        5 => {
            git(fixture, &repo, &["checkout", "--quiet", "side"]);
            write(&repo, "t.txt", "second pick\n");
            git(fixture, &repo, &["add", "t.txt"]);
            git(fixture, &repo, &["commit", "--quiet", "--message=second pick"]);
            let second: String = text(fixture, &repo, &["rev-parse", "HEAD"]);
            git(fixture, &repo, &["checkout", "--quiet", "main"]);
            attempt(fixture, &repo, &["cherry-pick", side.as_str(), second.as_str()]);
            resolve(fixture, &repo);
        }
        _ => {
            git(fixture, &repo, &["checkout", "--quiet", "-b", "tidy", "HEAD~1"]);
            write(&repo, "u.txt", "tidy\n");
            git(fixture, &repo, &["add", "u.txt"]);
            git(fixture, &repo, &["commit", "--quiet", "--message=tidy"]);
            git(fixture, &repo, &["checkout", "--quiet", "main"]);
            attempt(fixture, &repo, &["merge", "--no-ff", "--no-commit", "tidy"]);
            write(&repo, "u.txt", "tidy, then edited\n");
            git(fixture, &repo, &["add", "u.txt"]);
        }
    }
    return repo;
}

/// The command lines every conclusion state is compared by.
fn conclusion_cases(label: &'static str) -> Vec<Case> {
    let lines: [Case; 4] = [
        Case { index_commit: true, ..case("no-edit message", &["commit", "--no-edit"]) },
        Case { index_commit: true, ..case("typed message", &["commit", "-m", "typed"]) },
        Case { index_commit: true, ..case("editor that accepts the prepared message", &["commit"]) },
        Case { index_commit: true, ..case("commit all", &["commit", "-a", "-m", "all"]) },
    ];
    return lines
        .into_iter()
        .map(|line| {
            return Case { name: Box::leak(format!("{label}: {}", line.name).into_boxed_str()), ..line };
        })
        .collect();
}

/// What:
///  One conclusion state lands through the service,
/// plus the four command lines it is concluded by,
/// and leaves what native Git leaves.
/// Why:
///  The landing of each is the service's compare-and-swap,
/// and the person must find the commit,
/// the head,
///  the index and Git's own words as native Git leaves them.
fn a_conclusion_state_lands_as_native_git_leaves_it(fixture_name: &str, number: usize, label: &'static str) {
    let fixture: Fixture = fixture(fixture_name);
    let name: String = format!("{fixture_name}-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let template: PathBuf = base_repository(&fixture);
    let repo: PathBuf = conclusion_repository(&fixture, template.as_path(), "state", number);
    for line in &conclusion_cases(label) {
        let (native, served) = compare(&fixture, &repo, name.as_str(), line);
        // A conclusion removes its own markers: in the states that make a commit, none is left in either copy.
        if native.code == Some(0) {
            for (state, value) in served.left.iter().filter(|(entry, _)| { return ["state MERGE_HEAD", "state CHERRY_PICK_HEAD", "state REVERT_HEAD", "state MERGE_MSG", "state MERGE_MODE", "state SQUASH_MSG"].contains(&entry.as_str()); }) {
                assert_eq!(value, "(absent)", "{label}: {state} is removed by the landing");
            }
        }
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A conflicted merge is resolved and concluded by each of the four command lines.
#[test]
fn a_conflicted_merge_conclusion_lands_through_the_service_and_removes_the_state_native_git_removes() {
    return a_conclusion_state_lands_as_native_git_leaves_it("landing-conclusion-merge", 0, "conflicted merge, resolved");
}

/// A merge Git left without a commit (`--no-commit`) is concluded by each of the four command lines.
#[test]
fn a_merge_left_for_its_commit_lands_through_the_service_as_native_git_lands_it() {
    return a_conclusion_state_lands_as_native_git_leaves_it("landing-conclusion-merge-no-commit", 1, "merge without a commit");
}

/// A squash merge is concluded by each of the four command lines,
/// and its squash state is removed as native Git removes it.
#[test]
fn a_squash_merge_conclusion_lands_through_the_service_and_removes_its_squash_state() {
    return a_conclusion_state_lands_as_native_git_leaves_it("landing-conclusion-squash", 2, "squash merge");
}

/// A conflicted cherry-pick is resolved and concluded by each of the four command lines.
#[test]
fn a_conflicted_cherry_pick_conclusion_lands_through_the_service_and_removes_its_state() {
    return a_conclusion_state_lands_as_native_git_leaves_it("landing-conclusion-cherry-pick", 3, "conflicted cherry-pick, resolved");
}

/// A conflicted revert is resolved and concluded by each of the four command lines.
#[test]
fn a_conflicted_revert_conclusion_lands_through_the_service_and_removes_its_state() {
    return a_conclusion_state_lands_as_native_git_leaves_it("landing-conclusion-revert", 4, "conflicted revert, resolved");
}

/// A cherry-pick of two commits whose first conflicted is concluded by each of the four command lines,
/// with the sequencer the second pick left.
#[test]
fn a_sequencer_left_by_two_cherry_picks_lands_through_the_service_as_native_git_lands_it() {
    return a_conclusion_state_lands_as_native_git_leaves_it("landing-conclusion-two-picks", 5, "two picks, first resolved");
}

/// A merge of a side branch that brings a new file is concluded by each of the four command lines,
/// including the edit of the new file that `commit all` stages.
#[test]
fn a_merge_bringing_a_new_file_lands_through_the_service_as_native_git_lands_it() {
    return a_conclusion_state_lands_as_native_git_leaves_it("landing-conclusion-new-file", 6, "merge of a side branch with a new file");
}

/// A detached head is moved as `HEAD` itself:
/// the commit lands on it,
/// the branches stay,
/// and the head stays detached.
#[test]
fn a_commit_on_a_detached_head_moves_head_and_no_branch() {
    let fixture: Fixture = fixture("landing-detached");
    let base: PathBuf = base_repository(&fixture);
    git(&fixture, &base, &["checkout", "--quiet", "--detach"]);
    write(&base, "a.txt", "changed\n");
    write(&base, "b.txt", "changed b\n");
    git(&fixture, &base, &["add", "b.txt"]);
    let name: String = format!("landing-detached-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in [
        case("named path", &["commit", "-m", "detached", "--", "a.txt"]),
        Case { index_commit: true, ..case("index", &["commit", "-m", "detached"]) },
        Case { index_commit: true, ..case("all", &["commit", "-a", "-m", "detached"]) },
        Case { index_commit: true, ..case("amend", &["commit", "--amend", "--no-edit"]) },
    ] {
        let (native, served) = compare(&fixture, &base, name.as_str(), &entry);
        assert_eq!(native.code, Some(0), "{}", entry.name);
        let symbolic = served.left.iter().find(|(label, _)| { return label == "symbolic head"; }).map(|(_, value)| { return value.clone(); });
        assert_eq!(symbolic.as_deref(), Some(""), "{}: the head is still detached", entry.name);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A branch that does not exist yet gets its first commit by a compare against the all-zero name.
#[test]
fn the_first_commit_of_a_branch_lands_through_the_service() {
    let fixture: Fixture = fixture("landing-unborn");
    let base: PathBuf = fixture.root.join("unborn");
    std::fs::create_dir(&base).expect("directory");
    git(&fixture, &base, &["init", "--quiet", "--initial-branch=main"]);
    git(&fixture, &base, &["config", "maintenance.autoDetach", "false"]);
    write(&base, "a.txt", "first\n");
    write(&base, "b.txt", "second\n");
    write(&base, "c.txt", "untracked\n");
    git(&fixture, &base, &["add", "a.txt", "b.txt"]);
    let name: String = format!("landing-unborn-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in [
        Case { index_commit: true, ..case("index", &["commit", "-m", "first"]) },
        case("named path", &["commit", "-m", "first", "--", "a.txt"]),
        Case { index_commit: true, ..case("empty commit", &["commit", "--allow-empty", "-m", "first"]) },
        Case { index_commit: true, ..case("nothing to amend", &["commit", "--amend", "-m", "first"]) },
        Case { index_commit: true, full_output: true, ..case("editor-free message", &["commit", "--no-edit"]) },
    ] {
        compare(&fixture, &base, name.as_str(), &entry);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// `-a` stages the working copy of a conflicted file,
/// which is a resolution,
/// so the service commits it as native Git does.
/// A plain
///  commit with unmerged files is no longer refused as native Git refuses it:
/// it finishes the merge with a warning
///  (`binary_commit_during_merge_tests.rs`,
/// owner,
/// 2026-11-03:
/// "Warn and finish the merge").
#[test]
fn a_commit_with_all_staged_resolves_an_unmerged_file_as_native_git_does() {
    let fixture: Fixture = fixture("landing-unmerged");
    let base: PathBuf = base_repository(&fixture);
    diverged(&fixture, &base);
    attempt(&fixture, &base, &["merge", "side"]);
    write(&base, "c.txt", "edited elsewhere\n");
    let name: String = format!("landing-unmerged-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    // `-a` stages the working copy of the conflicted file, which is a resolution, so native Git commits it and so must the service.
    let resolved = Case { index_commit: true, ..case("all", &["commit", "-a", "-m", "x"]) };
    let (native, _served) = compare(&fixture, &base, name.as_str(), &resolved);
    assert_eq!(native.code, Some(0), "native Git commits the staged working copy");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// Nothing to commit is said in Git's own words with Git's own status,
/// and nothing lands.
#[test]
fn nothing_to_commit_is_said_in_the_words_of_native_git() {
    let fixture: Fixture = fixture("landing-nothing");
    let base: PathBuf = base_repository(&fixture);
    write(&base, "d.txt", "untracked\n");
    let name: String = format!("landing-nothing-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in [
        Case { index_commit: true, full_output: true, ..case("index, only untracked files", &["commit", "-m", "x"]) },
        Case { index_commit: true, full_output: true, ..case("all, only untracked files", &["commit", "-a", "-m", "x"]) },
        Case { full_output: true, ..case("a tracked path with no change", &["commit", "-m", "x", "--", "c.txt"]) },
    ] {
        let (native, _served) = compare(&fixture, &base, name.as_str(), &entry);
        assert_ne!(native.code, Some(0), "{}: native Git has nothing to commit", entry.name);
    }
    git(&fixture, &base, &["clean", "--force", "--quiet", "-d"]);
    let clean = Case { index_commit: true, full_output: true, ..case("a clean tree", &["commit", "-m", "x"]) };
    compare(&fixture, &base, name.as_str(), &clean);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A policy rewrite is installed into the worktree after the landing;
/// the commit holds the corrected file,
/// as a commit of
///  the corrected file by native Git does.
#[test]
fn a_policy_rewrite_lands_and_reaches_the_worktree() {
    let fixture: Fixture = fixture("landing-rewrite");
    let base: PathBuf = base_repository(&fixture);
    let name: String = format!("landing-rewrite-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let served_repo: PathBuf = copy_beside(&fixture, &base, "served");
    // Native Git is given the corrected file; the policy corrects the other copy.
    write(&base, "a.txt", "changed\n");
    write(&served_repo, "a.txt", "changed");
    let entry: Case = case("a missing final newline", &["commit", "-m", "corrected", "--", "a.txt"]);
    let before: usize = finished(&fixture);
    let native = run_native(&fixture, &base, &entry);
    let served = run_served(&fixture, &served_repo, name.as_str(), &entry);
    assert_eq!(served.code, native.code, "{}", served.output);
    for ((label, expected), (_, found)) in native.left.iter().zip(&served.left) {
        assert_eq!(found, expected, "{label}");
    }
    assert_eq!(std::fs::read(served_repo.join("a.txt")).expect("worktree copy"), b"changed\n");
    assert_eq!(finished(&fixture), before + 1, "the service landed it");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// Change lists the embedded route cannot carry are built by Git and landed:
/// more paths than a request may list,
/// names that
///  are not text,
/// a gitlink,
/// an executable,
/// a symbolic link,
/// and a file replaced by a directory.
#[test]
fn awkward_change_lists_land_as_native_git_commits_them() {
    use std::os::unix::ffi::OsStrExt;
    let fixture: Fixture = fixture("landing-odd");
    let base: PathBuf = base_repository(&fixture);
    let many: PathBuf = base.join("many");
    std::fs::create_dir(&many).expect("directory");
    for number in 0..git_policy_cli::service_protocol::MAX_CHANGES + 100 {
        std::fs::write(many.join(format!("f{number}.txt")), format!("{number}\n")).expect("file");
    }
    std::fs::write(base.join(std::ffi::OsStr::from_bytes(b"caf\xe9.txt")), "not text\n").expect("non-UTF-8 name");
    std::fs::write(base.join("run.sh"), "#!/bin/sh\n").expect("script");
    let mut permissions = std::fs::metadata(base.join("run.sh")).expect("metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    std::fs::set_permissions(base.join("run.sh"), permissions).expect("mode");
    std::os::unix::fs::symlink("a.txt", base.join("link")).expect("link");
    git(&fixture, &base, &["update-index", "--add", "--cacheinfo", &format!("160000,{},sub", text(&fixture, &base, &["rev-parse", "HEAD"]))]);
    std::fs::remove_file(base.join("c.txt")).expect("remove");
    std::fs::create_dir(base.join("c.txt")).expect("directory in place of the file");
    std::fs::write(base.join("c.txt/inner.txt"), "inside\n").expect("inner");
    git(&fixture, &base, &["add", "--all"]);
    let name: String = format!("landing-odd-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in [
        Case { index_commit: true, ..case("everything staged", &["commit", "-m", "awkward"]) },
        Case { index_commit: true, ..case("everything staged, with a message from a file", &["commit", "-F", "../message.txt"]) },
    ] {
        std::fs::write(fixture.root.join("message.txt"), "from a file\n").expect("message");
        compare(&fixture, &base, name.as_str(), &entry);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
