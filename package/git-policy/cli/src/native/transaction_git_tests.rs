//! Controls for the transaction Git runner against real Git.

use super::*;
use crate::test_support::{executable, fixture, git_context, git_text, remove, repository};

/// Requests carry their arguments and nothing else by default.
#[test]
fn requests_start_empty() {
    let request: GitRequest = GitRequest::new(Path::new("/repo"), &["status", "--short"]);
    assert_eq!(request.cwd, PathBuf::from("/repo"));
    assert_eq!(
        request.arguments,
        vec![OsString::from("status"), OsString::from("--short")]
    );
    assert!(!request.without_prefix);
    assert_eq!(request.index, None);
    assert_eq!(request.object_directory, None);
    assert!(request.environment.is_empty());
    assert!(request.unset.is_empty());
    assert_eq!(request.input, None);
    assert_eq!(
        command_text(&[OsString::from("update-ref"), OsString::from("-d")]),
        "git update-ref -d"
    );
    assert_eq!(command_text(&[]), "git");
}

/// Output,
///  exit codes and input reach the caller;
///  failures name the command.
#[test]
fn runs_capture_output_and_feed_input() {
    let root: PathBuf = fixture("transaction-git-run");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let mut hashing: GitRequest =
        GitRequest::new(repo.as_path(), &["hash-object", "-w", "--stdin"]);
    hashing.input = Some(b"content\n".to_vec());
    let hashed: GitOutput = run_git_checked(&context, &hashing).expect("hash-object");
    assert!(hashed.succeeded());
    assert_eq!(
        String::from_utf8_lossy(hashed.stdout.as_slice()).trim(),
        "d95f3ad14dee633a758d2e331151e950dd13e4ed"
    );
    // Large input never deadlocks against Git's output.
    let mut large: GitRequest = GitRequest::new(repo.as_path(), &["hash-object", "--stdin"]);
    large.input = Some(vec![b'x'; 4 * 1024 * 1024]);
    assert!(run_git_checked(&context, &large).is_ok());
    let missing: GitRequest = GitRequest::new(
        repo.as_path(),
        &["rev-parse", "--verify", "--quiet", "refs/heads/none"],
    );
    let output: GitOutput = run_git(&context, &missing).expect("rev-parse runs");
    assert_eq!(output.code, Some(1));
    assert!(!output.succeeded());
    let failure: GitFailure = run_git_checked(
        &context,
        &GitRequest::new(
            repo.as_path(),
            &["cat-file", "-t", "0000000000000000000000000000000000000001"],
        ),
    )
    .expect_err("missing object");
    assert!(
        failure
            .to_string()
            .starts_with("git cat-file -t 0000000000000000000000000000000000000001 failed: fatal:"),
        "{failure}"
    );
    let unstartable: GitContext = GitContext {
        real_git: root.join("no-git"),
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: None,
    };
    let start: GitFailure =
        run_git_checked(&unstartable, &GitRequest::new(repo.as_path(), &["status"]))
            .expect_err("no Git");
    assert!(
        start.0.starts_with("git status could not start: "),
        "{start}"
    );
    remove(root.as_path());
}

/// The global prefix,
///  the overlay,
///  removals,
///  additions,
///  index and store reach Git in order.
#[test]
fn environment_and_prefix_reach_git() {
    let root: PathBuf = fixture("transaction-git-environment");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut context: GitContext = git_context();
    context.global_prefix = vec![OsString::from("-c"), OsString::from("cli.probe=prefixed")];
    let prefixed: GitOutput = run_git_checked(
        &context,
        &GitRequest::new(repo.as_path(), &["config", "cli.probe"]),
    )
    .expect("prefix");
    assert_eq!(
        String::from_utf8_lossy(prefixed.stdout.as_slice()),
        "prefixed\n"
    );
    let mut bare: GitRequest = GitRequest::new(repo.as_path(), &["config", "cli.probe"]);
    bare.without_prefix = true;
    assert_eq!(run_git(&context, &bare).expect("unprefixed").code, Some(1));
    // `GIT_CONFIG_PARAMETERS` set by the overlay, then removed, then added back by the request.
    context.overlay.push((
        OsString::from("GIT_CONFIG_PARAMETERS"),
        OsString::from("'cli.probe'='overlay'"),
    ));
    let mut removed: GitRequest = GitRequest::new(repo.as_path(), &["config", "cli.probe"]);
    removed.without_prefix = true;
    removed.unset = vec![OsString::from("GIT_CONFIG_PARAMETERS")];
    assert_eq!(run_git(&context, &removed).expect("removed").code, Some(1));
    removed.environment = vec![(
        OsString::from("GIT_CONFIG_PARAMETERS"),
        OsString::from("'cli.probe'='request'"),
    )];
    let added: GitOutput = run_git_checked(&context, &removed).expect("added");
    assert_eq!(
        String::from_utf8_lossy(added.stdout.as_slice()),
        "request\n"
    );
    // A private index and object store receive the writes.
    let private_index: PathBuf = root.join("private.index");
    let store: PathBuf = root.join("objects");
    std::fs::create_dir(&store).expect("store");
    let mut update: GitRequest = GitRequest::new(
        repo.as_path(),
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            "100644,d95f3ad14dee633a758d2e331151e950dd13e4ed,new.txt",
        ],
    );
    update.without_prefix = true;
    update.index = Some(private_index.clone());
    run_git_checked(&context, &update).expect("private index write");
    assert!(private_index.exists());
    assert_eq!(git_text(repo.as_path(), &["ls-files"]), "");
    let mut write: GitRequest = GitRequest::new(repo.as_path(), &["hash-object", "-w", "--stdin"]);
    write.without_prefix = true;
    write.object_directory = Some(store.clone());
    write.input = Some(b"private\n".to_vec());
    let written: GitOutput = run_git_checked(&context, &write).expect("private object");
    let oid: String = String::from_utf8_lossy(written.stdout.as_slice())
        .trim()
        .to_string();
    assert!(store.join(&oid[..2]).join(&oid[2..]).exists());
    let unknown: GitOutput = run_git(
        &context,
        &GitRequest::new(repo.as_path(), &["cat-file", "-e", oid.as_str()]),
    )
    .expect("cat-file runs");
    assert!(!unknown.succeeded());
    assert_eq!(unknown.error_text(), "");
    remove(root.as_path());
}

/// Error text is trimmed and lossily decoded.
#[test]
fn error_text_is_trimmed() {
    let output: GitOutput = GitOutput {
        code: Some(128),
        stdout: Vec::new(),
        stderr: b"  fatal: \xff broken \n".to_vec(),
    };
    assert_eq!(output.error_text(), "fatal: \u{fffd} broken");
    assert!(
        !GitOutput {
            code: None,
            stdout: Vec::new(),
            stderr: Vec::new()
        }
        .succeeded()
    );
}

/// Input files are private and already unlinked.
#[cfg(unix)]
#[test]
fn input_files_leave_nothing_behind() {
    use std::io::Read;
    use std::os::unix::fs::MetadataExt;
    let root: PathBuf = fixture("transaction-git-input");
    let mut file: std::fs::File = input_file(b"bytes", Some(root.as_path())).expect("input file");
    let metadata: std::fs::Metadata = file.metadata().expect("metadata");
    assert_eq!(metadata.nlink(), 0);
    assert_eq!(metadata.mode() & 0o777, 0o600);
    let mut read: Vec<u8> = Vec::new();
    file.read_to_end(&mut read).expect("read");
    assert_eq!(read, b"bytes");
    let left: Vec<std::ffi::OsString> = std::fs::read_dir(root.join("cli-git-scratch"))
        .expect("the repository scratch directory exists")
        .map(|entry| { return entry.expect("entry").file_name(); })
        .collect();
    assert!(left.is_empty(), "nothing is left in the scratch directory: {left:?}");
    remove(root.as_path());
}

/// A request that carries input needs a repository: without one the failure names the defect and writes nothing.
#[test]
fn input_without_a_repository_is_refused_naming_the_defect() {
    let error: std::io::Error = input_file(b"bytes", None).expect_err("no repository");
    assert!(error.to_string().contains("no repository scratch directory"), "{error}");
}

/// A common directory that does not exist is an error, and the input file is never made elsewhere.
#[test]
fn input_in_a_missing_common_directory_fails_without_a_fallback() {
    let root: PathBuf = fixture("transaction-git-input-missing");
    let missing: PathBuf = root.join("no-such-common-directory");
    assert!(input_file(b"bytes", Some(missing.as_path())).is_err());
    assert!(!missing.exists());
    remove(root.as_path());
}

/// A fake Git that prints where its standard input lives, which is the input file's name before it was unlinked.
#[cfg(target_os = "linux")]
const FAKE_GIT_STDIN_PATH: &[u8] = b"#!/bin/sh\nreadlink /proc/$$/fd/0\n";

/// The input a command gets is a file in the repository scratch directory of its context, and is gone afterwards.
#[cfg(target_os = "linux")]
#[test]
fn a_command_reads_its_input_from_the_repository_scratch_directory() {
    let root: PathBuf = fixture("transaction-git-input-location");
    let program: PathBuf = root.join("fake-git");
    executable(program.as_path(), FAKE_GIT_STDIN_PATH);
    let common: PathBuf = root.join("common");
    std::fs::create_dir(&common).expect("common directory");
    let context: GitContext = GitContext {
        real_git: program,
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: Some(common.clone()),
    };
    let mut request: GitRequest = GitRequest::new(root.as_path(), &["hash-object", "--stdin"]);
    request.input = Some(b"12345".to_vec());
    let output: GitOutput = run_git(&context, &request).expect("the fake runs");
    let named: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_owned();
    let expected: String = format!("{}/cli-git-scratch/cli-git-input-", common.display());
    assert!(named.starts_with(expected.as_str()), "{named} should start with {expected}");
    assert_eq!(std::fs::read_dir(common.join("cli-git-scratch")).expect("scratch").count(), 0);
    remove(root.as_path());
}

/// A fake Git that counts its runs, refuses the first `FAKE_REFUSALS` of them the way a host out of threads does,
///  and otherwise prints how many input bytes it read.
const FAKE_GIT: &[u8] = b"#!/bin/sh\n\
count=$(cat \"$FAKE_COUNT\" 2>/dev/null || echo 0)\n\
count=$((count + 1))\n\
echo $count > \"$FAKE_COUNT\"\n\
bytes=$(wc -c | tr -d ' ')\n\
if [ \"$count\" -le \"$FAKE_REFUSALS\" ]; then\n\
  echo 'fatal: unable to create thread: Resource temporarily unavailable' >&2\n\
  exit 128\n\
fi\n\
if [ -n \"$FAKE_OTHER_FAILURE\" ]; then\n\
  echo 'fatal: bad object' >&2\n\
  exit 128\n\
fi\n\
echo \"read $bytes\"\n";

/// A context running the fake Git, a request for `arguments` with the fake's counters, and the count file.
fn fake_run(root: &Path, arguments: &[&str], refusals: u32, other_failure: bool) -> (GitContext, GitRequest, PathBuf) {
    let program: PathBuf = root.join("fake-git");
    executable(program.as_path(), FAKE_GIT);
    let count: PathBuf = root.join("count");
    let context: GitContext = GitContext {
        real_git: program,
        overlay: Vec::new(),
        global_prefix: Vec::new(),
        common_dir: Some(root.to_path_buf()),
    };
    let mut request: GitRequest = GitRequest::new(root, arguments);
    request.input = Some(b"12345".to_vec());
    request.environment = vec![
        (OsString::from("FAKE_COUNT"), count.clone().into_os_string()),
        (OsString::from("FAKE_REFUSALS"), OsString::from(refusals.to_string())),
        (OsString::from("FAKE_OTHER_FAILURE"), OsString::from(if other_failure { "1" } else { "" })),
    ];
    return (context, request, count);
}

/// How many times the fake Git ran.
fn runs_of(count: &Path) -> u32 {
    return std::fs::read_to_string(count).map_or(0, |text| { return text.trim().parse().unwrap_or(0); });
}

/// A pause that records nothing and waits for nothing, so the controls do not sleep.
fn no_pause(_wait: Duration) {}

/// A rerunnable command that Git reports as refused is run again with a fresh input file until it works.
#[test]
fn a_rerunnable_command_refused_inside_git_is_run_again_with_its_input() {
    let root: PathBuf = fixture("transaction-git-rerun");
    let (context, request, count) = fake_run(root.as_path(), &["rev-parse", "HEAD"], 2, false);
    let output: GitOutput =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect("the third run works");
    assert_eq!(output.code, Some(0));
    assert_eq!(String::from_utf8_lossy(output.stdout.as_slice()), "read 5\n", "the input file was fresh");
    assert_eq!(runs_of(count.as_path()), 3);
    remove(root.as_path());
}

/// A command that may not be run twice runs once, whatever Git says.
#[test]
fn a_writer_that_may_not_be_rerun_runs_once() {
    let root: PathBuf = fixture("transaction-git-once");
    let (context, request, count) = fake_run(root.as_path(), &["update-ref", "refs/heads/main", "abc"], 5, false);
    let output: GitOutput =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect("the run is reported");
    assert_eq!(output.code, Some(128));
    assert!(output.error_text().contains("Resource temporarily unavailable"));
    assert_eq!(runs_of(count.as_path()), 1);
    remove(root.as_path());
}

/// A failure that is not a refusal is not run again.
#[test]
fn another_failure_is_not_run_again() {
    let root: PathBuf = fixture("transaction-git-other");
    let (context, request, count) = fake_run(root.as_path(), &["rev-parse", "HEAD"], 0, true);
    let output: GitOutput =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect("the run is reported");
    assert_eq!(output.error_text(), "fatal: bad object");
    assert_eq!(runs_of(count.as_path()), 1);
    remove(root.as_path());
}

/// A refusal that outlasts the budget returns Git's own last output, after a bounded number of runs.
#[test]
fn a_refusal_past_the_budget_returns_gits_last_output() {
    let root: PathBuf = fixture("transaction-git-budget");
    let (context, request, count) = fake_run(root.as_path(), &["rev-parse", "HEAD"], 1000, false);
    let output: GitOutput =
        run_git_pausing(&context, &request, START_RETRY_BUDGET, &mut no_pause).expect("Git's last output");
    assert_eq!(output.code, Some(128));
    assert!(output.error_text().contains("unable to create thread"));
    assert_eq!(runs_of(count.as_path()), 15, "the same bound as a refused start");
    remove(root.as_path());
}

/// The allow-list: reads and idempotent writers may be rerun, other writers and unrelated shapes may not.
#[test]
fn only_harmless_commands_may_be_rerun() {
    let may = |arguments: &[&str], private_index: bool| -> bool {
        let mut request: GitRequest = GitRequest::new(Path::new("/repo"), arguments);
        if private_index {
            request.index = Some(PathBuf::from("/private.index"));
        }
        return may_be_rerun(&request);
    };
    assert!(may(&["rev-parse", "HEAD"], false));
    assert!(may(&["-c", "core.hooksPath=/x", "pack-objects", "--revs"], false));
    assert!(may(&["--git-dir=/shadow", "-C", "/dir", "index-pack", "--keep=m"], false));
    assert!(may(&["config", "--get", "a.b"], false));
    assert!(may(&["config", "--file", "config", "--null", "--list"], false));
    assert!(may(&["remote", "get-url", "origin"], false));
    assert!(may(&["add", "--all", "--", "a.txt"], true));
    assert!(may(&["reset", "--quiet", "abc", "--", "a.txt"], true));
    assert!(!may(&["add", "--all", "--", "a.txt"], false), "the real index is not rerun");
    assert!(!may(&["config", "a.b", "c"], false), "a config write is not a read");
    assert!(!may(&["remote", "add", "origin", "url"], false));
    assert!(!may(&["update-ref", "refs/heads/main", "abc"], false));
    assert!(!may(&["commit", "--message=x"], false));
    assert!(!may(&["push", "origin"], false));
    assert!(!may(&[], false));
    assert!(!may(&["-c", "a=b"], false));
}

/// Only Git's own words for a refused process or thread count as a refusal.
#[test]
fn only_a_refusal_by_the_system_counts() {
    let failure = |text: &str, code: Option<i32>| -> GitOutput {
        return GitOutput { code, stdout: Vec::new(), stderr: text.as_bytes().to_vec() };
    };
    assert!(refused_by_the_system(&failure("fatal: unable to create thread: Resource temporarily unavailable", Some(128))));
    assert!(refused_by_the_system(&failure("fatal: cannot fork() for pack-objects: Resource temporarily unavailable", None)));
    assert!(!refused_by_the_system(&failure("Resource temporarily unavailable", Some(0))), "a success is not a refusal");
    assert!(!refused_by_the_system(&failure("fatal: bad object", Some(128))));
}
