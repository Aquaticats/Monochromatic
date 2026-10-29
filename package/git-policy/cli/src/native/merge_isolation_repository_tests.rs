//! Controls for the decisions of the isolated merge that read a repository: the settings, the staged-work check, and
//! the first decisions of an attempt. Each runs real Git in a disposable repository.

use super::*;
use crate::forwarded_command::Route;
use crate::test_support::{REAL_GIT, fixture, git, git_context, git_text, isolated_overlay, remove, repository};
use crate::worktree_identity::resolve_worktree_identity;

/// A session over the repository at `repo`, the way `attempt` makes one.
fn session_in(repo: &Path) -> Session {
    let mut context: GitContext = git_context();
    context.global_prefix = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    return Session { context };
}

/// The settings read from a repository that holds the given `git config` entries, in the order given.
fn config_with(name: &str, entries: &[(&str, &str)]) -> MergeConfig {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for (key, value) in entries {
        git(repo.as_path(), &["config", "--add", key, value]);
    }
    let config: MergeConfig = read_config(&session_in(repo.as_path())).expect("the settings are read");
    remove(parent.as_path());
    return config;
}

/// A setting that changes the merge in a way this path does not reproduce defers the whole command to native Git,
/// whichever setting it is and wherever it falls among the others; a setting that is off, or none, defers nothing.
#[test]
fn only_the_settings_the_path_cannot_reproduce_defer_the_merge() {
    let deferring: [(&str, &[(&str, &str)]); 5] = [
        ("autostash", &[("merge.autostash", "true")]),
        ("verifysignatures", &[("merge.verifysignatures", "yes")]),
        ("branch-options", &[("branch.main.mergeoptions", "--no-ff")]),
        ("deferring-then-off", &[("merge.verifysignatures", "true"), ("merge.autostash", "false")]),
        ("off-then-deferring", &[("merge.verifysignatures", "false"), ("merge.autostash", "true")]),
    ];
    for (name, entries) in deferring {
        assert!(config_with(format!("merge-config-{name}").as_str(), entries).deferred, "{name}");
    }
    let quiet: [(&str, &[(&str, &str)]); 3] = [
        ("none", &[]),
        ("both-off", &[("merge.verifysignatures", "false"), ("merge.autostash", "no")]),
        ("honored", &[("merge.ff", "false"), ("merge.stat", "false"), ("commit.gpgsign", "true")]),
    ];
    for (name, entries) in quiet {
        assert!(!config_with(format!("merge-config-{name}").as_str(), entries).deferred, "{name}");
    }
}

/// The settings the path honors are read as Git reads them.
#[test]
fn the_honored_settings_are_read_from_the_repository() {
    let unset: MergeConfig = config_with("merge-config-honored-unset", &[]);
    assert_eq!((unset.stat, unset.fast_forward, unset.fast_forward_only, unset.sign_commits), (true, None, false, false));
    let set: MergeConfig = config_with(
        "merge-config-honored-set",
        &[("merge.ff", "false"), ("merge.stat", "false"), ("commit.gpgsign", "true")],
    );
    assert_eq!((set.stat, set.fast_forward, set.fast_forward_only, set.sign_commits), (false, Some(false), false, true));
    let only: MergeConfig = config_with("merge-config-honored-only", &[("merge.ff", "only")]);
    assert!(only.fast_forward_only && only.fast_forward.is_none());
}

/// Index entries of one path at the given stages, written with `update-index --index-info`: what a conflict leaves.
fn unmerge(repo: &Path, path: &str, stages: &[u8]) {
    let mut lines: String = String::new();
    for stage in stages {
        std::fs::write(repo.join(path), format!("{path} at stage {stage}\n")).expect("file");
        let blob: String = git_text(repo, &["hash-object", "-w", "--", path]);
        lines.push_str(format!("100644 {blob} {stage}\t{path}\n").as_str());
    }
    git(repo, &["rm", "--quiet", "--cached", "--force", "--ignore-unmatch", "--", path]);
    let mut child = std::process::Command::new(REAL_GIT)
        .current_dir(repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .args(["update-index", "--index-info"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("git starts");
    std::io::Write::write_all(&mut child.stdin.take().expect("piped input"), lines.as_bytes()).expect("entries written");
    assert!(child.wait().expect("git finishes").success(), "the entries of {path} were written");
}

/// An unmerged path means a conflict is in progress, which native Git reports itself, so the command is forwarded;
/// each way Git names an unmerged state is one of them.
#[test]
fn every_unmerged_state_is_forwarded() {
    let states: [(&str, &[u8]); 7] = [
        ("UU", &[1, 2, 3]),
        ("AA", &[2, 3]),
        ("DD", &[1]),
        ("DU", &[1, 3]),
        ("UD", &[1, 2]),
        ("AU", &[2]),
        ("UA", &[3]),
    ];
    for (code, stages) in states {
        let parent: PathBuf = fixture(format!("merge-unsaved-{code}").as_str());
        let repo: PathBuf = repository(parent.as_path(), "repo");
        std::fs::write(repo.join("c.txt"), b"base\n").expect("file");
        git(repo.as_path(), &["add", "--", "c.txt"]);
        git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
        unmerge(repo.as_path(), "c.txt", stages);
        let status: String = git_text(repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=no"]);
        assert_eq!(status, format!("{code} c.txt"), "the index holds the state under test");
        assert!(matches!(unsaved_paths(&session_in(repo.as_path())), Err(Bail::Forward(None))), "{code}");
        remove(parent.as_path());
    }
}

/// Ordinary staged and unstaged work is not a conflict: its paths are returned, one per entry.
#[test]
fn staged_and_unstaged_work_is_listed_by_path() {
    let parent: PathBuf = fixture("merge-unsaved-listed");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for file in ["a.txt", "b.txt", "c.txt", "d.txt"] {
        std::fs::write(repo.join(file), format!("{file} base\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--", "a.txt", "b.txt", "c.txt", "d.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"a staged\n").expect("file");
    std::fs::write(repo.join("n.txt"), b"added\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt", "n.txt"]);
    git(repo.as_path(), &["rm", "--quiet", "--", "b.txt"]);
    std::fs::write(repo.join("c.txt"), b"c edited, not staged\n").expect("file");
    let Ok(mut paths) = unsaved_paths(&session_in(repo.as_path())) else {
        panic!("staged and unstaged work is no reason to forward");
    };
    paths.sort();
    let expected: Vec<Vec<u8>> =
        ["a.txt", "b.txt", "c.txt", "n.txt"].iter().map(|name| { return name.as_bytes().to_vec(); }).collect();
    assert_eq!(paths, expected);
    remove(parent.as_path());
}

/// A repository where `feature` and `main` have diverged and another session staged `u.txt`.
fn diverged_with_staged_work(name: &str) -> (PathBuf, PathBuf) {
    return diverged_with_files(name, &[]);
}

/// A repository where `feature` and `main` have diverged and another session staged `u.txt`, whose base commit also
/// holds the `extra` files, which the merge does not touch.
fn diverged_with_files(name: &str, extra: &[&str]) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(name);
    let repo: PathBuf = repository(parent.as_path(), "repo");
    for file in ["u.txt", "f.txt", "h.txt"].iter().chain(extra.iter()) {
        std::fs::write(repo.join(file), format!("{file} base\nwith enough lines\nfor a rename to be found\n")).expect("file");
        git(repo.as_path(), &["add", "--", file]);
    }
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("f.txt"), b"f.txt feature\n").expect("file");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::fs::write(repo.join("dir/new.txt"), b"new\n").expect("file");
    git(repo.as_path(), &["add", "--", "dir/new.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature work"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("h.txt"), b"h.txt main\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main work"]);
    std::fs::write(repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(repo.as_path(), &["add", "--", "u.txt"]);
    return (parent, repo);
}

/// What `run_isolated_merge` makes of `arguments`, run as a command of the given route in `repo`.
fn attempted(
    repo: &Path,
    route: Route,
    arguments: &[&str],
    environment: &[(OsString, OsString)],
) -> IsolatedMerge {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    let identity: WorktreeIdentity =
        resolve_worktree_identity(Path::new(REAL_GIT), prefix.as_slice(), isolated_overlay().as_slice())
            .expect("identity");
    let mut words: Vec<OsString> = prefix.clone();
    words.extend(arguments.iter().map(OsString::from));
    let command: ResolvedCommand = ResolvedCommand {
        arguments: words,
        subcommand_index: 2,
        subcommand: Some(OsString::from(arguments[0])),
        route,
    };
    let overlay: Vec<(OsString, OsString)> = isolated_overlay();
    return run_isolated_merge(&IsolatedMergeInput {
        real_git: Path::new(REAL_GIT),
        overlay: overlay.as_slice(),
        identity: &identity,
        command: &command,
        environment,
    });
}

/// Whether the wrapper completed the merge with status 0.
fn is_completed(outcome: &IsolatedMerge) -> bool {
    return matches!(outcome, IsolatedMerge::Finished { code: 0, .. });
}

/// Only a built-in `merge` is completed: the same words under another command, or under a name that is not a
/// built-in, are native Git's, whatever the staged work, and nothing is written.
#[test]
fn only_a_builtin_merge_is_completed() {
    let (parent, repo): (PathBuf, PathBuf) = diverged_with_staged_work("merge-attempt-only-merges");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let forwarded: IsolatedMerge = IsolatedMerge::Forward { notice: None };
    assert_eq!(attempted(repo.as_path(), Route::Direct, &["status", "--no-edit", "feature"], &[]), forwarded);
    assert_eq!(attempted(repo.as_path(), Route::Unresolved, &["merge", "--no-edit", "feature"], &[]), forwarded);
    assert_eq!(attempted(repo.as_path(), Route::ShellAlias, &["merge", "--no-edit", "feature"], &[]), forwarded);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing was merged");
    assert!(!repo.join(".git/cli-git-merges").exists(), "no record was written");
    let direct: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
    assert!(is_completed(&direct), "the same fixture is completed as a built-in merge: {direct:?}");
    assert_ne!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head);
    remove(parent.as_path());
}

/// A name that is a range, not one commit, resolves to more than the two commits the merge needs; the command is
/// native Git's, which refuses it, and nothing is written.
#[test]
fn a_range_in_place_of_a_commit_is_native_gits() {
    let (parent, repo): (PathBuf, PathBuf) = diverged_with_staged_work("merge-attempt-range");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    for range in ["main..feature", "main...feature"] {
        let outcome: IsolatedMerge =
            attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "--message=range", range], &[]);
        assert_eq!(outcome, IsolatedMerge::Forward { notice: None }, "{range}");
    }
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing was merged");
    remove(parent.as_path());
}

/// A staged rename lists its original path as a field of its own, with no status letters, and a path may begin with
/// the letters of an unmerged state; such a rename is other work to keep, not a conflict, and the merge is completed.
#[test]
fn a_staged_rename_is_no_conflict_whatever_its_source_is_called() {
    for source in ["Uold.txt", "xUold.txt", "AAold.txt", "DDold.txt", "UUold.txt"] {
        let (parent, repo): (PathBuf, PathBuf) =
            diverged_with_files(format!("merge-unsaved-rename-{source}").as_str(), &[source]);
        git(repo.as_path(), &["mv", "--", source, "moved.txt"]);
        let status: String = git_text(repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=no"]);
        assert!(status.contains(format!("R  {source} -> moved.txt").as_str()), "{source}: {status}");
        let Ok(mut paths) = unsaved_paths(&session_in(repo.as_path())) else {
            panic!("{source}: a staged rename is no reason to forward");
        };
        paths.sort();
        let mut expected: Vec<Vec<u8>> =
            [source, "moved.txt", "u.txt"].iter().map(|name| { return name.as_bytes().to_vec(); }).collect();
        expected.sort();
        assert_eq!(paths, expected, "{source}");
        let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
        let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
        assert!(is_completed(&outcome), "{source}: {outcome:?}");
        assert_ne!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "{source}: merged");
        assert!(git_text(repo.as_path(), &["status", "--porcelain=v1"]).contains("moved.txt"), "{source}: rename kept");
        remove(parent.as_path());
    }
}

/// A conflict is still forwarded when it sits after a staged rename in the same status.
#[test]
fn a_conflict_after_a_staged_rename_is_still_forwarded() {
    let (parent, repo): (PathBuf, PathBuf) = diverged_with_files("merge-unsaved-rename-conflict", &["old.txt", "z.txt"]);
    git(repo.as_path(), &["mv", "--", "old.txt", "moved.txt"]);
    unmerge(repo.as_path(), "z.txt", &[1, 2, 3]);
    let status: String = git_text(repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=no"]);
    assert!(status.contains("R  old.txt -> moved.txt") && status.contains("UU z.txt"), "{status}");
    assert!(matches!(unsaved_paths(&session_in(repo.as_path())), Err(Bail::Forward(None))));
    remove(parent.as_path());
}

/// A branch with no history in common is merged only when `--allow-unrelated-histories` says so; without it the
/// command is native Git's, which refuses with its own words.
#[test]
fn unrelated_histories_are_merged_only_when_allowed() {
    let parent: PathBuf = fixture("merge-attempt-unrelated");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("u.txt"), b"u.txt base\n").expect("file");
    git(repo.as_path(), &["add", "--", "u.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--orphan", "other"]);
    std::fs::write(repo.join("o.txt"), b"o.txt other\n").expect("file");
    git(repo.as_path(), &["add", "--", "o.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=other root"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("u.txt"), b"u.txt staged by another session\n").expect("file");
    git(repo.as_path(), &["add", "--", "u.txt"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "other"], &[]);
    assert_eq!(refused, IsolatedMerge::Forward { notice: None });
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing was merged");
    let allowed: IsolatedMerge =
        attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "--allow-unrelated-histories", "other"], &[]);
    assert!(is_completed(&allowed), "{allowed:?}");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD^1"]), head);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD^2"]), git_text(repo.as_path(), &["rev-parse", "other"]));
    assert_eq!(std::fs::read(repo.join("o.txt")).expect("the other branch's file"), b"o.txt other\n");
    assert_eq!(git_text(repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt", "the staged work stayed staged");
    remove(parent.as_path());
}

/// An attempt recovers a dead merge's record first, before it reads the index: the checkout the dead merge left half
/// applied would otherwise read as another session's staged paths and refuse the merge.
#[test]
fn an_attempt_recovers_a_dead_merge_before_it_checks_the_staged_work() {
    let (parent, repo): (PathBuf, PathBuf) = diverged_with_staged_work("merge-attempt-recovers");
    let old_oid: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    let tree: String = git_text(repo.as_path(), &["merge-tree", "--write-tree", old_oid.as_str(), feature.as_str()]);
    let new_oid: String = git_text(
        repo.as_path(),
        &["commit-tree", tree.as_str(), "-p", old_oid.as_str(), "-p", feature.as_str(), "-m", "merge"],
    );
    let mut owner = current_transaction_owner("44444444-0000-4000-8000-000000000004", "2026-10-29T00:00:00.000Z")
        .expect("owner");
    owner.owner_identity = String::from("linux:1");
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: repo.to_string_lossy().into_owned(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: old_oid.clone(),
        new_oid: new_oid.clone(),
    };
    let record: PathBuf = write_merge_intent(repo.join(".git").as_path(), &intent).expect("the record");
    git(repo.as_path(), &["read-tree", "-m", "-u", old_oid.as_str(), new_oid.as_str()]);
    let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
    assert_eq!(outcome, IsolatedMerge::Forward { notice: None }, "the branch is merged once the dead merge is finished");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), new_oid, "the dead merge was finished");
    assert!(!record.exists(), "its record is gone");
    remove(parent.as_path());
}

/// `--no-edit` settles the question of the editor whatever `GIT_MERGE_AUTOEDIT` says, and without it the variable
/// decides: `yes` forwards with the notice, `no` completes the merge.
#[test]
fn the_editor_decision_follows_no_edit_then_the_environment() {
    let variable = |value: &str| -> Vec<(OsString, OsString)> {
        return vec![(OsString::from("GIT_MERGE_AUTOEDIT"), OsString::from(value))];
    };
    for (name, arguments, value, completes) in [
        ("no-edit-over-yes", ["merge", "--no-edit", "feature"], "yes", true),
        ("no-edit-over-no", ["merge", "--no-edit", "feature"], "no", true),
        ("yes", ["merge", "--verbose", "feature"], "yes", false),
        ("no", ["merge", "--verbose", "feature"], "no", true),
    ] {
        let (parent, repo): (PathBuf, PathBuf) =
            diverged_with_staged_work(format!("merge-attempt-edit-{name}").as_str());
        let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
        let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &arguments, variable(value).as_slice());
        if completes {
            assert!(is_completed(&outcome), "{name}: {outcome:?}");
            assert_ne!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "{name}: merged");
        } else {
            let IsolatedMerge::Forward { notice: Some(notice) } = outcome else {
                panic!("{name}: the merge would open an editor, so it is forwarded with the notice: {outcome:?}");
            };
            assert!(notice.contains("would open an editor"), "{name}: {notice}");
            assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "{name}: not merged");
        }
        remove(parent.as_path());
    }
}
