//! What:
//! A reader of where a Git repository's files are and what `HEAD` holds,
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
//! What a reference names (loose files and `packed-refs`)
//! and the reference a branch's upstream names
//! live in `git_reference_lookup.rs`,
//! so this file reads only the layout and `HEAD`;
//! those items are re-exported here,
//! so every caller keeps the path it had.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-refs.ts
//! export function findLayout(root: string, env: NativeEnvironment): GitLayout  // throws: ask Git
//! export function readHead(layout: GitLayout): Head  // throws: ask Git
//! export { lookupRef, lookupShort, trackingRef, tracksLocally } from './git-reference-lookup.ts'
//! ```

/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::ExplainedError;
/// Imports `fs` to read the repository files.
use std::fs;
/// Imports the borrowed and owned path types.
use std::os::unix::fs::MetadataExt;
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{
    Path,
    PathBuf,
};

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports the object name length shared with the object check.
use crate::git_objects::OBJECT_NAME_DIGITS;
/// Re-exports the configuration reader,
/// which lives in `git_settings.rs`.
pub use crate::git_settings::read_config;

/// Re-exports the reference lookup and the branch upstream,
/// which live in their own file (split out of this module),
/// so every caller keeps the path it had.
pub use crate::git_reference_lookup::{
    lookup_ref,
    lookup_short,
    tracking_ref,
    tracks_locally,
};

/// The program and the user id that only the tests of the Git reader use.
#[cfg(test)]
pub(crate) use crate::git_refs_environment::{
    GitProgram,
    current_user_id,
};

/// The environment and the Git program a native read runs with.
/// (split out of the git refs module).
pub(crate) use crate::git_refs_environment::{
    GIT_CALL_SECONDS,
    NativeEnvironment,
    SHARED_NAMESPACES,
};

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
pub(crate) fn is_set(name: &&&str) -> bool {
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
pub(crate) fn is_unsafe_component(part: &str) -> bool {
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
pub(crate) fn lacks_text(value: &Option<&str>) -> bool {
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
pub(crate) fn is_symbolic_link(path: &Path) -> bool {
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
    return match fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type().is_symlink(),
        Err(_) => false,
    };
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
pub(crate) fn trim_space(text: &str) -> &str {
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
pub(crate) fn is_object_name(text: &str) -> bool {
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
pub(crate) fn cannot_read(
    path: &Path,
    error: &std::io::Error,
) -> String {
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
    return fs::read_to_string(path).described(format_args!("{} cannot be read", path.display()));
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
            return Err(format!(
                "{} has no {required}, so it is not a Git directory",
                directory.display()
            ));
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
pub fn find_layout(
    root: &Path,
    environment: &NativeEnvironment,
) -> Result<GitLayout, String> {
    if let Some(name) = &environment.blocking_variable {
        return Err(format!(
            "{name} is set, so only Git can say where the repository is"
        ));
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
    let metadata = fs::symlink_metadata(&dot_git)
        .described(format_args!("{} cannot be read", dot_git.display()))?;
    if metadata.file_type().is_symlink() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!(
            "{} is a symbolic link, which only Git judges",
            dot_git.display()
        ));
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
            return Err(format!(
                "{} is a file but does not start with \"gitdir: \"",
                dot_git.display()
            ));
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
                return Err(format!(
                    "{} is empty, which Git refuses",
                    git_dir.join("commondir").display()
                ));
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
    for owned in [
        root,
        dot_git_path.as_path(),
        git_dir.as_path(),
        common_dir.as_path(),
    ] {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&error`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `owner: u32`.
        let owner: u32 = fs::symlink_metadata(owned)
            .described(format_args!("{} cannot be read", owned.display()))?
            .uid();
        if owner != environment.user_id {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!(
                "{} is owned by another user, so only Git decides whether it is safe to use",
                owned.display()
            ));
        }
    }
    return Ok(GitLayout {
        git_dir,
        common_dir,
    });
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
        return Err(format!(
            "{} is a symbolic link, which Git reads as a symbolic reference",
            path.display()
        ));
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
    return Err(format!(
        "{} holds neither a reference nor an object name",
        path.display()
    ));
}

/// What:
/// `#[cfg(test)] mod git_refs_tests;` declares the tests in `src/git_refs_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the plain-file reader of references and branch configuration,
/// over hand-made repository directories."
/// so the behavior of `git_refs` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // git_refs_tests.test.ts, run by the test runner only
/// ```
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "git_refs_tests.rs"]
mod git_refs_tests;
