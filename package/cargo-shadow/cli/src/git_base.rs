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
use std::path::{Path};
/// Imports the length of time a Git call may take.
use std::time::Duration;
/// What:
/// `use std::process::{Command, Output};` imports the process builder and the finished process's output.
/// Why:
/// `Command` describes the Git call;
/// `Output` is what a bounded run returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';
/// ```
use std::process::{Command, Output};

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
/// Imports the bounded child run that every Git call goes through.
use crate::bounded_run::{Bound, RunError, output_within, output_within_clean_environment};
/// What:
/// `use crate::git_native::native_base;` imports the choice of the trust base made from the repository files.
/// Why:
/// `resolve_trust_base_with` tries it before it asks Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { nativeBase } from 'crate/git_native';
/// ```
use crate::git_native::native_base_counting;
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

/// Which system configuration file the selected Git reads (split out of the git base module).
pub(crate) use crate::git_system_config_lookup::{system_config_of_git};

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
/// `fn batch_inputs() -> [String; 4]` lists the revisions the fallback asks Git about,
/// in the order they are written to it:
/// the default remote branches and `HEAD` first,
/// `@{upstream}` last.
/// Why:
/// `git cat-file --batch-check` stops with a fatal error on `@{upstream}` when the branch has none;
/// everything before that line is already printed,
/// so putting it last loses nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const batchInputs = (): string[] => ['origin/main^{commit}', 'origin/master^{commit}', 'HEAD^{commit}', '@{upstream}^{commit}'];
/// ```
fn batch_inputs() -> [String; 4] {
    return [
        // Macro `format!`: builds or prints text (or a list) from a template.
        format!("{}^{{commit}}", DEFAULT_REMOTE_BASES[0]),
        // Macro `format!`: builds or prints text (or a list) from a template.
        format!("{}^{{commit}}", DEFAULT_REMOTE_BASES[1]),
        // Macro `format!`: builds or prints text (or a list) from a template.
        format!("{DEFAULT_BASE}^{{commit}}"),
        // Macro `format!`: builds or prints text (or a list) from a template.
        format!("{UPSTREAM_BASE}^{{commit}}"),
    ];
}

/// What:
/// `fn commit_of_line(line: &str) -> Option<String>` reads one `git cat-file --batch-check` line:
/// the object name when the line is `<oid> commit <size>`,
/// `None` for `<input> missing` or `ambiguous`.
/// Why:
/// Only a line that says `commit` names a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commitOfLine = (line: string): string | undefined => { const [oid, kind] = line.split(/\s+/); return kind === 'commit' ? oid : undefined; };
/// ```
fn commit_of_line(line: &str) -> Option<String> {
    let mut words = line.split_whitespace();
    return match (words.next(), words.next()) {
        (Some(object), Some("commit")) => Some(object.to_string()),
        _ => None,
    };
}

/// What:
/// `fn run_git(environment, root, arguments, input) -> Result<Output, String>` runs Git within its bound.
/// Why:
/// Review finding (3a 9) and the owner's rule of 2026-11-03:
/// every Git call has a bound,
/// a timeout names what blocked and the
/// command to run again,
/// and the child is always reaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runGit(environment: NativeEnvironment, root: string, args: string[], input?: Buffer): Output  // throws a message
/// ```
pub(crate) fn run_git(environment: &NativeEnvironment, root: &Path, arguments: &[&str], input: Option<&[u8]>) -> Result<Output, String> {
    // Borrow `&environment.git.program`: lends the value without copying or giving it away.
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `command: Command`.
    let mut command: Command = Command::new(&environment.git.program);
    if environment.git.clean_environment {
        command.env_clear();
    }
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
    // Why:      They express one step of a pipeline without an index loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // list.map(f).filter(g)
    // ```
    for (name, value) in environment.git.environment.iter() {
        command.env(name.as_str(), value.as_str());
    }
    command.args(arguments).current_dir(root);
    // Typed binding: `bound: Bound`.
    let bound: Bound = Bound {
        limit: environment.git.bound,
        // Macro `format!`: builds or prints text (or a list) from a template.
        what: format!("`git {}` (cargo-shadow asks it which commit to compare with)", arguments.join(" ")),
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        rerun: crate::bounded_run::rerun_text(),
    };
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&mut command`: lends the value without copying or giving it away.
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    // A cleared environment is rebuilt from the variables set on the command, which the scope launcher must be told.
    let ran = if environment.git.clean_environment { output_within_clean_environment(&mut command, input, &bound) } else { output_within(&mut command, input, &bound) };
    return ran.map_err(run_error_text);
}

/// What:
/// `fn run_error_text(error: RunError) -> String` words a failed Git run:
/// the start failure of Git for an I/O error,
/// the error's own text for a timeout or another bound.
/// Why:
/// `Result::map_err` takes it by path instead of a closure.
fn run_error_text(error: RunError) -> String {
    return match error {
        RunError::Io(inner) => git_start_failure(inner),
        timed_out => timed_out.to_string(),
    };
}

/// How asking Git about one revision failed.
enum Candidate {
    /// Git answered that the revision names no commit (an ordinary negative answer).
    NotACommit,
    /// Git could not be asked or did not finish (a timeout,
    /// a start failure,
    /// a transport failure):
    /// nothing is known.
    Operational(
        /// What happened.
        String,
    ),
}

/// What:
/// `fn resolve_candidate(environment, root, base, starts) -> Result<String, Candidate>` asks Git which commit a revision names
/// and keeps an ordinary "no" apart from a failure to ask.
/// Why:
/// Review finding (4a 6):
/// a query that did not finish has not shown that the revision names no commit,
/// so the fallback must not
/// go on to a lower candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveCandidate(environment: NativeEnvironment, root: string, base: string, starts: { count: number }): string  // throws Candidate
/// ```
fn resolve_candidate(environment: &NativeEnvironment, root: &Path, base: &str, starts: &Cell<u32>) -> Result<String, Candidate> {
    starts.set(starts.get() + 1);
    // `None`: the present or the absent value of an `Option`.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&[`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let resolved = run_git(environment, root, &["rev-parse", "--verify", "--quiet", "--end-of-options", &format!("{base}^{{commit}}")], None).map_err(Candidate::Operational)?;
    // `git rev-parse --verify --quiet` answers "not a commit" with exit 1 and no output (checked with Git 2.56.0 for a missing name and
    // for a missing upstream); any other end is a failed Git, not a negative answer (round 6, 5a F4).
    match resolved.status.code() {
        Some(0) => {}
        Some(1) => return Err(Candidate::NotACommit),
        Some(code) => {
            return Err(Candidate::Operational(format!(
                "`git rev-parse` for {base} ended with exit status {code}: {}; no other candidate was tried",
                String::from_utf8_lossy(&resolved.stderr).trim()
            )));
        }
        None => {
            return Err(Candidate::Operational(format!(
                "`git rev-parse` for {base} was ended by a signal ({}); no other candidate was tried",
                String::from_utf8_lossy(&resolved.stderr).trim()
            )));
        }
    }
    return Ok(String::from_utf8_lossy(&resolved.stdout).trim().to_string());
}

/// What:
/// `fn resolve_commit(environment, root, base, starts) -> Result<String, String>` asks Git which commit a revision names,
/// with
/// the message to show when it names none.
/// Why:
/// It is the one start for an explicit base and for the base of CI.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveCommit(environment: NativeEnvironment, root: string, base: string, starts: { count: number }): string  // throws a message
/// ```
fn resolve_commit(environment: &NativeEnvironment, root: &Path, base: &str, starts: &Cell<u32>) -> Result<String, String> {
    // `match`: picks the arm whose pattern fits the value.
    return match resolve_candidate(environment, root, base, starts) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(commit) => Ok(commit),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(Candidate::NotACommit) => Err(not_a_commit_message(base)),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(Candidate::Operational(message)) => Err(message),
    };
}
/// What the one batched Git start answered.
enum Batch {
    /// Every candidate was answered:
    /// the commits of `origin/main`,
    /// `origin/master` and `HEAD`,
    /// and of `@{upstream}`
    /// (`None` when Git stopped at it because the branch has none,
    /// or it names no commit).
    Complete(
        /// The answers in the order of the batch inputs.
        Vec<Option<String>>,
    ),
    /// Git stopped before it answered every non-upstream candidate (a corrupt object,
    /// for example).
    /// Nothing can be concluded about the unanswered ones.
    Incomplete,
}

/// What:
/// `fn batch_answers(environment, root) -> Result<Batch, String>` starts Git once and returns,
/// for each input of `batch_inputs`
/// in order,
/// the commit it names or `None`.
/// Why:
/// A line is `<oid> commit <size>` for a commit and `<input> missing` (or `ambiguous`) otherwise.
/// `@{upstream}` is last because Git stops with a fatal error on a branch without one,
/// after the earlier lines are printed:
/// a missing last line is that expected end,
/// but a missing earlier line means Git stopped early (review finding 3a 5),
/// and an unanswered higher-priority query is never read as a missing revision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function batchAnswers(environment: NativeEnvironment, root: string): Batch  // throws when git cannot start
/// ```
fn batch_answers(environment: &NativeEnvironment, root: &Path) -> Result<Batch, String> {
    // Typed binding: `inputs: [String; 4]`.
    let inputs: [String; 4] = batch_inputs();
    // `Some`: the present or the absent value of an `Option`.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&[`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let finished = run_git(environment, root, &["cat-file", "--batch-check"], Some(format!("{}\n", inputs.join("\n")).as_bytes()))?;
    let text: String = String::from_utf8_lossy(&finished.stdout).to_string();
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // Typed binding: `lines: Vec<&str>`.
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < inputs.len() - 1 {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(Batch::Incomplete);
    }
    // Closure `|position|`: an anonymous function, like a TS arrow function.
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `answers: Vec<Option<String>>`.
    let mut answers: Vec<Option<String>> = Vec::new();
    for position in 0..inputs.len() {
        answers.push(lines.get(position).copied().and_then(commit_of_line));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(Batch::Complete(answers));
}

/// What:
/// `fn resolve_sequentially(environment, root, starts) -> TrustBase` resolves the candidates one Git start each,
/// in priority
/// order.
/// Why:
/// When the batch stopped early,
/// each candidate is asked on its own so a corrupt lower-priority one cannot hide a higher one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveSequentially(environment: NativeEnvironment, root: string, starts: { count: number }): TrustBase
/// ```
fn resolve_sequentially(environment: &NativeEnvironment, root: &Path, starts: &Cell<u32>) -> TrustBase {
    for candidate in [UPSTREAM_BASE, DEFAULT_REMOTE_BASES[0], DEFAULT_REMOTE_BASES[1]] {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        match resolve_candidate(environment, root, candidate, starts) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            Ok(commit) => return TrustBase { spelling: candidate.to_string(), commit: Ok(commit), git_starts: starts.get() },
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(Candidate::NotACommit) => {}
            // Review finding (4a 6): a query that did not finish says nothing about this candidate, so no lower one is chosen.
            Err(Candidate::Operational(message)) => return TrustBase { spelling: candidate.to_string(), commit: Err(message), git_starts: starts.get() },
        }
    }
    // Typed binding: `commit: Result<String, String>`.
    let commit: Result<String, String> = resolve_commit(environment, root, DEFAULT_BASE, starts);
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    return TrustBase { spelling: DEFAULT_BASE.to_string(), commit, git_starts: starts.get() };
}

/// What:
/// `fn resolve_batch(environment, root, starts) -> TrustBase` chooses the base with one Git start:
/// `@{upstream}`,
/// else `origin/main`,
/// else `origin/master`,
/// else `HEAD`.
/// Why:
/// The reader of the files declined,
/// so Git answers,
/// and it answers all candidates at once;
/// only when Git stopped early does the choice fall back to one start per candidate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveBatch(environment: NativeEnvironment, root: string, starts: { count: number }): TrustBase
/// ```
fn resolve_batch(environment: &NativeEnvironment, root: &Path, starts: &Cell<u32>) -> TrustBase {
    starts.set(starts.get() + 1);
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `answers: Vec<Option<String>>`.
    let answers: Vec<Option<String>> = match batch_answers(environment, root) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(Batch::Complete(found)) => found,
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(Batch::Incomplete) => return resolve_sequentially(environment, root, starts),
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
        let commit: Result<String, String> = resolve_commit(environment, root, &spelling, &starts);
        return TrustBase { spelling, commit, git_starts: starts.get() };
    }
    // The files answer for the plain layouts.
    if let Ok(native) = native_base_counting(root, in_ci, environment, &starts) {
        return TrustBase { spelling: native.spelling, commit: native.commit, git_starts: starts.get() };
    }
    // CI keeps its base.
    if in_ci {
        // Borrow `&starts`: lends the value without copying or giving it away.
        // Typed binding: `commit: Result<String, String>`.
        let commit: Result<String, String> = resolve_commit(environment, root, DEFAULT_BASE, &starts);
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return TrustBase { spelling: DEFAULT_BASE.to_string(), commit, git_starts: starts.get() };
    }
    // Delegated pick (2026-11-03): a local run compares with what is already on the upstream branch, so a lockfile that arrived
    // with a commit not pushed yet is new since the base; the first candidate that names a commit wins.
    // One Git start answers every candidate (review finding 2a 13).
    return resolve_batch(environment, root, &starts);
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
        // `let mut`: a binding that may change.
        // Typed binding: `command: Command`.
        let mut command: Command = Command::new("git");
        // Borrow `&object`: lends the value without copying or giving it away.
        command.args(["show", &object]).current_dir(&root_path);
        // Typed binding: `bound: Bound`.
        let bound: Bound = Bound {
            // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
            // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // TS has one type for this: T[] / Set / Map / number
            // ```
            limit: Duration::from_secs(crate::git_refs::GIT_CALL_SECONDS),
            // Macro `format!`: builds or prints text (or a list) from a template.
            what: format!("`git show {object}` (cargo-shadow reads the lockfile of the verification base)"),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            rerun: crate::bounded_run::rerun_text(),
        };
        // `None`: the present or the absent value of an `Option`.
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&mut command`: lends the value without copying or giving it away.
        return match output_within(&mut command, None, &bound) {
            Ok(shown) if shown.status.success() => format!("{}\n", String::from_utf8_lossy(&shown.stdout)),
            // A lockfile that cannot be read from the base is read as absent there, which checks more, never less.
            Err(error @ RunError::TimedOut { .. }) => {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!("cargo-shadow: warning: {error}; the lockfile is treated as absent at the base, so every version in it is checked");
                // Constructor `String::new()`: an empty owned collection or a boxed value.
                String::new()
            }
            _ => String::new(),
        };
    });
}

/// What:
/// `pub fn is_git_ignored(root: &Path, relative: &str) -> bool` says whether git ignores a path of the repository at `root`
/// (`git check-ignore`,
/// exit status 0).
/// Why:
/// A crate in a directory that git ignores (a gate's disposable build context under `node_modules`,
/// a scratch copy) is not a lockfile
/// of the repository:
/// there is no committed lockfile to compare with,
/// so every locked version would count as newly locked and be held
/// to the trust policy of versions that nobody added.
/// Only a clear "ignored" answer counts;
/// any other answer (not ignored,
/// no
/// repository,
/// a failure,
/// a timeout) leaves the crate under the catalog's policy,
/// so a crate that is merely new is still checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isGitIgnored(root: string, relative: string): boolean
/// ```
pub fn is_git_ignored(root: &Path, relative: &str) -> bool {
    // A plain child with a deadline, not a contained run: the question only reads, it writes nothing that needs a scope to stop, and
    // it must not start a scope (or take a fault meant for the first contained run) on every invocation of the shadow.
    let mut child = match Command::new("git")
        .args(["check-ignore", "--quiet", "--", relative])
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(started) => started,
        Err(_) => return false,
    };
    let deadline: std::time::Instant = std::time::Instant::now() + Duration::from_secs(crate::git_refs::GIT_CALL_SECONDS);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.code() == Some(0),
            Ok(None) => {}
            Err(_) => return false,
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
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
