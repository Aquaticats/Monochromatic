//! What:
//! Reads the lockfiles of a base revision with `git`,
//! for the frozen install under `trustLockfile`.
//! Why:
//! The base is the pushed range's previous commit or a pull request's base (`doc/decision/rust-dependency-catalog.md`,
//! amended by the owner on 2026-10-29).
//! CI names it in the environment variable below;
//! without it the base is `HEAD`,
//! so a
//! developer's uncommitted lockfile changes are the ones checked.
//! The old TypeScript `rust:verify` took the same revision as
//! `--base`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-base.ts
//! export async function createGitBaseLockReader({ root, base }): Promise<(crateDir: string) => string>
//! ```

/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `git_base_lock_reader`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::cell::Cell;
/// What:
/// `use std::path::Path;` imports
/// `Path`
/// so this file can name them unqualified.
/// Why:
/// `Path` is the borrowed view of a file path (sibling:
/// `PathBuf`,
/// the owned one);
/// the repository root is lent to the functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;
/// What:
/// `use std::io::Write;` imports the trait that adds `write_all` to a pipe.
/// Why:
/// The revisions to resolve are written to the standard input of the one Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // child.stdin.write(text)
/// ```
use std::io::Write;
/// What:
/// `use std::process::{Command, Stdio};` imports the process builder and the pipe settings.
/// Why:
/// `Command` starts Git; `Stdio::piped` connects its input and output to this process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';
/// ```
use std::process::{Command, Stdio};

/// What:
/// `use crate::constants::LOCKFILE_NAME;` imports `LOCKFILE_NAME` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LOCKFILE_NAME` is needed by the derive or macro expansions of this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::LOCKFILE_NAME;
/// What:
/// `use crate::git_native::native_base;` imports the choice of the trust base made from the repository files.
/// Why:
/// `resolve_trust_base_with` tries it before it asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { nativeBase } from 'crate/git_native';
/// ```
use crate::git_native::native_base;
/// What:
/// `use crate::git_refs::NativeEnvironment;` imports the description of where the user configuration lives and whether the environment forbids reading the files.
/// Why:
/// Tests pass a disposable one;
/// the shadow passes the process's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { NativeEnvironment } from 'crate/git_refs';
/// ```
use crate::git_refs::NativeEnvironment;

/// The environment variable that names the base revision.
pub const BASE_VARIABLE: &str = "CARGO_SHADOW_TRUST_BASE";

/// The base used when the variable is absent or empty.
pub const DEFAULT_BASE: &str = "HEAD";

/// The upstream branch of the current branch,
/// the first base a local run tries.
const UPSTREAM_BASE: &str = "@{upstream}";

/// The remote default branches a local run tries when the current branch has no upstream.
const DEFAULT_REMOTE_BASES: [&str; 2] = ["origin/main", "origin/master"];

/// What:
/// `fn git_start_failure(error: std::io::Error) -> String` words the failure to start `git`.
/// Why:
/// The base lock reader runs `git`,
/// and the user needs to know when it cannot even start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitStartFailure = (error: Error) => `could not start git: ${error.message}`;
/// ```
fn git_start_failure(error: std::io::Error) -> String {
    return format!("could not start git: {error}");
}

/// How a trust base was chosen and what it resolved to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBase {
    /// How the base is spelled (`CARGO_SHADOW_TRUST_BASE`,
    /// `@{upstream}`,
    /// `origin/main`,
    /// `HEAD`).
    pub spelling: String,
    /// The commit it names,
    /// or why it names none.
    pub commit: Result<String, String>,
    /// How many Git processes were started to choose it (none when the files answered).
    pub git_starts: u32,
}

/// What:
/// `fn resolve_commit(root, base) -> Result<String, String>` is the commit a revision names in the repository at `root`.
/// Why:
/// A remembered pass is keyed by the commit,
/// not by how the base is spelled,
/// so a base that moves is a different base.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveCommit(root: string, base: string): string  // git rev-parse --verify base^{commit}
/// ```
fn resolve_commit(root: &Path, base: &str, starts: &Cell<u32>) -> Result<String, String> {
    // Every Git process started here is counted, so a test (and a measurement) can say how many a choice cost.
    starts.set(starts.get() + 1);
    let resolved = Command::new("git")
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        .args(["rev-parse", "--verify", "--quiet", "--end-of-options", &format!("{base}^{{commit}}")])
        .current_dir(root)
        .output()
        .map_err(git_start_failure)?;
    if !resolved.status.success() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(not_a_commit_message(base));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&resolved.stdout`: lends the value without copying or giving it away.
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
    return Ok(String::from_utf8_lossy(&resolved.stdout).trim().to_string());
}

/// What:
/// `pub fn not_a_commit_message(base: &str) -> String` is the text that says a base names no commit.
/// Why:
/// Git and the file reader report the same failure,
/// so they use the same words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const notACommitMessage = (base: string): string => `The verification base ${JSON.stringify(base)} is not a commit...`;
/// ```
pub fn not_a_commit_message(base: &str) -> String {
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!(
        "The verification base {base:?} is not a commit in this repository.\nSet {BASE_VARIABLE} to a revision that exists locally (in CI, fetch it first), or leave it unset to compare with the upstream branch (HEAD in CI)."
    );
}

/// What:
/// `pub fn resolve_trust_base(root, in_ci, explicit) -> TrustBase` chooses the revision whose lockfiles count as trusted,
/// reading the running process's environment.
/// Why:
/// The shadow calls this;
/// the tests call `resolve_trust_base_with` with a disposable environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveTrustBase(root: string, inCi: boolean, explicit?: string): TrustBase
/// ```
pub fn resolve_trust_base(root: &Path, in_ci: bool, explicit: Option<&str>) -> TrustBase {
    // Borrow `&NativeEnvironment`: lends the value without copying or giving it away.
    return resolve_trust_base_with(root, in_ci, explicit, &NativeEnvironment::from_process());
}

/// What:
/// `fn batch_inputs() -> [String; 4]` lists the revisions the fallback asks Git about, in the order they are written to it:
/// the default remote branches and `HEAD` first, `@{upstream}` last.
/// Why:
/// `git cat-file --batch-check` stops with a fatal error on `@{upstream}` when the branch has none;
/// everything before that line is already printed, so putting it last loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const batchInputs = (): string[] => ['origin/main^{commit}', 'origin/master^{commit}', 'HEAD^{commit}', '@{upstream}^{commit}'];
/// ```
fn batch_inputs() -> [String; 4] {
    return [
        format!("{}^{{commit}}", DEFAULT_REMOTE_BASES[0]),
        format!("{}^{{commit}}", DEFAULT_REMOTE_BASES[1]),
        format!("{DEFAULT_BASE}^{{commit}}"),
        format!("{UPSTREAM_BASE}^{{commit}}"),
    ];
}

/// What:
/// `fn batch_answers(root: &Path) -> Result<Vec<Option<String>>, String>` starts Git once and returns, for each input of
/// `batch_inputs` in order, the commit it names or `None`.
/// Why:
/// A line is `<oid> commit <size>` for a commit and `<input> missing` (or `ambiguous`) otherwise;
/// an input whose line is absent (the fatal `@{upstream}` of a branch without one) names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function batchAnswers(root: string): (string | undefined)[]  // throws when git cannot start
/// ```
fn batch_answers(root: &Path) -> Result<Vec<Option<String>>, String> {
    let inputs: [String; 4] = batch_inputs();
    let mut child = Command::new("git")
        .args(["cat-file", "--batch-check"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(git_start_failure)?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(inputs.join("\n").as_bytes()).and_then(|()| { return stdin.write_all(b"\n"); }).map_err(git_start_failure)?;
    }
    let finished = child.wait_with_output().map_err(git_start_failure)?;
    let text: String = String::from_utf8_lossy(&finished.stdout).to_string();
    let mut lines = text.lines();
    let mut answers: Vec<Option<String>> = Vec::new();
    for _input in &inputs {
        let answer: Option<String> = lines.next().and_then(|line| {
            let mut words = line.split_whitespace();
            return match (words.next(), words.next()) {
                (Some(object), Some("commit")) => Some(object.to_string()),
                _ => None,
            };
        });
        answers.push(answer);
    }
    return Ok(answers);
}

/// What:
/// `fn resolve_batch(root: &Path, starts: &Cell<u32>) -> TrustBase` chooses the base with one Git start:
/// `@{upstream}`, else `origin/main`, else `origin/master`, else `HEAD`.
/// Why:
/// The reader of the files declined, so Git answers, and it answers all candidates at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveBatch(root: string, starts: { count: number }): TrustBase
/// ```
fn resolve_batch(root: &Path, starts: &Cell<u32>) -> TrustBase {
    starts.set(starts.get() + 1);
    let answers: Vec<Option<String>> = match batch_answers(root) {
        Ok(found) => found,
        Err(message) => return TrustBase { spelling: DEFAULT_BASE.to_string(), commit: Err(message), git_starts: starts.get() },
    };
    let order: [(usize, &str); 4] = [(3, UPSTREAM_BASE), (0, DEFAULT_REMOTE_BASES[0]), (1, DEFAULT_REMOTE_BASES[1]), (2, DEFAULT_BASE)];
    for (position, spelling) in order {
        if let Some(Some(commit)) = answers.get(position) {
            return TrustBase { spelling: spelling.to_string(), commit: Ok(commit.clone()), git_starts: starts.get() };
        }
    }
    return TrustBase { spelling: DEFAULT_BASE.to_string(), commit: Err(not_a_commit_message(DEFAULT_BASE)), git_starts: starts.get() };
}

/// What:
/// `pub fn resolve_trust_base_with(root, in_ci, explicit, environment) -> TrustBase` chooses the revision whose lockfiles count
/// as trusted:
/// an explicit `CARGO_SHADOW_TRUST_BASE` wins;
/// otherwise CI compares with `HEAD` and a local run with `@{upstream}`,
/// else `origin/main`,
/// else `origin/master`,
/// else `HEAD`.
/// Why:
/// The choice is read from the repository's files (`git_refs.rs`) without starting Git,
/// by the owner's rule "Invoke real git as
/// rarely as possible.";
/// a layout or setting the reader does not answer exactly (`git_refs::native_base` returns `Err`),
/// and an
/// explicit base,
/// which Git must verify,
/// are asked of Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveTrustBaseWith(root, inCi, explicit, environment): TrustBase
/// ```
pub fn resolve_trust_base_with(root: &Path, in_ci: bool, explicit: Option<&str>, environment: &NativeEnvironment) -> TrustBase {
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    // What:     `Cell<T>` holds a value that can be replaced through a shared reference (interior mutability; sibling: `RefCell<T>`, for non-copy values).
    // Why:      A method that takes `&self` still has to remember when it last made a request.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let starts: Cell<u32> = Cell::new(0);
    // An explicit base wins everywhere, and Git verifies that it names a commit.
    if let Some(found) = explicit
        && !found.trim().is_empty()
    {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Typed binding: `spelling: String`.
        let spelling: String = found.trim().to_string();
        // Borrow `&spelling`: lends the value without copying or giving it away.
        // Typed binding: `commit: Result<String, String>`.
        let commit: Result<String, String> = resolve_commit(root, &spelling, &starts);
        return TrustBase { spelling, commit, git_starts: starts.get() };
    }
    // The files answer for the plain layouts.
    if let Ok(native) = native_base(root, in_ci, environment) {
        return TrustBase { spelling: native.spelling, commit: native.commit, git_starts: 0 };
    }
    // CI keeps its base.
    if in_ci {
        // Borrow `&starts`: lends the value without copying or giving it away.
        // Typed binding: `commit: Result<String, String>`.
        let commit: Result<String, String> = resolve_commit(root, DEFAULT_BASE, &starts);
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return TrustBase { spelling: DEFAULT_BASE.to_string(), commit, git_starts: starts.get() };
    }
    // Delegated pick (2026-11-03): a local run compares with what is already on the upstream branch, so a lockfile that arrived
    // with a commit not pushed yet is new since the base; the first candidate that names a commit wins.
    // One Git start answers every candidate (review finding 2a 13).
    return resolve_batch(root, &starts);
}

/// What:
/// `pub fn git_lock_reader_at(root, commit) -> LockReader` reads one crate root's lockfile text at a commit
/// (empty when the lockfile did not exist there).
/// Why:
/// The base is resolved once,
/// so every read names the same commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function gitLockReaderAt(root: string, commit: string): (crateDir: string) => string
/// ```
#[cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(avoidable_closure, reason = "the returned reader keeps the repository root and the commit after this function returns")
)]
pub fn git_lock_reader_at(root: &Path, commit: &str) -> Box<dyn Fn(&str) -> String> {
    let root_path = root.to_path_buf();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `commit_text: String`.
    let commit_text: String = commit.to_string();
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    return Box::new(move |crate_dir: &str| -> String {
        // Typed binding: `object: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let object: String = format!("{commit_text}:{crate_dir}/{LOCKFILE_NAME}");
        return match Command::new("git").args(["show", &object]).current_dir(&root_path).output() {
            Ok(shown) if shown.status.success() => format!("{}\n", String::from_utf8_lossy(&shown.stdout)),
            _ => String::new(),
        };
    });
}

/// What:
/// `pub fn explicit_base_from_environment() -> Option<String>` is the base named by `CARGO_SHADOW_TRUST_BASE`,
/// when it is set
/// and not blank.
/// Why:
/// CI sets it to the pushed range's previous commit or the pull request's base;
/// it wins over every default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const explicitBaseFromEnvironment = (): string | undefined => process.env.CARGO_SHADOW_TRUST_BASE?.trim() || undefined;
/// ```
pub fn explicit_base_from_environment() -> Option<String> {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    return match std::env::var(BASE_VARIABLE) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        Ok(found) if !found.trim().is_empty() => Some(found.trim().to_string()),
        // `None`: the present or the absent value of an `Option`.
        _ => None,
    };
}
