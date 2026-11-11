//! Binary-level controls for `git commit` through the native wrapper:
//!  the transaction lands a commit,
//! applies the policies,
//! reports what blocked it,
//! leaves nothing behind,
//!  and replays or fails a commit that lost a landing race.
//! Every control runs the built executable on a disposable repository with the host's Git.

use super::support::{
    Fixture, Observed, bounded_for, fixture, git, observe, porcelain, remove, repository, run_direct,
    stderr_of, stderr_without_wait_notices, wrapper_first_path,
};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// The bound of one commit under host load,
/// in seconds;
/// the shared 5-second bound is for single Git commands.
pub(super) const COMMIT_BOUND_SECONDS: &str = "120";

/// Remove the fixture once nothing writes into it any more.
///  A landed commit may leave a maintenance child finishing for a moment under load,
/// and removing a directory that
///  still gains entries fails with "Directory not empty";
/// the removal is tried again for up to five seconds.
pub(super) fn remove_settled(fixture: &Fixture) {
    for _ in 0..50 {
        if std::fs::remove_dir_all(&fixture.root).is_ok() || !fixture.root.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    remove(fixture);
}

/// A command for the fixture's `bin/git` with the wrapper first on PATH and the commit bound.
pub(super) fn wrapped_commit(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Command {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    command.current_dir(directory).args(arguments);
    return command;
}

/// Run one wrapped commit command to its end.
pub(super) fn run_commit(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Observed {
    return observe(&mut wrapped_commit(fixture, directory, arguments), b"");
}

/// The two spellings of literal pathspec mode:
///  the global option and the variable.
#[derive(Clone, Copy, Debug)]
enum LiteralMode {
    /// `git --literal-pathspecs <command>`.
    Option,
    /// `GIT_LITERAL_PATHSPECS=1` in the command's environment.
    Variable,
}

/// Run one wrapped command under one literal pathspec mode.
fn run_under(fixture: &Fixture, directory: &Path, mode: LiteralMode, arguments: &[&str]) -> Observed {
    // `mut` allows the variable to be added to the command before it runs.
    let mut command: Command = match mode {
        LiteralMode::Option => {
            let mut prefixed: Vec<&str> = vec!["--literal-pathspecs"];
            prefixed.extend_from_slice(arguments);
            wrapped_commit(fixture, directory, prefixed.as_slice())
        }
        LiteralMode::Variable => wrapped_commit(fixture, directory, arguments),
    };
    if matches!(mode, LiteralMode::Variable) {
        command.env("GIT_LITERAL_PATHSPECS", "1");
    }
    return observe(&mut command, b"");
}

/// The text of a Git query against the repository.
pub(super) fn git_text(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    let output = git(fixture, repo, arguments);
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// Both streams of an observed command as text,
/// for a failure that says what Git or the wrapper said on either one.
fn stdout_and_stderr(observed: &Observed) -> String {
    return format!("stdout: {}stderr: {}", String::from_utf8_lossy(&observed.stdout), stderr_of(observed));
}

/// A repository with one tracked file whose worktree copy holds an uncommitted edit.
fn edited_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// Make the automatic maintenance a landing starts run to its end before the wrapper returns.
///  Detached,
/// Git 2.56.0 repacks even a repository this small in the background,
/// and its `tmp_pack_*` and `tmp_idx_*`
///  files are what `git count-objects -v` reports as garbage while the repack runs;
/// a check made right after the
///  wrapper returns then races with it.
pub(super) fn keep_maintenance_in_the_foreground(fixture: &Fixture, repo: &Path) {
    git(fixture, repo, &["config", "maintenance.autoDetach", "false"]);
}

/// The sorted file names in a pack directory,
/// for failure messages that name what a garbage count stands for.
fn pack_entries(pack_directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(pack_directory)
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .collect();
        })
        .unwrap_or_default();
    names.sort();
    return names;
}

/// What a finished transaction must leave:
/// no transaction directory,
/// no lock,
/// no shadow repository,
/// no `.keep`.
pub(super) fn assert_nothing_left(fixture: &Fixture, repo: &Path) {
    let git_dir: PathBuf = repo.join(".git");
    let registry: Vec<String> = std::fs::read_dir(git_dir.join("cli-git-transactions"))
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .collect();
        })
        .unwrap_or_default();
    assert_eq!(registry, Vec::<String>::new(), "no transaction or lock stays in the registry");
    assert!(!git_dir.join("index.lock").exists(), "the real index lock is released");
    let shadows: Vec<String> = std::fs::read_dir(git_dir.join("monochromatic-git-policy-cli/shadow"))
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .collect();
        })
        .unwrap_or_default();
    assert_eq!(shadows, Vec::<String>::new(), "no shadow repository stays");
    let pack_directory: PathBuf = git_dir.join("objects/pack");
    let keeps: String = git_text(fixture, repo, &["count-objects", "-v"]);
    assert!(keeps.contains("garbage: 0"), "{keeps}pack directory: {:?}", pack_entries(pack_directory.as_path()));
    let kept: Vec<String> = std::fs::read_dir(pack_directory)
        .map(|entries| {
            return entries
                .flatten()
                .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
                .filter(|name| { return name.ends_with(".keep"); })
                .collect();
        })
        .unwrap_or_default();
    assert_eq!(kept, Vec::<String>::new(), "no pack keeps the commit's objects alive");
}

/// How many small files the maintenance control stages,
/// so that loose objects far exceed `gc.auto`.
const FILLER_FILES: usize = 3000;

/// The automatic maintenance a landing hands to the service runs:
/// with detaching off and the repack threshold at 1,
/// the loose
///  objects of the filler are packed once the service has settled the work records,
/// and nothing of the repack stays behind.
///  The count before the commit is the positive control:
/// the filler is loose then.
///  Git 2.56.0 decides automatic maintenance with its default geometric strategy,
/// whose repack task reads
///  `maintenance.geometric-repack.auto` (6700 objects by default) and not `gc.auto`;
/// `gc.auto` of 1 alone leaves 3000 loose
///  objects alone even when `git maintenance run --auto` is started by hand,
/// so this control sets the threshold Git reads.
#[test]
fn the_maintenance_a_landing_hands_to_the_service_runs_once_the_work_settled() {
    let fixture: Fixture = fixture("commit-service-maintenance");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["config", "gc.auto", "1"]);
    std::fs::create_dir(repo.join("filler")).expect("filler directory");
    for number in 0..FILLER_FILES {
        std::fs::write(repo.join(format!("filler/{number}.txt")), format!("filler {number}\n")).expect("filler file");
    }
    git(&fixture, repo.as_path(), &["add", "--", "filler"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=filler"]);
    let before: String = git_text(&fixture, repo.as_path(), &["count-objects", "-v"]);
    assert!(!before.contains("count: 0\n"), "the filler is loose before the landing: {before}");
    // Set after the filler commit, so that commit's own automatic maintenance leaves the filler loose.
    git(&fixture, repo.as_path(), &["config", "maintenance.geometric-repack.auto", "1"]);
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=second", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    super::support::settle_service_work(repo.as_path());
    let counts: String = git_text(&fixture, repo.as_path(), &["count-objects", "-v"]);
    assert!(counts.contains("count: 0\n"), "the loose objects were packed: {counts}");
    assert!(counts.contains("garbage: 0\n"), "{counts}");
    assert!(!repo.join(".git/objects/maintenance.lock").exists(), "no maintenance still holds its lock");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An explicit-path commit lands the worktree content of the named path once and leaves nothing behind.
#[test]
fn an_explicit_path_commit_lands_once_and_leaves_nothing() {
    let fixture: Fixture = fixture("commit-explicit");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=second", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "3");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "second");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A message that spells the path named before it stays:
/// `git commit a.txt -m a.txt` lands the path `a.txt` with the message `a.txt`,
/// as native Git does,
/// because only a token in option position is removed.
#[test]
fn a_message_equal_to_a_path_stays_and_the_path_is_committed() {
    let fixture: Fixture = fixture("commit-message-equal-path");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let arguments: [&str; 6] = ["-c", "commit.cleanup=strip", "commit", "a.txt", "-m", "a.txt"];
    git(&fixture, repo.as_path(), &arguments);
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
        "a.txt",
        "native Git reads the later token as the message"
    );
    git(&fixture, repo.as_path(), &["reset", "--hard", "HEAD~1"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &arguments);
    assert_eq!(landed.code, Some(0), "{}", stdout_and_stderr(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "a.txt");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A long option is accepted by any unambiguous prefix,
/// so the value of `--mess` is the person's message even when it spells `-o` or `--only`,
/// and `git commit --mess -o -- a.txt` lands the message `-o` as native Git does.
#[test]
fn an_abbreviated_message_keeps_a_value_that_spells_a_flag() {
    let fixture: Fixture = fixture("commit-abbreviated-message");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let arguments: [&str; 7] = ["-c", "commit.cleanup=strip", "commit", "--mess", "-o", "--", "a.txt"];
    git(&fixture, repo.as_path(), &arguments);
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
        "-o",
        "native Git reads the token after the abbreviation as its value"
    );
    git(&fixture, repo.as_path(), &["reset", "--hard", "HEAD~1"]);
    std::fs::write(repo.join("a.txt"), b"one\ntwo\n").expect("edit a");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &arguments);
    assert_eq!(landed.code, Some(0), "{}", stdout_and_stderr(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "-o");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A final-newline correction lands in the commit and in the worktree copy,
/// and the summary names the path.
#[test]
fn a_final_newline_correction_lands_in_the_commit_and_the_worktree() {
    let fixture: Fixture = fixture("commit-newline");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("a.txt"), b"one\ntwo").expect("edit without the final newline");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=second", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("worktree copy"), b"one\ntwo\n");
    assert!(
        stderr_of(&landed).contains("\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\"passes\":1,\"changedPaths\":[\"a.txt\"]"),
        "{}",
        stderr_of(&landed)
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A final-newline correction lands in the commit and in the worktree copy,
/// and the summary names the path,
/// under both spellings of literal pathspec mode:
/// a commit's candidate list is read with pathspec magic whatever mode the caller set.
#[test]
fn a_final_newline_correction_lands_under_literal_pathspec_mode() {
    for (index, mode) in [LiteralMode::Option, LiteralMode::Variable]
        .into_iter()
        .enumerate()
    {
        let fixture: Fixture = fixture(format!("commit-literal-newline-{index}").as_str());
        let repo: PathBuf = edited_repository(&fixture, "repo");
        std::fs::write(repo.join("a.txt"), b"one\ntwo").expect("edit without the final newline");
        let landed: Observed = run_under(
            &fixture,
            repo.as_path(),
            mode,
            &["commit", "--message=second", "--", "a.txt"],
        );
        assert_eq!(landed.code, Some(0), "{mode:?}: {}", stderr_of(&landed));
        assert_eq!(
            git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]),
            "one\ntwo\n",
            "{mode:?}: the commit must hold the corrected bytes"
        );
        assert_eq!(
            std::fs::read(repo.join("a.txt")).expect("worktree copy"),
            b"one\ntwo\n",
            "{mode:?}: the worktree copy must take the corrected bytes"
        );
        assert!(
            stderr_of(&landed).contains(
                "\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\"passes\":1,\"changedPaths\":[\"a.txt\"]"
            ),
            "{mode:?}: {}",
            stderr_of(&landed)
        );
        assert_eq!(porcelain(&fixture, repo.as_path()), "", "{mode:?}");
        remove_settled(&fixture);
    }
}

/// `commit -a` stages every tracked change into the private index under both spellings of literal
/// pathspec mode,
/// and the real index still holds only what was staged before:
/// the private staging is a magic pathspec too.
#[test]
fn a_commit_of_all_tracked_changes_lands_under_literal_pathspec_mode() {
    for (index, mode) in [LiteralMode::Option, LiteralMode::Variable]
        .into_iter()
        .enumerate()
    {
        let fixture: Fixture = fixture(format!("commit-literal-all-{index}").as_str());
        let repo: PathBuf = edited_repository(&fixture, "repo");
        let landed: Observed = run_under(
            &fixture,
            repo.as_path(),
            mode,
            &["commit", "-a", "--no-only", "--message=all"],
        );
        assert_eq!(landed.code, Some(0), "{mode:?}: {}", stderr_of(&landed));
        assert_eq!(
            git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]),
            "one\ntwo\n",
            "{mode:?}: the tracked edit must land"
        );
        assert_eq!(porcelain(&fixture, repo.as_path()), "", "{mode:?}");
        remove_settled(&fixture);
    }
}

/// A staged deletion of a path a literal pathspec does not name is no concern of this commit,
/// under both spellings of literal pathspec mode:
/// `literal*.txt` names one file when the mode reads the star as text.
#[test]
fn a_staged_deletion_outside_a_literal_pathspec_is_not_this_commit() {
    for (index, mode) in [LiteralMode::Option, LiteralMode::Variable]
        .into_iter()
        .enumerate()
    {
        let fixture: Fixture = fixture(format!("commit-literal-deletion-{index}").as_str());
        let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
        std::fs::write(repo.join("literal*.txt"), b"star base\n").expect("star");
        std::fs::write(repo.join("literal-other.txt"), b"other base\n").expect("other");
        git(&fixture, repo.as_path(), &["add", "--", "literal*.txt", "literal-other.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
        keep_maintenance_in_the_foreground(&fixture, repo.as_path());
        git(&fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "literal-other.txt"]);
        std::fs::write(repo.join("literal*.txt"), b"star edit\n").expect("edit the literal star");
        let landed: Observed = run_under(
            &fixture,
            repo.as_path(),
            mode,
            &["commit", "--message=wrapped", "--", "literal*.txt"],
        );
        assert_eq!(landed.code, Some(0), "{mode:?}: {}", stderr_of(&landed));
        assert_eq!(
            git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
            "wrapped",
            "{mode:?}: the commit must land"
        );
        let tree: String = git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "HEAD"]);
        assert!(tree.contains("literal*.txt"), "{mode:?}: {tree}");
        assert_eq!(
            git_text(&fixture, repo.as_path(), &["show", "HEAD:literal-other.txt"]),
            "other base\n",
            "{mode:?}: the unselected path keeps the content the base had"
        );
        assert_eq!(
            git_text(&fixture, repo.as_path(), &["show", "HEAD:literal*.txt"]),
            "star edit\n",
            "{mode:?}: the selected path holds its worktree content"
        );
        assert!(
            porcelain(&fixture, repo.as_path()).contains("D  literal-other.txt"),
            "{mode:?}: the deletion of a path this selection does not name stays staged: {}",
            porcelain(&fixture, repo.as_path())
        );
        assert_nothing_left(&fixture, repo.as_path());
        remove_settled(&fixture);
    }
}

/// The only configuration file the native wrapper reads.
const CONFIG_FILE_NAME: &str = "monochromatic-git-policy-cli.config.jsonc";

/// A secret a commit stages by path is refused before it lands,
/// under both spellings of literal pathspec mode,
/// and the matched text never reaches an event.
#[test]
fn a_forbidden_string_stops_a_commit_under_literal_pathspec_mode() {
    let canary: String = ["LITERAL", "MODE", "CANARY"].join("-");
    for (index, mode) in [LiteralMode::Option, LiteralMode::Variable]
        .into_iter()
        .enumerate()
    {
        let fixture: Fixture = fixture(format!("commit-literal-secret-{index}").as_str());
        let repo: PathBuf = edited_repository(&fixture, "repo");
        std::fs::create_dir(repo.join("rules")).expect("rules directory");
        std::fs::write(repo.join("rules/private.txt"), format!("{canary}\n")).expect("rules");
        std::fs::write(
            repo.join(CONFIG_FILE_NAME),
            "{ \"policies\": { \"security/forbidden-strings\": [\"error\", { \"builtinRules\": false, \"rulesFile\": \"rules/private.txt\" }] } }\n",
        )
        .expect("configuration");
        git(
            &fixture,
            repo.as_path(),
            &[
                "add",
                "--",
                "rules/private.txt",
                CONFIG_FILE_NAME,
            ],
        );
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=config"]);
        std::fs::write(repo.join("leak.txt"), format!("token = {canary}\n")).expect("leak");
        let before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
        let refused: Observed = run_under(
            &fixture,
            repo.as_path(),
            mode,
            &["commit", "--message=leak", "--", "leak.txt"],
        );
        let reported: String = stderr_of(&refused);
        assert_eq!(refused.code, Some(1), "{mode:?}: {reported}");
        assert_eq!(
            git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]),
            before,
            "{mode:?}: nothing may land"
        );
        assert!(
            reported.contains("\"type\":\"finding\""),
            "{mode:?}: {reported}"
        );
        assert!(
            reported.contains("\"policyId\":\"security/forbidden-strings\""),
            "{mode:?}: {reported}"
        );
        assert!(
            reported.contains("\"trigger\":\"pre-forward\""),
            "{mode:?}: {reported}"
        );
        assert!(
            reported.contains("\"path\":\"leak.txt\""),
            "{mode:?}: {reported}"
        );
        assert!(
            !reported.contains(canary.as_str()),
            "{mode:?}: the matched text must not reach an event: {reported}"
        );
        remove_settled(&fixture);
    }
}

/// A dry run commits nothing and starts no transaction.
#[test]
fn a_dry_run_commits_nothing() {
    let fixture: Fixture = fixture("commit-dry-run");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let dry: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--dry-run", "--message=x", "a.txt"]);
    assert_eq!(dry.code, Some(0), "{}", stderr_of(&dry));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert!(!repo.join(".git/monochromatic-git-policy-cli").exists(), "a dry run creates no shadow area");
    remove_settled(&fixture);
}

/// `commit -a` commits every tracked change,
/// including a deletion.
#[test]
fn a_commit_of_all_tracked_changes_lands_them() {
    let fixture: Fixture = fixture("commit-all");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=more"]);
    std::fs::remove_file(repo.join("b.txt")).expect("remove b");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "-a", "--no-only", "--message=all"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "HEAD"]).trim(),
        "a.txt"
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A commit that names no path while commit-only enforcement is active is rejected with exit status 1.
#[test]
fn a_blocking_finding_stops_the_commit_with_status_one() {
    let fixture: Fixture = fixture("commit-blocked");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let blocked: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=x"]);
    assert_eq!(blocked.code, Some(1), "{}", stderr_of(&blocked));
    assert!(stderr_of(&blocked).contains("commit-only/pathspec-required"), "{}", stderr_of(&blocked));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(porcelain(&fixture, repo.as_path()), "M  a.txt\n");
    remove_settled(&fixture);
}

/// A correction that makes the commit identical to its base creates no commit and reconciles the copies.
#[test]
fn a_correction_that_restores_the_base_is_a_no_change_normalization() {
    let fixture: Fixture = fixture("commit-no-change");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("a.txt"), b"one").expect("remove the final newline");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let result: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=x", "a.txt"]);
    assert_eq!(result.code, Some(1), "{}", stderr_of(&result));
    assert!(
        stderr_of(&result).contains("\"code\":\"commit-normalization/no-change\""),
        "{}",
        stderr_of(&result)
    );
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("worktree copy"), b"one\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An amend replaces the head commit,
/// and an empty commit lands only with `--allow-empty`:
/// without the flag both the wrapper and real Git refuse it and nothing lands.
#[test]
fn an_amend_replaces_the_head_and_an_empty_commit_needs_its_flag() {
    let fixture: Fixture = fixture("commit-amend");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let amended: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--amend", "--message=amended", "a.txt"],
    );
    assert_eq!(amended.code, Some(0), "{}", stderr_of(&amended));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "2");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "amended");
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--no-only", "--message=empty"]);
    let refused_directly: Observed = run_direct(&fixture, repo.as_path(), &["commit", "--no-only", "--message=empty"]);
    assert_ne!(refused.code, Some(0), "{}", stderr_of(&refused));
    assert_eq!(refused.code, refused_directly.code, "{}", stderr_of(&refused));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "2");
    let empty: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--allow-empty", "--no-only", "--message=empty"],
    );
    assert_eq!(empty.code, Some(0), "{}", stderr_of(&empty));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "3");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An unknown option reaches Git,
/// which refuses it;
/// nothing lands.
#[test]
fn an_unknown_option_is_refused_by_git_and_nothing_lands() {
    let fixture: Fixture = fixture("commit-unknown-option");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--no-such-option", "a.txt"]);
    let direct: Observed = run_direct(&fixture, repo.as_path(), &["commit", "--no-such-option", "a.txt"]);
    assert_eq!(refused.code, direct.code, "{}", stderr_of(&refused));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    remove_settled(&fixture);
}

/// A termination signal sent to the wrapper alone while the commit sits in the editor ends the commit's Git,
/// leaves no
///  transaction behind,
/// and exits with 128 plus the signal number.
/// The wrapper runs with no `timeout` in front of it:
/// `timeout`
///  forwards a signal to its whole process group,
/// which would end Git and the editor directly and leave the wrapper's own
///  relay untested (`binary_fault_signal_tests.rs`).
#[test]
fn a_termination_signal_during_the_editor_ends_the_commit_and_leaves_nothing() {
    let fixture: Fixture = fixture("commit-signal");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let started: PathBuf = fixture.root.join("editor-started");
    let editor: PathBuf = super::support::stalling_editor(&fixture, started.as_path());
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let mut command: Command = super::fault_run::unbounded_wrapper(&fixture, repo.as_path(), &["commit", "a.txt"]);
    command.env("GIT_EDITOR", &editor);
    let mut running = super::fault_run::spawn_watched(&fixture, &mut command);
    running.wait_for(started.as_path());
    super::fault_run::send(running.id(), libc::SIGTERM);
    let finished = running.finish();
    assert_eq!(finished.status.code(), Some(143), "{}", String::from_utf8_lossy(finished.stderr.as_slice()));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// `git commit --patch` stages the hunks chosen on standard input into the private index,
/// and only those land.
#[test]
fn a_patch_selection_lands_the_chosen_hunk() {
    let fixture: Fixture = fixture("commit-patch");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let landed: Observed = observe(
        &mut wrapped_commit(&fixture, repo.as_path(), &["commit", "--patch", "--message=chosen"]),
        b"y\n",
    );
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "chosen");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// `git commit --interactive` runs the file menu against the private index the same way.
#[test]
fn an_interactive_selection_lands_the_chosen_file() {
    let fixture: Fixture = fixture("commit-interactive");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let landed: Observed = observe(
        &mut wrapped_commit(&fixture, repo.as_path(), &["commit", "--interactive", "--message=chosen"]),
        b"u\n1\n\nq\n",
    );
    assert_eq!(landed.code, Some(0), "{}{}", stderr_of(&landed), String::from_utf8_lossy(landed.stdout.as_slice()));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// With `--allow-empty`,
/// a correction that restores the base still creates the commit the user asked for.
#[test]
fn an_allowed_empty_commit_is_not_a_no_change_normalization() {
    let fixture: Fixture = fixture("commit-allow-empty-normalized");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("a.txt"), b"one").expect("remove the final newline");
    let before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    let landed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--allow-empty", "--message=empty on purpose", "a.txt"],
    );
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "3");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD^{tree}"]), before);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// `--include` commits the named path's worktree content together with what is already staged.
#[test]
fn include_commits_the_named_path_together_with_the_staged_changes() {
    let fixture: Fixture = fixture("commit-include");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--include", "--message=both", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:b.txt"]), "b\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// `--pathspec-from-file` names the committed paths from a file,
/// and `-` reads them from standard input.
#[test]
fn a_pathspec_file_selects_the_committed_paths() {
    let fixture: Fixture = fixture("commit-pathspec-file");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    std::fs::write(repo.join("list.txt"), b"a.txt\n").expect("list");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=listed", "--pathspec-from-file=list.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\n");
    assert!(!git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "HEAD"]).contains("b.txt"));
    std::fs::write(repo.join("a.txt"), b"one\ntwo\nthree\n").expect("edit a again");
    let from_input: Observed = observe(
        &mut wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=piped", "--pathspec-from-file=-"]),
        b"a.txt\n",
    );
    assert_eq!(from_input.code, Some(0), "{}", stderr_of(&from_input));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "one\ntwo\nthree\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(), "piped");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An explicit-path commit cannot record a deletion staged for a path that stays on disk:
/// the finding says so and
/// offers the index-backed form,
/// and that form records the deletion and keeps the file (issue 472).
#[test]
fn an_explicit_path_commit_of_a_staged_deletion_that_stays_on_disk_offers_the_index_form() {
    let fixture: Fixture = fixture("commit-staged-deletion");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=untrack", "a.txt"]);
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    let text: String = stderr_of(&refused);
    assert!(text.contains("commit-only/staged-deletion-worktree-present"), "{text}");
    assert!(text.contains("[\\\"a.txt\\\"]"), "{text}");
    assert!(text.contains("--no-only"), "{text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(porcelain(&fixture, repo.as_path()), "D  a.txt\n?? a.txt\n");
    assert_nothing_left(&fixture, repo.as_path());
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--no-only", "--message=untrack"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "HEAD"]).trim(), "");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("the file stays"), b"one\ntwo\n");
    remove_settled(&fixture);
}

/// A repository with `a.txt` and `b.txt` tracked and an uncommitted edit in each,
/// for the selections that name one path and exclude another.
fn two_edited_files(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"a base\n").expect("a");
    std::fs::write(repo.join("b.txt"), b"b base\n").expect("b");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "b.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"a edit\n").expect("edit a");
    std::fs::write(repo.join("b.txt"), b"b edit\n").expect("edit b");
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// An exclusion subtracts the exact path it names,
/// so `a.txt ':(exclude)a.txt'` is a selection no path matches:
/// nothing lands and both edits stay where the person left them.
#[test]
fn an_exclusion_of_an_exact_path_leaves_nothing_to_commit() {
    let fixture: Fixture = fixture("commit-exclusion-of-exact-path");
    let repo: PathBuf = two_edited_files(&fixture, "repo");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--message=selected", "--", "a.txt", ":(exclude)a.txt"],
    );
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]),
        head,
        "the excluded path may not land: {}",
        stderr_of(&refused)
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), " M a.txt\n M b.txt\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// The exclusion narrows the exact paths and takes no path the person did not name:
/// `a.txt b.txt ':(exclude)b.txt'` lands the edit of `a.txt` and leaves the edit of `b.txt` pending.
#[test]
fn an_exclusion_keeps_the_unselected_exact_path_out_of_the_commit() {
    let fixture: Fixture = fixture("commit-exclusion-narrows");
    let repo: PathBuf = two_edited_files(&fixture, "repo");
    let landed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--message=selected", "--", "a.txt", "b.txt", ":(exclude)b.txt"],
    );
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "a edit\n");
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["show", "HEAD:b.txt"]),
        "b base\n",
        "b.txt is excluded from this selection"
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), " M b.txt\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A `--pathspec-from-file` names the same paths the command line names,
/// so a staged deletion whose file stays on disk gets the same finding:
/// the commit is not made,
/// the file keeps its bytes,
/// and the deletion stays staged.
#[test]
fn a_pathspec_file_of_a_staged_deletion_that_stays_on_disk_gets_the_finding() {
    let fixture: Fixture = fixture("commit-pathspec-file-staged-deletion");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    std::fs::write(repo.join("list.txt"), b"a.txt\n").expect("list");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--message=wrapped", "--pathspec-from-file=list.txt"],
    );
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    let text: String = stderr_of(&refused);
    assert!(text.contains("commit-only/staged-deletion-worktree-present"), "{text}");
    assert!(text.contains("[\\\"a.txt\\\"]"), "{text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing may land");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("the file stays"), b"one\ntwo\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "D  a.txt\n?? a.txt\n?? list.txt\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A pathspec list read from standard input is the same selection as a list file,
/// so the staged deletion it names gets the finding too.
#[test]
fn a_pathspec_list_from_standard_input_of_a_staged_deletion_gets_the_finding() {
    let fixture: Fixture = fixture("commit-pathspec-input-staged-deletion");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "a.txt"]);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: Observed = observe(
        &mut wrapped_commit(
            &fixture,
            repo.as_path(),
            &["commit", "--message=wrapped", "--pathspec-from-file=-"],
        ),
        b"a.txt\n",
    );
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    assert!(
        stderr_of(&refused).contains("commit-only/staged-deletion-worktree-present"),
        "{}",
        stderr_of(&refused)
    );
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head, "nothing may land");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("the file stays"), b"one\ntwo\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "D  a.txt\n?? a.txt\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A staged deletion below an ignored directory gets the same finding,
/// not Git's hint to force-add the ignored path.
#[test]
fn a_staged_deletion_below_an_ignored_directory_is_not_answered_with_a_force_add_hint() {
    let fixture: Fixture = fixture("commit-staged-deletion-ignored");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    std::fs::create_dir(repo.join("tool")).expect("tool directory");
    std::fs::write(repo.join("tool").join("name"), b"local\n").expect("tool file");
    git(&fixture, repo.as_path(), &["add", "--", "tool/name"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join(".gitignore"), b"/tool/\n").expect("ignore rule");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore", "--", ".gitignore"]);
    keep_maintenance_in_the_foreground(&fixture, repo.as_path());
    git(&fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "tool/name"]);
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=untrack", "tool/name"]);
    assert_eq!(refused.code, Some(1), "{}", stderr_of(&refused));
    let text: String = stderr_of(&refused);
    assert!(text.contains("commit-only/staged-deletion-worktree-present"), "{text}");
    assert!(!text.contains("Use -f"), "{text}");
    assert!(!text.contains("ignored by one of your .gitignore files"), "{text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// Files an ignore rule matches but the real index stages are committed by a directory pathspec and by a file
/// pathspec,
/// with the worktree content,
/// as native Git commits them (issue 1807).
#[test]
fn explicit_paths_commit_staged_additions_that_an_ignore_rule_matches() {
    let fixture: Fixture = fixture("commit-staged-ignored");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join(".gitignore"), b"*.js\n").expect("ignore rule");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore", "--", ".gitignore"]);
    std::fs::create_dir(repo.join("lib")).expect("lib");
    std::fs::write(repo.join("lib/_a.js"), b"a\n").expect("_a");
    std::fs::write(repo.join("lib/_b.js"), b"b\n").expect("_b");
    git(&fixture, repo.as_path(), &["add", "--force", "--", "lib/_a.js", "lib/_b.js"]);
    let by_directory: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=lib", "--", "lib"]);
    assert_eq!(by_directory.code, Some(0), "{}", stderr_of(&by_directory));
    assert_eq!(git_text(&fixture, repo.as_path(), &["ls-tree", "--name-only", "-r", "HEAD", "lib"]), "lib/_a.js\nlib/_b.js\n");
    std::fs::write(repo.join("lib/_a.js"), b"a2\n").expect("edit _a");
    git(&fixture, repo.as_path(), &["add", "--force", "--", "lib/_a.js"]);
    std::fs::write(repo.join("lib/_a.js"), b"a3\n").expect("edit _a again");
    let by_file: Observed =
        run_commit(&fixture, repo.as_path(), &["commit", "--message=file", "--", "lib/_a.js"]);
    assert_eq!(by_file.code, Some(0), "{}", stderr_of(&by_file));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:lib/_a.js"]), "a3\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// What:
///  A named file of a worktree the command line selects with `--git-dir` and `--work-tree`,
///  committed from a directory outside that worktree:
///  the ignored one is refused with the named-ignored finding,
///  and the one nothing ignores is committed as an exact path.
/// Why:
///   Git reads a relative pathspec from the top of the selected worktree when the directory the command runs in
/// is outside it,
///  and the invocation directory holds neither file,
/// so a reader that asked it took both for patterns:
/// `git add --update` refused the ignored file with a pathspec failure instead of naming what to do,
///  and refused the file nothing ignores,
/// which an exact path commits.
#[test]
fn a_commit_of_a_selected_worktree_names_a_file_the_invocation_directory_does_not_hold() {
    let fixture: Fixture = fixture("commit-selected-worktree-outside");
    let first: PathBuf = repository(&fixture, OsStr::new("first"));
    let second: PathBuf = repository(&fixture, OsStr::new("second"));
    std::fs::write(second.join(".gitignore"), b"*.js\n").expect("ignore rule");
    git(&fixture, second.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, second.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(second.join("ignored.js"), b"ignored\n").expect("the ignored file only the selected worktree holds");
    std::fs::write(second.join("plain.txt"), b"plain\n").expect("the file nothing ignores only the selected worktree holds");
    let nested: PathBuf = first.join("nested");
    std::fs::create_dir(&nested).expect("nested");
    let git_dir: String = second.join(".git").to_str().expect("a UTF-8 fixture path").to_owned();
    let work_tree: String = second.to_str().expect("a UTF-8 fixture path").to_owned();
    let base: String = git_text(&fixture, second.as_path(), &["rev-parse", "HEAD"]);
    let refused: Observed = run_commit(
        &fixture,
        nested.as_path(),
        &["--git-dir", git_dir.as_str(), "--work-tree", work_tree.as_str(), "commit", "--message=ignored", "--", "ignored.js"],
    );
    let refusal: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{refusal}");
    assert!(refusal.contains("explicit-path-commit/named-path-ignored"), "the ignored file is named as ignored, not failed as a pathspec: {refusal}");
    assert!(refusal.contains("ignored.js"), "{refusal}");
    assert_eq!(git_text(&fixture, second.as_path(), &["rev-parse", "HEAD"]), base, "the refused commit landed nothing");
    let landed: Observed = run_commit(
        &fixture,
        nested.as_path(),
        &["--git-dir", git_dir.as_str(), "--work-tree", work_tree.as_str(), "commit", "--message=plain", "--", "plain.txt"],
    );
    let failure: String = stderr_of(&landed);
    assert_eq!(landed.code, Some(0), "the file nothing ignores is committed as an exact path: {failure}");
    assert_eq!(git_text(&fixture, second.as_path(), &["show", "HEAD:plain.txt"]), "plain\n");
    assert_nothing_left(&fixture, second.as_path());
    remove_settled(&fixture);
}

/// A repository where a force-added ignored file is held as intent-to-add and another tracked file is edited:
/// the shape a partial commit must record with both changes.
fn ignored_intent_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join(".gitignore"), b"ignored.txt\n").expect("ignore rule");
    std::fs::write(repo.join("a.txt"), b"a one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", ".gitignore", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("ignored.txt"), b"content\n").expect("the ignored file");
    git(fixture, repo.as_path(), &["add", "--force", "--intent-to-add", "--", "ignored.txt"]);
    std::fs::write(repo.join("a.txt"), b"a two\n").expect("edit a");
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// A force-added ignored file the real index holds as intent-to-add is an addition the selection names,
/// so an explicit-path commit records it with the worktree content,
/// as native Git commits it.
#[test]
fn an_explicit_path_commit_records_an_ignored_file_held_as_intent_to_add() {
    let fixture: Fixture = fixture("commit-ignored-intent");
    let wrapper: PathBuf = ignored_intent_repository(&fixture, "wrapped");
    let real: PathBuf = ignored_intent_repository(&fixture, "plain");
    let observed: Observed = run_commit(&fixture, wrapper.as_path(), &["commit", "--message=all", "--", "."]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    git(&fixture, real.as_path(), &["commit", "--quiet", "--message=all", "--", "."]);
    assert_eq!(
        git_text(&fixture, wrapper.as_path(), &["ls-tree", "-r", "HEAD"]),
        git_text(&fixture, real.as_path(), &["ls-tree", "-r", "HEAD"]),
        "the commit must hold native Git's tree for the same fixture"
    );
    assert_eq!(git_text(&fixture, wrapper.as_path(), &["show", "HEAD:ignored.txt"]), "content\n");
    assert_nothing_left(&fixture, wrapper.as_path());
    remove_settled(&fixture);
}

/// A repository where a tracked empty file is held as intent-to-add at the root and a file below `dir` is edited.
fn intent_empty_with_subdirectory(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("monochromatic-git-policy-cli.config.jsonc"), b"{ \"policies\": { \"require-root\": \"off\" } }\n").expect("settings");
    std::fs::create_dir(repo.join("dir")).expect("dir");
    std::fs::write(repo.join("empty"), b"").expect("the tracked empty file");
    std::fs::write(repo.join("dir").join("a.txt"), b"a one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "empty", "dir/a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "empty"]);
    git(fixture, repo.as_path(), &["add", "--intent-to-add", "--", "empty"]);
    std::fs::write(repo.join("dir").join("a.txt"), b"a two\n").expect("edit a");
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// A commit run from a subdirectory keeps a tracked empty file the real index holds as intent-to-add with the entry's
/// mode and object name equal to the base's,
/// where native Git leaves that path out of its tree.
/// Why the wrapper keeps it:
/// the file's removal is not the person's work,
/// and the scratch `git commit` builds its tree from the base for every
/// path the command does not name (`SPEC.md`,
/// "Explicit-path commit").
/// The repository turns `require-root` off,
/// because the shipped default refuses every command run below the root.
#[test]
fn an_intent_to_add_entry_equal_to_the_base_stays_in_the_commit_where_native_git_drops_it() {
    let fixture: Fixture = fixture("commit-intent-subdirectory");
    let wrapper: PathBuf = intent_empty_with_subdirectory(&fixture, "wrapped");
    let real: PathBuf = intent_empty_with_subdirectory(&fixture, "plain");
    let observed: Observed = run_commit(&fixture, wrapper.join("dir").as_path(), &["commit", "--message=sub", "--", "a.txt"]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    git(&fixture, real.join("dir").as_path(), &["commit", "--quiet", "--message=sub", "--", "a.txt"]);
    assert_eq!(
        git_text(&fixture, wrapper.as_path(), &["ls-tree", "-r", "HEAD", "empty"]),
        "100644 blob e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\tempty\n",
        "the tracked file stays in the commit"
    );
    assert_eq!(
        git_text(&fixture, real.as_path(), &["ls-tree", "-r", "HEAD", "empty"]),
        "",
        "and native Git leaves it out of the same case, which is the difference this control records"
    );
    assert_eq!(git_text(&fixture, wrapper.as_path(), &["show", "HEAD:dir/a.txt"]), "a two\n");
    assert_nothing_left(&fixture, wrapper.as_path());
    remove_settled(&fixture);
}

/// A tracked empty file the real index holds as intent-to-add,
/// with the entry's mode and object name equal to the base's.
fn intent_empty_repository(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    std::fs::write(repo.join("a.txt"), b"a one\n").expect("a");
    std::fs::write(repo.join("empty"), b"").expect("the tracked empty file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt", "empty"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["rm", "--quiet", "--cached", "--", "empty"]);
    git(fixture, repo.as_path(), &["add", "--intent-to-add", "--", "empty"]);
    std::fs::write(repo.join("a.txt"), b"a two\n").expect("edit a");
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    return repo;
}

/// A commit whose tree native Git builds from the private index keeps that tracked file too.
/// Why this matters:
///  a detached `HEAD` makes the service defer to native `git commit`
///  (`config_needs_native` and `forced` in `service_prepare.rs`),
///  which writes its tree from the private index,
///  where an intent-to-add mark equal to the base's entry leaves that path out:
///  a deletion the person did not make.
#[test]
fn an_intent_to_add_entry_equal_to_the_base_stays_in_a_commit_native_git_builds() {
    let fixture: Fixture = fixture("commit-intent-detached");
    let repo: PathBuf = intent_empty_repository(&fixture, "wrapped");
    git(&fixture, repo.as_path(), &["switch", "--detach", "--quiet"]);
    let observed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=detached", "--", "a.txt"]);
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["ls-tree", "-r", "HEAD", "empty"]),
        "100644 blob e69de29bb2d1d6434b8b29ae775ad8c2e48c5391\tempty\n",
        "the tracked file stays in the commit Git built from the private index"
    );
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "HEAD:a.txt"]), "a two\n", "the named path carries the edit");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An ignored path nothing stages is an input refusal,
/// not an engine failure:
/// exit `1`,
/// a finding that names the path and both ways forward,
/// nothing landed and nothing staged,
/// and the same command lands once the file is force-added (issue 6278).
#[test]
fn an_explicit_path_commit_of_an_unstaged_ignored_path_names_the_way_forward() {
    let fixture: Fixture = fixture("commit-unstaged-ignored");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join(".gitignore"), b"*.js\n").expect("ignore rule");
    git(&fixture, repo.as_path(), &["add", "--", ".gitignore"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=ignore", "--", ".gitignore"]);
    std::fs::write(repo.join("c.js"), b"c\n").expect("c");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let staged_before: String = git_text(&fixture, repo.as_path(), &["diff", "--cached", "--name-status"]);
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=c", "--", "a.txt", "c.js"]);
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("\"type\":\"core-finding\""), "{text}");
    assert!(text.contains("explicit-path-commit/named-path-ignored"), "{text}");
    assert!(text.contains("[\\\"c.js\\\"]"), "{text}");
    assert!(text.contains("this commit was not made and nothing was staged by it"), "{text}");
    assert!(!text.contains("engine-failure"), "{text}");
    assert!(!text.contains("transaction-failed"), "{text}");
    assert!(!text.contains("git log --oneline -3"), "the transaction-failure hint: {text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "the named tracked path did not land without the ignored one");
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["diff", "--cached", "--name-status"]),
        staged_before,
        "the refusal staged nothing"
    );
    assert_nothing_left(&fixture, repo.as_path());
    git(&fixture, repo.as_path(), &["add", "--force", "--", "c.js"]);
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=c", "--", "a.txt", "c.js"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "--name-only", "--format=", "HEAD"]), "a.txt\nc.js\n");
    remove_settled(&fixture);
}

/// The bytes of a repository's real index,
///  so a refusal can prove the index it never wrote is what it was.
fn index_bytes(repo: &Path) -> Vec<u8> {
    return std::fs::read(repo.join(".git").join("index")).expect("the real index");
}

/// An option written after `--` is a pathspec to Git,
/// so the commit is refused as the caller's input error:
/// exit `1`,
/// a finding naming the token,
/// and the remedy that options belong before the separator (issue 6283).
#[test]
fn an_option_written_after_the_separator_is_refused_as_a_pathspec_that_matches_nothing() {
    let fixture: Fixture = fixture("commit-option-after-separator");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let status_before: String = porcelain(&fixture, repo.as_path());
    let staged_before: String = git_text(&fixture, repo.as_path(), &["diff", "--cached", "--name-status"]);
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--", "a.txt", "--message=second"]);
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("\"type\":\"core-finding\""), "{text}");
    assert!(text.contains("explicit-path-commit/pathspec-matches-nothing"), "{text}");
    assert!(
        text.contains("pathspec '--message=second' did not match any files"),
        "the token the caller wrote, named as native Git names it: {text}"
    );
    assert!(
        text.contains("git commit --message <text> -- <paths>"),
        "the remedy for an option written after the separator: {text}"
    );
    assert!(!text.contains("engine-failure"), "{text}");
    assert!(!text.contains("git add --update"), "the report names no step of the wrapper: {text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "nothing landed: {text}");
    assert_eq!(porcelain(&fixture, repo.as_path()), status_before, "nothing was staged: {text}");
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["diff", "--cached", "--name-status"]),
        staged_before,
        "the staged state the caller had is the one left: {text}"
    );
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// Every one of the twenty stopped records of 2026-11-10 wrote the message after the paths,
/// with the path still in the selection and a word of the message as an element of its own
/// (`doc/planning/cli-git-commits-that-stopped-after-their-add-steps.md`),
/// so this is that shape:
/// exit `1`,
/// a finding naming the token Git refused,
/// the remedy that options belong before `--`,
/// and neither the path nor the message staged (issue 6283).
#[test]
fn a_message_written_after_the_paths_is_refused_as_the_pathspec_it_became() {
    let fixture: Fixture = fixture("commit-message-after-paths");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let status_before: String = porcelain(&fixture, repo.as_path());
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--", "a.txt", "-m", "one two"]);
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("\"type\":\"core-finding\""), "{text}");
    assert!(text.contains("explicit-path-commit/pathspec-matches-nothing"), "{text}");
    assert!(text.contains("pathspec '-m' did not match any files"), "the token Git refused: {text}");
    assert!(text.contains("git commit --message <text> -- <paths>"), "the remedy: {text}");
    assert!(!text.contains("engine-failure"), "{text}");
    assert!(!text.contains("git add --update"), "the report names no step of the wrapper: {text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "nothing landed: {text}");
    assert_eq!(porcelain(&fixture, repo.as_path()), status_before, "nothing was staged: {text}");
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A plain misspelled path is the same refusal,
/// with no advice about the separator,
/// because nothing in it starts with a hyphen.
#[test]
fn a_misspelled_path_is_refused_as_a_pathspec_that_matches_nothing() {
    let fixture: Fixture = fixture("commit-misspelled-path");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=misspelled", "--", "nope.txt"]);
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("\"type\":\"core-finding\""), "{text}");
    assert!(text.contains("explicit-path-commit/pathspec-matches-nothing"), "{text}");
    assert!(text.contains("pathspec 'nope.txt' did not match any files"), "{text}");
    assert!(!text.contains("options belong before"), "nothing here starts with a hyphen: {text}");
    assert!(!text.contains("engine-failure"), "{text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "nothing landed: {text}");
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A magic pathspec Git answers for itself stays Git's answer:
/// the wrapper predicts no match of its own,
/// so a glob selector nothing matches is the same refusal,
/// naming the selector exactly as the caller wrote it.
#[test]
fn a_magic_pathspec_that_matches_nothing_is_refused_as_the_same_input_error() {
    let fixture: Fixture = fixture("commit-magic-matches-nothing");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=magic", "--", ":(glob)nothing/*"]);
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("explicit-path-commit/pathspec-matches-nothing"), "{text}");
    assert!(text.contains("pathspec ':(glob)nothing/*' did not match any files"), "{text}");
    assert!(!text.contains("options belong before"), "the selector does not start with a hyphen: {text}");
    assert!(!text.contains("engine-failure"), "{text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "nothing landed: {text}");
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// One good path and one that matches nothing lands nothing at all:
/// the good path is staged into the private index before the bad one is refused,
/// so this is the control that the refusal leaves the real index alone.
#[test]
fn one_path_that_matches_nothing_withholds_the_whole_commit() {
    let fixture: Fixture = fixture("commit-mixed-paths");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let status_before: String = porcelain(&fixture, repo.as_path());
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=mix", "--", "a.txt", "nope.txt"]);
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("pathspec 'nope.txt' did not match any files"), "{text}");
    assert!(!text.contains("engine-failure"), "{text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "nothing landed: {text}");
    assert_eq!(porcelain(&fixture, repo.as_path()), status_before, "nothing was staged: {text}");
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// The most bytes a pathspec file may hold and still be read as elements;
/// a file above it is passed to Git as the file the caller named
/// (`ITEMS_BYTES_LIMIT` in `commit_selection_staging.rs`).
const LARGEST_READ_PATHSPEC_FILE_BYTES: usize = 64 * 1024;

/// A list file whose one matching path is repeated past that bound,
/// and whose last path matches nothing.
fn list_past_the_reader_bound(missing: &str) -> String {
    let entry: &str = "a.txt\n";
    let repetitions: usize = LARGEST_READ_PATHSPEC_FILE_BYTES / entry.len();
    return format!("{}{missing}\n", entry.repeat(repetitions));
}

/// A pathspec file with more bytes than one command line may carry is Git's to read,
/// and a token in it that matches nothing is the same refusal the command line gets:
/// exit `1`,
/// the finding,
/// and the token Git named,
/// instead of an engine failure that names a step of the wrapper (issue 6283).
#[test]
fn a_pathspec_file_too_large_for_one_command_line_is_refused_as_a_pathspec_that_matches_nothing() {
    let fixture: Fixture = fixture("commit-large-pathspec-file");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let listed: String = list_past_the_reader_bound("nope.txt");
    assert!(
        listed.len() > LARGEST_READ_PATHSPEC_FILE_BYTES,
        "the list file is past the bound the wrapper reads elements within"
    );
    let list: PathBuf = fixture.root.join("pathspecs.txt");
    std::fs::write(list.as_path(), listed.as_bytes()).expect("the list file");
    let option: String = format!("--pathspec-from-file={}", list.display());
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=listed", option.as_str()]);
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("\"type\":\"core-finding\""), "{text}");
    assert!(text.contains("explicit-path-commit/pathspec-matches-nothing"), "{text}");
    assert!(text.contains("pathspec 'nope.txt' did not match any files"), "the token Git refused: {text}");
    assert!(!text.contains("engine-failure"), "{text}");
    assert!(!text.contains("transaction-failed"), "{text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "nothing landed: {text}");
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// Git words that refusal in the language of the locale it inherits,
/// and the wrapper reads those words,
/// so the staging start that is read happens under the C locale:
/// a German locale still ends as the finding rather than an engine failure.
#[test]
fn a_pathspec_refusal_is_understood_whatever_the_persons_locale() {
    let fixture: Fixture = fixture("commit-pathspec-german-locale");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let mut command: Command = wrapped_commit(&fixture, repo.as_path(), &["commit", "--message=miss", "--", "nope.txt"]);
    // The three variables a person's shell carries, so Git translates its words unless the wrapper's own start says not to.
    command.env("LANG", "de_DE.utf8").env("LC_ALL", "de_DE.utf8").env("LANGUAGE", "de_DE");
    let refused: Observed = observe(&mut command, b"");
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("explicit-path-commit/pathspec-matches-nothing"), "{text}");
    assert!(text.contains("pathspec 'nope.txt' did not match any files"), "the token Git refused: {text}");
    assert!(!text.contains("engine-failure"), "{text}");
    assert!(!text.contains("Pfadspezifikation"), "the wrapper's start reads English words only: {text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "nothing landed: {text}");
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A pathspec that itself holds the wording Git closes a refusal with is one pathspec,
/// so Git's refusal of it is the caller's input error naming that whole token:
/// exit `1`,
/// the finding,
/// and no engine failure (issue 6283).
#[test]
fn a_pathspec_that_holds_the_refusal_wording_is_refused_as_the_one_pathspec_it_is() {
    let fixture: Fixture = fixture("commit-pathspec-holds-the-wording");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let head_before: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let token: &str = "x' did not match any files";
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=miss", "--", token]);
    let text: String = stderr_of(&refused);
    assert_eq!(refused.code, Some(1), "{text}");
    assert!(text.contains("\"type\":\"core-finding\""), "{text}");
    assert!(text.contains("explicit-path-commit/pathspec-matches-nothing"), "{text}");
    assert!(
        text.contains("pathspec 'x' did not match any files' did not match any files"),
        "the token Git refused, read whole: {text}"
    );
    assert!(!text.contains("engine-failure"), "{text}");
    assert!(!text.contains("transaction-failed"), "{text}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head_before, "nothing landed: {text}");
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// Another failure of the same staging start is not a pathspec that matches nothing,
/// so it keeps the engine-failure report and exit `2` it had before issue 6283:
/// here Git could not read the content its pathspec matched.
#[test]
fn a_staging_failure_that_is_not_a_pathspec_matching_nothing_stays_an_engine_failure() {
    use std::os::unix::fs::PermissionsExt;
    let fixture: Fixture = fixture("commit-unreadable-match");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    let directory: PathBuf = repo.join("dir");
    std::fs::create_dir(directory.as_path()).expect("dir");
    let unreadable: PathBuf = directory.join("x.txt");
    std::fs::write(unreadable.as_path(), b"x\n").expect("x");
    git(&fixture, repo.as_path(), &["add", "--", "dir/x.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=dir", "--", "dir/x.txt"]);
    std::fs::write(unreadable.as_path(), b"x\ny\n").expect("edit x");
    let permitted = |mode: u32| {
        let mut permissions = std::fs::metadata(unreadable.as_path()).expect("the file").permissions();
        permissions.set_mode(mode);
        std::fs::set_permissions(unreadable.as_path(), permissions).expect("the file's mode");
    };
    permitted(0o000);
    let index_before: Vec<u8> = index_bytes(repo.as_path());
    let refused: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--message=perm", "--", "dir"]);
    let text: String = stderr_of(&refused);
    // The mode is readable again before the fixture is removed, so no cleanup depends on it.
    permitted(0o644);
    assert_eq!(refused.code, Some(2), "{text}");
    assert!(text.contains("\"type\":\"engine-failure\""), "{text}");
    assert!(text.contains("\"code\":\"transaction-failed\""), "{text}");
    assert!(text.contains("Permission denied"), "Git's own words for the failure: {text}");
    assert!(!text.contains("pathspec-matches-nothing"), "{text}");
    assert_eq!(index_bytes(repo.as_path()), index_before, "the real index is byte for byte what it was: {text}");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A repository nothing was staged in has no index file;
/// an explicit-path commit there reads it as empty instead of
///  failing on the missing file.
#[test]
fn an_explicit_path_commit_works_before_any_index_exists() {
    let fixture: Fixture = fixture("commit-no-index");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    // Git writes an index file whenever it commits; a repository nothing was staged in has none, so remove it.
    std::fs::remove_file(repo.join(".git/index")).expect("the index Git wrote for the initial commit");
    assert!(!repo.join(".git/index").exists(), "the premise: no index file yet");
    std::fs::write(repo.join("first.txt"), b"one\n").expect("first file");
    let observed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=first", "--", "first.txt"],
    );
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(
        git_text(&fixture, &repo, &["show", "--name-only", "--format=", "HEAD"]),
        "first.txt\n"
    );
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    remove_settled(&fixture);
}

/// An explicit-path commit leaves every other staged entry (a rename,
/// a new file) exactly as it was and says nothing
///  about staged entries moving.
#[test]
fn an_explicit_path_commit_keeps_other_staged_entries_without_a_warning() {
    let fixture: Fixture = fixture("commit-keeps-staged");
    let repo: PathBuf = edited_repository(&fixture, "repo");
    std::fs::write(repo.join("b.txt"), b"base\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=b", "--", "b.txt"]);
    git(&fixture, repo.as_path(), &["mv", "b.txt", "moved.txt"]);
    std::fs::write(repo.join("c.txt"), b"new\n").expect("c");
    git(&fixture, repo.as_path(), &["add", "--", "c.txt"]);
    let staged_before: String = git_text(&fixture, &repo, &["diff", "--cached", "--name-status"]);
    let observed: Observed = run_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=edit a", "--", "a.txt"],
    );
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert!(
        !stderr_of(&observed).contains("outside the paths it committed"),
        "{}",
        stderr_of(&observed)
    );
    assert_eq!(
        git_text(&fixture, &repo, &["diff", "--cached", "--name-status"]),
        staged_before
    );
    remove_settled(&fixture);
}

/// The paths the head commit holds,
/// one per line,
/// as native Git lists them.
fn head_paths(fixture: &Fixture, repo: &Path) -> String {
    return git_text(fixture, repo, &["show", "--name-only", "--format=", "HEAD"]);
}

/// Two repositories holding the same shapes:
/// a tracked file edited,
/// a tracked file below a directory edited,
/// one addition staged,
/// and a file nothing staged beside each of them.
fn twins_with_unstaged_neighbours(fixture: &Fixture, name: &str) -> (PathBuf, PathBuf) {
    let wrapped: PathBuf = repository(fixture, OsStr::new(format!("{name}-wrapped").as_str()));
    let native: PathBuf = repository(fixture, OsStr::new(format!("{name}-native").as_str()));
    for repo in [wrapped.clone(), native.clone()] {
        std::fs::write(repo.join("a.txt"), b"alpha\n").expect("a");
        std::fs::create_dir(repo.join("dir")).expect("dir");
        std::fs::write(repo.join("dir/c.txt"), b"c1\n").expect("c");
        git(fixture, repo.as_path(), &["add", "--", "a.txt", "dir/c.txt"]);
        git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
        std::fs::write(repo.join("a.txt"), b"alpha edited\n").expect("edit a");
        std::fs::write(repo.join("dir/c.txt"), b"c2\n").expect("edit c");
        std::fs::write(repo.join("dir/staged.txt"), b"staged\n").expect("staged");
        git(fixture, repo.as_path(), &["add", "--", "dir/staged.txt"]);
        std::fs::write(repo.join("scratch.txt"), b"scratch\n").expect("scratch");
        std::fs::write(repo.join("dir/never-added.txt"), b"never\n").expect("never");
        keep_maintenance_in_the_foreground(fixture, repo.as_path());
    }
    return (wrapped, native);
}

/// What:
///  A directory,
/// the whole worktree,
/// and a wildcard pathspec commit what native Git commits:
/// tracked paths and staged additions,
/// and no file nothing staged.
/// Why:
///   The private index was staged with `git add --all`,
/// which swept in every untracked file the pathspec matched,
/// so a commit of `.` published a scratch file nobody staged
/// (owner:
/// "never discard user data").
///  The same argv runs in a twin repository through native Git,
/// and the two commits,
/// statuses
///  and exit codes are compared.
#[test]
fn a_pathspec_that_is_a_directory_or_a_pattern_commits_no_file_nothing_staged() {
    let fixture: Fixture = fixture("commit-pathspec-untracked");
    for (number, pathspec) in [".", "dir", "*"].iter().enumerate() {
        let name: String = format!("pathspec-untracked-{number}");
        let (wrapped, native): (PathBuf, PathBuf) = twins_with_unstaged_neighbours(&fixture, name.as_str());
        let arguments: [&str; 5] = ["commit", "--quiet", "--message=x", "--", pathspec];
        let wrapped_ran: Observed = run_commit(&fixture, wrapped.as_path(), &arguments);
        let native_ran: Observed = run_direct(&fixture, native.as_path(), &arguments);
        assert_eq!(wrapped_ran.code, native_ran.code, "pathspec {pathspec}: {}", stderr_of(&wrapped_ran));
        assert_eq!(head_paths(&fixture, wrapped.as_path()), head_paths(&fixture, native.as_path()), "pathspec {pathspec}: the commit holds what native Git commits");
        assert_eq!(
            porcelain(&fixture, wrapped.as_path()),
            porcelain(&fixture, native.as_path()),
            "pathspec {pathspec}: the worktree is left as native Git leaves it"
        );
        assert!(porcelain(&fixture, wrapped.as_path()).contains("?? scratch.txt"), "pathspec {pathspec}: {}", porcelain(&fixture, wrapped.as_path()));
    }
    remove_settled(&fixture);
}

/// What:
///  A pathspec mode given as a global option selects the paths native Git selects:
/// `--noglob-pathspecs` makes `*.txt` literal (nothing matches,
/// exit 1),
/// makes `st*r.txt` literal (the file of that name only),
/// and `--icase-pathspecs` matches a staged `A.TXT` through `a.txt`.
/// Why:
///   The private staging,
/// the staged-addition listing and the carried check runs dropped the caller's global options
///  (`GitRequest::without_prefix`),
/// so a mode written as an option did not reach the pathspecs the caller wrote,
/// where the same mode as a variable did.
///  Every case runs in a twin repository through native Git and compares exit codes,
///  commits and statuses.
#[test]
fn a_pathspec_mode_given_as_a_global_option_selects_what_native_git_selects() {
    let fixture: Fixture = fixture("commit-pathspec-mode");
    // `*.txt` under noglob matches no file, so native Git refuses the commit and changes nothing.
    {
        let (wrapped, native): (PathBuf, PathBuf) = twins_with_unstaged_neighbours(&fixture, "mode-noglob-none");
        let arguments: [&str; 6] = ["--noglob-pathspecs", "commit", "--quiet", "--message=x", "--", "*.txt"];
        let wrapped_ran: Observed = run_commit(&fixture, wrapped.as_path(), &arguments);
        let native_ran: Observed = run_direct(&fixture, native.as_path(), &arguments);
        // The refusal is the caller's input, exit status 1 like native Git's own refusal of the same pathspec, and the
        // finding names the token it wrote (`README.md` "Output and exit status", issue 6283).
        assert_eq!(wrapped_ran.code, Some(1), "noglob-none: {}", stderr_of(&wrapped_ran));
        assert_eq!(native_ran.code, Some(1), "the control: native Git refuses a literal pathspec that matches nothing");
        assert!(stderr_of(&wrapped_ran).contains("pathspec '*.txt' did not match any files"), "noglob-none: {}", stderr_of(&wrapped_ran));
        assert_eq!(head_paths(&fixture, wrapped.as_path()), head_paths(&fixture, native.as_path()), "noglob-none: nothing lands");
        assert_eq!(porcelain(&fixture, wrapped.as_path()), porcelain(&fixture, native.as_path()), "noglob-none");
    }
    // `st*r.txt` under noglob names the file of that name, not every sibling its glob would match.
    {
        let (wrapped, native): (PathBuf, PathBuf) = (repository(&fixture, OsStr::new("mode-star-wrapped")), repository(&fixture, OsStr::new("mode-star-native")));
        for repo in [wrapped.clone(), native.clone()] {
            std::fs::write(repo.join("st*r.txt"), b"star\n").expect("the literal name");
            std::fs::write(repo.join("stxr.txt"), b"sibling\n").expect("the sibling");
            git(&fixture, repo.as_path(), &["--noglob-pathspecs", "add", "--", "st*r.txt"]);
            keep_maintenance_in_the_foreground(&fixture, repo.as_path());
        }
        let arguments: [&str; 6] = ["--noglob-pathspecs", "commit", "--quiet", "--message=x", "--", "st*r.txt"];
        let wrapped_ran: Observed = run_commit(&fixture, wrapped.as_path(), &arguments);
        let native_ran: Observed = run_direct(&fixture, native.as_path(), &arguments);
        assert_eq!(wrapped_ran.code, native_ran.code, "noglob-star: {}", stderr_of(&wrapped_ran));
        assert_eq!(head_paths(&fixture, wrapped.as_path()), head_paths(&fixture, native.as_path()), "noglob-star: the commit holds what native Git commits");
        assert!(porcelain(&fixture, wrapped.as_path()).contains("?? stxr.txt"), "noglob-star: {}", porcelain(&fixture, wrapped.as_path()));
    }
    // `a.txt` under icase names the staged `A.TXT`.
    {
        let (wrapped, native): (PathBuf, PathBuf) = (repository(&fixture, OsStr::new("mode-icase-wrapped")), repository(&fixture, OsStr::new("mode-icase-native")));
        for repo in [wrapped.clone(), native.clone()] {
            std::fs::write(repo.join("A.TXT"), b"upper\n").expect("the upper-case name");
            git(&fixture, repo.as_path(), &["add", "--", "A.TXT"]);
            keep_maintenance_in_the_foreground(&fixture, repo.as_path());
        }
        let arguments: [&str; 6] = ["--icase-pathspecs", "commit", "--quiet", "--message=x", "--", "a.txt"];
        let wrapped_ran: Observed = run_commit(&fixture, wrapped.as_path(), &arguments);
        let native_ran: Observed = run_direct(&fixture, native.as_path(), &arguments);
        assert_eq!(wrapped_ran.code, native_ran.code, "icase: {}", stderr_of(&wrapped_ran));
        assert_eq!(native_ran.code, Some(0), "the control: native Git commits the staged file the icase pathspec names");
        assert_eq!(head_paths(&fixture, wrapped.as_path()), head_paths(&fixture, native.as_path()), "icase: the commit holds what native Git commits");
        assert_eq!(porcelain(&fixture, wrapped.as_path()), porcelain(&fixture, native.as_path()), "icase");
    }
    remove_settled(&fixture);
}

/// The reflog subjects of the head's last `count` entries,
/// newest first.
fn reflog_subjects(fixture: &Fixture, repo: &Path, count: &str) -> String {
    return git_text(fixture, repo, &["reflog", "show", "--format=%gs", format!("--max-count={count}").as_str(), "HEAD"]);
}

/// What:
///  A commit landed by the service writes the reflog message native Git writes for the same commit:
/// `commit: <subject>` for an ordinary commit,
/// `commit (amend): <subject>` for an amend,
/// and `commit (initial): <subject>` for the first commit of a branch.
/// Why:
///   The landing moved the branch with `update-ref --stdin` and no message,
/// so every row of `git reflog`,
///  the recovery tool a lost commit is found with,
/// read blank.
///  The same sequence runs in a twin repository through native Git and the subjects are compared.
#[test]
fn a_landed_commit_writes_the_reflog_message_native_git_writes() {
    let fixture: Fixture = fixture("commit-reflog");
    let wrapped: PathBuf = repository(&fixture, OsStr::new("reflog-wrapped"));
    let native: PathBuf = repository(&fixture, OsStr::new("reflog-native"));
    for repo in [wrapped.clone(), native.clone()] {
        std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
        git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
        keep_maintenance_in_the_foreground(&fixture, repo.as_path());
    }
    let plain: [&str; 5] = ["commit", "--quiet", "--message=plain", "--", "a.txt"];
    assert_eq!(run_commit(&fixture, wrapped.as_path(), &plain).code, Some(0));
    assert_eq!(run_direct(&fixture, native.as_path(), &plain).code, Some(0));
    std::fs::write(wrapped.join("a.txt"), b"one\ntwo\n").expect("edit wrapped");
    std::fs::write(native.join("a.txt"), b"one\ntwo\n").expect("edit native");
    let amend: [&str; 5] = ["commit", "--quiet", "--amend", "--message=amended", "a.txt"];
    let amended: Observed = run_commit(&fixture, wrapped.as_path(), &amend);
    assert_eq!(amended.code, Some(0), "{}", stderr_of(&amended));
    assert_eq!(run_direct(&fixture, native.as_path(), &amend).code, Some(0));
    assert_eq!(reflog_subjects(&fixture, wrapped.as_path(), "1"), reflog_subjects(&fixture, native.as_path(), "1"), "the amend's row");
    assert_eq!(
        reflog_subjects(&fixture, wrapped.as_path(), "1").as_str(),
        "commit (amend): amended\n",
        "the amend's wording"
    );
    // The first commit of a branch is `commit (initial)`.
    let unborn_wrapped: PathBuf = fixture.root.join("unborn-wrapped");
    let unborn_native: PathBuf = fixture.root.join("unborn-native");
    for repo in [unborn_wrapped.clone(), unborn_native.clone()] {
        std::fs::create_dir(repo.as_path()).expect("unborn repository");
        git(&fixture, repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
        std::fs::write(repo.join("first.txt"), b"first\n").expect("first");
        // The file is staged, so the commit names a path both sides know: an untracked exact path is the wrapper's own
        // deliberate difference in the other direction.
        git(&fixture, repo.as_path(), &["add", "--", "first.txt"]);
        keep_maintenance_in_the_foreground(&fixture, repo.as_path());
    }
    let initial: [&str; 5] = ["commit", "--quiet", "--message=first", "--", "first.txt"];
    let landed: Observed = run_commit(&fixture, unborn_wrapped.as_path(), &initial);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(run_direct(&fixture, unborn_native.as_path(), &initial).code, Some(0));
    assert_eq!(reflog_subjects(&fixture, unborn_wrapped.as_path(), "1"), reflog_subjects(&fixture, unborn_native.as_path(), "1"), "the initial row");
    assert_eq!(
        reflog_subjects(&fixture, unborn_wrapped.as_path(), "1").as_str(),
        "commit (initial): first\n",
        "the initial commit's wording"
    );
    remove_settled(&fixture);
}

/// What:
///  A commit that concludes a stopped merge writes the reflog row native Git writes for it,
/// `commit (merge): <first line>`.
/// Why:
///   The merge's state is another kind the landing has to tell apart,
/// and its row is the one a person reads to find the merge commit after a `reset`.
///  The same sequence runs in a twin repository through native Git and the subjects are compared.
#[test]
fn a_merge_conclusion_writes_the_reflog_message_native_git_writes() {
    let fixture: Fixture = fixture("commit-reflog-merge");
    let wrapped: PathBuf = repository(&fixture, OsStr::new("reflog-merge-wrapped"));
    let native: PathBuf = repository(&fixture, OsStr::new("reflog-merge-native"));
    for repo in [wrapped.clone(), native.clone()] {
        std::fs::write(repo.join("f.txt"), b"a\nb\n").expect("f");
        git(&fixture, repo.as_path(), &["add", "--", "f.txt"]);
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base", "--", "f.txt"]);
        git(&fixture, repo.as_path(), &["branch", "side"]);
        std::fs::write(repo.join("f.txt"), b"a\nMAIN\n").expect("main edit");
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
        git(&fixture, repo.as_path(), &["checkout", "--quiet", "side"]);
        std::fs::write(repo.join("f.txt"), b"a\nSIDE\n").expect("side edit");
        git(&fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
        git(&fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
        let merged: Observed = run_direct(&fixture, repo.as_path(), &["merge", "--quiet", "side"]);
        assert_eq!(merged.code, Some(1), "the control: the merge stops on the conflict");
        std::fs::write(repo.join("f.txt"), b"a\nRESOLVED\n").expect("resolution");
        git(&fixture, repo.as_path(), &["add", "--", "f.txt"]);
        keep_maintenance_in_the_foreground(&fixture, repo.as_path());
    }
    let concluded: Observed = run_commit(&fixture, wrapped.as_path(), &["commit", "--no-only", "--message=merged"]);
    assert_eq!(concluded.code, Some(0), "{}", stderr_of(&concluded));
    assert_eq!(run_direct(&fixture, native.as_path(), &["commit", "--quiet", "--message=merged"]).code, Some(0));
    assert_eq!(reflog_subjects(&fixture, wrapped.as_path(), "1"), reflog_subjects(&fixture, native.as_path(), "1"), "the merge row");
    assert_eq!(reflog_subjects(&fixture, wrapped.as_path(), "1").as_str(), "commit (merge): merged\n", "the merge's wording");
    remove_settled(&fixture);
}

/// What:
///  A commit Git itself builds lands in a reftable repository whose own directory name is not UTF-8.
/// Why:
///   A reftable shadow is the one shadow Git itself creates with `init`,
///  and the path handed to that `init` was a spelling of the shadow's path:
///  Git initialized a directory under the replacement character,
///  the creation then protected a path that did not exist,
///  and the commit stopped instead of landing.
///  A message file that is not text is Git's to read,
/// so Git builds this commit in the shadow repository.
#[test]
fn a_commit_git_builds_lands_in_a_reftable_repository_whose_directory_is_not_utf8() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let fixture: Fixture = fixture("commit-reftable-not-utf8-directory");
    let repo: PathBuf = fixture.root.join(OsString::from_vec(b"reftable-\xef".to_vec()));
    std::fs::create_dir(repo.as_path()).expect("repository directory");
    git(&fixture, repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    keep_maintenance_in_the_foreground(&fixture, repo.as_path());
    let base: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    // A message file that is not text is Git's to read, so Git builds this commit in the shadow repository, and Git's own
    //  warning about the encoding is the only thing on standard error.
    std::fs::write(repo.join("message.txt"), b"one \xe9\n").expect("message file");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--file=message.txt", "--", "a.txt"]);
    assert_eq!(landed.code, Some(0), "the commit lands: {}", stderr_of(&landed));
    let said: String = stderr_without_wait_notices(&landed);
    assert!(!said.contains("failed:"), "no step of the shadow creation failed: {said}");
    assert!(!said.contains("monochromatic-git-policy-cli:"), "the wrapper adds no words of its own: {said}");
    assert_ne!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), base, "the branch moved to the new commit");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// What:
///  A commit lands in a worktree whose Git directory name ends with a space,
/// selected with `--git-dir`.
/// Why:
///   The route reads the Git directory from Git's answer,
/// and it removed a trailing byte that belongs to the name:
///  it resolved a directory that does not exist and reached no service,
///  so the commit stopped instead of landing in the repository the person selected.
#[test]
fn a_commit_lands_in_a_worktree_whose_git_directory_name_ends_with_a_space() {
    use std::ffi::OsString;
    let fixture: Fixture = fixture("commit-git-dir-trailing-space");
    let worktree: PathBuf = fixture.root.join("worktree");
    std::fs::create_dir(worktree.as_path()).expect("worktree directory");
    let metadata: PathBuf = fixture.root.join("metadata ");
    git(
        &fixture,
        fixture.root.as_path(),
        &[
            "init",
            "--quiet",
            "--initial-branch=main",
            "--separate-git-dir",
            metadata.to_str().expect("UTF-8 fixture name"),
            worktree.to_str().expect("UTF-8 worktree"),
        ],
    );
    git(&fixture, worktree.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    std::fs::write(worktree.join("a.txt"), b"one\n").expect("a");
    git(&fixture, worktree.as_path(), &["add", "--", "a.txt"]);
    keep_maintenance_in_the_foreground(&fixture, worktree.as_path());
    let base: String = git_text(&fixture, worktree.as_path(), &["rev-parse", "HEAD"]);
    let arguments: Vec<OsString> = vec![
        OsString::from(format!("--git-dir={}", metadata.display())),
        OsString::from("commit"),
        OsString::from("--message=one"),
        OsString::from("--"),
        OsString::from("a.txt"),
    ];
    let mut command: Command = bounded_for(
        &fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(&fixture).as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    command.current_dir(worktree.as_path()).args(arguments.as_slice());
    let landed: Observed = observe(&mut command, b"");
    assert_eq!(landed.code, Some(0), "the commit lands: {}", stderr_of(&landed));
    assert_ne!(git_text(&fixture, worktree.as_path(), &["rev-parse", "HEAD"]), base, "the branch moved to the new commit");
    assert_nothing_left(&fixture, worktree.as_path());
    remove_settled(&fixture);
}
