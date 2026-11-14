//! What:
//!  Commits whose options are a form real Git refuses,
//!  through the built wrapper and the resident service:
//!  each is refused with the status and the words the real Git of this host
//!  prints,
//!  and leaves no commit,
//!  no reference and no record of the commit service.
//! Why:
//!  Native preparation removes the tokens Git validates,
//!  so before the change these commands reached the private `git commit` as
//!  ordinary ones and made a commit real Git would have refused
//!  (finding 8 of the audit of 0.6.1-alpha.14;
//!  `-i` with `-o` from the review of the commit-argument fixes).
//!  The owner's answer of 2026-11-12 is "Refuse as Git does",
//!  with Git's own message,
//!  so every control here compares the wrapper's answer with the answer real
//!  Git gives the same command line in a repository in the same state,
//!  never with a message this program keeps.
//!  The one recorded exception stays and is the last control in this file:
//!  a commit that names an untracked path commits it
//!  (`doc/decision/cli-git-policies-platform.md`,
//!  "Transactional commit gate").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const real = spawnSync('/usr/bin/git', ['commit', '-i', '-o', '-m', 'x', '--', 'a.txt'], { cwd: twin });
//! const wrapped = throughService(['commit', '-i', '-o', '-m', 'x', '--', 'a.txt']);
//! expect(wrapped).toEqual(real); expect(head).toBe(headBefore);
//! ```

use super::commit::{git_text, remove_settled};
use super::service_commit::{Running, runtime_for, start, through_service, through_service_with};
use super::support::{Fixture, bounded_for, direct, fixture, git, repository, wrapper_first_path};
use super::support_program_names_and_time_bound::REAL_GIT;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// What a refusal must leave as it was:
///  the commit `HEAD` names,
///  how many commits there are,
///  every reference,
///  whether the commit's timing record was written,
///  and what the service's journal holds.
/// Why:
///   "No commit,
/// no reference and no record" is one statement about all of these,
///  and a refusal that staged a path or reached the service would move one of them.
#[derive(Debug, Eq, PartialEq)]
struct State {
    /// The commit `HEAD` points at.
    head: String,
    /// The number of commits `HEAD` reaches.
    commits: String,
    /// Every reference with the object it names.
    refs: String,
    /// Whether the commit's timing record exists.
    timing_record: bool,
    /// Every name under the service's journal directory.
    journal: Vec<String>,
}

/// The names under `directory`,
/// relative to it,
/// sorted.
fn names_under(directory: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return found;
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        let name: String = path.strip_prefix(directory).unwrap_or(path.as_path()).display().to_string();
        found.push(name);
        if path.is_dir() {
            for inner in names_under(path.as_path()) {
                found.push(format!("{}/{inner}", entry.file_name().display()));
            }
        }
    }
    found.sort();
    return found;
}

/// The state of one repository and the fixture's journal.
fn state(fixture: &Fixture, repo: &Path) -> State {
    return State {
        head: git_text(fixture, repo, &["rev-parse", "HEAD"]).trim().to_owned(),
        commits: git_text(fixture, repo, &["rev-list", "--count", "HEAD"]).trim().to_owned(),
        refs: git_text(fixture, repo, &["for-each-ref", "--format=%(refname) %(objectname)"]),
        timing_record: repo.join(".git/monochromatic-git-policy-cli-commit-timing.jsonl").exists(),
        journal: names_under(fixture.root.join("journal").as_path()),
    };
}

/// A repository with one commit and one tracked file whose edit is staged,
///  so every row below has a path to name and a `HEAD` to amend or to fix up.
fn a_staged_change(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    git(fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), "one\ntwo\n").expect("edit a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    return repo;
}

/// What:
///  Run one commit command line as real Git and through the wrapper in two
///  repositories in the same state,
///  and require the same status and the same words,
///  with nothing changed in the wrapped one.
/// Why:
///   The comparison is what "Git's own message" means:
///  a release that rewords a message changes both sides together,
///  and a wrapper that answered anything of its own fails here.
fn refuses_as_git_does(fixture_name: &str, arguments: &[&str]) {
    let fixture: Fixture = fixture(fixture_name);
    let real_repo: PathBuf = a_staged_change(&fixture, "real");
    let wrapped_repo: PathBuf = a_staged_change(&fixture, "wrapped");
    std::fs::write(wrapped_repo.join("paths.txt"), "a.txt\n").expect("pathspecs");
    std::fs::write(real_repo.join("paths.txt"), "a.txt\n").expect("pathspecs");
    let name: String = format!("{fixture_name}-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let before: State = state(&fixture, wrapped_repo.as_path());
    let mut git_command: std::process::Command = direct(&fixture);
    git_command.current_dir(real_repo.as_path());
    let real: Output = git_command.args(arguments).output().expect("real Git runs");
    let wrapped: Output = through_service(&fixture, wrapped_repo.as_path(), name.as_str(), arguments);
    assert_eq!(
        wrapped.status.code(),
        real.status.code(),
        "the status real Git answered {arguments:?} with; Git said: {}; the wrapper said: {}",
        String::from_utf8_lossy(real.stderr.as_slice()),
        String::from_utf8_lossy(wrapped.stderr.as_slice())
    );
    assert_eq!(
        String::from_utf8_lossy(wrapped.stderr.as_slice()),
        String::from_utf8_lossy(real.stderr.as_slice()),
        "the words of {arguments:?}"
    );
    assert_ne!(real.status.code(), Some(0), "the row must be one real Git refuses: {arguments:?}");
    assert_eq!(state(&fixture, wrapped_repo.as_path()), before, "nothing changed: no commit, no reference, no record");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// `-i` with `-o`:
/// the row the review of the commit-argument fixes found.
#[test]
fn include_with_only_is_refused() {
    refuses_as_git_does("commit-form-include-only", &["commit", "-i", "-o", "-m", "x", "--", "a.txt"]);
}

/// The long spellings of the same row,
///  which a transform could miss where it read only the short ones.
#[test]
fn include_with_only_is_refused_in_long_spellings() {
    refuses_as_git_does(
        "commit-form-include-only-long",
        &["commit", "--include", "--only", "--message", "x", "--", "a.txt"],
    );
}

/// `-i` with `-a`:
/// one of the rows the audit found.
#[test]
fn include_with_all_is_refused() {
    refuses_as_git_does("commit-form-include-all", &["commit", "-i", "-a", "-m", "x", "--", "a.txt"]);
}

/// `-i`,
/// `-o` and `-a` together:
///  Git names all three,
///  so the wrapper must print that whole sentence.
#[test]
fn include_with_only_and_all_is_refused() {
    refuses_as_git_does("commit-form-include-only-all", &["commit", "-i", "-o", "-a", "-m", "x", "--", "a.txt"]);
}

/// `-a` with `--interactive`:
///  the selection count Git makes treats the two interactive spellings as one.
#[test]
fn all_with_interactive_is_refused() {
    refuses_as_git_does("commit-form-all-interactive", &["commit", "-a", "--interactive"]);
}

/// `--fixup=reword:` with a path.
#[test]
fn rewording_with_a_path_is_refused() {
    refuses_as_git_does("commit-form-reword-path", &["commit", "--fixup=reword:HEAD", "--", "a.txt"]);
}

/// `--fixup=reword:` with `-a`.
#[test]
fn rewording_with_all_is_refused() {
    refuses_as_git_does("commit-form-reword-all", &["commit", "--fixup=reword:HEAD", "-a"]);
}

/// `--fixup=reword:` with `-i`.
#[test]
fn rewording_with_include_is_refused() {
    refuses_as_git_does("commit-form-reword-include", &["commit", "--fixup=reword:HEAD", "-i"]);
}

/// `--fixup=reword:` with `--patch`.
#[test]
fn rewording_with_patch_is_refused() {
    refuses_as_git_does("commit-form-reword-patch", &["commit", "--fixup=reword:HEAD", "--patch"]);
}

/// `--fixup=reword:` with `--interactive`.
#[test]
fn rewording_with_interactive_is_refused() {
    refuses_as_git_does("commit-form-reword-interactive", &["commit", "--fixup=reword:HEAD", "--interactive"]);
}

/// `--pathspec-from-file` with pathspec arguments.
#[test]
fn a_pathspec_file_with_paths_is_refused() {
    refuses_as_git_does(
        "commit-form-pathspec-file-with-paths",
        &["commit", "--pathspec-from-file=paths.txt", "-m", "x", "--", "a.txt"],
    );
}

/// What:
///  The recorded exception still commits:
///  a commit that names an untracked path records it,
///  where real Git refuses the pathspec.
/// Why:
///  `doc/decision/cli-git-policies-platform.md`,
///  "Transactional commit gate",
/// records the selection of untracked paths as chosen,
///  and the owner's answer of 2026-11-12 keeps it.
///  A refusal here would take away what an explicit-path commit is for.
#[test]
fn a_commit_that_names_an_untracked_path_still_commits() {
    let fixture: Fixture = fixture("commit-form-untracked-path");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, repo.as_path(), &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("u.txt"), "untracked\n").expect("u");
    let name: String = format!("commit-form-untracked-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let before: String = git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim().to_owned();
    let committed: Output = through_service(&fixture, repo.as_path(), name.as_str(), &["commit", "-m", "x", "--", "u.txt"]);
    assert_eq!(
        committed.status.code(),
        Some(0),
        "the recorded exception commits: {}",
        String::from_utf8_lossy(committed.stderr.as_slice())
    );
    let after: String = git_text(&fixture, repo.as_path(), &["rev-list", "--count", "HEAD"]).trim().to_owned();
    assert_ne!(after, before, "the commit landed");
    assert_eq!(git_text(&fixture, repo.as_path(), &["show", "--name-only", "--format=", "HEAD"]).trim(), "u.txt");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// What:
///  The fixture's `bin/git` with the wrapper first on `PATH`,
///  in a directory of the row's own,
///  under the bound the package's commit controls use.
/// Why:
///  The arranged rows name directories and repositories inside two twins,
///  so the command must run where the row says rather than in the fixture root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wrappedHere = (fixture, cwd, args) => boundedFor(fixture, join(fixture.root, 'bin/git'), wrapperFirstPath(fixture), COMMIT_BOUND);
/// ```
fn wrapped_here(fixture: &Fixture, directory: &Path, arguments: &[&str]) -> Command {
    let mut command: Command = bounded_for(
        fixture,
        fixture.root.join("bin/git").as_path(),
        wrapper_first_path(fixture).as_os_str(),
        super::commit::COMMIT_BOUND_SECONDS,
    );
    command.current_dir(directory).args(arguments);
    return command;
}

/// What:
///  Every file of a tree with its bytes,
///  the Git directories' own files included but their object stores left out,
/// so a commit anywhere in the tree,
///  a staged path,
///  a moved reference or a leftover scratch file all show as a difference.
/// Why:
///  A row that names a directory may have more than one repository below it,
///  and what must stay unchanged is the whole tree:
///  the composition that landed a commit in an unrelated repository changed a
///  tree this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const everythingUnder = (root) => walk(root).filter((p) => !p.endsWith('/.git/objects')).map((p) => [p, read(p)]);
/// ```
fn everything_under(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found: Vec<(String, Vec<u8>)> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            let path: PathBuf = entry.path();
            let name: String = path.strip_prefix(root).unwrap_or(path.as_path()).display().to_string();
            if name.ends_with(".git/objects") || name.contains(".git/objects/") {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            found.push((name, std::fs::read(path.as_path()).unwrap_or_default()));
        }
    }
    found.sort();
    return found;
}

/// What:
///  One row whose command must be refused with real Git's own status and bytes,
///  leaving the whole tree as it was,
///  with a layout,
///  arguments and variables of the row's own.
/// Why:
///   The rows below exercise the directory options and the environment,
///  which name paths inside each twin,
///  and the shapes whose `--` is an option's value;
///  each must be refused in Git's words and must change nothing anywhere.
fn refuses_as_git_does_arranged(
    fixture_name: &str,
    arrange: fn(&Fixture, &Path),
    arguments: fn(&Path) -> Vec<String>,
    cwd: fn(&Path) -> PathBuf,
    environment: &[(&str, &str)],
) {
    let fixture: Fixture = fixture(fixture_name);
    let real_root: PathBuf = fixture.root.join("real");
    let wrapped_root: PathBuf = fixture.root.join("wrapped");
    for root in [&real_root, &wrapped_root] {
        std::fs::create_dir_all(root.as_path()).expect("twin root");
        arrange(&fixture, root.as_path());
    }
    let name: String = format!("{fixture_name}-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let before: Vec<(String, Vec<u8>)> = everything_under(wrapped_root.as_path());
    let real_arguments: Vec<String> = arguments(real_root.as_path());
    let mut git_command: Command = direct(&fixture);
    git_command.current_dir(cwd(real_root.as_path()));
    for (variable, value) in environment {
        git_command.env(variable, value);
    }
    let real: Output = git_command.args(real_arguments.as_slice()).output().expect("real Git runs");
    let wrapped_arguments: Vec<String> = arguments(wrapped_root.as_path());
    let wrapped_words: Vec<&str> = wrapped_arguments.iter().map(String::as_str).collect();
    let mut wrapped_command: Command = wrapped_here(&fixture, cwd(wrapped_root.as_path()).as_path(), wrapped_words.as_slice());
    wrapped_command.env("XDG_RUNTIME_DIR", runtime_for(name.as_str()));
    for (variable, value) in environment {
        wrapped_command.env(variable, value);
    }
    let wrapped: Output = wrapped_command.output().expect("the wrapped commit runs");
    assert_eq!(
        wrapped.status.code(),
        real.status.code(),
        "the status of {wrapped_arguments:?}; Git said: {}; the wrapper said: {}",
        String::from_utf8_lossy(real.stderr.as_slice()),
        String::from_utf8_lossy(wrapped.stderr.as_slice())
    );
    assert_eq!(
        String::from_utf8_lossy(wrapped.stderr.as_slice()),
        String::from_utf8_lossy(real.stderr.as_slice()),
        "the words of {wrapped_arguments:?}"
    );
    assert_ne!(real.status.code(), Some(0), "the row is one real Git refuses: {real_arguments:?}");
    assert_eq!(
        everything_under(wrapped_root.as_path()),
        before,
        "nothing changed anywhere: no commit, no staged path, no reference, no leftover"
    );
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A repository of its own with one commit,
///  so a row can place several beside one another.
fn a_repository_at(path: &Path) {
    std::fs::create_dir_all(path).expect("directory");
    git_at(path, &["init", "--quiet", "--initial-branch=main"]);
    git_at(path, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(path.join("a.txt"), "one\n").expect("a");
    git_at(path, &["add", "--", "a.txt"]);
    git_at(path, &["commit", "--quiet", "--message=base"]);
}

/// Real Git with the fixture's hermetic environment,
///  for a row that makes a repository of its own inside a twin.
fn direct_here() -> Command {
    let mut command: Command = Command::new(REAL_GIT);
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid");
    return command;
}

/// Real Git with the fixture's environment,
///  in a directory a row made for itself.
fn git_at(directory: &Path, arguments: &[&str]) {
    let mut command: Command = direct_here();
    command.current_dir(directory).args(arguments);
    let output: Output = command.output().expect("real Git runs");
    assert!(output.status.success(), "{arguments:?}: {}", String::from_utf8_lossy(output.stderr.as_slice()));
}

/// What:
///  A staged change in the repository at `root`,
///  and a staged file whose name is the option the row's message spells.
/// Why:
///  The shape that once landed a commit had a staged file named `-i`,
///  which the mangled question turned into a pathspec.
fn a_staged_change_and_a_staged_file_named(path: &Path, name: &str) {
    a_repository_at(path);
    std::fs::write(path.join("a.txt"), "one\ntwo\n").expect("edit");
    git_at(path, &["add", "--", "a.txt"]);
    std::fs::write(path.join(name), "named\n").expect("the named file");
    git_at(path, &["add", "--", name]);
}

/// A message that is the separator,
///  with a staged file named `-i`:
///  the question must be a dry run,
/// so Git refuses and nothing is committed.
#[test]
fn a_message_named_separator_is_refused_with_gits_words_and_lands_nothing() {
    refuses_as_git_does_arranged(
        "commit-form-message-separator",
        |_fixture, root| { a_staged_change_and_a_staged_file_named(root, "-i"); },
        |_root| { vec!["commit".to_owned(), "-m".to_owned(), "--".to_owned(), "-i".to_owned()] },
        |root| { return root.to_path_buf(); },
        &[],
    );
}

/// A message file named `--`,
///  with a staged file named `-i`:
///  the question must be a dry run.
#[test]
fn a_message_file_named_separator_is_refused_with_gits_words() {
    refuses_as_git_does_arranged(
        "commit-form-message-file-separator",
        |_fixture, root| {
            a_staged_change_and_a_staged_file_named(root, "-i");
            std::fs::write(root.join("--"), "a message\n").expect("the message file");
        },
        |_root| { vec!["commit".to_owned(), "-F".to_owned(), "--".to_owned(), "-i".to_owned()] },
        |root| { return root.to_path_buf(); },
        &[],
    );
}

/// A pathspec file named `--`,
///  followed by `-a`:
///  the question must ask about this command and not about another.
#[test]
fn a_pathspec_file_named_separator_is_refused_with_gits_words() {
    refuses_as_git_does_arranged(
        "commit-form-pathspec-file-separator",
        |_fixture, root| {
            a_repository_at(root);
            std::fs::write(root.join("a.txt"), "one\ntwo\n").expect("edit");
            git_at(root, &["add", "--", "a.txt"]);
            std::fs::write(root.join("--"), "a.txt\n").expect("the pathspec file");
        },
        |_root| { vec!["commit".to_owned(), "--pathspec-from-file".to_owned(), "--".to_owned(), "-a".to_owned()] },
        |root| { return root.to_path_buf(); },
        &[],
    );
}

/// A relative `-C`:
/// the question must be asked where the person's Git is,
///  so Git's words are about the person's command and not about a directory that is not there.
#[test]
fn a_relative_directory_option_is_applied_once() {
    refuses_as_git_does_arranged(
        "commit-form-relative-c",
        |_fixture, root| { a_repository_at(&root.join("sub")); },
        |_root| { vec!["-C".to_owned(), "sub".to_owned(), "commit".to_owned(), "-i".to_owned(), "-o".to_owned(), "-m".to_owned(), "x".to_owned()] },
        |root| { return root.to_path_buf(); },
        &[],
    );
}

/// A chain of `-C`:
/// each is applied once,
/// in the order the person wrote them.
#[test]
fn a_chain_of_directory_options_is_applied_once() {
    refuses_as_git_does_arranged(
        "commit-form-chain-of-c",
        |_fixture, root| { a_repository_at(&root.join("one").join("two")); },
        |_root| {
            vec![
                "-C".to_owned(),
                "one".to_owned(),
                "-C".to_owned(),
                "two".to_owned(),
                "commit".to_owned(),
                "-i".to_owned(),
                "-o".to_owned(),
                "-m".to_owned(),
                "x".to_owned(),
            ]
        },
        |root| { return root.to_path_buf(); },
        &[],
    );
}

/// The composition the review reported:
///  a relative `-C sub` and a message named `--` with `-i` beside it,
///  while `sub/sub` is also a repository with a staged file named `-i`.
///  Nothing may be committed anywhere,
///  least of all in the repository the command never named.
#[test]
fn a_commit_under_a_relative_directory_option_never_lands_in_a_nested_repository() {
    refuses_as_git_does_arranged(
        "commit-form-nested-repository",
        |_fixture, root| {
            a_repository_at(root);
            a_repository_at(&root.join("sub"));
            a_staged_change_and_a_staged_file_named(&root.join("sub").join("sub"), "-i");
        },
        |_root| { vec!["-C".to_owned(), "sub".to_owned(), "commit".to_owned(), "-m".to_owned(), "--".to_owned(), "-i".to_owned()] },
        |root| { return root.to_path_buf(); },
        &[],
    );
}

/// A repository named by `--git-dir` and `--work-tree`:
///  the question must read the repository the command names,
/// once.
#[test]
fn a_named_git_directory_is_asked_once() {
    refuses_as_git_does_arranged(
        "commit-form-named-git-dir",
        |_fixture, root| {
            a_repository_at(&root.join("work"));
            std::fs::rename(root.join("work").join(".git"), root.join("elsewhere.git")).expect("a Git directory beside its work tree");
        },
        |root| {
            vec![
                "--git-dir".to_owned(),
                root.join("elsewhere.git").display().to_string(),
                "--work-tree".to_owned(),
                root.join("work").display().to_string(),
                "commit".to_owned(),
                "-i".to_owned(),
                "-o".to_owned(),
                "-m".to_owned(),
                "x".to_owned(),
            ]
        },
        |root| { return root.join("work"); },
        &[],
    );
}

/// A repository named by the environment:
///  `GIT_DIR` and `GIT_WORK_TREE` reach the question as the person gave them.
#[test]
fn a_git_dir_in_the_environment_is_asked_once() {
    refuses_as_git_does_arranged(
        "commit-form-git-dir-variable",
        |_fixture, root| {
            a_repository_at(&root.join("work"));
            std::fs::rename(root.join("work").join(".git"), root.join("elsewhere.git")).expect("a Git directory beside its work tree");
        },
        |_root| { vec!["commit".to_owned(), "-i".to_owned(), "-o".to_owned(), "-m".to_owned(), "x".to_owned()] },
        |root| { return root.join("work"); },
        &[],
    );
}

/// What:
///  A pathless `-o` beside `--fixup=amend:` is a commit real Git makes,
///  so the wrapper must make it too.
/// Why:
///  Git's own condition allows an empty change when the fixup names the `amend` prefix,
///  with or without `--amend`;
///  refusing it breaks "Commits should never fail".
#[test]
fn a_pathless_only_beside_an_amending_fixup_commits() {
    let fixture: Fixture = fixture("commit-form-amending-fixup");
    let real_repo: PathBuf = a_staged_change(&fixture, "real");
    let wrapped_repo: PathBuf = a_staged_change(&fixture, "wrapped");
    let name: String = format!("commit-form-amending-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let real_before: String = git_text(&fixture, real_repo.as_path(), &["rev-list", "--count", "HEAD"]).trim().to_owned();
    let wrapped_before: String = git_text(&fixture, wrapped_repo.as_path(), &["rev-list", "--count", "HEAD"]).trim().to_owned();
    // `--fixup=amend:` opens the editor with the message Git built, so both arms are given one that keeps it. The policy
    //  escape hatch is the wrapper's own word, so only the wrapped arm is given it: with it, the commit-only policy stands
    //  aside and this control measures the form rule alone.
    let mut git_command: Command = direct(&fixture);
    git_command.current_dir(real_repo.as_path()).env("GIT_EDITOR", "/bin/true");
    let real: Output = git_command.args(["commit", "-o", "--fixup=amend:HEAD"]).output().expect("real Git runs");
    let wrapped: Output = through_service_with(
        &fixture,
        wrapped_repo.as_path(),
        name.as_str(),
        &["commit", "-o", "--fixup=amend:HEAD", "--no-enforce-only"],
        &[("GIT_EDITOR", "/bin/true")],
    );
    assert_eq!(
        wrapped.status.code(),
        real.status.code(),
        "the status both answers share; Git said: {}; the wrapper said: {}",
        String::from_utf8_lossy(real.stderr.as_slice()),
        String::from_utf8_lossy(wrapped.stderr.as_slice())
    );
    assert_eq!(real.status.code(), Some(0), "real Git makes this commit: {}", String::from_utf8_lossy(real.stderr.as_slice()));
    assert_eq!(
        git_text(&fixture, wrapped_repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(),
        (wrapped_before.parse::<u32>().expect("a count") + 1).to_string(),
        "the wrapper made the same one commit"
    );
    assert_eq!(git_text(&fixture, real_repo.as_path(), &["rev-list", "--count", "HEAD"]).trim(), (real_before.parse::<u32>().expect("a count") + 1).to_string());
    assert_eq!(
        git_text(&fixture, wrapped_repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
        git_text(&fixture, real_repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
        "the same subject as Git's"
    );
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
