//! What:
//!  The name a forced `checkout`, `switch`, `reset` or `restore` writes the tree of,
//!  and what the reading answers when Git cannot list that tree.
//! Why:
//!  `git ls-tree topic` fails for a branch that exists only as `refs/remotes/origin/topic`,
//!  while `git checkout -f topic` creates the local branch from it and writes that tree
//!  (measured on this host with native Git 2.56.0,
//!  2026-11-13);
//!  a reading that took the failed listing for "this form writes nothing" let the forced form
//!  overwrite an untracked file it never looked at,
//!  which is the loss these controls keep closed.
//!  The other half is the opposite mistake:
//!  a tree Git knows and cannot list is a refusal,
//!  never an empty answer,
//!  because what such a command writes is unknown rather than nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pathsTheTreeWrites(facts, { kind: 'tree', revision: 'topic', guesses: true }) === ['b.txt']
//! // pathsTheTreeWrites(facts, { kind: 'tree', revision: 'topic', guesses: false }) === []
//! ```

use super::paths_the_tree_writes;
use crate::command_test_support::{git, repository_with_tracked_file};
use crate::command_written_paths::{PathspecTarget, WrittenEntries, WrittenPaths};
use crate::repo_read::repository_facts::{GitFacts, git_facts};
use crate::test_support::{REAL_GIT, isolated_overlay, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A provider for real Git after `-C <repository>`.
fn facts_in(repository: &Path) -> GitFacts {
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repository.as_os_str().to_os_string()];
    return git_facts(Path::new(REAL_GIT), prefix.as_slice(), isolated_overlay().as_slice());
}

/// The written-path reading of a whole tree the form names,
/// with the guess of a remote branch set the way a `checkout` or a `reset` sets it.
fn tree_of(repository: &Path, revision: &str, guesses: bool) -> Result<Vec<Vec<u8>>, String> {
    let mut facts: GitFacts = facts_in(repository);
    return paths_the_tree_writes(
        &mut facts,
        &WrittenPaths::Tree {
            revision: Some(OsString::from(revision)),
            entries: WrittenEntries::EveryEntry,
            guesses_a_remote_branch: guesses,
        },
        repository,
    );
}

/// What:
///  A repository whose `topic` commit holds `b.txt` and whose only ref to it is a remote-tracking one,
/// with a linked worktree on `main`.
/// Why:
///  That is the state a forced `checkout topic` guesses the local branch from,
///  and the state `git ls-tree topic` cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = repositoryWithOnlyARemoteTrackingBranch('remote-only-branch', 'origin');
/// ```
fn repository_with_only_a_remote_tracking_branch(name: &str, remote: &str) -> (PathBuf, PathBuf) {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_tracked_file(name);
    git(repository.as_path(), &["checkout", "--quiet", "-b", "topic"]);
    std::fs::write(repository.join("b.txt"), b"from the topic tree\n").expect("the topic's file");
    git(repository.as_path(), &["add", "--", "b.txt"]);
    git(repository.as_path(), &["commit", "--quiet", "--message=topic", "--", "b.txt"]);
    let topic: String = String::from_utf8_lossy(&git(repository.as_path(), &["rev-parse", "topic"]).stdout).trim().to_owned();
    git(repository.as_path(), &["checkout", "--quiet", "main"]);
    git(repository.as_path(), &["branch", "--delete", "--force", "topic"]);
    let reference: String = format!("refs/remotes/{remote}/topic");
    git(repository.as_path(), &["update-ref", reference.as_str(), topic.as_str()]);
    // Git guesses only from the remote-tracking branch of a configured remote: with no
    //  `remote.<name>` at all it refuses the short name
    //  (`error: pathspec 'topic' did not match any file(s) known to git`, measured on this host
    //  with native Git 2.56.0, 2026-11-13).
    git(repository.as_path(), &["remote", "add", remote, "/nonexistent-remote"]);
    return (directory, repository);
}

/// The paths of the `topic` commit's tree:
/// the file it adds and the one it inherited from `main`.
fn the_topics_tree() -> Vec<Vec<u8>> {
    return vec![b"b.txt".to_vec(), b"tracked.txt".to_vec()];
}

/// A forced `checkout` or `switch` writes the tree of the branch it creates from a remote-tracking
/// branch of that name,
/// so the reading names that tree's paths.
///  `git ls-tree topic` itself exits nonzero in this state,
///  which is the control that this test needs the guess.
#[test]
fn a_remote_tracking_branch_of_the_name_is_the_tree_a_checkout_writes() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-remote-only", "origin");
    // The control: the listing Git does without the guess fails, so an empty answer would be wrong.
    let direct = crate::command_test_support::git_status(repository.as_path(), &["ls-tree", "-r", "--name-only", "topic"]);
    assert!(!direct.status.success(), "the control needs `git ls-tree topic` to fail here: {direct:?}");
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(the_topics_tree()));
    // A reset names a revision and never the branch a short name stands for.
    assert_eq!(tree_of(repository.as_path(), "topic", false), Ok(Vec::<Vec<u8>>::new()));
    // A name Git resolves without any guess is read with and without it.
    assert_eq!(tree_of(repository.as_path(), "origin/topic", true), Ok(the_topics_tree()));
    assert_eq!(tree_of(repository.as_path(), "origin/topic", false), Ok(the_topics_tree()));
    remove(directory.as_path());
}

/// A remote's own name may hold a slash,
/// and Git guesses the branch from the ref that begins with that name.
#[test]
fn a_remote_whose_name_holds_a_slash_is_read_whole() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-slashed-remote", "a/b");
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(the_topics_tree()));
    remove(directory.as_path());
}

/// Two remotes carrying the branch are an ambiguity Git refuses,
/// and `checkout.defaultRemote` names the one Git then writes.
#[test]
fn two_remotes_are_ambiguous_unless_the_default_remote_names_one() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-two-remotes", "origin");
    let topic: String = String::from_utf8_lossy(&git(repository.as_path(), &["rev-parse", "refs/remotes/origin/topic"]).stdout).trim().to_owned();
    git(repository.as_path(), &["update-ref", "refs/remotes/second/topic", topic.as_str()]);
    git(repository.as_path(), &["remote", "add", "second", "/nonexistent-second"]);
    // The control: real Git refuses the ambiguous name and writes nothing.
    let direct = crate::command_test_support::git_status(repository.as_path(), &["ls-tree", "-r", "--name-only", "topic"]);
    assert!(!direct.status.success());
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(Vec::<Vec<u8>>::new()));
    git(repository.as_path(), &["config", "checkout.defaultRemote", "second"]);
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(the_topics_tree()));
    remove(directory.as_path());
}

/// `checkout.guess=false` makes Git refuse the short name,
/// so the reading resolves nothing and leaves the command to Git.
#[test]
fn a_repository_that_turns_the_guess_off_resolves_nothing() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-guess-off", "origin");
    git(repository.as_path(), &["config", "checkout.guess", "false"]);
    let direct = crate::command_test_support::git_status(repository.as_path(), &["ls-tree", "-r", "--name-only", "topic"]);
    assert!(!direct.status.success());
    assert_eq!(tree_of(repository.as_path(), "topic", true), Ok(Vec::<Vec<u8>>::new()));
    remove(directory.as_path());
}

/// A name Git knows nothing of writes no path:
/// Git refuses the form itself,
/// and the guard must not refuse a command Git would not have run.
#[test]
fn a_name_git_does_not_know_writes_no_path() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-unknown-name", "origin");
    assert_eq!(tree_of(repository.as_path(), "nosuchbranch", true), Ok(Vec::<Vec<u8>>::new()));
    remove(directory.as_path());
}

/// What:
///  A tree Git knows and cannot list is a failure of the reading,
///  never "this form writes nothing".
/// Why:
///  The two answers mean opposite things for a guard:
///  a name Git does not know is a form Git refuses itself,
///  while an object it knows and cannot read says nothing about what the command would write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // remove the tree object of `other`, then pathsTheTreeWrites(facts, 'other') throws
/// ```
#[test]
fn a_tree_that_cannot_be_listed_is_a_failure() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch("written-unreadable-tree", "origin");
    let tree: String = String::from_utf8_lossy(&git(repository.as_path(), &["rev-parse", "refs/remotes/origin/topic^{tree}"]).stdout).trim().to_owned();
    let object: PathBuf = repository
        .join(".git")
        .join("objects")
        .join(&tree[..2])
        .join(&tree[2..]);
    std::fs::remove_file(object.as_path()).expect("the tree object");
    let answer: Result<Vec<Vec<u8>>, String> = tree_of(repository.as_path(), "origin/topic", false);
    match answer {
        Err(cause) => assert!(cause.contains("could not read the tree `origin/topic` names"), "{cause}"),
        Ok(found) => panic!("an unreadable tree was read as {found:?}"),
    }
    remove(directory.as_path());
}

//region The second review's readings of a pathspec file and of a killed query

/// The written-path reading of a pathspec file one command names.
fn from_the_file(
    repository: &Path,
    file: &str,
    target: PathspecTarget,
    nul_separated: bool,
) -> Result<Vec<Vec<u8>>, String> {
    let mut facts: GitFacts = facts_in(repository);
    return paths_the_tree_writes(
        &mut facts,
        &WrittenPaths::PathspecFile {
            target,
            file: OsString::from(file),
            nul_separated,
            entries: WrittenEntries::EveryEntry,
            guesses_a_remote_branch: true,
        },
        repository,
    );
}

/// What:
///  A repository whose `topic` commit holds `b.txt`, reachable as `refs/remotes/origin/topic`,
/// with a remote configured, and a directory for the lists the controls write.
/// Why:
///  A forced `checkout` of such a branch writes over an untracked file at its tree's paths,
///  and its name is the one a zero-byte pathspec file turns back into a whole-tree write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { directory, repository } = fixtureWithARemoteBranch('facts-pathspec-file', 'origin');
/// ```
fn fixture_with_a_remote_branch(name: &str) -> (PathBuf, PathBuf) {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_only_a_remote_tracking_branch(name, "origin");
    return (directory, repository);
}

/// A zero-byte pathspec file is no pathspec at all to Git,
/// which then switches the branch by force and writes the whole tree,
/// so the reading answers the tree's paths and not nothing.
#[test]
fn an_empty_pathspec_file_is_the_whole_tree_git_writes() {
    let (directory, repository): (PathBuf, PathBuf) = fixture_with_a_remote_branch("facts-empty-pathspec-file");
    let list: PathBuf = directory.join("empty-list");
    std::fs::write(list.as_path(), b"").expect("the zero-byte list");
    // The control: Git does not resolve `topic` on its own, so the answer needs the guess as well.
    assert_eq!(
        from_the_file(repository.as_path(), list.to_str().expect("utf-8 path"), PathspecTarget::Tree(Some(OsString::from("topic"))), false),
        Ok(the_topics_tree())
    );
    // `restore` refuses an empty list itself, so its answer is nothing.
    assert_eq!(
        from_the_file(repository.as_path(), list.to_str().expect("utf-8 path"), PathspecTarget::Index, false),
        Ok(Vec::<Vec<u8>>::new())
    );
    // A checkout with no tree-ish is a no-op to Git, so it writes nothing either.
    assert_eq!(
        from_the_file(repository.as_path(), list.to_str().expect("utf-8 path"), PathspecTarget::Tree(None), false),
        Ok(Vec::<Vec<u8>>::new())
    );
    remove(directory.as_path());
}

/// A line that begins with a double quote is the C-quoted name Git reads,
/// so the reading asks about that name and not about the quoted text.
#[test]
fn a_quoted_entry_is_the_name_git_reads() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-quoted-pathspec");
    let tabbed: String = String::from("a\tb.txt");
    git(repository.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repository.join(tabbed.as_str()), b"from the other tree\n").expect("the tabbed name");
    git(repository.as_path(), &["add", "--", tabbed.as_str()]);
    git(repository.as_path(), &["commit", "--quiet", "--message=tabbed"]);
    git(repository.as_path(), &["checkout", "--quiet", "main"]);
    let quoted: PathBuf = directory.join("quoted-list");
    std::fs::write(quoted.as_path(), b"\"a\\tb.txt\"\n").expect("the quoted list");
    let written: Result<Vec<Vec<u8>>, String> = from_the_file(
        repository.as_path(),
        quoted.to_str().expect("utf-8 path"),
        PathspecTarget::Tree(Some(OsString::from("other"))),
        false,
    );
    assert_eq!(written, Ok(vec![b"a\tb.txt".to_vec()]), "the quoted entry is the name Git reads");
    remove(directory.as_path());
}

/// An empty line and a line Git calls badly quoted are Git's own refusals,
/// so the reading refuses rather than answering that nothing is written.
#[test]
fn an_empty_or_badly_quoted_line_is_a_refusal() {
    let (directory, repository): (PathBuf, PathBuf) = fixture_with_a_remote_branch("facts-bad-lines");
    for (name, contents, wanted) in [
        ("empty-line", b"b.txt\n\n".to_vec(), "holds an empty pathspec"),
        ("bad-quote", b"\"b\\qb.txt\"\n".to_vec(), "badly quoted"),
        ("unterminated", b"\"b.txt\n".to_vec(), "badly quoted"),
    ] {
        let list: PathBuf = directory.join(name);
        std::fs::write(list.as_path(), contents.as_slice()).expect("the list");
        let answer: Result<Vec<Vec<u8>>, String> = from_the_file(
            repository.as_path(),
            list.to_str().expect("utf-8 path"),
            PathspecTarget::Tree(Some(OsString::from("origin/topic"))),
            false,
        );
        match answer {
            Err(cause) => assert!(cause.contains(wanted), "{name}: {cause}"),
            Ok(found) => panic!("{name} was read as {found:?}"),
        }
    }
    remove(directory.as_path());
}

/// What:
///  A real Git replaced by a program that ends itself on one query and is Git for every other.
///  `&str` names the word the shim dies on.
/// Why:
///  A child a signal ends answers nothing at all,
///  which is not the answer "Git does not know the name" and not "the repository leaves guessing off";
///  the guard must refuse rather than read a killed probe as nothing to guard.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shim = writeShimThatDiesOn('checkout.guess', realGitPath);
/// ```
fn a_git_that_dies_on(word: &str, directory: &Path) -> PathBuf {
    let shim: PathBuf = directory.join("git-that-dies");
    let script: String = format!(
        "#!/bin/sh\nfor each in \"$@\"; do\n  if [ \"$each\" = \"{word}\" ]; then kill -9 $$; fi\ndone\nexec {REAL_GIT} \"$@\"\n"
    );
    std::fs::write(shim.as_path(), script.as_bytes()).expect("the shim");
    let mut permissions = std::fs::metadata(shim.as_path()).expect("the shim").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    std::fs::set_permissions(shim.as_path(), permissions).expect("the shim runs");
    return shim;
}

/// A query the system ended by a signal is a failure of the reading,
/// for the listing,
/// for the question whether Git knows the name,
/// and for the repository's setting on guessing a remote branch.
#[test]
fn a_query_a_signal_ended_is_a_failure_and_never_an_answer() {
    let (directory, repository): (PathBuf, PathBuf) = fixture_with_a_remote_branch("facts-killed-queries");
    for word in ["checkout.guess", "ls-tree", "rev-parse"] {
        let shim: PathBuf = a_git_that_dies_on(word, directory.as_path());
        let prefix: Vec<OsString> = vec![OsString::from("-C"), repository.as_os_str().to_os_string()];
        // The in-process readers answer `config` without starting a child, and a control about a killed child needs
        //  the child: the same switch the wrapper's own README documents for that.
        let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
        overlay.push((OsString::from(crate::repo_read_hook::SWITCH_VARIABLE), OsString::from("off")));
        let mut facts: GitFacts = git_facts(shim.as_path(), prefix.as_slice(), overlay.as_slice());
        // The shape the review's case has: a forced `checkout` of a remote-only name, which asks the repository
        //  whether it leaves the guess on before it can read the tree.
        let answer: Result<Vec<Vec<u8>>, String> = paths_the_tree_writes(
            &mut facts,
            &WrittenPaths::Tree {
                revision: Some(OsString::from("topic")),
                entries: WrittenEntries::EveryEntry,
                guesses_a_remote_branch: true,
            },
            repository.as_path(),
        );
        match answer {
            Err(cause) => assert!(cause.contains("ended by a signal"), "{word}: {cause}"),
            Ok(found) => panic!("a {word} child a signal ended was read as {found:?}"),
        }
    }
    remove(directory.as_path());
}

/// What:
///  A pipe named by `--pathspec-from-file` is still read,
///  which is what a process substitution names.
/// Why:
///  The byte bound is on the read and not on the kind of file:
///  `--pathspec-from-file=<(printf 'b.txt\n')` is a spelling Git runs,
///  so refusing a pipe would refuse a command that writes what its words say.
///  A writer is started first, so the read has its bytes and no wait is possible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fromTheFile(repo, fifoWrittenWith('b.txt\n'), tree('other')) === ['b.txt']
/// ```
#[test]
fn a_pipe_the_form_names_is_read() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-pathspec-file-pipe");
    git(repository.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repository.join("b.txt"), b"from the other tree\n").expect("the tracked file");
    git(repository.as_path(), &["add", "--", "b.txt"]);
    git(repository.as_path(), &["commit", "--quiet", "--message=other"]);
    git(repository.as_path(), &["checkout", "--quiet", "main"]);
    let pipe: PathBuf = directory.join("list.pipe");
    let made: std::process::ExitStatus = std::process::Command::new("mkfifo")
        .arg(pipe.as_path())
        .status()
        .expect("mkfifo runs");
    assert!(made.success(), "mkfifo must create {}", pipe.display());
    // The writer waits for the read, as a process substitution's own writer does.
    let writer: std::process::Child = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("printf 'b.txt\\n' > {}", pipe.display()))
        .spawn()
        .expect("the writer starts");
    let answer: Result<Vec<Vec<u8>>, String> = from_the_file(
        repository.as_path(),
        pipe.to_str().expect("utf-8 path"),
        PathspecTarget::Tree(Some(OsString::from("other"))),
        false,
    );
    let mut writer: std::process::Child = writer;
    let _ = writer.wait();
    assert_eq!(answer, Ok(vec![b"b.txt".to_vec()]), "a pipe's bytes are the pathspecs Git reads");
    remove(directory.as_path());
}

//endregion

//region The shapes a forced move has, each measured against real Git

/// What:
///  Every entry below the worktree root, with the bytes it holds,
///  in one stable order.
/// Why:
///  What real Git writes for one move of one shape is the set of entries that appear or change;
///  that set is the oracle the reading's written paths are compared with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = (root) => walk(root).map((p) => `${p}:${contentHash(p)}`).sort();
/// ```
fn the_worktree_state(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::<String>::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.file_name() == std::ffi::OsStr::new(".git") {
                continue;
            }
            let path: PathBuf = entry.path();
            let shown: String = path.strip_prefix(root).unwrap_or(path.as_path()).display().to_string();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                found.push(format!("dir {shown}"));
                pending.push(path);
                continue;
            }
            let bytes: Vec<u8> = std::fs::read(path.as_path()).unwrap_or_default();
            found.push(format!(
                "file {shown} {}",
                bytes.iter().map(|byte| return format!("{byte:02x}")).collect::<String>()
            ));
        }
    }
    found.sort();
    return found;
}

/// The written paths one forced move answers, as text.
fn moved_paths(repository: &Path, sources: &[&str], destination: &str) -> Result<Vec<String>, String> {
    let mut facts: GitFacts = facts_in(repository);
    let answered: Result<Vec<Vec<u8>>, String> = paths_the_tree_writes(
        &mut facts,
        &WrittenPaths::MoveInto {
            sources: sources.iter().map(|source| return OsString::from(*source)).collect(),
            destination: OsString::from(destination),
        },
        repository,
    );
    return answered.map(|paths| {
        return paths.iter().map(|path| return String::from_utf8_lossy(path).into_owned()).collect::<Vec<String>>();
    });
}

/// What:
///  The paths real Git writes for one forced move of one shape,
///  read as the entries that appeared or changed,
///  and the move's own exit code.
/// Why:
///  A reading that names a path Git did not write is the false refusal the third review found,
///  and the only oracle for "what Git writes" is Git's own run on the same shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const realMove = (repo, sources, destination) => { before; git mv -f; diff(before, after) };
/// ```
fn what_real_git_wrote(repository: &Path, sources: &[&str], destination: &str) -> (Option<i32>, Vec<(bool, String)>) {
    let before: Vec<String> = the_worktree_state(repository);
    let mut arguments: Vec<String> = vec![String::from("mv"), String::from("-f")];
    arguments.extend(sources.iter().map(|source| return String::from(*source)));
    arguments.push(String::from(destination));
    let words: Vec<&str> = arguments.iter().map(|word| return word.as_str()).collect::<Vec<&str>>();
    let ran = crate::command_test_support::git_status(repository, words.as_slice());
    let after: Vec<String> = the_worktree_state(repository);
    let mut written: Vec<(bool, String)> = Vec::<(bool, String)>::new();
    for entry in &after {
        if before.contains(entry) {
            continue;
        }
        // An entry of the state is `<kind> <path>` for a directory or `<kind> <path> <content>` for a file.
        let mut parts = entry.splitn(3, ' ');
        let kind: &str = parts.next().unwrap_or_default();
        let path: &str = parts.next().unwrap_or_default();
        written.push((kind == "dir", String::from(path)));
    }
    let code: Option<i32> = ran.status.code();
    return (code, written);
}

/// What:
///  One shape of a forced `git mv`: its label, its setup, its words.
///  A `struct` is a record with named fields.
/// Why:
///  The shapes are a table the third review asked for, and one row per shape keeps the oracle and the
///  reading side by side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rows = [{ label: 'one-source-into-an-untracked-dir', setup, sources: ['a.txt'], destination: 'b' }];
/// ```
struct MoveShape {
    /// What the row is called.
    label: &'static str,
    /// The entries it puts in the worktree before the move.
    setup: fn(&Path),
    /// The sources the move names.
    sources: &'static [&'static str],
    /// The destination it names.
    destination: &'static str,
}

/// A repository with the entries every row starts from:
/// a tracked `a.txt`, `x.txt` and `d1/inner.txt`.
fn a_repository_for_moving(name: &str) -> (PathBuf, PathBuf) {
    let (directory, repo): (PathBuf, PathBuf) = repository_with_tracked_file(name);
    std::fs::write(repo.join("a.txt"), b"tracked a\n").expect("a.txt");
    std::fs::write(repo.join("x.txt"), b"tracked x\n").expect("x.txt");
    std::fs::create_dir(repo.join("d1")).expect("d1");
    std::fs::write(repo.join("d1/inner.txt"), b"inside d1\n").expect("d1/inner.txt");
    git(repo.as_path(), &["add", "--", "a.txt", "x.txt", "d1/inner.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    return (directory, repo);
}

/// The shapes a forced move has, each compared with what real Git does on the same shape:
/// a destination directory is moved into and never written over,
/// a destination that is not one is renamed onto,
/// and the shapes Git refuses write nothing, so the guard names nothing and Git answers.
#[test]
fn every_shape_of_a_forced_move_is_read_the_way_git_writes_it() {
    fn no_setup(_repo: &Path) {}
    fn an_untracked_directory(repo: &Path) {
        std::fs::create_dir(repo.join("b")).expect("b");
        std::fs::write(repo.join("b/keep.txt"), b"keep\n").expect("b/keep.txt");
    }
    fn a_tracked_directory(repo: &Path) {
        std::fs::create_dir(repo.join("t")).expect("t");
        std::fs::write(repo.join("t/keep.txt"), b"keep\n").expect("t/keep.txt");
        git(repo, &["add", "--", "t/keep.txt"]);
        git(repo, &["commit", "--quiet", "--message=t"]);
    }
    fn a_tracked_file_at_the_destination(repo: &Path) {
        std::fs::write(repo.join("dest.txt"), b"dest\n").expect("dest.txt");
        git(repo, &["add", "--", "dest.txt"]);
        git(repo, &["commit", "--quiet", "--message=dest"]);
    }
    fn a_directory_at_the_destination(repo: &Path) {
        std::fs::create_dir(repo.join("d2")).expect("d2");
        std::fs::write(repo.join("d2/keep.txt"), b"keep\n").expect("d2/keep.txt");
    }
    fn a_directory_where_the_source_lands(repo: &Path) {
        std::fs::create_dir_all(repo.join("b/a.txt")).expect("b/a.txt");
    }
    let rows: [MoveShape; 12] = [
        MoveShape { label: "one-source-into-an-untracked-directory", setup: an_untracked_directory, sources: &["a.txt"], destination: "b" },
        MoveShape { label: "one-source-into-a-tracked-directory", setup: a_tracked_directory, sources: &["a.txt"], destination: "t" },
        MoveShape { label: "several-sources-into-a-directory", setup: an_untracked_directory, sources: &["a.txt", "x.txt"], destination: "b" },
        MoveShape { label: "a-source-onto-a-tracked-file", setup: a_tracked_file_at_the_destination, sources: &["a.txt"], destination: "dest.txt" },
        MoveShape { label: "a-directory-onto-a-directory", setup: a_directory_at_the_destination, sources: &["d1"], destination: "d2" },
        MoveShape { label: "a-destination-that-is-not-there", setup: no_setup, sources: &["a.txt"], destination: "new.txt" },
        MoveShape { label: "a-directory-onto-a-name-that-is-not-there", setup: no_setup, sources: &["d1"], destination: "newdir" },
        MoveShape { label: "a-trailing-slash-on-an-existing-directory", setup: an_untracked_directory, sources: &["a.txt"], destination: "b/" },
        MoveShape { label: "a-trailing-slash-on-a-missing-directory", setup: no_setup, sources: &["a.txt"], destination: "b/" },
        MoveShape { label: "several-sources-onto-a-file", setup: a_tracked_file_at_the_destination, sources: &["a.txt", "x.txt"], destination: "dest.txt" },
        MoveShape { label: "a-directory-onto-an-existing-file", setup: a_tracked_file_at_the_destination, sources: &["d1"], destination: "dest.txt" },
        MoveShape { label: "a-directory-where-the-source-lands", setup: a_directory_where_the_source_lands, sources: &["a.txt"], destination: "b" },
    ];
    for row in rows {
        let (directory, repository): (PathBuf, PathBuf) = a_repository_for_moving(row.label);
        (row.setup)(repository.as_path());
        let answered: Result<Vec<String>, String> = moved_paths(repository.as_path(), row.sources, row.destination);
        let (code, written): (Option<i32>, Vec<(bool, String)>) =
            what_real_git_wrote(repository.as_path(), row.sources, row.destination);
        match answered {
            Ok(paths) => {
                if code == Some(0) {
                    // What the reading names is a path Git wrote, and every entry Git wrote is named or lies under
                    //  a directory the reading names: a moved directory is one entry for the guard, and everything
                    //  inside it came with it.
                    for named in &paths {
                        assert!(
                            written.iter().any(|(_, path)| return path == named),
                            "{}: the reading named {named}, which Git did not write ({written:?})",
                            row.label
                        );
                    }
                    for (_, path) in &written {
                        let covered: bool = paths
                            .iter()
                            .any(|named| return path == named || path.starts_with(format!("{named}/").as_str()));
                        assert!(covered, "{}: Git wrote {path}, which the reading did not name ({paths:?})", row.label);
                    }
                } else {
                    // Git refused this shape and wrote nothing, so the reading names nothing either.
                    assert!(paths.is_empty(), "{}: Git refused (code {code:?}) and the reading named {paths:?}", row.label);
                }
            }
            Err(cause) => panic!("{}: the reading failed: {cause}", row.label),
        }
        remove(directory.as_path());
    }
}

//endregion

//region The fourth review: the bytes of a pathspec file (2026-11-14)

/// What:
///  A pathspec file larger than the byte bound is refused by its size,
///  and a list within the bound is read.
/// Why:
///  The read runs on a thread this process stops waiting for at
///  `PATHSPEC_FILE_READ_CEILING`,
///  and that thread went on reading the whole file into memory with no bound,
///  so a file larger than this host's memory raised the wrapper's memory after the command had
///  already been refused
///  (found by reading; the file itself is not made here).
///  The small list is the positive control: the refusal below is the size and not a reader that
///  refuses everything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fromTheFile(repo, sparseFileOf(PATHSPEC_FILE_READ_LIMIT + 1), tree('topic')) // throws naming the size
/// fromTheFile(repo, 'b.txt\n', tree('topic'))                                  // ['b.txt']
/// ```
#[test]
fn a_pathspec_file_larger_than_the_bound_is_refused_by_its_size() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_tracked_file("facts-pathspec-file-size");
    git(repository.as_path(), &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repository.join("b.txt"), b"from the other tree\n").expect("the tracked file");
    git(repository.as_path(), &["add", "--", "b.txt"]);
    git(repository.as_path(), &["commit", "--quiet", "--message=other"]);
    git(repository.as_path(), &["checkout", "--quiet", "main"]);
    let small: PathBuf = directory.join("small-list");
    std::fs::write(small.as_path(), b"b.txt\n").expect("the small list");
    assert_eq!(
        from_the_file(
            repository.as_path(),
            small.to_str().expect("utf-8 path"),
            PathspecTarget::Tree(Some(OsString::from("other"))),
            false
        ),
        Ok(vec![b"b.txt".to_vec()]),
        "a list within the bound is read"
    );
    // A sparse file: the bytes cost no disk, and reading it is what the bound must stop.
    let large: PathBuf = directory.join("over-the-bound");
    let made: std::fs::File = std::fs::File::create(large.as_path()).expect("the large list");
    made.set_len(crate::wait_ceilings::PATHSPEC_FILE_READ_LIMIT + 1).expect("its length");
    drop(made);
    let answer: Result<Vec<Vec<u8>>, String> = from_the_file(
        repository.as_path(),
        large.to_str().expect("utf-8 path"),
        PathspecTarget::Tree(Some(OsString::from("other"))),
        false,
    );
    match answer {
        Err(cause) => assert!(
            cause.contains("more than") && cause.contains("bytes"),
            "the refusal must name the size: {cause}"
        ),
        Ok(found) => panic!("a file past the bound was read as {found:?}"),
    }
    remove(directory.as_path());
}

//endregion
