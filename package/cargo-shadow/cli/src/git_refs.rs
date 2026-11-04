//! What:
//! A reader of a Git repository's `HEAD`,
//! branch configuration and references (loose files and `packed-refs`),
//! for the plain file layouts only.
//! Why:
//! The shadow runs before every cargo command,
//! and the owner's rules are "Invoke real git as rarely as possible."
//! and
//! "embed instead of spawning" (`package/git-policy/cli/SPEC.md`).
//! Every function answers `Err(reason)` for a layout or setting it cannot read exactly:
//! the reftable reference format,
//! any repository extension,
//! a configuration `include` or `includeIf`,
//! a symbolic reference where a commit was expected,
//! an ambiguous short name,
//! a reference outside `refs/heads`,
//! `refs/remotes` and `refs/tags`,
//! and the `GIT_*` variables that move the repository or the configuration.
//! The caller then asks Git once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-refs.ts
//! export function findLayout(root: string, env: NativeEnvironment): GitLayout  // throws: ask Git
//! export function lookupRef(layout: GitLayout, name: string): string | undefined  // throws: ask Git
//! export function trackingRef(config: ParsedConfig, layout: GitLayout, branch: string): string | undefined  // throws: ask Git
//! ```

/// Imports `fs` to read the repository files.
use std::fs;
/// Imports the borrowed and owned path types.
use std::os::unix::fs::MetadataExt;
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports the parsed configuration and the lexer entry point.
use crate::git_config::ParsedConfig;
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports the object name length shared with the object check.
use crate::git_objects::OBJECT_NAME_DIGITS;
// Imports `crate::git_refspec::{is_plain_ref_name, map...`.
/// Imports the refspec checks and the mapping through them.
use crate::git_refspec::{is_plain_ref_name, map_through_refspecs};
/// Re-exports the configuration reader,
/// which lives in `git_settings.rs`.
pub use crate::git_settings::read_config;

/// The environment variables that move the repository,
/// its references,
/// its objects or its configuration;
/// when one is set the answer is Git's,
/// not this reader's.
const BLOCKING_VARIABLES: [&str; 17] = [
    "GIT_DIR",
    "GIT_COMMON_DIR",
    "GIT_WORK_TREE",
    "GIT_NAMESPACE",
    "GIT_CONFIG",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_NOSYSTEM",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CEILING_DIRECTORIES",
    "GIT_DISCOVERY_ACROSS_FILESYSTEM",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_REPLACE_REF_BASE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REF_PARANOIA",
];

/// The reference namespaces whose files live in the common directory and that the reader looks up.
const SHARED_NAMESPACES: [&str; 3] = ["refs/heads/", "refs/remotes/", "refs/tags/"];

/// The place the process environment puts Git's inputs,
/// so tests can give a disposable home.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeEnvironment {
    /// The home directory (`HOME`),
    /// where `~/.gitconfig` and `~/.config/git/config` live.
    pub home: Option<PathBuf>,
    /// `XDG_CONFIG_HOME`,
    /// when set and not empty.
    pub xdg_config_home: Option<PathBuf>,
    /// The system configuration file.
    /// This is `/etc/gitconfig`,
    /// which distribution builds of Git read;
    /// a Git built for another prefix reads elsewhere and is not seen
    /// (the limit is stated in the README of this package).
    pub system_config: Option<PathBuf>,
    /// The name of the first blocking variable that is set.
    pub blocking_variable: Option<String>,
    /// The Git program that answers when this reader declines.
    pub git: GitProgram,
    /// The user id Git compares the repository's owner with (its dubious-ownership check).
    pub user_id: u32,
}

/// How the Git program that answers a declined choice is started and bounded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitProgram {
    /// The program to start.
    pub program: PathBuf,
    /// The longest one Git call may take (owner rule of 2026-11-03:
    /// "nothing should ever wait forever or indefinitely");
    /// a bound chosen for local repository reads,
    /// not measured.
    pub bound: std::time::Duration,
    /// Start Git with an empty environment plus `environment` instead of this process's own.
    pub clean_environment: bool,
    /// Variables set for the Git call.
    pub environment: Vec<(String, String)>,
}

/// The bound of one Git call,
/// in seconds.
/// Measured 2026-11-04 on this repository (load average 6.2 to 6.9 on 32 CPUs,
/// capped scopes,
/// 3 runs each,
/// `~/temp/agent/bench_bounds.py` and `bench_online.py`):
/// `git cat-file --batch-check` with the four inputs took 0.007 s to 0.018 s,
/// `git rev-parse --verify HEAD^{commit}` 0.007 s to 0.008 s,
/// `git show HEAD:<dir>/Cargo.lock` 0.008 s to 0.009 s and `git var GIT_CONFIG_SYSTEM` 0.006 s to 0.007 s,
/// all in a warm repository.
/// A minute is about 3,000 times the slowest of these and leaves room for a cold disk or a promisor fetch,
/// neither of which was measured.
/// The owner's rule of 2026-11-03 justifies that a bound exists;
/// the number is a Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"), veto open.
pub const GIT_CALL_SECONDS: u64 = 60;

/// The methods of the type above.
impl Default for GitProgram {
    /// What:
    /// `fn default() -> GitProgram` is the Git on the search path,
    /// with the process environment and the standard bound.
    /// Why:
    /// Production starts `git` as the user's shell would.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const defaultGit = { program: 'git', boundSeconds: 60, cleanEnvironment: false, environment: [] };
    /// ```
    fn default() -> GitProgram {
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
        // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: T[] / Set / Map / number
        // ```
        return GitProgram { program: PathBuf::from("git"), bound: std::time::Duration::from_secs(GIT_CALL_SECONDS), clean_environment: false, environment: Vec::new() };
    }
}

/// What:
/// `pub fn current_user_id() -> u32` is the effective user id of this process.
/// Why:
/// Git refuses a repository that another user owns unless it is declared safe;
/// the reader leaves such a repository to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const currentUserId = (): number => process.geteuid();
/// ```
pub fn current_user_id() -> u32 {
    return rustix::process::geteuid().as_raw();
}

/// The constructor that reads the process environment.
impl NativeEnvironment {
    /// What:
    /// `pub fn from_process() -> NativeEnvironment` reads the inputs from the process environment.
    /// Why:
    /// Production code asks the process;
    /// tests build the struct by hand.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromProcess(): NativeEnvironment
    /// ```
    pub fn from_process() -> NativeEnvironment {
        let home: Option<PathBuf> = std::env::var_os("HOME").map(PathBuf::from);
        let xdg_config_home: Option<PathBuf> = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
            _ => None,
        };
        // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
        // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // (a, b) => body
        // ```
        // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
        // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // String(value)  // JS strings are always owned values
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
        // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
        // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const name: Type = value;
        // ```
        // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
        // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let blocking_variable: Option<String> = BLOCKING_VARIABLES.iter().find(is_set).map(ToString::to_string);
        // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
        // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Array.from(iterable)  // or [...iterable]
        // ```
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        // Adapter `.map(`: transforms or selects items with a closure.
        // Typed binding: `other_system_configs: Vec<PathBuf>`.
        return NativeEnvironment { home, xdg_config_home, system_config: None, blocking_variable, git: GitProgram::default(), user_id: current_user_id() };
    }
}

/// Where a repository's files are.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitLayout {
    /// The directory of this working tree's own files (`HEAD`).
    pub git_dir: PathBuf,
    /// The directory of the shared files (`config`,
    /// `refs`,
    /// `packed-refs`,
    /// `objects`).
    pub common_dir: PathBuf,
}

/// What `HEAD` holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Head {
    /// A symbolic reference to a branch,
    /// with the full reference name.
    Branch(
        /// The full reference name,
        /// for example `refs/heads/main`.
        String,
    ),
    /// A detached `HEAD`:
    /// an object name.
    Detached(
        /// The object name.
        String,
    ),
}

/// The base the reader chose,
/// before the lockfile is read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBase {
    /// How Git would spell it.
    pub spelling: String,
    /// The commit,
    /// or the message to show when the base names no commit.
    pub commit: Result<String, String>,
    /// How many Git starts the reader itself needed (the one question about the system configuration,
    /// once per Git program).
    pub git_starts: u32,
}

/// What:
/// `fn is_set(name: &&str) -> bool` says whether the environment variable of that name is set.
/// Why:
/// It names the test that finds the first blocking variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSet = (name: string): boolean => process.env[name] !== undefined;
/// ```
fn is_set(name: &&&str) -> bool {
    return std::env::var_os(name).is_some();
}

/// What:
/// `fn is_git_space(character: char) -> bool` is Git's `isspace`:
/// space,
/// tab,
/// newline and carriage return only.
/// Why:
/// Rust's own whitespace test also drops Unicode spaces such as the no-break space,
/// which are legal inside a reference name or a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isGitSpace = (character: string): boolean => ' \t\n\r'.includes(character);
/// ```
fn is_git_space(character: char) -> bool {
    return matches!(character, ' ' | '\t' | '\n' | '\r');
}

/// What:
/// `fn is_unsafe_component(part: &str) -> bool` says whether a path component of a reference name is empty,
/// `.` or `..`.
/// Why:
/// Such a component would leave or confuse the repository directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isUnsafeComponent = (part: string): boolean => part === '' || part === '.' || part === '..';
/// ```
fn is_unsafe_component(part: &str) -> bool {
    return part.is_empty() || part == ".." || part == ".";
}

/// What:
/// `fn lacks_text(value: &Option<&str>) -> bool` says whether a variable occurrence has no value or an empty one.
/// Why:
/// Git rejects such a `branch.<name>.remote`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lacksText = (value: string | undefined): boolean => value === undefined || value === '';
/// ```
fn lacks_text(value: &Option<&str>) -> bool {
    return value.is_none_or(str::is_empty);
}

/// What:
/// `fn is_line_end(character: char) -> bool` says whether a character ends a line (`\n` or `\r`).
/// Why:
/// Review finding (3a 3):
/// Git drops only the line ending from the path in a `.git` file or a `commondir` file;
/// a space or tab at the end of the path belongs to the directory name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLineEnd = (character: string): boolean => character === '\n' || character === '\r';
/// ```
fn is_line_end(character: char) -> bool {
    // Macro `matches!`: builds or prints text (or a list) from a template.
    return matches!(character, '\n' | '\r');
}

/// What:
/// `fn is_symbolic_link(path: &Path) -> bool` says whether the path is a symbolic link (without following it).
/// Why:
/// Review finding (3a 1):
/// Git reads a symbolic-link `HEAD` or reference as a symbolic reference
/// (`core.preferSymlinkRefs`);
/// reading through it would see the target's object name instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSymbolicLink = (path: string): boolean => lstatSync(path, { throwIfNoEntry: false })?.isSymbolicLink() ?? false;
/// ```
fn is_symbolic_link(path: &Path) -> bool {
    // Closure `|metadata|`: an anonymous function, like a TS arrow function.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // Adapter `.map(`: transforms or selects items with a closure.
    return fs::symlink_metadata(path).map(|metadata| { return metadata.file_type().is_symlink(); }).unwrap_or(false);
}

/// What:
/// `fn trim_space(text: &str) -> &str` trims the characters Git's `isspace` knows (space,
/// tab,
/// newline,
/// carriage return).
/// Why:
/// Rust's `trim` also drops Unicode spaces such as the no-break space,
/// which are legal inside a reference name or a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trimSpace = (text: string): string => text.replace(/^[ \t\r\n]+|[ \t\r\n]+$/g, '');
/// ```
fn trim_space(text: &str) -> &str {
    // Closure `|character|`: an anonymous function, like a TS arrow function.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    return text.trim_matches(is_git_space);
}

/// What:
/// `fn is_object_name(text: &str) -> bool` says whether the text is a SHA-1 object name of lowercase or uppercase hexadecimal digits.
/// Why:
/// Other lengths belong to other object formats,
/// which the reader leaves to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isObjectName = (text: string): boolean => /^[0-9a-fA-F]{40}$/.test(text);
/// ```
fn is_object_name(text: &str) -> bool {
    // Closure `|character|`: an anonymous function, like a TS arrow function.
    return text.len() == OBJECT_NAME_DIGITS && text.as_bytes().iter().all(u8::is_ascii_hexdigit);
}

/// What:
/// `fn cannot_read(path: &Path, error: &std::io::Error) -> String` words a read failure with the path.
/// Why:
/// Every error names what could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cannotRead = (path: string, error: Error) => `${path} cannot be read: ${error}`;
/// ```
fn cannot_read(path: &Path, error: &std::io::Error) -> String {
    return format!("{} cannot be read: {error}", path.display());
}

/// What:
/// `fn read_text(path: &Path) -> Result<String, String>` reads a text file.
/// Why:
/// Failures carry the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readText = (path: string): string => readFileSync(path, 'utf8');
/// ```
fn read_text(path: &Path) -> Result<String, String> {
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    return fs::read_to_string(path).map_err(|error| { return cannot_read(path, &error); });
}

/// What:
/// `fn read_optional(path: &Path) -> Result<Option<String>, String>` reads a text file that may be absent.
/// Why:
/// Only a missing file is absence;
/// a permission error or invalid UTF-8 is a failure the reader does not guess around.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readOptional = (path: string): string | undefined => ...
/// ```
pub(crate) fn read_optional(path: &Path) -> Result<Option<String>, String> {
    return match fs::read_to_string(path) {
        // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
        // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Ok(value)  is  return value;
        // // Err(error) is  throw error;   (the caller sees it as a returned value)
        // ```
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        Ok(text) => Ok(Some(text)),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(cannot_read(path, &error)),
    };
}

/// What:
/// `fn check_git_directory(directory: &Path) -> Result<(), String>` requires the files every Git directory has.
/// Why:
/// A directory without `HEAD`,
/// `objects` and `refs` is not one Git would accept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGitDirectory(directory: string): void  // throws
/// ```
fn check_git_directory(directory: &Path) -> Result<(), String> {
    for required in ["HEAD", "objects", "refs"] {
        if !directory.join(required).exists() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{} has no {required}, so it is not a Git directory", directory.display()));
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// What:
/// `pub fn find_layout(root: &Path, environment: &NativeEnvironment) -> Result<GitLayout, String>` finds the Git directory of a
/// working tree and its common directory.
/// Why:
/// A `.git` file says `gitdir: ` and then a path that keeps any leading space (only trailing whitespace is dropped);
/// a `commondir` file is absent or it is read exactly,
/// and any other failure is the caller's cue to ask Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findLayout(root: string, env: NativeEnvironment): GitLayout  // throws: ask Git
/// ```
pub fn find_layout(root: &Path, environment: &NativeEnvironment) -> Result<GitLayout, String> {
    if let Some(name) = &environment.blocking_variable {
        return Err(format!("{name} is set, so only Git can say where the repository is"));
    }
    let dot_git: PathBuf = root.join(".git");
    // `.clone()`: an independent owned copy.
    // Typed binding: `dot_git_path: PathBuf`.
    let dot_git_path: PathBuf = dot_git.clone();
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&dot_git`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Review finding (4a 5): Git inspects the `.git` entry itself (an `lstat`), so a link is not followed and is Git's to judge.
    let metadata = fs::symlink_metadata(&dot_git).map_err(|error| { return cannot_read(&dot_git, &error); })?;
    if metadata.file_type().is_symlink() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{} is a symbolic link, which only Git judges", dot_git.display()));
    }
    let git_dir: PathBuf = if metadata.is_dir() {
        dot_git
    } else {
        let text: String = read_text(&dot_git)?;
        // `Some`: the present or the absent value of an `Option`.
        // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
        // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (value !== undefined) { const inner = value; ... }
        // ```
        let Some(target) = text.strip_prefix("gitdir: ") else {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{} is a file but does not start with \"gitdir: \"", dot_git.display()));
        };
        // Closure `|character|`: an anonymous function, like a TS arrow function.
        // Typed binding: `path: &str`.
        // Macro `matches!`: builds or prints text (or a list) from a template.
        let path: &str = target.trim_end_matches(is_line_end);
        if path.is_empty() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{} names no directory", dot_git.display()));
        }
        root.join(path)
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    // Borrow `&git_dir.join`: lends the value without copying or giving it away.
    // Typed binding: `common_dir: PathBuf`.
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let common_dir: PathBuf = match read_optional(&git_dir.join("commondir"))? {
        // `Some`: the present or the absent value of an `Option`.
        // Closure `|character|`: an anonymous function, like a TS arrow function.
        // Macro `matches!`: builds or prints text (or a list) from a template.
        Some(text) => {
            // Typed binding: `path: &str`.
            let path: &str = text.trim_end_matches(is_line_end);
            if path.is_empty() {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                return Err(format!("{} is empty, which Git refuses", git_dir.join("commondir").display()));
            }
            git_dir.join(path)
        }
        // `None`: the present or the absent value of an `Option`.
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        None => git_dir.clone(),
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&common_dir`: lends the value without copying or giving it away.
    check_git_directory(&common_dir)?;
    // Review finding (3a 8): Git refuses a repository another user owns unless it is declared safe; that decision is Git's.
    for owned in [root, dot_git_path.as_path(), git_dir.as_path(), common_dir.as_path()] {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&error`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `owner: u32`.
        let owner: u32 = fs::symlink_metadata(owned).map_err(|error| { return cannot_read(owned, &error); })?.uid();
        if owner != environment.user_id {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{} is owned by another user, so only Git decides whether it is safe to use", owned.display()));
        }
    }
    return Ok(GitLayout { git_dir, common_dir });
}

/// What:
/// `pub fn read_head(layout: &GitLayout) -> Result<Head, String>` reads `HEAD`.
/// Why:
/// It is a symbolic reference or an object name;
/// anything else is Git's to explain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readHead(layout: GitLayout): Head  // throws
/// ```
pub fn read_head(layout: &GitLayout) -> Result<Head, String> {
    let path: PathBuf = layout.git_dir.join("HEAD");
    // Borrow `&path`: lends the value without copying or giving it away.
    if is_symbolic_link(&path) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{} is a symbolic link, which Git reads as a symbolic reference", path.display()));
    }
    let text: String = read_text(&path)?;
    // Borrow `&text`: lends the value without copying or giving it away.
    // Typed binding: `trimmed: &str`.
    let trimmed: &str = trim_space(&text);
    if let Some(name) = trimmed.strip_prefix("ref:") {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Ok(Head::Branch(trim_space(name).to_string()));
    }
    if is_object_name(trimmed) {
        return Ok(Head::Detached(trimmed.to_string()));
    }
    return Err(format!("{} holds neither a reference nor an object name", path.display()));
}

/// What:
/// `fn check_ref_name(name: &str) -> Result<(), String>` refuses a name that could leave the repository directory.
/// Why:
/// The name becomes a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRefName(name: string): void  // throws
/// ```
fn check_ref_name(name: &str) -> Result<(), String> {
    // Closure `|part|`: an anonymous function, like a TS arrow function.
    // Typed binding: `escapes: bool`.
    let escapes: bool = name.is_empty() || name.starts_with('/') || name.split('/').any(is_unsafe_component);
    if escapes {
        return Err(format!("{name:?} is not a reference name this reader will turn into a path"));
    }
    return Ok(());
}

/// What:
/// `fn read_reference(layout: &GitLayout, name: &str) -> Result<Option<String>, String>` reads the object name stored in one
/// reference,
/// loose first and then packed,
/// in the common directory.
/// Why:
/// A loose file shadows `packed-refs`;
/// a symbolic reference or anything that is not an object name is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readReference(layout: GitLayout, name: string): string | undefined  // throws
/// ```
fn read_reference(layout: &GitLayout, name: &str) -> Result<Option<String>, String> {
    check_ref_name(name)?;
    let loose: PathBuf = layout.common_dir.join(name);
    // Borrow `&loose`: lends the value without copying or giving it away.
    if is_symbolic_link(&loose) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{} is a symbolic link, which Git reads as a symbolic reference", loose.display()));
    }
    match fs::read_to_string(&loose) {
        Ok(text) => {
            // Borrow `&text`: lends the value without copying or giving it away.
            // Typed binding: `trimmed: &str`.
            let trimmed: &str = trim_space(&text);
            if is_object_name(trimmed) {
                return Ok(Some(trimmed.to_string()));
            }
            return Err(format!("{} holds something other than an object name (a symbolic reference?)", loose.display()));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound || loose.is_dir() => {}
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Borrow `&loose`: lends the value without copying or giving it away.
        Err(error) => return Err(cannot_read(&loose, &error)),
    }
    // `Some`: the present or the absent value of an `Option`.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&layout.common_dir.join`: lends the value without copying or giving it away.
    let Some(packed) = read_optional(&layout.common_dir.join("packed-refs"))? else {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        return Ok(None);
    };
    for line in packed.lines() {
        if line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        if let Some((object, reference)) = line.split_once(' ')
            && reference == name
            && is_object_name(object)
        {
            return Ok(Some(object.to_string()));
        }
    }
    return Ok(None);
}

/// What:
/// `pub fn lookup_ref(layout: &GitLayout, name: &str) -> Result<Option<String>, String>` reads the object name stored in a full
/// reference name under `refs/heads/`,
/// `refs/remotes/` or `refs/tags/`.
/// Why:
/// Other namespaces include the worktree-private ones (`refs/worktree/`,
/// `refs/bisect/`,
/// `refs/rewritten/`) whose files are not in
/// the common directory;
/// they are Git's to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lookupRef(layout: GitLayout, name: string): string | undefined  // throws: ask Git
/// ```
pub fn lookup_ref(layout: &GitLayout, name: &str) -> Result<Option<String>, String> {
    // Closure `|namespace|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    if !SHARED_NAMESPACES.iter().any(|namespace| { return name.starts_with(namespace); }) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{name:?} is outside refs/heads, refs/remotes and refs/tags, so only Git reads it"));
    }
    return read_reference(layout, name);
}

/// What:
/// `pub fn lookup_short(layout: &GitLayout, name: &str) -> Result<Option<String>, String>` resolves a short name the way Git's
/// reference lookup does for a remote-tracking branch (`origin/main`),
/// or answers `Err` when anything else could also be named by it.
/// Why:
/// Git tries `name`,
/// `refs/name`,
/// `refs/tags/name`,
/// `refs/heads/name`,
/// `refs/remotes/name` and `refs/remotes/name/HEAD` in that
/// order and warns when several exist;
/// the reader answers only when exactly the remote-tracking reference exists and nothing earlier does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lookupShort(layout: GitLayout, name: string): string | undefined  // throws: ask Git
/// ```
pub fn lookup_short(layout: &GitLayout, name: &str) -> Result<Option<String>, String> {
    check_ref_name(name)?;
    // Borrow `&layout.common_dir`: lends the value without copying or giving it away.
    for directory in [&layout.common_dir, &layout.git_dir] {
        if directory.join(name).exists() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{name:?} also names a file in the repository directory"));
        }
    }
    for prefix in ["refs/", "refs/tags/", "refs/heads/"] {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        if read_reference(layout, &format!("{prefix}{name}"))?.is_some() {
            return Err(format!("{name:?} is ambiguous: {prefix}{name} exists"));
        }
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&format`: lends the value without copying or giving it away.
    // Typed binding: `found: Option<String>`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let found: Option<String> = lookup_ref(layout, &format!("refs/remotes/{name}"))?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&format`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    if found.is_none() && read_reference(layout, &format!("refs/remotes/{name}/HEAD"))?.is_some() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{name:?} resolves through refs/remotes/{name}/HEAD, which only Git reads"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(found);
}

/// What:
/// `fn single_value<'a>(values: &[Option<&'a str>], key: &str) -> Result<Option<&'a str>, String>` takes the last value of a
/// variable and refuses an occurrence without a value or an empty one.
/// Why:
/// Git reports a missing value for `branch.<name>.remote` as an error instead of reading it as a string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function singleValue(values: (string | undefined)[], key: string): string | undefined  // throws on a bare key
/// ```
fn single_value<'a>(values: &[Option<&'a str>], key: &str) -> Result<Option<&'a str>, String> {
    // Closure `|value|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    if values.iter().any(lacks_text) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{key} is written without a value, which Git rejects"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    return Ok(values.last().copied().flatten());
}

/// What:
/// `pub fn tracking_ref(config: &ParsedConfig, layout: &GitLayout, branch: &str) -> Result<Option<String>, String>` returns the
/// reference that `@{upstream}` of a branch names:
/// the branch's first `merge` value,
/// mapped through the remote's fetch refspecs (or as is for the remote `.`).
/// Why:
/// Git's `@{upstream}` uses the first `branch.<name>.merge` value and the last `branch.<name>.remote`,
/// and rejects the remote when any of its refspecs is invalid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trackingRef(config: ParsedConfig, layout: GitLayout, branch: string): string | undefined  // throws: ask Git
/// ```
pub fn tracking_ref(config: &ParsedConfig, layout: &GitLayout, branch: &str) -> Result<Option<String>, String> {
    // `Some`: the present or the absent value of an `Option`.
    // Typed binding: `remotes: Vec<Option<&str>>`.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let remotes: Vec<Option<&str>> = config.values("branch", Some(branch), "remote");
    // `Some`: the present or the absent value of an `Option`.
    // Typed binding: `merges: Vec<Option<&str>>`.
    let merges: Vec<Option<&str>> = config.values("branch", Some(branch), "merge");
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&remotes`: lends the value without copying or giving it away.
    // Typed binding: `remote_name: Option<&str>`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let remote_name: Option<&str> = single_value(&remotes, &format!("branch.{branch}.remote"))?;
    // Closure `|value|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    if merges.iter().any(Option::is_none) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("branch.{branch}.merge is written without a value, which Git rejects"));
    }
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    let (Some(remote), Some(Some(merge))) = (remote_name, merges.first().copied()) else {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        return Ok(None);
    };
    if !merge.starts_with("refs/") || !is_plain_ref_name(merge) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("branch.{branch}.merge is {merge:?}, not a plain full reference name"));
    }
    if remote != "." && !is_plain_ref_name(remote) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("branch.{branch}.remote is {remote:?}, which is not a plain remote name"));
    }
    for legacy in ["remotes", "branches"] {
        if layout.common_dir.join(legacy).join(remote).exists() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("the remote {remote:?} is also described by a legacy {legacy}/ file, which only Git reads"));
        }
    }
    // `Some`: the present or the absent value of an `Option`.
    // Typed binding: `specs: Vec<Option<&str>>`.
    let specs: Vec<Option<&str>> = config.values("remote", Some(remote), "fetch");
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `texts: Vec<&str>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut texts: Vec<&str> = Vec::new();
    for spec in specs {
        // `match`: picks the arm whose pattern fits the value.
        match spec {
            // `Some`: the present or the absent value of an `Option`.
            Some(text) => texts.push(text),
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            None => return Err(format!("remote.{remote}.fetch is written without a value, which Git rejects")),
        }
    }
    // Borrow `&texts`: lends the value without copying or giving it away.
    // The remote "." is the local repository: its own fetch refspecs map first, and only without a match the merge name is the
    // local reference (review finding 3a 2).
    return match map_through_refspecs(&texts, merge)? {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        Some(mapped) => Ok(Some(mapped)),
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        None if remote == "." => Ok(Some(merge.to_string())),
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        None => Ok(None),
    };
}

/// What:
/// `pub fn tracks_locally(config: &ParsedConfig, branch: &str) -> bool` says whether the branch's upstream is in this repository
/// (its remote is `.`).
/// Why:
/// Git resolves such a merge name with its name expansion when the exact reference is missing,
/// so a missing reference there is not a plain "no upstream yet".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function tracksLocally(config: ParsedConfig, branch: string): boolean
/// ```
pub fn tracks_locally(config: &ParsedConfig, branch: &str) -> bool {
    // `Some`: the present or the absent value of an `Option`.
    return config.last("branch", Some(branch), "remote") == Some(".");
}
