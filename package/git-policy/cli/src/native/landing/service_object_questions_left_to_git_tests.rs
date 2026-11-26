//! Controls for the object questions a declined in-process reader leaves to Git.
//! Why:
//!  The reader declines a reftable store and a files store holding a replace ref,
//! and each question below then starts Git.
//!  A question that silently answered nothing would take a wrong answer back to the landing,
//! so each is asked of both kinds of declined repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const repo of [reftable, filesWithReplace]) { expect(commitBytes(repo, older)).toBe(gitCatFile(repo, older)); }
//! ```

use super::*;
use crate::recording_git::{
    recorded_starts,
    recording_context,
};
use crate::test_support::{
    FIXTURE_GIT_BOUND,
    REAL_GIT,
    fixture,
    git,
    git_output,
    git_text,
    real_git_within,
    remove,
};
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// A repository whose object reader declines,
/// with two commits:
/// a reftable one,
/// or a files one holding a replace ref.
fn a_declined_repository(
    root: &Path,
    kind: &str,
) -> PathBuf {
    let repo: PathBuf = a_plain_repository(root, kind, kind == "reftable");
    // The reftable backend keeps the replace ref in its tables and the files backend in `refs/replace`,
    // so one command makes both kinds a store the reader declines.
    let older: String = git_text(repo.as_path(), &["rev-parse", "HEAD~1"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["replace", older.as_str(), head.as_str()]);
    return repo;
}

/// A repository the reader opens,
/// with two commits,
/// unless its ref store is the reftable one.
fn a_plain_repository(
    root: &Path,
    name: &str,
    reftable: bool,
) -> PathBuf {
    let repo: PathBuf = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    if reftable {
        git(
            repo.as_path(),
            &[
                "init",
                "--quiet",
                "--ref-format=reftable",
                "--initial-branch=main",
            ],
        );
    } else {
        git(
            repo.as_path(),
            &["init", "--quiet", "--initial-branch=main"],
        );
    }
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("a.txt"), "two\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    return repo;
}

/// The two kinds of declined repository,
/// built under one fixture.
fn both_kinds(root: &Path) -> Vec<(&'static str, PathBuf)> {
    return vec![
        ("reftable", a_declined_repository(root, "reftable")),
        (
            "files with a replace ref",
            a_declined_repository(root, "files"),
        ),
    ];
}

/// The commit names of a repository's last two commits,
/// the older first.
fn last_two(repo: &Path) -> (String, String) {
    return (
        git_text(repo, &["rev-parse", "HEAD~1"]),
        git_text(repo, &["rev-parse", "HEAD"]),
    );
}

/// One real Git run with input,
/// in the hermetic fixture environment.
fn git_with_input(
    repo: &Path,
    arguments: &[&str],
    input: &str,
) -> Vec<u8> {
    let environment: Vec<(OsString, OsString)> = fixture_environment(repo);
    let words: Vec<OsString> = arguments
        .iter()
        .map(|argument| {
            return OsString::from(argument);
        })
        .collect();
    let answer = real_git_within(
        Path::new(REAL_GIT),
        Some(repo),
        environment.as_slice(),
        words.as_slice(),
        Some(input.as_bytes()),
        FIXTURE_GIT_BOUND,
    );
    assert!(
        answer.output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(answer.output.stderr.as_slice())
    );
    return answer.output.stdout;
}

/// The environment a fixture Git runs with.
fn fixture_environment(repo: &Path) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (
            OsString::from("GIT_CONFIG_GLOBAL"),
            OsString::from("/nonexistent-global-config"),
        ),
        (
            OsString::from("HOME"),
            repo.join("no-home").into_os_string(),
        ),
    ];
}

/// Drop a loose object the fixture Git wrote.
fn remove_loose_object(
    repo: &Path,
    name: &str,
) {
    let path: PathBuf = repo
        .join(".git")
        .join("objects")
        .join(&name[..2])
        .join(&name[2..]);
    std::fs::remove_file(path.as_path())
        .unwrap_or_else(|error| panic!("remove {}: {error}", path.display()));
}

/// A tree of the fixture's object store with one blob below a subtree.
fn a_tree_with_one_blob(repo: &Path) -> (String, String) {
    let blob: String = String::from_utf8(git_with_input(
        repo,
        &["hash-object", "-w", "--stdin"],
        "gone\n",
    ))
    .expect("UTF-8")
    .trim()
    .to_owned();
    let subtree_line: String = format!("100644 blob {blob}\tsub.txt\n");
    let subtree: String =
        String::from_utf8(git_with_input(repo, &["mktree"], subtree_line.as_str()))
            .expect("UTF-8")
            .trim()
            .to_owned();
    let root_line: String = format!("040000 tree {subtree}\tsub\n");
    let root: String = String::from_utf8(git_with_input(repo, &["mktree"], root_line.as_str()))
        .expect("UTF-8")
        .trim()
        .to_owned();
    return (root, blob);
}

/// `commit_bytes` prints the commit Git reads in a declined repository,
/// the replacement a replace ref names included,
/// and reads nothing for a name Git has no commit of.
#[test]
fn commit_bytes_answers_with_the_commit_git_reads() {
    let root: PathBuf = fixture("object-questions-commit-bytes");
    for (kind, repo) in both_kinds(root.as_path()) {
        let (older, _head) = last_two(repo.as_path());
        let expected: Vec<u8> =
            git_output(repo.as_path(), &["cat-file", "commit", older.as_str()]).stdout;
        assert_eq!(
            commit_bytes(
                &crate::test_support::git_context(),
                repo.as_path(),
                older.as_str()
            ),
            Some(expected),
            "{kind}: the bytes Git prints"
        );
        // Positive control: the fixture's replacement makes Git's answer differ from the object the files hold.
        let unreplaced: Vec<u8> = git_output(
            repo.as_path(),
            &["--no-replace-objects", "cat-file", "commit", older.as_str()],
        )
        .stdout;
        assert_ne!(
            git_output(repo.as_path(), &["cat-file", "commit", older.as_str()]).stdout,
            unreplaced,
            "{kind}: the fixture replaces the older commit"
        );
        let absent: String = "1".repeat(older.len());
        assert_eq!(
            commit_bytes(
                &crate::test_support::git_context(),
                repo.as_path(),
                absent.as_str()
            ),
            None,
            "{kind}: a name Git has no commit of"
        );
    }
    remove(root.as_path());
}

/// `objects_missing` sees a blob a pruned subtree names in a declined repository:
/// the subtree is present,
/// the blob below it is gone,
/// so the whole tree is not complete.
#[test]
fn objects_missing_sees_a_blob_a_subtree_names() {
    let root: PathBuf = fixture("object-questions-objects-missing");
    for (kind, repo) in both_kinds(root.as_path()) {
        let common_dir: PathBuf = repo.join(".git");
        let (tree, blob) = a_tree_with_one_blob(repo.as_path());
        let context = crate::test_support::git_context();
        assert_eq!(
            objects_missing(
                &context,
                repo.as_path(),
                common_dir.as_path(),
                &[tree.as_str()]
            ),
            Some(false),
            "{kind}: the tree is complete"
        );
        remove_loose_object(repo.as_path(), blob.as_str());
        // Positive control: Git itself no longer finds the blob the subtree names.
        assert_eq!(
            git_output(repo.as_path(), &["cat-file", "-e", blob.as_str()])
                .status
                .code(),
            Some(1),
            "{kind}: the blob is gone"
        );
        assert_eq!(
            objects_missing(
                &context,
                repo.as_path(),
                common_dir.as_path(),
                &[tree.as_str()]
            ),
            Some(true),
            "{kind}: the subtree names a gone blob"
        );
    }
    remove(root.as_path());
}

/// A request larger than one command line still reaches Git,
/// because the names travel on standard input.
/// Why:
///  One name on the command line costs its own bytes plus the terminator,
/// so the count is derived from the host's own limit rather than a guessed number.
#[test]
fn a_request_too_large_for_one_command_line_reaches_git() {
    let root: PathBuf = fixture("object-questions-large-request");
    let repo: PathBuf = a_declined_repository(root.as_path(), "files");
    let common_dir: PathBuf = repo.join(".git");
    let context = crate::test_support::git_context();
    // SAFETY: `sysconf` only reads the process's argument limit and touches no memory of ours.
    let host_limit: i64 = unsafe { libc::sysconf(libc::_SC_ARG_MAX) };
    let limit: usize =
        usize::try_from(host_limit).expect("the host answers a positive argument limit");
    let name: usize = 40;
    let count: usize = limit / (name + 1) + 1;
    let names: Vec<String> = (1..=count)
        .map(|index| {
            return format!("{index:0name$}", name = name);
        })
        .collect();
    let bytes: usize = count * (name + 1);
    assert!(
        bytes > limit,
        "the request's names would not fit one command line: {bytes} bytes against {limit}"
    );
    let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
    assert_eq!(
        objects_missing(
            &context,
            repo.as_path(),
            common_dir.as_path(),
            borrowed.as_slice()
        ),
        Some(true),
        "a request this large still reaches Git"
    );
    remove(root.as_path());
}

/// `object_missing` sees a commit Git built once its object is gone,
/// and a present one is not missing.
#[test]
fn object_missing_sees_a_gone_commit_git_built() {
    let root: PathBuf = fixture("object-questions-object-missing");
    for (kind, repo) in both_kinds(root.as_path()) {
        let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
        let built: String = git_text(
            repo.as_path(),
            &["commit-tree", tree.as_str(), "-m", "built"],
        );
        let context = crate::test_support::git_context();
        assert!(
            !object_missing(&context, repo.as_path(), built.as_str()),
            "{kind}: the commit is present"
        );
        remove_loose_object(repo.as_path(), built.as_str());
        assert_eq!(
            git_output(repo.as_path(), &["cat-file", "-e", built.as_str()])
                .status
                .code(),
            Some(1),
            "{kind}: the commit is gone"
        );
        assert!(
            object_missing(&context, repo.as_path(), built.as_str()),
            "{kind}: the gone commit is missing"
        );
    }
    remove(root.as_path());
}

/// `built_commits_missing` asks each commit of a request:
/// one gone commit among present ones is missing,
/// present ones are not,
/// and no commit is not.
#[test]
fn built_commits_missing_asks_every_commit() {
    let root: PathBuf = fixture("object-questions-built-commits");
    for (kind, repo) in both_kinds(root.as_path()) {
        let (_older, head) = last_two(repo.as_path());
        let absent: String = "2".repeat(head.len());
        let context = crate::test_support::git_context();
        assert!(
            !built_commits_missing(&context, repo.as_path(), &[head.as_str()]),
            "{kind}: the commit is present"
        );
        assert!(
            built_commits_missing(&context, repo.as_path(), &[head.as_str(), absent.as_str()]),
            "{kind}: the second commit is gone"
        );
        assert!(
            !built_commits_missing(&context, repo.as_path(), &[]),
            "{kind}: no commit is asked"
        );
    }
    remove(root.as_path());
}

/// What one landing begins with,
/// as the service's landing entry begins it:
/// the bytes a commit read before it are not answered from.
fn begin_a_landing() {
    crate::landing_state_carried_by_its_task::forget_the_commits_read_by_an_earlier_landing();
}

/// A landing asks Git once for a commit it reads twice,
/// and the answer is still the bytes Git prints.
#[test]
fn a_landing_reads_a_commit_once() {
    let root: PathBuf = fixture("object-questions-read-once");
    let repo: PathBuf = a_declined_repository(root.as_path(), "reftable");
    let (_older, head) = last_two(repo.as_path());
    let (context, log) = recording_context(root.as_path());
    let answer: Vec<u8> = crate::landing_state_carried_by_its_task::in_a_landing(|| {
        begin_a_landing();
        let first: Vec<u8> = commit_bytes(&context, repo.as_path(), head.as_str()).expect("read");
        let second: Vec<u8> = commit_bytes(&context, repo.as_path(), head.as_str()).expect("read");
        assert_eq!(first, second, "the second read answers the first");
        return first;
    });
    assert_eq!(
        answer,
        git_output(repo.as_path(), &["cat-file", "commit", head.as_str()]).stdout,
        "the bytes Git prints"
    );
    let starts: Vec<String> = recorded_starts(log.as_path());
    assert_eq!(
        starts.len(),
        1,
        "one Git start for one commit read twice: {starts:?}"
    );
    assert!(
        starts[0].contains("cat-file commit"),
        "the start is the commit question: {starts:?}"
    );
    remove(root.as_path());
}

/// What:
///  A landing does not answer with the bytes an earlier landing read,
///  so a replace ref added between two landings is what the second one's commit question reads.
/// Why:
///  The state the memory lives in belongs to the service's loop,
/// which enters it once (`land_batch_timed`),
///  so a memory that is not emptied hands the first landing's bytes to every later one:
///  an amend of that commit then stands on the commit's own parents where Git reads the replacement's.
#[test]
fn a_landing_does_not_answer_with_the_commits_an_earlier_landing_read() {
    let root: PathBuf = fixture("object-questions-two-landings");
    let repo: PathBuf = a_declined_repository(root.as_path(), "reftable");
    let (_older, head) = last_two(repo.as_path());
    let (context, log) = recording_context(root.as_path());
    // One scope around both landings, which is the one the service's loop enters.
    crate::landing_state_carried_by_its_task::in_a_landing(|| {
        begin_a_landing();
        let first: Vec<u8> = commit_bytes(&context, repo.as_path(), head.as_str()).expect("read");
        begin_a_landing();
        let second: Vec<u8> = commit_bytes(&context, repo.as_path(), head.as_str()).expect("read");
        assert_eq!(first, second, "both landings read the bytes Git prints");
    });
    let starts: Vec<String> = recorded_starts(log.as_path());
    assert_eq!(
        starts.len(),
        2,
        "each landing asks Git for the commit again: {starts:?}"
    );
    remove(root.as_path());
}

/// The names the cut-write control asks about:
/// two object names,
/// so the input file holds more bytes than the child may write.
const TWO_NAMES: [&str; 2] = [
    "1111111111111111111111111111111111111111",
    "2222222222222222222222222222222222222222",
];

/// The number of bytes the child that cuts a write short may write into one file:
/// fewer than the two names it asks about need.
const A_CUT_WRITE_LIMIT: u64 = 64;

/// What:
///  A question whose input file cannot be made is not an answer.
/// Why:
///  `objects_missing` writes the names into a private file and hands Git that file;
///  when the file cannot be made,
///  nothing was asked,
///  so saying "not missing" would report content present that nothing looked at.
///  The caller keeps its own answer instead.
#[test]
fn a_question_whose_input_file_cannot_be_made_is_not_an_answer() {
    let root: PathBuf = fixture("object-questions-input-failed");
    let repo: PathBuf = a_declined_repository(root.as_path(), "files");
    // A regular file where the repository's scratch directory belongs: no file can be made there.
    let blocked: PathBuf = root.join("not-a-directory");
    std::fs::write(blocked.as_path(), "x").expect("file");
    let context = crate::test_support::git_context();
    assert_eq!(
        objects_missing(
            &context,
            repo.as_path(),
            blocked.as_path(),
            TWO_NAMES.as_slice()
        ),
        None,
        "a question that could not be asked is not an answer"
    );
    remove(root.as_path());
}

/// What:
///  A write of the input file that is cut short is not an answer either,
///  and leaves no part of the file behind.
/// Why:
///  A full disk or a file-size limit cuts the write,
///  and a partial file would be the input of a later command;
///  the question says it failed and the scratch directory keeps nothing.
#[test]
fn a_question_whose_input_write_is_cut_short_is_not_an_answer_and_leaves_nothing() {
    let root: PathBuf = fixture("object-questions-input-cut");
    let repo: PathBuf = a_declined_repository(root.as_path(), "files");
    // The repository is made here, before the child's file-size limit exists, and the child asks the question in it.
    let status: std::process::ExitStatus =
        crate::file_size_limit_child_test_support::run_child_under_file_size_limit(
            "landing::service_object_questions_left_to_git::tests::child_asks_the_objects_question_under_a_file_size_limit",
            A_CUT_WRITE_LIMIT,
            root.as_path(),
        );
    assert!(
        status.success(),
        "the child's question must be a failure, not an answer: {status:?}"
    );
    let scratch: PathBuf = repo
        .join(".git")
        .join(crate::scratch_location::REPOSITORY_SCRATCH_NAME);
    let left: Vec<String> = match std::fs::read_dir(scratch.as_path()) {
        Ok(listing) => listing
            .flatten()
            .map(|entry| {
                return entry.file_name().to_string_lossy().into_owned();
            })
            .collect(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => panic!(
            "the scratch directory {} cannot be read ({error})",
            scratch.display()
        ),
    };
    assert_eq!(
        left,
        Vec::<String>::new(),
        "no part of the cut input file stays"
    );
    remove(root.as_path());
}

/// Child of the control above:
/// ask the objects question in a process that may write no file past the limit,
/// so the write of the input file is cut short.
#[test]
#[ignore = "started as a child process, under a file-size limit, by the control beside it"]
fn child_asks_the_objects_question_under_a_file_size_limit() {
    let root: PathBuf = PathBuf::from(
        std::env::var_os(crate::file_size_limit_child_test_support::LIMITED_DIRECTORY_VARIABLE)
            .expect("directory"),
    );
    let repo: PathBuf = root.join("files");
    let common_dir: PathBuf = repo.join(".git");
    let context = crate::test_support::git_context();
    assert_eq!(
        objects_missing(
            &context,
            repo.as_path(),
            common_dir.as_path(),
            TWO_NAMES.as_slice()
        ),
        None,
        "the input file was written whole, so the limit is not cutting this write"
    );
}
