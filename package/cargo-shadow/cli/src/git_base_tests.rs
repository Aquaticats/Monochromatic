//! The unit tests of the choice of the trust base,
//! against real throwaway git repositories.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
use std::path::{Path, PathBuf};
// Imports `std::process::Command`.
use std::process::Command;

// Imports `crate::git_base::{git_lock_reader_at, resol...`.
use crate::git_base::{git_lock_reader_at, resolve_trust_base};
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

/// The system git,
/// not the `git` on PATH:
/// that one may be a policy wrapper that refuses a bare `git commit` in a fixture.
const SYSTEM_GIT: &str = "/usr/bin/git";

fn git(directory: &Path, arguments: &[&str]) -> String {
    let output = Command::new(SYSTEM_GIT)
        .args(["-c", "user.name=tester", "-c", "user.email=tester@example.invalid", "-c", "commit.gpgsign=false", "-c", "init.defaultBranch=main"])
        .args(arguments)
        .current_dir(directory)
        .output()
        // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
        // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
        // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // value ?? fallback
        // ```
        .expect("start git");
    // Borrow `&output.stderr`: lends the value without copying or giving it away.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    // Borrow `&output.stdout`: lends the value without copying or giving it away.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    return String::from_utf8_lossy(&output.stdout).trim().to_string();
}

/// A repository whose branch `main` tracks a bare remote at its first commit and has one more commit that was not pushed.
/// Returns the working directory,
/// the pushed commit and the local-only commit.
fn repository_ahead_of_its_upstream() -> (tempfile::TempDir, PathBuf, String, String) {
    let root = scratch();
    // Typed binding: `remote: PathBuf`.
    let remote: PathBuf = root.path().join("remote.git");
    // Typed binding: `work: PathBuf`.
    let work: PathBuf = root.path().join("work");
    // Borrow `&remote`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::fs::create_dir_all(&remote).expect("remote dir");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::fs::create_dir_all(work.join("package/a/one")).expect("work dir");
    // Borrow `&remote`: lends the value without copying or giving it away.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    git(&remote, &["init", "--bare", "--quiet"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["init", "--quiet"]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::fs::write(work.join("package/a/one/Cargo.lock"), "pushed lock\n").expect("lock");
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["add", "--", "package/a/one/Cargo.lock"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["commit", "--quiet", "-m", "pushed"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(&work, &["remote", "add", "origin", remote.to_str().expect("utf8")]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    // Typed binding: `pushed: String`.
    let pushed: String = git(&work, &["rev-parse", "HEAD"]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::fs::write(work.join("package/a/one/Cargo.lock"), "merged lock\n").expect("lock");
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["commit", "--quiet", "--all", "-m", "merged, not pushed"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    // Typed binding: `local: String`.
    let local: String = git(&work, &["rev-parse", "HEAD"]);
    return (root, work, pushed, local);
}

/// Delegated pick 1 (2026-11-03):
/// a local run compares with the upstream branch,
/// so a lockfile that arrived with a commit that is
/// not on the upstream yet is new since the base.
/// The remembered pass is keyed by the commit.
#[test]
fn a_local_run_compares_with_the_upstream_branch_not_head() {
    let (_root, work, pushed, local) = repository_ahead_of_its_upstream();

    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    // Borrow `&work`: lends the value without copying or giving it away.
    let base = resolve_trust_base(&work, false, None);

    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
    // Why:      The copy can be stored or moved elsewhere while the original stays in place.
    // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // structuredClone(value)  // or [...list] / {...object}
    // ```
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(base.commit, Ok(pushed.clone()));
    assert_ne!(pushed, local);
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(base.spelling, "@{upstream}");
}

/// CI keeps `HEAD`,
/// and an explicit `CARGO_SHADOW_TRUST_BASE` wins everywhere.
#[test]
fn ci_compares_with_head_and_an_explicit_base_wins() {
    let (_root, work, pushed, local) = repository_ahead_of_its_upstream();

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&work`: lends the value without copying or giving it away.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(resolve_trust_base(&work, true, None).commit, Ok(local.clone()));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&work`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(resolve_trust_base(&work, false, Some(&local)).commit, Ok(local));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&work`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(resolve_trust_base(&work, true, Some(&pushed)).commit, Ok(pushed));
}

/// Without an upstream the base falls back to the remote's default branch,
/// and without that to `HEAD`.
#[test]
fn without_an_upstream_the_base_is_origin_main_and_without_a_remote_it_is_head() {
    let (_root, work, pushed, local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["checkout", "--quiet", "-b", "feature"]);

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&work`: lends the value without copying or giving it away.
    let fallback = resolve_trust_base(&work, false, None);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fallback.commit, Ok(pushed));
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fallback.spelling, "origin/main");

    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["remote", "remove", "origin"]);
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&work`: lends the value without copying or giving it away.
    let alone = resolve_trust_base(&work, false, None);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(alone.commit, Ok(local));
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(alone.spelling, "HEAD");
}

/// The reader names a commit and reads the lockfile text there,
/// empty when it did not exist.
#[test]
fn the_reader_reads_a_lockfile_at_the_commit_and_an_absent_one_as_empty() {
    let (_root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    let read = git_lock_reader_at(&work, &pushed);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read("package/a/one").starts_with("pushed lock"));
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(read("package/a/two"), "");
}

/// A base that names no commit is an error naming it,
/// not a panic.
#[test]
fn a_base_that_names_no_commit_is_an_error() {
    let (_root, work, _pushed, _local) = repository_ahead_of_its_upstream();

    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&work`: lends the value without copying or giving it away.
    let base = resolve_trust_base(&work, false, Some("no-such-revision"));

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(base.commit.expect_err("not a commit").contains("no-such-revision"));
}

// Owner rules (2026-11-01, `package/git-policy/cli/SPEC.md`): "Invoke real git as rarely as possible." and "embed instead of
// spawning". The trust base is read from the repository's files; the tests count the Git processes a choice started and compare
// the answer with the system git as the oracle.

// Imports `crate::git_base::{TrustBase, resolve_trust_...`.
use crate::git_base::{TrustBase, resolve_trust_base_with};
// Imports `crate::git_refs::NativeEnvironment`.
use crate::git_refs::NativeEnvironment;

/// A disposable environment:
/// an empty home,
/// no system configuration,
/// no blocking variable.
fn isolated_environment(root: &Path) -> NativeEnvironment {
    // Typed binding: `home: PathBuf`.
    let home: PathBuf = root.join("home");
    // Borrow `&home`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::fs::create_dir_all(&home).expect("home");
    // `Some`: the present or the absent value of an `Option`.
    return NativeEnvironment { home: Some(home), xdg_config_home: None, system_config: Some(root.join("no-system-config")), blocking_variable: None, git: crate::git_refs::GitProgram::default(), user_id: crate::git_refs::current_user_id() };
}

fn oracle(work: &Path, revision: &str) -> String {
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return git(work, &["rev-parse", "--verify", &format!("{revision}^{{commit}}")]);
}

fn chosen(root: &tempfile::TempDir, work: &Path, in_ci: bool) -> TrustBase {
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&isolated_environment`: lends the value without copying or giving it away.
    return resolve_trust_base_with(work, in_ci, None, &isolated_environment(root.path()));
}

#[test]
fn a_plain_repository_is_answered_from_its_files_with_no_git_process() {
    let (root, work, pushed, local) = repository_ahead_of_its_upstream();

    // Borrow `&root`: lends the value without copying or giving it away.
    let local_run = chosen(&root, &work, false);
    // Borrow `&root`: lends the value without copying or giving it away.
    let ci_run = chosen(&root, &work, true);

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((local_run.spelling.as_str(), local_run.commit.clone(), local_run.git_starts), ("@{upstream}", Ok(pushed), 0));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((ci_run.spelling.as_str(), ci_run.commit.clone(), ci_run.git_starts), ("HEAD", Ok(local), 0));
}

#[test]
fn packed_references_are_read_as_loose_ones_are() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["pack-refs", "--all", "--prune"]);

    // Borrow `&root`: lends the value without copying or giving it away.
    let base = chosen(&root, &work, false);

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((base.commit.clone(), base.git_starts), (Ok(pushed), 0));
}

#[test]
fn without_an_upstream_the_fallbacks_are_read_from_the_files_too() {
    let (root, work, pushed, local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["checkout", "--quiet", "-b", "feature"]);
    // Borrow `&root`: lends the value without copying or giving it away.
    let from_origin = chosen(&root, &work, false);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["remote", "remove", "origin"]);
    // Borrow `&root`: lends the value without copying or giving it away.
    let alone = chosen(&root, &work, false);

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((from_origin.spelling.as_str(), from_origin.commit.clone(), from_origin.git_starts), ("origin/main", Ok(pushed), 0));
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((alone.spelling.as_str(), alone.commit.clone(), alone.git_starts), ("HEAD", Ok(local), 0));
}

#[test]
fn a_detached_head_has_no_upstream_and_is_its_own_base() {
    let (root, work, _pushed, local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["checkout", "--quiet", "--detach"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["remote", "remove", "origin"]);

    // Borrow `&root`: lends the value without copying or giving it away.
    let base = chosen(&root, &work, false);

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((base.commit.clone(), base.git_starts), (Ok(local), 0));
}

#[test]
fn a_linked_worktree_reads_its_own_head_and_the_shared_configuration_and_references() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // Typed binding: `linked: PathBuf`.
    let linked: PathBuf = root.path().join("linked");
    // Borrow `&work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(&work, &["worktree", "add", "--quiet", "-b", "topic", linked.to_str().expect("utf8")]);
    // Borrow `&linked`: lends the value without copying or giving it away.
    git(&linked, &["branch", "--quiet", "--set-upstream-to=origin/main"]);

    // Borrow `&root`: lends the value without copying or giving it away.
    let base = chosen(&root, &linked, false);

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((base.spelling.as_str(), base.commit.clone(), base.git_starts), ("@{upstream}", Ok(pushed.clone()), 0));
    // Borrow `&linked`: lends the value without copying or giving it away.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(oracle(&linked, "@{upstream}"), pushed);
}

/// Each layout or setting the reader cannot answer exactly costs one Git start,
/// and the answer is still Git's.
fn assert_asked_git(base: &TrustBase, expected: &str) {
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(base.git_starts >= 1, "the files should not have answered: {base:?}");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(base.commit, Ok(expected.to_string()));
}

#[test]
fn a_configuration_include_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::fs::write(root.path().join("extra.config"), "[user]\n\tname = extra\n").expect("extra");
    // Borrow `&work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(&work, &["config", "include.path", root.path().join("extra.config").to_str().expect("utf8")]);

    // Borrow `&chosen`: lends the value without copying or giving it away.
    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn a_conditional_include_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::fs::write(root.path().join("extra.config"), "[user]\n\tname = extra\n").expect("extra");
    // Borrow `&work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(&work, &["config", "includeIf.gitdir:/nowhere/.path", root.path().join("extra.config").to_str().expect("utf8")]);

    // Borrow `&chosen`: lends the value without copying or giving it away.
    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn an_include_in_the_users_configuration_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // Typed binding: `environment: NativeEnvironment`.
    let environment: NativeEnvironment = isolated_environment(root.path());
    // `.clone()`: an independent owned copy.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    std::fs::write(environment.home.clone().expect("home").join(".gitconfig"), "[include]\n\tpath = /nowhere\n").expect("user config");

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&work`: lends the value without copying or giving it away.
    let base = resolve_trust_base_with(&work, false, None, &environment);

    // Borrow `&base`: lends the value without copying or giving it away.
    assert_asked_git(&base, &pushed);
}

#[test]
fn a_blocking_environment_variable_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `environment: NativeEnvironment`.
    let mut environment: NativeEnvironment = isolated_environment(root.path());
    // `Some`: the present or the absent value of an `Option`.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    environment.blocking_variable = Some("GIT_NAMESPACE".to_string());

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&resolve_trust_base_with`: lends the value without copying or giving it away.
    assert_asked_git(&resolve_trust_base_with(&work, false, None, &environment), &pushed);
}

#[test]
fn a_worktree_specific_configuration_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["config", "extensions.worktreeConfig", "true"]);

    // Borrow `&chosen`: lends the value without copying or giving it away.
    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn a_tag_that_shadows_the_remote_default_branch_is_asked_of_git() {
    let (root, work, _pushed, local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["checkout", "--quiet", "-b", "feature"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["tag", "origin/main", "HEAD"]);

    // `git rev-parse origin/main` prefers the tag, which is the local commit.
    assert_asked_git(&chosen(&root, &work, false), &local);
}

#[test]
fn a_negative_fetch_refspec_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["config", "--add", "remote.origin.fetch", "^refs/heads/nothing"]);

    // Borrow `&chosen`: lends the value without copying or giving it away.
    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn a_symbolic_reference_where_a_commit_is_expected_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["checkout", "--quiet", "-b", "feature"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["symbolic-ref", "refs/remotes/origin/main", "refs/remotes/origin/other"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["update-ref", "refs/remotes/origin/other", &pushed]);

    // Borrow `&chosen`: lends the value without copying or giving it away.
    assert_asked_git(&chosen(&root, &work, false), &pushed);
}

#[test]
fn the_reftable_reference_format_is_asked_of_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();
    // Borrow `&work`: lends the value without copying or giving it away.
    git(&work, &["config", "extensions.refStorage", "reftable"]);

    // The setting alone is what the reader refuses; the repository still has files, so Git (which ignores the setting while the
    // files exist) is not asked to read a reftable store here.
    let environment: NativeEnvironment = isolated_environment(root.path());
    // Borrow `&work`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(crate::git_native::native_base(&work, false, &environment).is_err());
    // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
    // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // void expr;   // or try { expr } catch {}
    // ```
    let _ = pushed;
}

#[test]
fn an_explicit_base_is_verified_by_git() {
    let (root, work, pushed, _local) = repository_ahead_of_its_upstream();

    // `Some`: the present or the absent value of an `Option`.
    // Borrow `&work`: lends the value without copying or giving it away.
    let base = resolve_trust_base_with(&work, false, Some(&pushed), &isolated_environment(root.path()));

    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!((base.commit.clone(), base.git_starts), (Ok(pushed), 1));
}
