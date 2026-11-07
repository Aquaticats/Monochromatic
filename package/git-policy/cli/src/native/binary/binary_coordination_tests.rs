//! What:
//!  Index writers through the built executable:
//!  a `git add` beside a foreign `index.lock` ends as native Git ends it,
//! at once and with Git's own words,
//!  and a `git rebase --exec 'git add'` finishes because a nested wrapper waits for nothing.
//! Why:
//!  A forwarded index writer competes for the index only through Git's own `index.lock`,
//! as it does without the wrapper;
//!  the one holder it waits for is the commit service,
//! whose hold ends by itself.
//! A writer that waited for a foreign lock,
//!  or for a lock its own ancestor held,
//! would hang where native Git does not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await runWrapped(['rebase', '--exec', 'git add -- b.txt', base])).toMatchObject({ code: 0 });
//! ```

/// The shared fixtures and bounded process helpers.
use super::support::{
    Fixture, Observed, fixture, git, observe, remove, repository, run_direct, stderr_of, wrapped,
};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

/// Commit `name` with the given text,
///  so the repository has a history to rebase.
fn commit_file(fixture: &Fixture, repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("file");
    git(fixture, repo, &["add", "--", name]);
    git(fixture, repo, &["commit", "--quiet", "--message", name]);
}

/// Wait for a spawned command to finish,
///  with its output.
fn finish(child: Child) -> Output {
    return child.wait_with_output().expect("the command finishes");
}

/// `git rebase --exec 'git add -- b.txt'` finishes:
///  the wrapper that the rebase's exec starts waits for nothing its ancestor holds.
#[test]
fn a_nested_git_add_inside_rebase_does_not_deadlock() {
    let fixture: Fixture = fixture("coordination-rebase");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    commit_file(&fixture, repo.as_path(), "a.txt", "one\n");
    commit_file(&fixture, repo.as_path(), "b.txt", "two\n");
    // `git rebase` puts its own libexec directory first on PATH for exec steps, so a bare `git` there
    // is real Git and never reaches the wrapper; the exec step names the wrapper's path instead.
    let wrapper_path: String = fixture.root.join("bin/git").display().to_string();
    let exec_step: String = format!("{wrapper_path} add -- b.txt");
    let mut command: Command = wrapped(&fixture);
    command
        .current_dir(&repo)
        .args(["rebase", "--exec", exec_step.as_str(), "HEAD~1"]);
    let output: Output = finish(command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("start the wrapped rebase"));
    assert_eq!(
        output.status.code(),
        Some(0),
        "the rebase must finish, not time out: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    remove(&fixture);
}

/// A foreign program's `index.lock` is Git's refusal:
/// the wrapped `git add` ends with the status and the words native
///  `git add` gives for the same lock,
/// does not wait for it,
/// and leaves it where it is.
#[test]
fn a_foreign_index_lock_is_refused_as_native_git_refuses_it() {
    let fixture: Fixture = fixture("coordination-foreign-lock");
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    commit_file(&fixture, repo.as_path(), "a.txt", "one\n");
    std::fs::write(repo.join("b.txt"), "two\n").expect("file to add");
    std::fs::write(repo.join(".git/index.lock"), b"").expect("a lock nobody can be proven to hold");
    let wrapped_run: Observed = observe(wrapped(&fixture).current_dir(&repo).args(["add", "--", "b.txt"]), b"");
    let native_run: Observed = run_direct(&fixture, repo.as_path(), &["add", "--", "b.txt"]);
    assert_eq!(native_run.code, Some(128), "{}", stderr_of(&native_run));
    assert_eq!(wrapped_run.code, native_run.code, "{}", stderr_of(&wrapped_run));
    assert_eq!(stderr_of(&wrapped_run), stderr_of(&native_run), "Git's own words, and nothing of the wrapper's");
    assert!(repo.join(".git/index.lock").exists(), "the foreign lock is left in place");
    remove(&fixture);
}
