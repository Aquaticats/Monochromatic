//! What:
//!  A `git pull` that merges,
//! through the built executable:
//! with another session's staged work in the index it ends as
//!  native Git's pull ends on a clean copy,
//! and every pull the wrapper does not complete keeps native Git's own run.
//! Why:
//!  Native Git's pull runs `git merge` as a child of Git (`builtin/pull.c`,
//! `run_merge`),
//! so the merge never reached the
//!  wrapper and kept native Git's failures:
//! a staged index,
//! a held `index.lock`,
//! a late ref update.
//! The owner's rules are
//!  "Auto merging should never fail."
//! and "Manual merge should never fail too.",
//! so the merge phase of a pull goes
//!  through the same isolated merge as `git merge` (`doc/planning/cli-git-merge-auto-resolution.md`,
//! "Pull").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ours = await wrapped(['pull', '--no-rebase', '--no-edit'], dirtyCopy); const native = await direct(['pull', '--no-rebase', '--no-edit'], cleanCopy);
//! expect(await commitShape(ours)).toEqual(await commitShape(native));
//! ```

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::held_lock_support::{Ended, run_acting_on_line};
use super::merge_isolation::{copy_of, diverged, snapshot, stage_unrelated_work};
use super::support::{Fixture, Observed, direct, fixture, git, observe, repository, without_wait_notices};
use std::path::{Path, PathBuf};

/// A repository whose `main` follows a bare remote that holds the `feature` history as its `main`,
/// so a pull merges.
fn diverged_with_remote(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = diverged(fixture, "dirty");
    let remote: PathBuf = fixture.root.join("remote.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
    git(fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(fixture, repo.as_path(), &["push", "--quiet", "origin", "feature:main"]);
    git(fixture, repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(fixture, repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    return repo;
}

/// Run a wrapped command to its end,
/// without the wait notices a slow step may print.
fn run(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    let observed: Observed = observe(&mut wrapped_commit(fixture, repo, arguments), b"");
    return Observed { stderr: without_wait_notices(observed.stderr.as_slice()), ..observed };
}

/// Run native Git directly,
/// the oracle on a clean copy.
fn native(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Observed {
    return observe(direct(fixture).current_dir(repo).args(arguments), b"");
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

/// What the commit at `HEAD` holds apart from the time it was made:
/// its tree,
/// how many parents it has,
/// and its words and names.
/// Two pulls made
///  in different seconds make commits that differ in name only.
fn head_shape(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return vec![
        git_text(fixture, repo, &["rev-parse", "HEAD^{tree}"]),
        git_text(fixture, repo, &["log", "-1", "--format=%P"]).split_whitespace().count().to_string(),
        git_text(fixture, repo, &["log", "-1", "--format=%an <%ae>|%cn <%ce>|%B"]),
    ];
}

/// A pull that merges over another session's staged work is completed,
/// leaves that work as it was,
/// and equals native Git's
///  pull on a clean copy in tree,
/// parents,
/// message and words.
#[test]
fn a_pull_that_merges_is_completed_over_staged_work_and_equals_native_gits() {
    let fixture: Fixture = fixture("pull-isolated-oracle");
    let dirty: PathBuf = diverged_with_remote(&fixture);
    let clean: PathBuf = copy_of(&fixture, dirty.as_path(), "clean");
    stage_unrelated_work(&fixture, dirty.as_path());
    let staged: Vec<String> = git_text(&fixture, dirty.as_path(), &["diff", "--cached", "--name-status"]).lines().map(String::from).collect();
    let arguments: [&str; 3] = ["pull", "--no-rebase", "--no-edit"];
    let theirs: Observed = native(&fixture, clean.as_path(), &arguments);
    assert_eq!(theirs.code, Some(0), "native Git pulls into the clean copy: {}", String::from_utf8_lossy(&theirs.stderr));
    let ours: Observed = run(&fixture, dirty.as_path(), &arguments);
    assert_eq!(ours.code, Some(0), "{}", String::from_utf8_lossy(&ours.stderr));
    assert_eq!(String::from_utf8_lossy(&ours.stdout), String::from_utf8_lossy(&theirs.stdout), "output");
    assert_eq!(String::from_utf8_lossy(&ours.stderr), String::from_utf8_lossy(&theirs.stderr), "diagnostics");
    assert_eq!(commit_shape(&fixture, dirty.as_path()), commit_shape(&fixture, clean.as_path()));
    let after: Vec<String> = git_text(&fixture, dirty.as_path(), &["diff", "--cached", "--name-status", "HEAD"]).lines().map(String::from).collect();
    assert_eq!(after, staged, "the other session's staged work is still staged and nothing else is");
    assert!(!snapshot(&fixture, dirty.as_path()).status.is_empty(), "the staged work is still there");
    remove_settled(&fixture);
}

/// The same pull into a clean index,
/// which the isolated merge completes since it completes a clean-index merge,
/// equals native Git's.
#[test]
fn a_pull_into_a_clean_index_equals_native_gits() {
    let fixture: Fixture = fixture("pull-isolated-clean");
    let ours_repo: PathBuf = diverged_with_remote(&fixture);
    let theirs_repo: PathBuf = copy_of(&fixture, ours_repo.as_path(), "clean");
    let arguments: [&str; 3] = ["pull", "--no-rebase", "--no-edit"];
    let theirs: Observed = native(&fixture, theirs_repo.as_path(), &arguments);
    let ours: Observed = run(&fixture, ours_repo.as_path(), &arguments);
    assert_eq!((ours.code, theirs.code), (Some(0), Some(0)), "{}", String::from_utf8_lossy(&ours.stderr));
    assert_eq!(String::from_utf8_lossy(&ours.stdout), String::from_utf8_lossy(&theirs.stdout), "output");
    assert_eq!(String::from_utf8_lossy(&ours.stderr), String::from_utf8_lossy(&theirs.stderr), "diagnostics");
    assert_eq!(commit_shape(&fixture, ours_repo.as_path()), commit_shape(&fixture, theirs_repo.as_path()));
    remove_settled(&fixture);
}

/// A pull whose `pull.rebase` is written more than once,
/// or as a number Git reads as zero,
/// merges the way Git's own reading of that setting says,
/// so it is completed over another session's staged work.
/// Why:
///   The wrapper read the first of several values and a short list of spellings,
/// so a valid setting Git reads as false kept the pull native,
/// where native Git's merge
///  refused over staged work:
/// the setting then decided the refusal,
/// not the person's staged bytes.
#[test]
fn a_pull_whose_rebase_setting_is_written_twice_or_as_a_number_is_completed() {
    let cases: [(&str, &[(&str, &str)]); 3] = [
        ("duplicate", &[("pull.rebase", "true"), ("pull.rebase", "false")]),
        ("numeric-zero", &[("pull.rebase", "00")]),
        ("branch-duplicate", &[("branch.main.rebase", "true"), ("branch.main.rebase", "false")]),
    ];
    for (name, entries) in cases {
        let fixture: Fixture = fixture(format!("pull-rebase-{name}").as_str());
        let ours_repo: PathBuf = diverged_with_remote(&fixture);
        for (key, value) in entries {
            git(&fixture, ours_repo.as_path(), &["config", "--add", key, value]);
        }
        // Git's own reading of the setting, which is what the wrapped pull must follow.
        assert_eq!(git_text(&fixture, ours_repo.as_path(), &["config", "--bool", "--get", entries[0].0]).trim(), "false", "{name}: the setting this control writes");
        let clean_repo: PathBuf = copy_of(&fixture, ours_repo.as_path(), "clean");
        stage_unrelated_work(&fixture, ours_repo.as_path());
        let staged: Vec<String> = git_text(&fixture, ours_repo.as_path(), &["diff", "--cached", "--name-status"]).lines().map(String::from).collect();
        let arguments: [&str; 2] = ["pull", "--no-edit"];
        let theirs: Observed = native(&fixture, clean_repo.as_path(), &arguments);
        assert_eq!(theirs.code, Some(0), "{name}: native Git merges into the clean copy: {}", String::from_utf8_lossy(theirs.stderr.as_slice()));
        let ours: Observed = run(&fixture, ours_repo.as_path(), &arguments);
        assert_eq!(ours.code, Some(0), "{name}: {}", String::from_utf8_lossy(ours.stderr.as_slice()));
        assert_eq!(commit_shape(&fixture, ours_repo.as_path()), commit_shape(&fixture, clean_repo.as_path()), "{name}: the merge equals native Git's");
        let after: Vec<String> = git_text(&fixture, ours_repo.as_path(), &["diff", "--cached", "--name-status", "HEAD"]).lines().map(String::from).collect();
        assert_eq!(after, staged, "{name}: the other session's staged work is still staged and nothing else is");
        remove_settled(&fixture);
    }
}

/// A pull the wrapper does not read exactly,
/// or that Git refuses by its own advice,
/// is native Git's whole,
/// in words and status.
#[test]
fn a_pull_native_git_refuses_or_the_wrapper_does_not_read_is_native_gits() {
    let cases: [(&str, &[&str]); 4] = [
        ("advice", &["pull", "--no-edit"]),
        ("ff-only", &["pull", "--ff-only"]),
        ("rebase", &["pull", "--rebase"]),
        ("abbreviation", &["pull", "--no-reb", "--no-edit"]),
    ];
    for (name, arguments) in cases {
        let fixture: Fixture = fixture(format!("pull-native-{name}").as_str());
        let ours_repo: PathBuf = diverged_with_remote(&fixture);
        let theirs_repo: PathBuf = copy_of(&fixture, ours_repo.as_path(), "clean");
        let theirs: Observed = native(&fixture, theirs_repo.as_path(), arguments);
        let ours: Observed = run(&fixture, ours_repo.as_path(), arguments);
        assert_eq!(ours.code, theirs.code, "{name}: {}", String::from_utf8_lossy(&ours.stderr));
        assert_eq!(String::from_utf8_lossy(&ours.stdout), String::from_utf8_lossy(&theirs.stdout), "{name}: output");
        assert_eq!(String::from_utf8_lossy(&ours.stderr), String::from_utf8_lossy(&theirs.stderr), "{name}: diagnostics");
        // The two runs are made one after the other and Git stamps a commit it makes with the second it is made in, so their commit names
        //  differ whenever the runs straddle a second boundary: what must be equal is the commit's content, not its name.
        assert_eq!(head_shape(&fixture, ours_repo.as_path()), head_shape(&fixture, theirs_repo.as_path()), "{name}: head");
        remove_settled(&fixture);
    }
}

/// A repository whose `main` is one commit behind a bare remote and whose fetch refspec updates the branches,
/// so a fetch moves
///  the checked-out branch.
fn behind_with_mirror_refspec(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, std::ffi::OsStr::new("repo"));
    let remote: PathBuf = fixture.root.join("remote.git");
    git(fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "remote.git"]);
    git(fixture, repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    std::fs::write(repo.join("f.txt"), b"one\n").expect("file");
    git(fixture, repo.as_path(), &["add", "--", "f.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=one"]);
    std::fs::write(repo.join("f.txt"), b"two\n").expect("file");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    git(fixture, repo.as_path(), &["push", "--quiet", "origin", "main"]);
    git(fixture, repo.as_path(), &["reset", "--quiet", "--hard", "HEAD~1"]);
    git(fixture, repo.as_path(), &["config", "remote.origin.fetch", "+refs/heads/*:refs/heads/*"]);
    git(fixture, repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(fixture, repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    return repo;
}

/// A fetch that moves the checked-out branch takes the working tree along,
/// then the pull merges,
/// as in native Git.
#[test]
fn a_fetch_that_moves_the_checked_out_branch_updates_the_working_tree_as_native_git_does() {
    let fixture: Fixture = fixture("pull-moved-head");
    let repo: PathBuf = behind_with_mirror_refspec(&fixture);
    let clean: PathBuf = copy_of(&fixture, repo.as_path(), "clean");
    let arguments: [&str; 3] = ["pull", "--no-rebase", "--no-edit"];
    let theirs: Observed = native(&fixture, clean.as_path(), &arguments);
    let ours: Observed = run(&fixture, repo.as_path(), &arguments);
    assert_eq!((ours.code, theirs.code), (Some(0), Some(0)), "{}", String::from_utf8_lossy(&ours.stderr));
    assert_eq!(String::from_utf8_lossy(&ours.stdout), String::from_utf8_lossy(&theirs.stdout), "output");
    assert_eq!(String::from_utf8_lossy(&ours.stderr), String::from_utf8_lossy(&theirs.stderr), "diagnostics");
    assert_eq!(std::fs::read_to_string(repo.join("f.txt")).expect("file"), "two\n", "the working tree followed the fetch");
    assert_eq!(git_text(&fixture, repo.as_path(), &["status", "--porcelain=v1"]), "", "nothing is left modified");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), git_text(&fixture, clean.as_path(), &["rev-parse", "HEAD"]));
    remove_settled(&fixture);
}

/// A fetch that moves the checked-out branch while another program holds `index.lock`:
/// the pull says what it waits for and
///  takes the working tree along when the lock is gone,
/// where native Git's `read-tree` step would have died on the lock.
#[test]
fn a_pull_whose_fetch_moved_the_branch_waits_for_a_held_index_lock() {
    let fixture: Fixture = fixture("pull-moved-head-lock");
    let repo: PathBuf = behind_with_mirror_refspec(&fixture);
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(&lock, b"").expect("another program holds the lock");
    let mut command = wrapped_commit(&fixture, repo.as_path(), &["pull", "--no-rebase", "--no-edit"]);
    command.env("CLI_GIT_WAIT_NOTICE_SECONDS", "1");
    let ended: Ended = run_acting_on_line(&mut command, "waits for the index lock", &mut || {
        std::fs::remove_file(&lock).expect("the other program lets go");
    });
    assert_eq!(ended.code, Some(0), "{}", ended.stderr);
    assert_eq!(std::fs::read_to_string(repo.join("f.txt")).expect("file"), "two\n", "the working tree followed the fetch");
    assert_eq!(git_text(&fixture, repo.as_path(), &["status", "--porcelain=v1"]), "", "nothing is left modified");
    remove_settled(&fixture);
}

/// The elapsed times of one arm,
/// in milliseconds,
/// sorted.
fn sorted_milliseconds(times: &[std::time::Duration]) -> Vec<u128> {
    let mut milliseconds: Vec<u128> = times.iter().map(std::time::Duration::as_millis).collect();
    milliseconds.sort_unstable();
    return milliseconds;
}

/// A measurement for the handover,
/// not a control:
/// the same pull timed with native Git (twice,
/// so the spread of one arm shows),
///  with the wrapper's split pull,
/// with a pull the wrapper leaves to native Git (`--autostash`) and with the typed merge of the
///  fetched branch,
/// run in turns on fresh copies of one repository,
/// each with a remote of its own,
/// under the load the host has.
///  Once for a pull that merges and once for a pull that fast-forwards.
/// It prints the sorted times;
/// nothing is asserted.
#[test]
#[ignore = "a measurement for the handover, not a control"]
fn measure_the_split_pull_against_native_git() {
    const ROUNDS: usize = 9;
    const ARMS: [&str; 5] = ["native first", "wrapper split", "wrapper leaves it to native", "native second", "wrapper typed merge"];
    for scenario in ["merging", "fast-forwarding"] {
        let fixture: Fixture = fixture(format!("pull-measure-{scenario}").as_str());
        let template: PathBuf = diverged_with_remote(&fixture);
        if scenario == "fast-forwarding" {
            git(&fixture, template.as_path(), &["reset", "--quiet", "--hard", "feature~1"]);
        }
        git(&fixture, template.as_path(), &["fetch", "--quiet", "origin"]);
        let mut times: Vec<Vec<std::time::Duration>> = vec![Vec::new(); ARMS.len()];
        for round in 0..ROUNDS {
            for arm in 0..ARMS.len() {
                let copy: PathBuf = copy_of(&fixture, template.as_path(), format!("copy-{scenario}-{round}-{arm}").as_str());
                // Each copy pulls from a remote of its own, because a merge the wrapper completes is pushed and would turn the
                // next copy's pull into a fast-forward.
                let own_remote: PathBuf = copy_of(&fixture, fixture.root.join("remote.git").as_path(), format!("remote-{scenario}-{round}-{arm}.git").as_str());
                git(&fixture, copy.as_path(), &["remote", "set-url", "origin", own_remote.to_str().expect("UTF-8")]);
                let started = std::time::Instant::now();
                let observed: Observed = match arm {
                    0 | 3 => native(&fixture, copy.as_path(), &["pull", "--no-rebase", "--no-edit"]),
                    1 => observe(&mut wrapped_commit(&fixture, copy.as_path(), &["pull", "--no-rebase", "--no-edit"]), b""),
                    2 => observe(&mut wrapped_commit(&fixture, copy.as_path(), &["pull", "--no-rebase", "--autostash", "--no-edit"]), b""),
                    _ => observe(&mut wrapped_commit(&fixture, copy.as_path(), &["merge", "--no-edit", "origin/main"]), b""),
                };
                times[arm].push(started.elapsed());
                assert_eq!(observed.code, Some(0), "{scenario} arm {arm}: {}", String::from_utf8_lossy(&observed.stderr));
                if round == 0 {
                    eprintln!("MEASURE {scenario} {} reflog: {}", ARMS[arm], git_text(&fixture, copy.as_path(), &["reflog", "-1", "--format=%gs"]));
                }
            }
        }
        for arm in 0..ARMS.len() {
            eprintln!("MEASURE {scenario} {} {:?}", ARMS[arm], sorted_milliseconds(times[arm].as_slice()));
        }
        remove_settled(&fixture);
    }
}
