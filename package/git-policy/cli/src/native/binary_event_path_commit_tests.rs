//! What:
//!  Event paths of a file whose name is not UTF-8 through a landed `git commit`,
//!  `git add` at `error` severity and a replay conflict,
//!  through the built executable.
//! Why:
//!  The reader of an event relies on "no `pathBytes` means `path` is the exact name",
//!  so every event on the commit path that names a file must carry the exact bytes of a name that is not UTF-8,
//!  and a commit must not stop only because a repository holds such a name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const event = JSON.parse(stderr); expect(Buffer.from(event.pathBytes, 'base64')).toEqual(name);
//! ```

/// Import the shared fixtures and the commit controls' helpers.
use super::commit::{
    COMMIT_BOUND_SECONDS, assert_nothing_left, await_marker, git_text, keep_maintenance_in_the_foreground,
    remove_settled,
};
use super::support::{Fixture, Observed, bounded_for, fixture, git, observe, porcelain, repository, stderr_of, wrapper_first_path};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// A name that is not UTF-8:
///  `bad`,
///  the byte 0xFF,
///  `.txt`.
const BAD_NAME: &[u8] = b"bad\xff.txt";

/// `BAD_NAME` as the event's readable `path`.
const BAD_TEXT: &str = "bad\u{fffd}.txt";

/// `BAD_NAME` as the event's `pathBytes`,
///  the output of `base64`.
const BAD_BYTES: &str = "YmFk/y50eHQ=";

/// The name as an operating-system string for arguments and file creation.
fn bad_name() -> OsString {
    return OsString::from_vec(BAD_NAME.to_vec());
}

/// A wrapped command with the commit bound,
///  taking operating-system string arguments.
fn wrapped_os(fixture: &Fixture, directory: &Path, arguments: &[OsString]) -> Command {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        COMMIT_BOUND_SECONDS,
    );
    command.current_dir(directory).args(arguments);
    return command;
}

/// Run one wrapped command to its end.
fn run_os(fixture: &Fixture, directory: &Path, arguments: &[OsString]) -> Observed {
    return observe(&mut wrapped_os(fixture, directory, arguments), b"");
}

/// Operating-system string arguments from literals and one name.
fn commit_arguments(extra: &[&str], name: &OsStr) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = vec![OsString::from("commit")];
    arguments.extend(extra.iter().map(|argument| { return OsString::from(argument); }));
    arguments.push(OsString::from("--"));
    arguments.push(name.to_os_string());
    return arguments;
}

/// A repository holding one commit and the configuration text,
///  when the control needs one.
fn configured(fixture: &Fixture, configuration: Option<&str>) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    keep_maintenance_in_the_foreground(fixture, repo.as_path());
    if let Some(text) = configuration {
        std::fs::write(repo.join("cli-git.config.jsonc"), text).expect("repository configuration");
    }
    return repo;
}

/// The names `git ls-tree -z --name-only HEAD` lists,
///  as exact bytes.
fn committed_names(fixture: &Fixture, repo: &Path) -> Vec<Vec<u8>> {
    let listing: Vec<u8> = git(fixture, repo, &["ls-tree", "-z", "--name-only", "HEAD"]).stdout;
    return listing
        .split(|byte| { return *byte == 0; })
        .filter(|name| { return !name.is_empty(); })
        .map(<[u8]>::to_vec)
        .collect();
}

/// The bytes of the file `BAD_NAME` in `HEAD`,
///  read by naming the file with its exact bytes.
fn head_blob(fixture: &Fixture, repo: &Path) -> Vec<u8> {
    let mut spec: Vec<u8> = b"HEAD:".to_vec();
    spec.extend_from_slice(BAD_NAME);
    return git(fixture, repo, &[OsString::from("show"), OsString::from_vec(spec)]).stdout;
}

/// A commit of a name that is not UTF-8 lands it with its exact bytes and prints nothing,
///  with the file alone or beside a tracked name that is not UTF-8.
#[test]
fn a_commit_of_a_name_that_is_not_utf8_lands_with_its_exact_bytes() {
    let fixture: Fixture = fixture("event-path-commit-clean");
    let repo: PathBuf = configured(&fixture, None);
    std::fs::write(repo.join(bad_name()), b"fine\n").expect("file");
    let landed: Observed = run_os(&fixture, repo.as_path(), &commit_arguments(&["--message=bad"], bad_name().as_os_str()));
    assert_eq!(
        (landed.code, stderr_of(&landed)),
        (Some(0), String::new())
    );
    assert_eq!(committed_names(&fixture, repo.as_path()), vec![BAD_NAME.to_vec()]);
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    // The name is now tracked in the base; a commit of another file must still land.
    std::fs::write(repo.join("other.txt"), b"fine\n").expect("other");
    let other: Observed = run_os(&fixture, repo.as_path(), &commit_arguments(&["--message=other"], OsStr::new("other.txt")));
    assert_eq!((other.code, stderr_of(&other)), (Some(0), String::new()));
    assert_eq!(
        committed_names(&fixture, repo.as_path()),
        vec![BAD_NAME.to_vec(), b"other.txt".to_vec()]
    );
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An untracked name that is not UTF-8 beside a committed file does not stop the commit.
#[test]
fn an_untracked_name_that_is_not_utf8_does_not_stop_a_commit() {
    let fixture: Fixture = fixture("event-path-commit-untracked");
    let repo: PathBuf = configured(&fixture, None);
    std::fs::write(repo.join(bad_name()), b"left alone\n").expect("untracked file");
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    let landed: Observed = run_os(&fixture, repo.as_path(), &commit_arguments(&["--message=a"], OsStr::new("a.txt")));
    assert_eq!((landed.code, stderr_of(&landed)), (Some(0), String::new()));
    assert_eq!(committed_names(&fixture, repo.as_path()), vec![b"a.txt".to_vec()]);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A correction on a name that is not UTF-8 lands in the commit and the worktree,
///  and the summary lists the exact bytes.
#[test]
fn a_commit_correction_of_a_name_that_is_not_utf8_reports_the_exact_bytes() {
    let fixture: Fixture = fixture("event-path-commit-fix");
    let repo: PathBuf = configured(&fixture, None);
    std::fs::write(repo.join(bad_name()), b"no newline").expect("file");
    let landed: Observed = run_os(&fixture, repo.as_path(), &commit_arguments(&["--message=bad"], bad_name().as_os_str()));
    assert_eq!(
        (landed.code, stderr_of(&landed)),
        (
            Some(0),
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\
                 \"passes\":1,\"changedPaths\":[\"{BAD_TEXT}\"],\"changedPathBytes\":[\"{BAD_BYTES}\"]}}\n"
            )
        )
    );
    assert_eq!(committed_names(&fixture, repo.as_path()), vec![BAD_NAME.to_vec()]);
    assert_eq!(std::fs::read(repo.join(bad_name())).expect("worktree copy"), b"no newline\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A selection made with `--include` applies no correction, so a finding at `error` severity blocks the commit and
///  names the exact bytes, and nothing lands.
#[test]
fn a_blocking_commit_finding_names_the_exact_bytes() {
    let fixture: Fixture = fixture("event-path-commit-blocked");
    let repo: PathBuf = configured(&fixture, Some("{ \"policies\": { \"final-newline\": \"error\" } }\n"));
    std::fs::write(repo.join(bad_name()), b"fine\n").expect("base file");
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join(bad_name()), b"no newline").expect("edit");
    let blocked: Observed = run_os(
        &fixture,
        repo.as_path(),
        &commit_arguments(&["--include", "--message=bad"], bad_name().as_os_str()),
    );
    assert_eq!(
        (blocked.code, stderr_of(&blocked)),
        (
            Some(1),
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\
                 \"policyId\":\"final-newline\",\"severity\":\"error\",\
                 \"code\":\"final-newline/noncanonical-final-newline\",\
                 \"message\":\"Non-empty text file must end with exactly one LF byte.\",\
                 \"path\":\"{BAD_TEXT}\",\"pathBytes\":\"{BAD_BYTES}\",\"fix\":\"none\"}}\n"
            )
        )
    );
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    remove_settled(&fixture);
}

/// `git add` at `error` severity stops with the exact bytes and stages nothing.
#[test]
fn an_add_blocked_at_error_severity_names_the_exact_bytes() {
    let fixture: Fixture = fixture("event-path-add-blocked");
    let repo: PathBuf = configured(&fixture, Some("{ \"policies\": { \"final-newline\": \"error\" } }\n"));
    std::fs::write(repo.join(bad_name()), b"no newline").expect("file");
    let blocked: Observed = run_os(
        &fixture,
        repo.as_path(),
        &[OsString::from("add"), OsString::from("--"), bad_name()],
    );
    assert_eq!(
        (blocked.code, stderr_of(&blocked)),
        (
            Some(1),
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\
                 \"policyId\":\"final-newline\",\"severity\":\"error\",\
                 \"code\":\"final-newline/noncanonical-final-newline\",\
                 \"message\":\"Non-empty text file must end with exactly one LF byte.\",\
                 \"path\":\"{BAD_TEXT}\",\"pathBytes\":\"{BAD_BYTES}\",\"fix\":\"none\"}}\n"
            )
        )
    );
    assert_eq!(git(&fixture, repo.as_path(), &["ls-files", "-z"]).stdout, Vec::<u8>::new());
    remove_settled(&fixture);
}

/// A replay conflict on a name that is not UTF-8 lists the readable names and a parallel list of the exact bytes.
#[test]
fn a_replay_conflict_on_a_name_that_is_not_utf8_lists_the_exact_bytes() {
    let fixture: Fixture = fixture("event-path-conflict");
    let repo: PathBuf = configured(&fixture, None);
    std::fs::write(repo.join(bad_name()), b"one\n").expect("base file");
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join(bad_name()), b"one\ntwo\n").expect("edit");
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("marker directory");
    let slow: Child = wrapped_os(&fixture, repo.as_path(), &commit_arguments(&["--message=slow"], bad_name().as_os_str()))
        .env("CLI_GIT_TEST_ONLY_PHASE_SIGNAL", format!("preparation-done:pause:{}", markers.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the slow commit starts");
    await_marker(markers.join("preparation-done.reached").as_path());
    std::fs::write(repo.join(bad_name()), b"other\n").expect("winner edit");
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=winner"]);
    let winning: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(markers.join("preparation-done.release"), b"").expect("release the slow commit");
    let finished = slow.wait_with_output().expect("the slow commit ends");
    let stderr: String = String::from_utf8_lossy(finished.stderr.as_slice()).into_owned();
    assert_eq!(finished.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("\"code\":\"concurrent-commit/replay-conflict\""), "{stderr}");
    assert!(stderr.contains(format!("\"paths\":[\"{BAD_TEXT}\"],\"pathsBytes\":[\"{BAD_BYTES}\"]").as_str()), "{stderr}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]).trim(), winning.trim());
    remove_settled(&fixture);
}

/// Names that are UTF-8 keep their old shape:
///  a backslash is escaped by JSON only,
///  and a name holding the replacement character itself, correctly encoded, gets no `pathBytes`.
#[test]
fn names_that_are_utf8_keep_the_old_shape() {
    let fixture: Fixture = fixture("event-path-utf8");
    let repo: PathBuf = configured(&fixture, None);
    let replacement: OsString = OsString::from("caf\u{fffd}.txt");
    for name in [OsString::from("a\\b.txt"), replacement.clone()] {
        std::fs::write(repo.join(&name), b"no newline").expect("file");
    }
    let added: Observed = run_os(
        &fixture,
        repo.as_path(),
        &[OsString::from("add"), OsString::from("--"), OsString::from("a\\b.txt"), replacement],
    );
    let line = |name: &str, sequence: u64| -> String {
        return format!(
            "{{\"schemaVersion\":1,\"sequence\":{sequence},\"type\":\"finding\",\"trigger\":\"pre-forward\",\
             \"policyId\":\"final-newline\",\"severity\":\"warn\",\
             \"code\":\"final-newline/noncanonical-final-newline\",\
             \"message\":\"Non-empty text file must end with exactly one LF byte.\",\
             \"path\":\"{name}\",\"fix\":\"none\"}}\n"
        );
    };
    assert_eq!(
        (added.code, stderr_of(&added)),
        (Some(0), format!("{}{}", line("a\\\\b.txt", 0), line("caf\u{fffd}.txt", 1)))
    );
    git(&fixture, repo.as_path(), &["reset", "--quiet"]);
    let committed: Observed = run_os(
        &fixture,
        repo.as_path(),
        &[
            OsString::from("commit"),
            OsString::from("--message=utf-8"),
            OsString::from("--"),
            OsString::from("a\\b.txt"),
            OsString::from("caf\u{fffd}.txt"),
        ],
    );
    assert_eq!(
        (committed.code, stderr_of(&committed)),
        (
            Some(0),
            "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\"passes\":1,\
             \"changedPaths\":[\"a\\\\b.txt\",\"caf\u{fffd}.txt\"]}\n"
                .to_string()
        )
    );
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// The configuration that turns the forbidden-strings policy on with a rules file holding one needle.
fn forbidden_configuration(repo: &Path, needle: &str) {
    std::fs::create_dir(repo.join("rules")).expect("rules directory");
    std::fs::write(repo.join("rules/private.txt"), format!("{needle}\n")).expect("rules");
    std::fs::write(
        repo.join("cli-git.config.jsonc"),
        "{ \"policies\": { \"final-newline\": \"off\", \"security/forbidden-strings\": [\"error\", { \"builtinRules\": false, \"rulesFile\": \"rules/private.txt\" }] } }\n",
    )
    .expect("configuration");
}

/// The forbidden-strings finding keeps the scanner's masked display of the name and never carries the bytes,
///  on `git add` and on a commit,
///  because the exact bytes would undo the masking of a matched segment.
#[test]
fn the_forbidden_strings_finding_for_a_name_that_is_not_utf8_has_no_exact_bytes() {
    let fixture: Fixture = fixture("event-path-forbidden");
    let repo: PathBuf = configured(&fixture, None);
    let needle: String = ["PLANTED", "PATH", "NEEDLE"].join("_");
    forbidden_configuration(repo.as_path(), needle.as_str());
    // Not the rules file's own bytes: a blob that `HEAD` already holds is not scanned again, and the commit below
    // would otherwise find the rules file's blob in `HEAD`.
    std::fs::write(repo.join(bad_name()), format!("{needle} in a file of its own\n")).expect("file");
    let expected: String = "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"finding\",\"trigger\":\"pre-forward\",\
         \"policyId\":\"security/forbidden-strings\",\"severity\":\"error\",\
         \"code\":\"security/forbidden-strings/forbidden-string\",\
         \"message\":\"Forbidden string matched at line 1 (rule 0).\",\"path\":\"bad\\\\xff.txt\",\"fix\":\"none\"}\n"
        .to_string();
    let added: Observed = run_os(&fixture, repo.as_path(), &[OsString::from("add"), OsString::from("--"), bad_name()]);
    assert_eq!((added.code, stderr_of(&added)), (Some(1), expected.clone()));
    git(&fixture, repo.as_path(), &["add", "--", "cli-git.config.jsonc", "rules/private.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=configuration"]);
    let committed: Observed = run_os(&fixture, repo.as_path(), &commit_arguments(&["--include", "--message=bad"], bad_name().as_os_str()));
    assert_eq!((committed.code, stderr_of(&committed)), (Some(1), expected));
    remove_settled(&fixture);
}

/// Index mode (`commit -a`) lands an edit of a tracked name that is not UTF-8, corrected,
///  and the summary lists the exact bytes.
#[test]
fn a_commit_of_all_tracked_changes_lands_a_name_that_is_not_utf8() {
    let fixture: Fixture = fixture("event-path-commit-all");
    let repo: PathBuf = configured(&fixture, None);
    std::fs::write(repo.join(bad_name()), b"one\n").expect("base file");
    git(&fixture, repo.as_path(), &["add", "--all"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join(bad_name()), b"one\ntwo").expect("edit");
    let landed: Observed = run_os(
        &fixture,
        repo.as_path(),
        &[OsString::from("commit"), OsString::from("-a"), OsString::from("--no-only"), OsString::from("--message=all")],
    );
    assert_eq!(
        (landed.code, stderr_of(&landed)),
        (
            Some(0),
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\
                 \"passes\":1,\"changedPaths\":[\"{BAD_TEXT}\"],\"changedPathBytes\":[\"{BAD_BYTES}\"]}}\n"
            )
        )
    );
    assert_eq!(head_blob(&fixture, repo.as_path()), b"one\ntwo\n");
    assert_eq!(std::fs::read(repo.join(bad_name())).expect("worktree copy"), b"one\ntwo\n");
    assert_eq!(committed_names(&fixture, repo.as_path()), vec![BAD_NAME.to_vec()]);
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A repository whose base commit tracks `name` with the content `one\n`,
///  and whose worktree copy then holds `edited`.
fn tracked(fixture: &Fixture, name: &OsStr, edited: &[u8]) -> PathBuf {
    let repo: PathBuf = configured(fixture, None);
    std::fs::write(repo.join(name), b"one\n").expect("base file");
    git(fixture, repo.as_path(), &["add", "--all"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join(name), edited).expect("edit");
    return repo;
}

/// A name staged with plain Git commits in index mode with its exact bytes.
#[test]
fn a_staged_name_that_is_not_utf8_commits_in_index_mode() {
    let fixture: Fixture = fixture("event-path-index-mode");
    let repo: PathBuf = configured(&fixture, None);
    std::fs::write(repo.join(bad_name()), b"fine\n").expect("file");
    git(&fixture, repo.as_path(), &[OsString::from("add"), OsString::from("--"), bad_name()]);
    let landed: Observed = run_os(
        &fixture,
        repo.as_path(),
        &[OsString::from("commit"), OsString::from("--no-only"), OsString::from("--message=staged")],
    );
    assert_eq!((landed.code, stderr_of(&landed)), (Some(0), String::new()));
    assert_eq!(committed_names(&fixture, repo.as_path()), vec![BAD_NAME.to_vec()]);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A correction that restores the base makes a no-change normalization,
///  and its summary lists the exact bytes.
#[test]
fn a_no_change_normalization_of_a_name_that_is_not_utf8_reports_the_exact_bytes() {
    let fixture: Fixture = fixture("event-path-no-change");
    let repo: PathBuf = tracked(&fixture, bad_name().as_os_str(), b"one");
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    let result: Observed = run_os(&fixture, repo.as_path(), &commit_arguments(&["--message=x"], bad_name().as_os_str()));
    let stderr: String = stderr_of(&result);
    assert_eq!(result.code, Some(1), "{stderr}");
    assert!(
        stderr.starts_with(
            format!(
                "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"fix-summary\",\"trigger\":\"pre-forward\",\
                 \"passes\":1,\"changedPaths\":[\"{BAD_TEXT}\"],\"changedPathBytes\":[\"{BAD_BYTES}\"]}}\n"
            )
            .as_str()
        ),
        "{stderr}"
    );
    assert!(stderr.contains("\"code\":\"commit-normalization/no-change\""), "{stderr}");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_eq!(std::fs::read(repo.join(bad_name())).expect("worktree copy"), b"one\n");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A configured `pre-commit` hook that re-stages a name that is not UTF-8 has its change kept in the commit.
#[test]
fn a_pre_commit_hook_restaging_a_name_that_is_not_utf8_is_kept() {
    let fixture: Fixture = fixture("event-path-hook");
    let repo: PathBuf = tracked(&fixture, bad_name().as_os_str(), b"one\ntwo\n");
    super::support::configure_pre_commit_hook(
        &fixture,
        repo.as_path(),
        b"#!/bin/sh\nname=$(printf 'bad\\377.txt')\nprintf 'hooked\\n' >> \"$name\"\ngit add -- \"$name\"\n",
    );
    let landed: Observed = run_os(&fixture, repo.as_path(), &commit_arguments(&["--message=hooked"], bad_name().as_os_str()));
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    assert_eq!(head_blob(&fixture, repo.as_path()), b"one\ntwo\nhooked\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// An amend replaces the head commit with the exact bytes of a name that is not UTF-8.
#[test]
fn an_amend_of_a_name_that_is_not_utf8_lands_it() {
    let fixture: Fixture = fixture("event-path-amend");
    let repo: PathBuf = tracked(&fixture, bad_name().as_os_str(), b"one\ntwo\n");
    let landed: Observed = run_os(
        &fixture,
        repo.as_path(),
        &commit_arguments(&["--amend", "--message=amended"], bad_name().as_os_str()),
    );
    assert_eq!((landed.code, stderr_of(&landed)), (Some(0), String::new()));
    assert_eq!(head_blob(&fixture, repo.as_path()), b"one\ntwo\n");
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), "2");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A commit that lost the landing race replays onto the winner and lands both files,
///  `name` among them, without a conflict.
fn replayed_names(fixture_name: &str, name: &OsStr) -> (Fixture, PathBuf, String) {
    let fixture: Fixture = fixture(fixture_name);
    let repo: PathBuf = tracked(&fixture, name, b"one\ntwo\n");
    std::fs::write(repo.join("b.txt"), b"b\n").expect("b");
    git(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("marker directory");
    let slow: Child = wrapped_os(&fixture, repo.as_path(), &commit_arguments(&["--message=slow"], name))
        .env("CLI_GIT_TEST_ONLY_PHASE_SIGNAL", format!("preparation-done:pause:{}", markers.display()))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the slow commit starts");
    await_marker(markers.join("preparation-done.reached").as_path());
    let fast: Observed = run_os(&fixture, repo.as_path(), &commit_arguments(&["--message=fast"], OsStr::new("b.txt")));
    assert_eq!(fast.code, Some(0), "{}", stderr_of(&fast));
    std::fs::write(markers.join("preparation-done.release"), b"").expect("release the slow commit");
    let finished = slow.wait_with_output().expect("the slow commit ends");
    let stderr: String = String::from_utf8_lossy(finished.stderr.as_slice()).into_owned();
    assert_eq!(finished.status.code(), Some(0), "{stderr}");
    assert!(stderr.contains("\"type\":\"commit-replayed\""), "{stderr}");
    return (fixture, repo, stderr);
}

/// A replayed commit of a name that is not UTF-8 lands it with its exact bytes.
#[test]
fn a_replayed_commit_of_a_name_that_is_not_utf8_lands_it() {
    let (fixture, repo, _stderr): (Fixture, PathBuf, String) = replayed_names("event-path-replay", bad_name().as_os_str());
    assert_eq!(committed_names(&fixture, repo.as_path()), vec![b"b.txt".to_vec(), BAD_NAME.to_vec()]);
    assert_eq!(head_blob(&fixture, repo.as_path()), b"one\ntwo\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    remove_settled(&fixture);
}

/// A replayed commit of a UTF-8 name with a non-ASCII character lands it once under its own name.
#[test]
fn a_replayed_commit_of_a_non_ascii_utf8_name_lands_it() {
    let (fixture, repo, _stderr): (Fixture, PathBuf, String) = replayed_names("event-path-replay-utf8", OsStr::new("café.txt"));
    assert_eq!(committed_names(&fixture, repo.as_path()), vec![b"b.txt".to_vec(), "café.txt".as_bytes().to_vec()]);
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    remove_settled(&fixture);
}

/// A commit killed after its ref moved is completed by recovery from the journal,
///  which holds the exact bytes of a name that is not UTF-8:
///  the worktree copy receives its correction.
#[test]
fn recovery_completes_a_killed_commit_of_a_name_that_is_not_utf8() {
    let fixture: Fixture = fixture("event-path-recovery");
    let repo: PathBuf = tracked(&fixture, bad_name().as_os_str(), b"one\ntwo");
    let markers: PathBuf = fixture.root.join("markers");
    std::fs::create_dir(&markers).expect("marker directory");
    // The kill marker ends the wrapper with SIGKILL, which `observe` would take for its own time bound.
    let killed = wrapped_os(&fixture, repo.as_path(), &commit_arguments(&["--message=killed"], bad_name().as_os_str()))
        .env("CLI_GIT_TEST_ONLY_PHASE_SIGNAL", format!("ref-updated:kill:{}", markers.display()))
        .stdin(Stdio::null())
        .output()
        .expect("the killed commit runs");
    assert!(markers.join("ref-updated.reached").exists(), "the wrapper reached the marker: {}", String::from_utf8_lossy(&killed.stderr));
    let recovered: Observed = run_os(&fixture, repo.as_path(), &[OsString::from("status"), OsString::from("--porcelain=v1")]);
    assert_eq!((recovered.code, stderr_of(&recovered)), (Some(0), String::new()));
    assert_eq!(committed_names(&fixture, repo.as_path()), vec![BAD_NAME.to_vec()]);
    assert_eq!(head_blob(&fixture, repo.as_path()), b"one\ntwo\n");
    assert_eq!(std::fs::read(repo.join(bad_name())).expect("worktree copy"), b"one\ntwo\n");
    assert_eq!(porcelain(&fixture, repo.as_path()), "");
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// A linter that fails stands in for the real one, because the control is about the event and not about Markdown.
fn failing_linter_path(fixture: &Fixture) -> OsString {
    let directory: PathBuf = fixture.root.join("linter");
    std::fs::create_dir_all(&directory).expect("linter directory");
    super::support::executable(
        directory.join("monochromatic-lint").as_path(),
        b"#!/bin/sh\necho 'monochromatic-lint: broken on purpose' >&2\nexit 3\n",
    );
    let mut path: OsString = fixture.root.join("bin").into_os_string();
    path.push(":");
    path.push(directory);
    path.push(":/usr/bin:/bin");
    return path;
}

/// A Markdown file the linter could not check, named by bytes that are not UTF-8, stops `git add` and a commit
///  with an engine failure that carries the exact bytes, and nothing is staged or landed.
#[test]
fn an_unchecked_markdown_name_that_is_not_utf8_is_reported_with_its_exact_bytes() {
    let fixture: Fixture = fixture("event-path-markdown");
    let repo: PathBuf = configured(&fixture, Some("{ \"policies\": { \"final-newline\": \"off\", \"markdown/autofix\": \"warn\" } }\n"));
    let name: OsString = OsString::from_vec(b"caf\xe9.md".to_vec());
    std::fs::write(repo.join(".lfsconfig"), b"[lfs]\n\turl = https://lfs.example.com/org/repo\n").expect("lfsconfig");
    std::fs::write(repo.join(".gitattributes"), b"*.png filter=lfs diff=lfs merge=lfs -text\n").expect("attributes");
    std::fs::write(repo.join(&name), b"# T\n").expect("markdown");
    let path: OsString = failing_linter_path(&fixture);
    let reason: &str = "monochromatic-lint exited with status 3, which its one-rule configuration never produces: broken on purpose";
    let expected = |sequence_type: &str| -> String {
        return format!(
            "{{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"policy-incomplete\",\
             \"message\":\"{reason}. The Markdown candidate was not checked and nothing was changed; fix the cause, \
             or pass --no-enforce-markdown/autofix to skip this policy for one command.\",\
             \"trigger\":\"{sequence_type}\",\"policyId\":\"markdown/autofix\",\
             \"path\":\"caf\u{fffd}.md\",\"pathBytes\":\"Y2Fm6S5tZA==\"}}\n"
        );
    };
    let mut add: Command = wrapped_os(&fixture, repo.as_path(), &[OsString::from("add"), OsString::from("--"), name.clone()]);
    add.env("PATH", &path);
    let added: Observed = observe(&mut add, b"");
    assert_eq!((added.code, stderr_of(&added)), (Some(2), expected("pre-forward")));
    assert_eq!(git(&fixture, repo.as_path(), &["ls-files", "-z"]).stdout, Vec::<u8>::new());
    git(&fixture, repo.as_path(), &[OsString::from("add"), OsString::from("--"), name.clone()]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join(&name), b"# T\n\nedited\n").expect("edit");
    let mut commit: Command = wrapped_os(&fixture, repo.as_path(), &commit_arguments(&["--message=edited"], name.as_os_str()));
    commit.env("PATH", &path);
    let committed: Observed = observe(&mut commit, b"");
    assert_eq!((committed.code, stderr_of(&committed)), (Some(2), expected("pre-forward")));
    assert_eq!(git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]), head);
    assert_nothing_left(&fixture, repo.as_path());
    remove_settled(&fixture);
}
