//! The differential tests of the plain-file base reader:
//! Git itself is the oracle.
//! Each case builds a disposable repository with an isolated home,
//! asks `/usr/bin/git` which base it would choose,
//! and requires the
//! reader to either decline (so Git is asked) or give exactly Git's spelling and commit.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
use std::fs;
// Imports `std::path::{Path, PathBuf}`.
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

// Imports `crate::git_base::resolve_trust_base_with`.
use crate::git_base::resolve_trust_base_with;
// Imports `crate::git_native::native_base`.
use crate::git_native::native_base;
// Imports `crate::git_refs::{NativeBase, NativeEnviron...`.
use crate::git_refs::{GitProgram, NativeBase, NativeEnvironment, current_user_id};
// Imports `crate::test_scratch::scratch`.
use crate::test_scratch::scratch;

/// The real Git binary;
/// the `git` on the search path may be a policy wrapper.
const SYSTEM_GIT: &str = "/usr/bin/git";

/// The spellings the base choice tries,
/// in order.
const CANDIDATES: [&str; 4] = ["@{upstream}", "origin/main", "origin/master", "HEAD"];

/// A disposable repository with two commits:
/// `first` (the older) and `second` (the one `HEAD` points at on the branch `topic`).
pub(crate) struct Fixture {
    /// The directory that owns everything the case creates.
    pub(crate) root: tempfile::TempDir,
    /// The working directory of the repository.
    pub(crate) work: PathBuf,
    /// The older commit.
    pub(crate) first: String,
    /// The newer commit,
    /// `HEAD` of `topic`.
    pub(crate) second: String,
}

/// Runs the real Git in a directory with an environment that contains nothing but a disposable home.
pub(crate) fn run_git(root: &Path, directory: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new(SYSTEM_GIT)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("LC_ALL", "C")
        .args(["-c", "user.name=tester", "-c", "user.email=tester@example.invalid", "-c", "commit.gpgsign=false", "-c", "protocol.file.allow=always"])
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
    if !output.status.success() {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        return None;
    }
    // `Some`: the present or the absent value of an `Option`.
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
}

/// Runs Git and requires success.
pub(crate) fn git(fixture_root: &Path, directory: &Path, arguments: &[&str]) -> String {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    return run_git(fixture_root, directory, arguments).unwrap_or_else(|| panic!("git {arguments:?} failed in {}", directory.display()));
}

/// The reader's environment:
/// the disposable home and no system configuration.
pub(crate) fn environment(root: &Path) -> NativeEnvironment {
    // `Some`: the present or the absent value of an `Option`.
    return NativeEnvironment { home: Some(root.join("home")), xdg_config_home: None, system_config: Some(root.join("no-system-config")), blocking_variable: None, git: GitProgram::default(), user_id: current_user_id() };
}

/// Builds the repository on the branch `branch`.
pub(crate) fn fixture(branch: &str) -> Fixture {
    let root = scratch();
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(root.path().join("home")).expect("home");
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let work: PathBuf = root.path().join("work");
    // Borrow `&work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&work).expect("work");
    // Borrow `&work`: lends the value without copying or giving it away.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    git(root.path(), &work, &["init", "--quiet", "--initial-branch", branch]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(work.join("file"), "one\n").expect("file");
    // Borrow `&work`: lends the value without copying or giving it away.
    git(root.path(), &work, &["add", "file"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(root.path(), &work, &["commit", "--quiet", "--message", "one"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    // Typed binding: `first: String`.
    let first: String = git(root.path(), &work, &["rev-parse", "HEAD"]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(work.join("file"), "two\n").expect("file");
    // Borrow `&work`: lends the value without copying or giving it away.
    git(root.path(), &work, &["commit", "--quiet", "--all", "--message", "two"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    // Typed binding: `second: String`.
    let second: String = git(root.path(), &work, &["rev-parse", "HEAD"]);
    return Fixture { root, work, first, second };
}

/// Replaces the repository configuration with the plain core section followed by `text`.
pub(crate) fn write_config(fixture: &Fixture, text: &str) {
    // Typed binding: `core: &str`.
    let core: &str = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n";
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(fixture.work.join(".git/config"), format!("{core}{text}")).expect("config");
}

/// Writes a loose reference by hand,
/// the way a fetch or a broken tool could.
pub(crate) fn write_ref(directory: &Path, name: &str, object: &str) {
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = directory.join(name);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(path.parent().expect("parent")).expect("ref directory");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(path, format!("{object}\n")).expect("ref");
}

/// What Git chooses:
/// the first candidate that names a commit,
/// with its commit.
pub(crate) fn oracle(fixture: &Fixture, directory: &Path) -> Option<(String, String)> {
    for spelling in CANDIDATES {
        // Typed binding: `revision: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let revision: String = format!("{spelling}^{{commit}}");
        // `Some`: the present or the absent value of an `Option`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        // Borrow `&[`: lends the value without copying or giving it away.
        if let Some(commit) = run_git(fixture.root.path(), directory, &["rev-parse", "--verify", "--quiet", "--end-of-options", &revision]) {
            // `Some`: the present or the absent value of an `Option`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Some((spelling.to_string(), commit));
        }
    }
    // `None`: the present or the absent value of an `Option`.
    return None;
}

/// Requires the reader to decline or to agree with Git,
/// in the directory `directory`.
pub(crate) fn assert_agrees_at(fixture: &Fixture, directory: &Path, label: &str) {
    // Typed binding: `expected: Option<(String, String)>`.
    let expected: Option<(String, String)> = oracle(fixture, directory);
    // Borrow `&environment`: lends the value without copying or giving it away.
    // Typed binding: `native: Result<NativeBase, String>`.
    let native: Result<NativeBase, String> = native_base(directory, false, &environment(fixture.root.path()));
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    match native {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        Err(_) => {}
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(base) => {
            // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
            // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // (a, b) => body
            // ```
            // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
            // Why:      The copy can be stored or moved elsewhere while the original stays in place.
            // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // structuredClone(value)  // or [...list] / {...object}
            // ```
            // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
            // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { return compute(); } catch { return undefined; }
            // ```
            // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
            // Why:      They express one step of a pipeline without an index loop.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // list.map(f).filter(g)
            // ```
            // Typed binding: `found: Option<(String, String)>`.
            let found: Option<(String, String)> = base.commit.clone().ok().map(|commit| { return (base.spelling.clone(), commit); });
            // Macro `assert_eq!`: builds or prints text (or a list) from a template.
            assert_eq!(found, expected, "{label}: the reader answered {base:?} but Git chooses {expected:?}");
        }
    }
}

/// Requires the reader to decline or to agree with Git,
/// in the repository's working directory.
pub(crate) fn assert_agrees(fixture: &Fixture, label: &str) {
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    assert_agrees_at(fixture, &fixture.work, label);
}

/// Requires the reader to decline.
pub(crate) fn assert_declines(fixture: &Fixture, label: &str) {
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // Typed binding: `native: Result<NativeBase, String>`.
    let native: Result<NativeBase, String> = native_base(&fixture.work, false, &environment(fixture.root.path()));
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(native.is_err(), "{label}: the reader answered {native:?} where only Git can say");
}

/// Makes the branch `base` point at the older commit,
/// so a correct upstream selection is distinguishable from `HEAD`.
pub(crate) fn base_branch_at_first(fixture: &Fixture) {
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.first]);
}

/// The branch section that makes `topic` track `refs/heads/base` of this repository.
pub(crate) const TOPIC_TRACKS_BASE: &str = "[branch \"topic\"]\n\tremote = .\n\tmerge = refs/heads/base\n";

/// A control:
/// the plain upstream case agrees,
/// so the other cases test the difference and not the harness.
#[test]
fn a_plain_local_upstream_agrees_with_git() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, TOPIC_TRACKS_BASE);

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "plain upstream");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let answered = native_base(&fixture.work, false, &environment(fixture.root.path())).expect("the plain case is answered from the files");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(answered.commit, Ok(fixture.first.clone()));
}

/// Review finding (2a 1):
/// a comment that ends in a backslash does not continue onto the next line,
/// so the include below it is real.
#[test]
fn a_comment_ending_in_a_backslash_does_not_hide_the_include_below_it() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Typed binding: `included: PathBuf`.
    let included: PathBuf = fixture.root.path().join("upstream.conf");
    // Borrow `&included`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&included, TOPIC_TRACKS_BASE).expect("included file");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(&fixture, &format!("\t# a comment \\\n[include]\n\tpath = {}\n", included.display()));

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "comment then include");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_declines(&fixture, "comment then include");
}

/// Review finding (2a 2):
/// every Git spelling of true for `extensions.worktreeConfig` makes the reader leave the answer to Git.
#[test]
fn every_true_spelling_of_the_worktree_configuration_extension_is_left_to_git() {
    for spelling in ["true", "TRUE", "yes", "on", "1", "2", "-1"] {
        let fixture = fixture("topic");
        // Borrow `&fixture`: lends the value without copying or giving it away.
        base_branch_at_first(&fixture);
        // Borrow `&fixture`: lends the value without copying or giving it away.
        write_config(&fixture, "");
        fs::write(
            fixture.work.join(".git/config"),
            // Macro `format!`: builds or prints text (or a list) from a template.
            format!("[core]\n\trepositoryformatversion = 1\n[extensions]\n\tworktreeConfig = {spelling}\n"),
        )
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        .expect("config");
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        fs::write(fixture.work.join(".git/config.worktree"), TOPIC_TRACKS_BASE).expect("worktree config");

        // Borrow `&fixture`: lends the value without copying or giving it away.
        assert_agrees(&fixture, spelling);
        // Borrow `&fixture`: lends the value without copying or giving it away.
        assert_declines(&fixture, spelling);
    }
}

/// Review finding (2a 3):
/// with several `merge` values Git uses the first.
#[test]
fn the_first_of_several_merge_values_is_the_upstream() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "[branch \"topic\"]\n\tremote = .\n\tmerge = refs/heads/base\n\tmerge = refs/heads/topic\n");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "two merge values");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let answered = native_base(&fixture.work, false, &environment(fixture.root.path())).expect("answered");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(answered.commit, Ok(fixture.first.clone()));
}

/// Review finding (2a 4):
/// a dotted section name followed by a quoted subsection is one flat name in Git.
#[test]
fn a_dotted_section_with_a_quoted_subsection_names_the_dotted_branch() {
    let fixture = fixture("topic.sub");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.first]);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "[branch.topic \"sub\"]\n\tremote = .\n\tmerge = refs/heads/base\n");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "dotted section and quoted subsection");
}

/// Review finding (2a 4):
/// a quoted subsection may contain a closing bracket,
/// an escaped quote and a hash sign.
#[test]
fn a_quoted_subsection_may_hold_a_bracket_an_escaped_quote_and_a_hash() {
    let fixture = fixture("topic\"]#suffix");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.first]);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "[branch \"topic\\\"]#suffix\"]\n\tremote = .\n\tmerge = refs/heads/base\n");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "bracket in subsection");
}

/// Review finding (2a 5):
/// a quoted value keeps its trailing characters,
/// even a no-break space that is a legal reference-name character.
#[test]
fn a_quoted_merge_value_keeps_a_trailing_no_break_space() {
    let fixture = fixture("topic");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["branch", "base", &fixture.second]);
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["branch", "base\u{a0}", &fixture.first]);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "[branch \"topic\"]\n\tremote = .\n\tmerge = \"refs/heads/base\u{a0}\"\n");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "no-break space");
}

/// Review finding (2a 6):
/// a key without a value is not the string `true`;
/// Git refuses the missing value.
#[test]
fn a_remote_key_without_a_value_is_not_a_remote_named_true() {
    let fixture = fixture("topic");
    // Borrow `&fixture.work.join`: lends the value without copying or giving it away.
    write_ref(&fixture.work.join(".git"), "refs/remotes/true/base", &fixture.first);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "[branch \"topic\"]\n\tremote\n\tmerge = refs/heads/base\n[remote \"true\"]\n\tfetch = +refs/heads/*:refs/remotes/true/*\n");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "bare remote key");
}

/// Review finding (2a 7):
/// references of a linked worktree's private namespaces live in that worktree's own directory.
#[test]
fn a_private_reference_of_a_linked_worktree_is_not_looked_for_in_the_common_directory() {
    let fixture = fixture("topic");
    // Typed binding: `linked: PathBuf`.
    let linked: PathBuf = fixture.root.path().join("linked");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(fixture.root.path(), &fixture.work, &["worktree", "add", "--quiet", "-b", "topic2", linked.to_str().expect("utf8"), "HEAD"]);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "[branch \"topic2\"]\n\tremote = .\n\tmerge = refs/worktree/base\n");
    // Borrow `&linked`: lends the value without copying or giving it away.
    git(fixture.root.path(), &linked, &["update-ref", "refs/worktree/base", &fixture.first]);

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees_at(&fixture, &linked, "private reference");
}

/// Review finding (2a 8):
/// the short name `origin/main` also resolves through `refs/remotes/origin/main/HEAD`.
#[test]
fn a_short_name_resolves_through_the_remote_head_of_a_remote_named_like_a_branch() {
    let fixture = fixture("topic");
    // Typed binding: `dot_git: PathBuf`.
    let dot_git: PathBuf = fixture.work.join(".git");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["update-ref", "refs/remotes/origin/main/stable", &fixture.first]);
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["symbolic-ref", "refs/remotes/origin/main/HEAD", "refs/remotes/origin/main/stable"]);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, "");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(dot_git.join("refs/remotes/origin/main/HEAD").exists());
    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "remote HEAD of a remote named origin/main");
}

/// Review finding (2a 9):
/// an object name in a reference is evidence of a commit only when the object is one (tags peel,
/// blobs and
/// missing objects are refused).
#[test]
fn an_annotated_tag_a_blob_and_a_missing_object_in_a_remote_reference_are_not_taken_for_commits() {
    let tag_fixture = fixture("topic");
    // Borrow `&tag_fixture.work`: lends the value without copying or giving it away.
    git(tag_fixture.root.path(), &tag_fixture.work, &["tag", "-a", "-m", "tag", "v1", &tag_fixture.first]);
    // Borrow `&tag_fixture.work`: lends the value without copying or giving it away.
    // Typed binding: `tag_object: String`.
    let tag_object: String = git(tag_fixture.root.path(), &tag_fixture.work, &["rev-parse", "v1"]);
    // Borrow `&tag_fixture.work.join`: lends the value without copying or giving it away.
    write_ref(&tag_fixture.work.join(".git"), "refs/remotes/origin/main", &tag_object);
    // Borrow `&tag_fixture`: lends the value without copying or giving it away.
    assert_agrees(&tag_fixture, "annotated tag");

    let blob_fixture = fixture("topic");
    // Borrow `&blob_fixture.work`: lends the value without copying or giving it away.
    // Typed binding: `blob: String`.
    let blob: String = git(blob_fixture.root.path(), &blob_fixture.work, &["rev-parse", "HEAD:file"]);
    // Borrow `&blob_fixture.work.join`: lends the value without copying or giving it away.
    write_ref(&blob_fixture.work.join(".git"), "refs/remotes/origin/main", &blob);
    // Borrow `&blob_fixture`: lends the value without copying or giving it away.
    assert_agrees(&blob_fixture, "blob");

    let missing_fixture = fixture("topic");
    // Borrow `&missing_fixture.work.join`: lends the value without copying or giving it away.
    write_ref(&missing_fixture.work.join(".git"), "refs/remotes/origin/main", "1234567890123456789012345678901234567890");
    // Borrow `&missing_fixture`: lends the value without copying or giving it away.
    assert_agrees(&missing_fixture, "missing object");
}

/// Review finding (2a 11):
/// the path after `gitdir: ` keeps its leading space,
/// and a `commondir` that cannot be read is not absent.
#[test]
fn a_gitdir_pointer_keeps_the_space_in_its_path() {
    let fixture = fixture("topic");
    // Typed binding: `outer: PathBuf`.
    let outer: PathBuf = fixture.root.path().join("outer");
    // Borrow `&outer`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&outer).expect("outer");
    // Borrow `&outer`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(fixture.root.path(), &outer, &["clone", "--quiet", "--bare", fixture.work.to_str().expect("utf8"), "git-a"]);
    // Borrow `&outer`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(fixture.root.path(), &outer, &["clone", "--quiet", "--bare", fixture.work.to_str().expect("utf8"), " git-a"]);
    // Borrow `&outer.join`: lends the value without copying or giving it away.
    git(fixture.root.path(), &outer.join(" git-a"), &["update-ref", "refs/heads/topic", &fixture.first]);
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(outer.join(".git"), "gitdir:  git-a\n").expect("pointer");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees_at(&fixture, &outer, "gitdir with a leading space");
}

/// Review finding (2a 11):
/// an unreadable `commondir` is a failure,
/// not the absence of one.
#[test]
fn an_unreadable_commondir_is_not_an_absent_one() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = fixture("topic");
    // Typed binding: `linked: PathBuf`.
    let linked: PathBuf = fixture.root.path().join("linked");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    git(fixture.root.path(), &fixture.work, &["worktree", "add", "--quiet", "--detach", linked.to_str().expect("utf8"), "HEAD"]);
    // Typed binding: `commondir: PathBuf`.
    let commondir: PathBuf = fixture.work.join(".git/worktrees/linked/commondir");
    // Borrow `&commondir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&commondir, fs::Permissions::from_mode(0o000)).expect("chmod");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees_at(&fixture, &linked, "unreadable commondir");
    // Borrow `&commondir`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&commondir, fs::Permissions::from_mode(0o644)).expect("chmod back");
}

/// Review finding (2a 12):
/// an unknown repository extension makes Git refuse the repository,
/// so the reader must not answer.
#[test]
fn an_unknown_repository_extension_is_left_to_git() {
    let fixture = fixture("topic");
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(fixture.work.join(".git/config"), "[core]\n\trepositoryformatversion = 1\n[extensions]\n\tsomethingnew = yes\n").expect("config");

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "unknown extension");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_declines(&fixture, "unknown extension");
}

/// Review finding (2a 12):
/// a fetch refspec that Git rejects is not skipped because an earlier one matched.
#[test]
fn an_invalid_refspec_after_a_matching_one_is_left_to_git() {
    let fixture = fixture("topic");
    // Borrow `&fixture.work.join`: lends the value without copying or giving it away.
    write_ref(&fixture.work.join(".git"), "refs/remotes/origin/tracked/base", &fixture.first);
    write_config(
        // Borrow `&fixture`: lends the value without copying or giving it away.
        &fixture,
        "[branch \"topic\"]\n\tremote = origin\n\tmerge = refs/heads/base\n[remote \"origin\"]\n\tfetch = +refs/heads/*:refs/remotes/origin/tracked/*\n\tfetch = refs/heads/**:refs/remotes/origin/**\n",
    );

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "invalid second refspec");
}

/// Review finding (2a 13):
/// when the reader declines,
/// one Git start answers every candidate.
#[test]
fn a_declined_choice_costs_exactly_one_git_start() {
    let fixture = fixture("topic");
    // Typed binding: `included: PathBuf`.
    let included: PathBuf = fixture.root.path().join("empty.conf");
    // Borrow `&included`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::write(&included, "").expect("included");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    write_config(&fixture, &format!("[include]\n\tpath = {}\n", included.display()));

    // `None`: the present or the absent value of an `Option`.
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // The Git is a logging stand-in around the real one, so the process budget is observed from outside and not read from the
    // reader's own counter (review finding 4a, missing tests), and the fallback runs the same Git and home as the oracle.
    let log: PathBuf = fixture.root.path().join("invocations");
    // Typed binding: `stand_in: PathBuf`.
    let stand_in: PathBuf = fixture.root.path().join("logging-git");
    // Borrow `&stand_in`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `format!`: builds or prints text (or a list) from a template.
    fs::write(&stand_in, format!("#!/bin/sh\necho \"$@\" >> {}\nexec {SYSTEM_GIT} \"$@\"\n", log.display())).expect("stand-in");
    // Borrow `&stand_in`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::set_permissions(&stand_in, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `controlled: NativeEnvironment`.
    let mut controlled: NativeEnvironment = environment(fixture.root.path());
    controlled.git = GitProgram {
        program: stand_in,
        // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
        // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: T[] / Set / Map / number
        // ```
        bound: std::time::Duration::from_secs(30),
        clean_environment: true,
        // Macro `vec!`: builds or prints text (or a list) from a template.
        environment: vec![
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            ("HOME".to_string(), fixture.root.path().join("home").display().to_string()),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            ("GIT_CONFIG_NOSYSTEM".to_string(), "1".to_string()),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            ("LC_ALL".to_string(), "C".to_string()),
        ],
    };
    // `None`: the present or the absent value of an `Option`.
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    let base = resolve_trust_base_with(&fixture.work, false, None, &controlled);
    // Borrow `&log`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(fs::read_to_string(&log).expect("the invocation log").lines().count(), 1, "the processes actually started");

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(base.git_starts, 1, "{base:?}");
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(base.spelling, "HEAD");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(base.commit, Ok(fixture.second.clone()));
}

/// After `git gc` the objects and references are packed;
/// the reader still answers from the files and agrees with Git.
#[test]
fn a_packed_repository_is_answered_from_its_pack_files() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, TOPIC_TRACKS_BASE);
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["gc", "--quiet", "--aggressive"]);

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!fixture.work.join(".git/refs/heads/base").exists(), "gc packed the references");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "packed repository");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    let answered = native_base(&fixture.work, false, &environment(fixture.root.path())).expect("answered from pack files");
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `.clone()`: an independent owned copy.
    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(answered.commit, Ok(fixture.first.clone()));
}

/// A SHA-256 repository has an object format extension and is Git's to read.
#[test]
fn a_sha256_repository_is_left_to_git() {
    let root = scratch();
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(root.path().join("home")).expect("home");
    // Typed binding: `work: PathBuf`.
    let work: PathBuf = root.path().join("work");
    // Borrow `&work`: lends the value without copying or giving it away.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    fs::create_dir_all(&work).expect("work");
    // Borrow `&work`: lends the value without copying or giving it away.
    git(root.path(), &work, &["init", "--quiet", "--object-format=sha256", "--initial-branch", "topic"]);
    // Borrow `&work`: lends the value without copying or giving it away.
    git(root.path(), &work, &["commit", "--quiet", "--allow-empty", "--message", "one"]);
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let fixture = Fixture { root, work, first: String::new(), second: String::new() };

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "sha256");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_declines(&fixture, "sha256");
}

/// The object check accepts a commit,
/// loose or packed,
/// and refuses a tag,
/// a tree,
/// a blob and an object that does not exist.
#[test]
fn only_a_commit_stored_whole_is_confirmed() {
    use crate::git_objects::confirm_commit;
    let fixture = fixture("topic");
    // Typed binding: `common: PathBuf`.
    let common: PathBuf = fixture.work.join(".git");
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["tag", "-a", "-m", "tag", "v1", &fixture.first]);
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // Typed binding: `tag: String`.
    let tag: String = git(fixture.root.path(), &fixture.work, &["rev-parse", "v1"]);
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // Typed binding: `tree: String`.
    let tree: String = git(fixture.root.path(), &fixture.work, &["rev-parse", "HEAD^{tree}"]);
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    // Typed binding: `blob: String`.
    let blob: String = git(fixture.root.path(), &fixture.work, &["rev-parse", "HEAD:file"]);

    for loose_or_packed in ["loose", "packed"] {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // Borrow `&common`: lends the value without copying or giving it away.
        // Macro `assert_eq!`: builds or prints text (or a list) from a template.
        assert_eq!(confirm_commit(&common, &fixture.first), Ok(()), "{loose_or_packed}: a commit");
        // Borrow `&common`: lends the value without copying or giving it away.
        // Macro `assert!`: builds or prints text (or a list) from a template.
        assert!(confirm_commit(&common, &tag).is_err(), "{loose_or_packed}: a tag");
        // Borrow `&common`: lends the value without copying or giving it away.
        // Macro `assert!`: builds or prints text (or a list) from a template.
        assert!(confirm_commit(&common, &tree).is_err(), "{loose_or_packed}: a tree");
        // Borrow `&common`: lends the value without copying or giving it away.
        // Macro `assert!`: builds or prints text (or a list) from a template.
        assert!(confirm_commit(&common, &blob).is_err(), "{loose_or_packed}: a blob");
        // Borrow `&common`: lends the value without copying or giving it away.
        // Macro `assert!`: builds or prints text (or a list) from a template.
        assert!(confirm_commit(&common, "1234567890123456789012345678901234567890").is_err(), "{loose_or_packed}: a missing object");
        // Borrow `&fixture.work`: lends the value without copying or giving it away.
        git(fixture.root.path(), &fixture.work, &["gc", "--quiet"]);
    }
}

/// A replace reference changes what Git reads for an object,
/// so the reader leaves the answer to Git.
#[test]
fn a_replace_reference_is_left_to_git() {
    let fixture = fixture("topic");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    base_branch_at_first(&fixture);
    // Borrow `&fixture`: lends the value without copying or giving it away.
    write_config(&fixture, TOPIC_TRACKS_BASE);
    // Borrow `&fixture.work`: lends the value without copying or giving it away.
    git(fixture.root.path(), &fixture.work, &["replace", &fixture.first, &fixture.second]);

    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_agrees(&fixture, "replace reference");
    // Borrow `&fixture`: lends the value without copying or giving it away.
    assert_declines(&fixture, "replace reference");
}
