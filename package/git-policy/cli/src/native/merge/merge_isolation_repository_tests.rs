//! Controls for the decisions of the isolated merge that read a repository:
//! the settings,
//! the staged-work check,
//! and
//! the first decisions of an attempt.
//! Each runs real Git in a disposable repository.

use super::*;
use crate::forwarded_command::ResolvedCommand;
use crate::child_environment::environment_with_overlay;
use crate::merge_isolation_git_reads::unsaved_paths;
use crate::merge_isolation_journal::{MergeIntent, write_merge_intent};
use crate::transaction_owner::current_transaction_owner;
use std::ffi::OsString;
use crate::forwarded_command::Route;
use crate::test_support::{REAL_GIT, fixture, git, git_context, git_text, isolated_overlay, remove, repository};
use crate::worktree_identity::resolve_worktree_identity;

/// A session over the repository at `repo`,
/// the way `attempt` makes one.
fn session_in(repo: &Path) -> Session {
    let mut context: GitContext = git_context();
    context.global_prefix = vec![OsString::from("-C"), repo.as_os_str().to_owned()];
    return Session { context };
}

/// The settings read from a repository that holds the given `git config` entries,
/// in the order given.
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
/// whichever setting it is and wherever it falls among the others;
/// a setting that is off,
/// or none,
/// defers nothing.
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

/// Index entries of one path at the given stages,
/// written with `update-index --index-info`:
/// what a conflict leaves.
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

/// An unmerged path means a conflict is in progress,
/// which native Git reports itself,
/// so the command is forwarded;
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

/// Ordinary staged and unstaged work is not a conflict:
/// its paths are returned,
/// one per entry.
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

/// A repository where `feature` and `main` have diverged and another session staged `u.txt`,
/// whose base commit also
/// holds the `extra` files,
/// which the merge does not touch.
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

/// What `run_isolated_merge` makes of `arguments`,
/// run as a command of the given route in `repo`.
fn attempted(
    repo: &Path,
    route: Route,
    arguments: &[&str],
    environment: &[(OsString, OsString)],
) -> IsolatedMerge {
    return attempted_with(Path::new(REAL_GIT), repo, route, arguments, environment);
}

/// `attempted` with the Git executable given,
/// so a control can stand in for it.
fn attempted_with(
    real_git: &Path,
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
        // A merge reached here runs no shell alias.
        shell_command: None,
    };
    // The variables of the command reach its Git children as the person's own environment does,
    // so a control that names an editor
    //  names the editor the merge resolves.
    let overlay: Vec<(OsString, OsString)> = environment_with_overlay(environment, isolated_overlay().as_slice());
    return run_isolated_merge(&IsolatedMergeInput {
        real_git,
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

/// Only a built-in `merge` is completed:
/// the same words under another command,
/// or under a name that is not a
/// built-in,
/// are native Git's,
/// whatever the staged work,
/// and nothing is written.
#[test]
fn only_a_builtin_merge_is_completed() {
    let (parent, repo): (PathBuf, PathBuf) = diverged_with_staged_work("merge-attempt-only-merges");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let forwarded: IsolatedMerge = IsolatedMerge::Forward { notice: None };
    assert_eq!(attempted(repo.as_path(), Route::Direct, &["status", "--no-edit", "feature"], &[]), forwarded);
    assert_eq!(attempted(repo.as_path(), Route::Unresolved, &["merge", "--no-edit", "feature"], &[]), forwarded);
    assert_eq!(attempted(repo.as_path(), Route::ShellAlias, &["merge", "--no-edit", "feature"], &[]), forwarded);
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing was merged");
    assert!(!repo.join(".git/monochromatic-git-policy-cli-merges").exists(), "no record was written");
    let direct: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
    assert!(is_completed(&direct), "the same fixture is completed as a built-in merge: {direct:?}");
    assert_ne!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head);
    remove(parent.as_path());
}

/// A name that is a range,
/// not one commit,
/// resolves to more than the two commits the merge needs;
/// the command is
/// native Git's,
/// which refuses it,
/// and nothing is written.
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

/// A staged rename lists its original path as a field of its own,
/// with no status letters,
/// and a path may begin with
/// the letters of an unmerged state;
/// such a rename is other work to keep,
/// not a conflict,
/// and the merge is completed.
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

/// A branch with no history in common is merged only when `--allow-unrelated-histories` says so;
/// without it the
/// command is native Git's,
/// which refuses with its own words.
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

/// An attempt recovers a dead merge's record first,
/// before it reads the index:
/// the checkout the dead merge left half
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
        repository_root: repo.to_path_buf(),
        target_ref: Some(String::from("refs/heads/main")),
        old_oid: old_oid.clone(),
        new_oid: new_oid.clone(),
        side_ref: None,
        carried: None,
    };
    let record: PathBuf = write_merge_intent(repo.join(".git").as_path(), &intent).expect("the record");
    git(repo.as_path(), &["read-tree", "-m", "-u", old_oid.as_str(), new_oid.as_str()]);
    let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
    assert_eq!(outcome, IsolatedMerge::Forward { notice: None }, "the branch is merged once the dead merge is finished");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), new_oid, "the dead merge was finished");
    assert!(!record.exists(), "its record is gone");
    remove(parent.as_path());
}

/// What:
///  A merge that opens an editor takes its message from the person's editor,
/// and every option that settles the question keeps
///  the merge away from it.
/// Why:
///   `--no-edit` declines the editor and `-m` gives the message (`prepare_to_commit`,
/// `builtin/merge.c`);
/// without
///  either,
/// `GIT_MERGE_AUTOEDIT` or a terminal decides.
/// The staged work is carried in every case,
/// which is what native
///  Git cannot do.
#[test]
fn the_editor_decision_follows_no_edit_then_the_environment() {
    let variable = |value: &str, editor: &Path| -> Vec<(OsString, OsString)> {
        return vec![
            (OsString::from("GIT_MERGE_AUTOEDIT"), OsString::from(value)),
            (OsString::from("GIT_EDITOR"), editor.as_os_str().to_owned()),
        ];
    };
    for (name, arguments, value, writes) in [
        ("no-edit-over-yes", ["merge", "--no-edit", "feature"], "yes", false),
        ("no-edit-over-no", ["merge", "--no-edit", "feature"], "no", false),
        ("message-over-yes", ["merge", "--message=given by -m", "feature"], "yes", false),
        ("yes", ["merge", "--verbose", "feature"], "yes", true),
        ("no", ["merge", "--verbose", "feature"], "no", false),
    ] {
        let (parent, repo): (PathBuf, PathBuf) =
            diverged_with_staged_work(format!("merge-attempt-edit-{name}").as_str());
        let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
        let editor: PathBuf = parent.join("editor");
        crate::test_support::executable(editor.as_path(), b"#!/bin/sh\nprintf 'written in the editor\\n' >> \"$1\"\n");
        let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &arguments, variable(value, editor.as_path()).as_slice());
        assert!(is_completed(&outcome), "{name}: the staged work is carried: {outcome:?}");
        assert_ne!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "{name}: merged");
        let message: String = git_text(repo.as_path(), &["log", "-1", "--format=%B"]);
        assert_eq!(message.contains("written in the editor"), writes, "{name}: the message the merge carries: {message:?}");
        assert_eq!(
            git_text(repo.as_path(), &["status", "--porcelain=v1", "--", "u.txt"]),
            "M  u.txt",
            "{name}: the work the index held is still staged"
        );
        remove(parent.as_path());
    }
}

/// The branch moves inside Git's own fast-forward,
/// after it installed the merge's paths and before its ref update,
/// which
/// then fails (`cannot lock ref 'HEAD': is at <winner> but expected <old>`):
/// the stand-in for Git plays the winner's commit
/// and the loser's install once,
/// then reports Git's failure.
/// The paths go back to the winner's content and the merge is made
/// again on the winner,
/// so it lands with the winner as its first parent and nothing half applied.
#[test]
fn a_ref_update_that_loses_the_race_inside_gits_fast_forward_is_made_again_on_the_new_head() {
    let (parent, repo): (PathBuf, PathBuf) = diverged_with_staged_work("merge-attempt-inside-race");
    let shim: PathBuf = parent.join("racing-git");
    let marker: PathBuf = parent.join("raced");
    let script: String = format!(
        "#!/bin/sh\nfor argument in \"$@\"; do\n  if [ \"$argument\" = --ff-only ] && [ ! -e '{marker}' ]; then\n    : > '{marker}'\n    for merged; do :; done\n    cd '{repo}' || exit 1\n    old=$({real} rev-parse HEAD)\n    echo 'committed meanwhile' > z.txt\n    {real} add -- z.txt\n    {real} -c user.name=Racer -c user.email=racer@example.invalid commit --quiet --message=meanwhile -- z.txt\n    winner=$({real} rev-parse HEAD)\n    {real} read-tree -m -u \"$old\" \"$merged\"\n    echo \"fatal: update_ref failed for ref 'HEAD': cannot lock ref 'HEAD': is at $winner but expected $old\" >&2\n    exit 128\n  fi\ndone\nexec {real} \"$@\"\n",
        marker = marker.display(),
        repo = repo.display(),
        real = REAL_GIT
    );
    crate::test_support::executable(shim.as_path(), script.as_bytes());
    let outcome: IsolatedMerge = attempted_with(shim.as_path(), repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
    assert!(marker.exists(), "the race was played");
    assert!(is_completed(&outcome), "{outcome:?}");
    let winner: String = git_text(repo.as_path(), &["rev-parse", "HEAD^1"]);
    assert_eq!(git_text(repo.as_path(), &["log", "-1", "--format=%s", winner.trim()]).trim(), "meanwhile", "the merge sits on the winner");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD^2"]), git_text(repo.as_path(), &["rev-parse", "feature"]));
    assert_eq!(git_text(repo.as_path(), &["status", "--porcelain=v1"]), "M  u.txt", "only the other session's work differs from the head");
    assert!(repo.join("z.txt").exists() && repo.join("dir/new.txt").exists(), "both sides are in the checkout");
    assert!(!repo.join(".git/MERGE_HEAD").exists());
    assert_eq!(std::fs::read_dir(repo.join(".git/monochromatic-git-policy-cli-merges")).map_or(0, Iterator::count), 0, "no record is left");
    remove(parent.as_path());
}

/// What:
///  A named pipe where the merge writes or deletes a path,
///  and the sentence that stops the merge there.
/// Why:
///  A pipe holds no bytes,
/// so no layer of the carry can hold it,
/// and opening one for reading waits for a writer that
///  nothing promises:
/// the merge would wait without an end while it holds the checkout,
/// and Git's own update of the
///  checkout would remove the pipe without a copy (owner's rules
/// "nothing should ever wait forever or indefinitely",
///  "never discard user data").
/// The deadline is what turns the wait this control forbids into a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answered = finishedWithinAt(PATIENCE, () => attempted(repo, ['merge', '--no-ff', '--no-edit', 'feature'])); expect(answered?.code).toBe(1);
/// ```
#[cfg(unix)]
#[test]
fn a_named_pipe_where_the_merge_changes_a_path_stops_the_merge_without_waiting() {
    /// `is_fifo` reads the type of the entry a named pipe is.
    use std::os::unix::fs::FileTypeExt;
    let shapes: [(&str, fn(&str) -> (PathBuf, PathBuf), &str); 2] = [
        ("added", an_untracked_pipe_the_merge_adds, "pipe"),
        ("tracked", a_pipe_over_a_tracked_path_the_merge_changes, "tracked.txt"),
    ];
    for (label, build, path) in shapes {
        let (parent, repo): (PathBuf, PathBuf) = build(label);
        let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
        let wanted: PathBuf = repo.clone();
        let answered: Option<IsolatedMerge> = crate::parallel_test_support::finished_within_at(
            crate::parallel_test_support::PATIENCE,
            move || {
                return attempted(wanted.as_path(), Route::Direct, &["merge", "--no-ff", "--no-edit", "feature"], &[]);
            },
        );
        let Some(outcome) = answered else {
            panic!("{label}: the merge waited for a writer of the named pipe at {path} that never came");
        };
        let (code, stdout, stderr): (i32, String, String) = match outcome {
            IsolatedMerge::Finished { code, stdout, stderr } => (code, stdout, stderr),
            other => panic!("{label}: the merge is the wrapper's to stop, and it was forwarded: {other:?}"),
        };
        assert_eq!(code, 1, "{label}: {stderr}");
        assert!(stdout.is_empty(), "{label}: nothing is printed about a merge that was not made: {stdout}");
        assert!(stderr.contains(path), "{label}: the path is named: {stderr}");
        assert!(stderr.contains("a named pipe, a socket or a device"), "{label}: {stderr}");
        assert!(stderr.contains("nothing in the worktree, the index or the branch changed"), "{label}: {stderr}");
        let kind: std::fs::FileType = std::fs::symlink_metadata(repo.join(path)).expect("the pipe is still in the checkout").file_type();
        assert!(kind.is_fifo(), "{label}: the pipe is untouched");
        assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "{label}: nothing was merged");
        assert!(git_text(repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/cli-git/"]).is_empty(), "{label}: no ref keeps anything");
        remove(parent.as_path());
    }
}

/// A repository where `feature` adds `pipe` and `main` holds an untracked named pipe where it would be written.
#[cfg(unix)]
fn an_untracked_pipe_the_merge_adds(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(format!("merge-special-pipe-{name}").as_str());
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("base.txt"), b"base\n").expect("file");
    git(repo.as_path(), &["add", "--", "base.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("pipe"), b"the branch's bytes\n").expect("file");
    git(repo.as_path(), &["add", "--", "pipe"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=add-pipe"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    a_named_pipe(repo.join("pipe").as_path());
    return (parent, repo);
}

/// A repository where `feature` changes `tracked.txt`,
/// whose worktree copy on `main` is a named pipe.
#[cfg(unix)]
fn a_pipe_over_a_tracked_path_the_merge_changes(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(format!("merge-special-tracked-{name}").as_str());
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("tracked.txt"), b"base\n").expect("file");
    git(repo.as_path(), &["add", "--", "tracked.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("tracked.txt"), b"the branch's bytes\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    std::fs::remove_file(repo.join("tracked.txt")).expect("the tracked copy makes way for the pipe");
    a_named_pipe(repo.join("tracked.txt").as_path());
    let status: String = git_text(repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]);
    assert_eq!(status, " M tracked.txt", "the pipe over a tracked path is what Git's status reports, which is how the capture met it: {status}");
    return (parent, repo);
}

/// Put a named pipe at `path`,
/// which stands for the entry a program left where a file was,
/// and holds no bytes for a merge to carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// execFileSync('mkfifo', [path]);
/// ```
#[cfg(unix)]
fn a_named_pipe(path: &Path) {
    let made: std::process::ExitStatus = std::process::Command::new("mkfifo").arg(path).status().expect("mkfifo");
    assert!(made.success(), "mkfifo must create {}", path.display());
}

/// What:
///  An edit at a path whose index entry carries `assume-unchanged` or `skip-worktree` is carried,
///  and the merge is made.
/// Why:
///  Git's status reads both marks as "do not compare the worktree copy",
/// so the edit entered no capture and Git's own
///  update of the checkout refused the merge with `merge-isolation/not-applied`
/// (a cross-vendor audit of
///  0.6.1-alpha.10).
/// The merge is work the wrapper completes over unsaved work,
/// and the person's bytes are theirs
///  (`SPEC.md`,
/// "Merges and commits never fail",
/// "Carry edits forward").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const mark of ['--assume-unchanged', '--skip-worktree']) expect(attempted(repo, ['merge', '--no-ff', '--no-edit', 'feature'])).toHaveCode(0);
/// ```
#[test]
fn an_edit_an_index_mark_hides_is_carried_and_the_merge_is_made() {
    for mark in ["--assume-unchanged", "--skip-worktree"] {
        let (parent, repo): (PathBuf, PathBuf) = diverged_with_a_hidden_edit(mark, mark);
        let status: String = git_text(repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]);
        assert_eq!(status, "", "{mark}: Git's own status hides the edit, which is the premise of this control: {status}");
        let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
        let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-ff", "--no-edit", "feature"], &[]);
        assert!(is_completed(&outcome), "{mark}: the merge is completed over the hidden edit: {outcome:?}");
        assert_ne!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "{mark}: the merge commit landed");
        let content: String = std::fs::read_to_string(repo.join("a.txt")).expect("a.txt");
        assert!(content.contains("the person's own line"), "{mark}: the hidden edit is carried: {content}");
        assert!(content.contains("the other branch's line"), "{mark}: the merge's own line is there: {content}");
        assert!(git_text(repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/cli-git/"]).is_empty(), "{mark}: the capture's ref went with the merge");
        remove(parent.as_path());
    }
}

/// A repository where `feature` changes `a.txt`
/// and `main` holds a person's edit at that path whose bytes the given index mark hides.
fn diverged_with_a_hidden_edit(name: &str, mark: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(format!("merge-hidden-{name}").as_str());
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"base\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("a.txt"), b"base\nthe other branch's line\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    git(repo.as_path(), &["update-index", mark, "--", "a.txt"]);
    std::fs::write(repo.join("a.txt"), b"base\nthe person's own line\n").expect("file");
    return (parent, repo);
}

/// What:
///  How many parents the branch's commit has,
///  which is one for the commit a fast-forward moves the branch to and two for a merge commit.
/// Why:
///   `rev-list --parents` prints the commit and then its parents,
///  so the count of its fields says what a fast-forward
///  promises:
/// no merge commit was made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parents = (repo) => gitText(repo, ['rev-list', '--parents', '-n1', 'HEAD']).split(' ').length - 1;
/// ```
fn parents_of_the_head(repo: &Path) -> usize {
    return git_text(repo, &["rev-list", "--parents", "-n1", "HEAD"]).split(' ').count() - 1;
}

/// What:
///  A repository whose `feature` is strictly ahead of `main` by one commit that changes `a.txt`.
/// Why:
///   A merge of it is a fast-forward,
///  which is the shape the wrapper used to hand to native Git whole,
///  so every control about it
///  builds the same branch and adds the person's state alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = aFastForwardedBranch('ff-plain'); // main is feature's parent
/// ```
fn a_fast_forwarded_branch(name: &str) -> (PathBuf, PathBuf) {
    let parent: PathBuf = fixture(format!("merge-ff-{name}").as_str());
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("a.txt"), b"one\ntwo\nthree\n").expect("file");
    std::fs::write(repo.join("keep.txt"), b"keep\n").expect("file");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\nthree\nthe other branch's line\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=feature"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    return (parent, repo);
}

/// What:
///  The same branch,
///  whose one incoming commit adds `new.txt`,
///  which this repository's ignore rules cover.
/// Why:
///   An ignored untracked file at a path the incoming commit adds is the file native Git's fast-forward
///  overwrites without a word,
/// which is the loss the carry closed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = aFastForwardedBranchAdding('ignored', 'new.txt', true);
/// ```
fn a_fast_forwarded_branch_adding(name: &str, added: &str, ignored: bool) -> (PathBuf, PathBuf) {
    let (parent, repo): (PathBuf, PathBuf) = a_fast_forwarded_branch(name);
    if ignored {
        std::fs::write(repo.join(".gitignore"), format!("{added}\n")).expect("file");
        git(repo.as_path(), &["add", "--", ".gitignore"]);
        git(repo.as_path(), &["commit", "--quiet", "--message=ignore"]);
        git(repo.as_path(), &["switch", "--quiet", "feature"]);
        git(repo.as_path(), &["rebase", "--quiet", "main"]);
    }
    std::fs::write(repo.join(added), b"the branch's own line\n").expect("file");
    git(repo.as_path(), &["add", "--force", "--", added]);
    git(repo.as_path(), &["commit", "--quiet", "--message=add"]);
    git(repo.as_path(), &["switch", "--quiet", "main"]);
    return (parent, repo);
}

/// What:
///  A fast-forward in a worktree with nothing unsaved in the way is still native Git's,
///  which is the control that keeps the ordinary case as cheap as it was.
/// Why:
///  The whole command is handed over,
///  so the wrapper changes nothing and Git's own fast-forward runs it;
///  a wrapper that carried this
///  case would pay the carry's reads on every fast-forward a person types.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(attempted(repo, ['merge', '--no-edit', 'feature'])).toEqual({ kind: 'forward' });
/// ```
#[test]
fn a_fast_forward_with_nothing_unsaved_is_native_gits() {
    let (parent, repo): (PathBuf, PathBuf) = a_fast_forwarded_branch("plain");
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
    assert_eq!(outcome, IsolatedMerge::Forward { notice: None });
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), head, "the wrapper moved nothing, so native Git runs it");
    assert!(git_text(repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/cli-git/"]).is_empty(), "no capture was made");
    remove(parent.as_path());
}

/// What:
///  A fast-forward over an unsaved edit in a file the incoming commit changes is carried,
///  and the branch still moves by a
///  fast-forward alone.
/// Why:
///  Native Git refuses such a merge with `Your local changes ... would be overwritten by merge`,
///  while the same
///  merge of a diverged branch carries the edit;
///  the owner's answer "Carry edits forward" and "Manual merge should never fail
///  too."
/// are what the fast-forward shape owed as well (`SPEC.md`,
/// "Carry edits forward").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(attempted(repo, ['merge', '--no-edit', 'feature'])).toHaveCode(0); expect(repo.a.txt).toHold(both lines); expect(head).toBe(feature);
/// ```
#[test]
fn a_fast_forward_over_an_unsaved_edit_is_carried() {
    let (parent, repo): (PathBuf, PathBuf) = a_fast_forwarded_branch("unsaved");
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    std::fs::write(repo.join("a.txt"), b"the person's own line\ntwo\nthree\n").expect("file");
    let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
    assert!(is_completed(&outcome), "{outcome:?}");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), feature, "the branch moved to the incoming commit itself");
    assert_eq!(parents_of_the_head(repo.as_path()), 1, "no merge commit was made");
    let content: String = std::fs::read_to_string(repo.join("a.txt")).expect("a.txt");
    assert!(content.contains("the person's own line"), "the unsaved edit is carried: {content}");
    assert!(content.contains("the other branch's line"), "the merge's own line is there: {content}");
    assert_eq!(git_text(repo.as_path(), &["status", "--porcelain=v1", "--", "a.txt"]), " M a.txt", "the edit is still unsaved");
    assert!(git_text(repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/cli-git/"]).is_empty(), "the capture's ref went with the merge");
    remove(parent.as_path());
}

/// What:
///  A fast-forward keeps an ignored file of the person's at a path its commit adds,
///  where native Git overwrites it with no copy and no
///  notice.
/// Why:
///   The bytes are the only copy of what the person wrote,
///  and the carry of a diverged merge already kept them;
///  the same
///  work at the same path cannot be kept or lost depending on whether the other side is strictly ahead
///  (owner's rule
///  "never discard user data";
///  `SPEC.md`,
/// "Carry edits forward",
/// which names the ignored untracked files of the
///  person's).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(attempted(repo, ['merge', '--no-edit', 'feature'])).toHaveCode(0); expect(repo['new.txt']).toHold("PERSON-MARKER");
/// ```
#[test]
fn a_fast_forward_keeps_an_ignored_file_at_a_path_it_adds() {
    let (parent, repo): (PathBuf, PathBuf) = a_fast_forwarded_branch_adding("ignored", "new.txt", true);
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    std::fs::write(repo.join("new.txt"), b"PERSON-MARKER 7f3a\n").expect("file");
    assert_eq!(git_text(repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]), "", "the ignore rule hides the file, which is the premise");
    let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
    assert!(is_completed(&outcome), "{outcome:?}");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), feature, "the branch moved to the incoming commit itself");
    let content: String = std::fs::read_to_string(repo.join("new.txt")).expect("new.txt");
    assert!(content.contains("PERSON-MARKER 7f3a"), "the person's bytes are carried: {content}");
    assert!(content.contains("the branch's own line"), "the commit's own line is there: {content}");
    remove(parent.as_path());
}

/// What:
///  An edit that an `assume-unchanged` or `skip-worktree` entry hides is carried over a fast-forward too.
/// Why:
///   Every status read treats a marked path as "do not compare",
///  so the question about the paths a fast-forward writes
///  has to compare the bytes with the entry to see the edit at all;
///  the carry of a diverged merge does that
///  (`merge_isolation_marks_the_status_hides.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const mark of ['--assume-unchanged', '--skip-worktree']) expect(attempted(repo, ['merge', '--no-edit', 'feature'])).toHaveCode(0);
/// ```
#[test]
fn a_fast_forward_over_an_edit_an_index_mark_hides_is_carried() {
    for mark in ["--assume-unchanged", "--skip-worktree"] {
        let (parent, repo): (PathBuf, PathBuf) = a_fast_forwarded_branch(mark.trim_start_matches("--"));
        let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
        git(repo.as_path(), &["update-index", mark, "--", "a.txt"]);
        std::fs::write(repo.join("a.txt"), b"the person's own line\ntwo\nthree\n").expect("file");
        assert_eq!(git_text(repo.as_path(), &["status", "--porcelain=v1", "--untracked-files=all"]), "", "{mark}: the mark hides the edit, which is the premise");
        let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
        assert!(is_completed(&outcome), "{mark}: {outcome:?}");
        assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), feature, "{mark}: the branch moved to the incoming commit");
        let content: String = std::fs::read_to_string(repo.join("a.txt")).expect("a.txt");
        assert!(content.contains("the person's own line"), "{mark}: the hidden edit is carried: {content}");
        assert!(content.contains("the other branch's line"), "{mark}: the merge's own line is there: {content}");
        remove(parent.as_path());
    }
}

/// What:
///  The bytes of a `--ff-only` fast-forward are carried as any other fast-forward's,
///  and a merge that cannot fast-forward stays native Git's,
///  whose refusal keeps that option's promise.
/// Why:
///  `--ff-only` states that the branch may move only by a fast-forward;
///  carrying the person's work does not
///  break that promise,
/// because the branch still moves to the incoming commit alone,
///  and a merge that would make a
///  commit has to keep Git's own refusal (`fatal: Not possible to fast-forward`,
///  exit 128).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(attempted(repo, ['merge', '--no-edit', '--ff-only', 'feature'])).toHaveCode(0); expect(attempted(diverged, same)).toEqual({ kind: 'forward' });
/// ```
#[test]
fn an_ff_only_fast_forward_is_carried_and_a_diverged_one_is_native_gits() {
    let (parent, repo): (PathBuf, PathBuf) = a_fast_forwarded_branch("ff-only");
    let feature: String = git_text(repo.as_path(), &["rev-parse", "feature"]);
    std::fs::write(repo.join("a.txt"), b"the person's own line\ntwo\nthree\n").expect("file");
    let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "--ff-only", "feature"], &[]);
    assert!(is_completed(&outcome), "{outcome:?}");
    assert_eq!(git_text(repo.as_path(), &["rev-parse", "HEAD"]), feature, "the branch moved to the incoming commit itself");
    assert_eq!(parents_of_the_head(repo.as_path()), 1, "no merge commit was made");
    assert!(std::fs::read_to_string(repo.join("a.txt")).expect("a.txt").contains("the person's own line"), "the edit is carried");
    remove(parent.as_path());
    let (diverged_parent, diverged_repo): (PathBuf, PathBuf) = a_fast_forwarded_branch("ff-only-diverged");
    std::fs::write(diverged_repo.join("keep.txt"), b"a commit main has and feature does not\n").expect("file");
    git(diverged_repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    let head: String = git_text(diverged_repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: IsolatedMerge = attempted(diverged_repo.as_path(), Route::Direct, &["merge", "--no-edit", "--ff-only", "feature"], &[]);
    assert_eq!(refused, IsolatedMerge::Forward { notice: None }, "the refusal is native Git's, with its own words");
    assert_eq!(git_text(diverged_repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing was changed");
    remove(diverged_parent.as_path());
}

/// What:
///  An entry standing at an ancestor of a path the merge writes is kept whole beside it,
///  and the line names the copy,
///  in a
///  merge that creates a commit and in one that fast-forwards.
/// Why:
///   Git's own update removes whatever stands on the way to the directory an added path needs:
/// measured,
///  an ignored regular file `dir` was gone after a merge that added `dir/x.txt`,
///  with no copy and no notice;
///  a directory and the person's file cannot share one path,
///  so the merge keeps its own content there and the
///  person's entry is kept beside it
///  (`SPEC.md`,
/// the copies a merge keeps beside a path).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const shape of ['fast-forward', 'merge-commit']) expect(merged(repo, shape).stderr).toMatch(/kept whole at/); expect(dirIsADirectory).toBe(true);
/// ```
#[test]
fn an_entry_at_an_ancestor_of_an_added_path_is_kept_whole_in_both_shapes() {
    for (label, fast_forward) in [("fast-forward", true), ("merge-commit", false)] {
        let parent: PathBuf = fixture(format!("merge-ancestor-{label}").as_str());
        let repo: PathBuf = repository(parent.as_path(), "repo");
        std::fs::write(repo.join("a.txt"), b"one\ntwo\nthree\n").expect("file");
        std::fs::write(repo.join(".gitignore"), b"dir\n").expect("file");
        git(repo.as_path(), &["add", "--all"]);
        git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
        git(repo.as_path(), &["switch", "--quiet", "--create", "feature"]);
        std::fs::create_dir(repo.join("dir")).expect("the directory the added path needs");
        std::fs::write(repo.join("dir/x.txt"), b"the branch's own line\n").expect("file");
        git(repo.as_path(), &["add", "--force", "--", "dir/x.txt"]);
        git(repo.as_path(), &["commit", "--quiet", "--message=add"]);
        git(repo.as_path(), &["switch", "--quiet", "main"]);
        // The branch's directory is empty once the branch left the checkout, and the person's ignored FILE takes that name:
        //  a checkout may leave it behind or prune it, so whatever is there goes.
        if repo.join("dir").is_dir() {
            std::fs::remove_dir(repo.join("dir")).expect("the emptied directory");
        }
        if !fast_forward {
            std::fs::write(repo.join("a.txt"), b"one\ntwo\nthree\na commit main has\n").expect("file");
            git(repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
        }
        std::fs::write(repo.join("dir"), b"PERSON-MARKER 7f3a\n").expect("the person's ignored file");
        let outcome: IsolatedMerge = attempted(repo.as_path(), Route::Direct, &["merge", "--no-edit", "feature"], &[]);
        let (code, stderr): (i32, String) = match outcome {
            IsolatedMerge::Finished { code, stderr, .. } => (code, stderr),
            other => panic!("{label}: the merge is the wrapper's to carry, and it was forwarded: {other:?}"),
        };
        assert_eq!(code, 0, "{label}: {stderr}");
        assert!(stderr.contains("kept whole at"), "{label}: the line names the copy: {stderr}");
        assert!(repo.join("dir").is_dir(), "{label}: the merge wrote the directory its path needs");
        assert_eq!(git_text(repo.as_path(), &["ls-files", "--", "dir/x.txt"]), "dir/x.txt", "{label}: the added path is in the index");
        let kept: Vec<String> = std::fs::read_dir(repo.as_path())
            .expect("the worktree")
            .flatten()
            .filter(|entry| { return entry.file_name().to_string_lossy().starts_with(".cli-git-recovery"); })
            .map(|entry| { return std::fs::read_to_string(entry.path()).unwrap_or_default(); })
            .collect();
        assert_eq!(kept.len(), 1, "{label}: one copy waits beside the path, and it holds the person's bytes: {kept:?}");
        assert!(kept[0].contains("PERSON-MARKER 7f3a"), "{label}: {kept:?}");
        let records: usize = std::fs::read_dir(repo.join(".git/monochromatic-git-policy-cli-kept-copies")).map_or(0, Iterator::count);
        assert_eq!(records, 1, "{label}: the copy is recorded, so a later command decides it again");
        remove(parent.as_path());
    }
}
