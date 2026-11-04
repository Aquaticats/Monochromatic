//! What:
//! The choice of the trust base made from the repository's files without starting Git:
//! in CI `HEAD`;
//! locally `@{upstream}`,
//! else `origin/main`,
//! else `origin/master`,
//! else `HEAD`.
//! Why:
//! The owner's rules are "Invoke real git as rarely as possible."
//! and "embed instead of spawning"
//! (`package/git-policy/cli/SPEC.md`);
//! `git_refs.rs` reads the files and this module applies the choice,
//! so each file stays under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-native.ts
//! export function nativeBase(root: string, inCi: boolean, environment: NativeEnvironment): NativeBase  // throws: ask Git
//! ```

/// What:
/// `use std::path::Path;` imports `Path`,
/// the borrowed view of a file path (sibling:
/// `PathBuf`,
/// the owned one).
/// Why:
/// The repository root is lent to `native_base`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::Path;

/// What:
/// `use crate::git_config::ParsedConfig;` imports the parsed configuration type.
/// Why:
/// `native_base` reads the configuration once and passes it to the upstream lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ParsedConfig } from 'crate/git_config';
/// ```
use crate::git_config::ParsedConfig;
/// What:
/// `use crate::git_objects::confirm_commit;` imports the check that an object name names a commit.
/// Why:
/// A reference file alone does not show the object exists or is a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { confirmCommit } from 'crate/git_objects';
/// ```
use crate::git_objects::confirm_commit;
/// What:
/// `use crate::git_refs::{GitLayout, Head, NativeBase, NativeEnvironment, find_layout, lookup_ref, lookup_short, read_config, read_head, tracking_ref};` imports the file readers this choice is made from.
/// Why:
/// Each step of the choice (layout,
/// configuration,
/// `HEAD`,
/// upstream,
/// short names) is one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { GitLayout, Head, NativeBase, NativeEnvironment, findLayout, lookupRef, lookupShort, readConfig, readHead, trackingRef } from 'crate/git_refs';
/// ```
use crate::git_refs::{GitLayout, Head, NativeBase, NativeEnvironment, find_layout, lookup_ref, lookup_short, read_config, read_head, tracking_ref, tracks_locally};

/// What:
/// `fn lookup_tracked(config, layout, branch, tracking) -> Result<Option<String>, String>` reads the upstream reference and
/// declines when a local upstream is missing.
/// Why:
/// For the remote `.` Git expands a missing merge name (`refs/heads/refs/heads/base`),
/// which this reader does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lookupTracked(config: ParsedConfig, layout: GitLayout, branch: string, tracking: string): string | undefined  // throws: ask Git
/// ```
fn lookup_tracked(config: &ParsedConfig, layout: &GitLayout, branch: &str, tracking: &str) -> Result<Option<String>, String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Typed binding: `found: Option<String>`.
    let found: Option<String> = lookup_ref(layout, tracking)?;
    if found.is_none() && tracks_locally(config, branch) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
        // Why:      Text building and printing are written as one call instead of string concatenation.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // `..${x}..`   // template literal
        // console.error(text);   // eprintln!
        // [a, b]            // vec![a, b]
        // ```
        return Err(format!("the local upstream {tracking} does not exist as written, and Git expands such names"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(found);
}

/// What:
/// `fn confirmed(layout, object) -> Result<Result<String, String>, String>` accepts an object name from a reference or `HEAD` as a
/// commit only after the object files show it is one.
/// Why:
/// A reference file proves neither that the object exists nor that it is a commit (a tag peels,
/// a blob is refused);
/// anything the object files cannot show leaves the answer to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function confirmed(layout: GitLayout, object: string): Result<string, string>  // throws: ask Git
/// ```
fn confirmed(layout: &GitLayout, object: String) -> Result<Result<String, String>, String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&layout.common_dir`: lends the value without copying or giving it away.
    confirm_commit(&layout.common_dir, &object)?;
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(Ok(object));
}

/// What:
/// `fn head_base(layout) -> NativeBase` is `HEAD` as a base:
/// its spelling and the commit it names.
/// Why:
/// CI compares with `HEAD`,
/// and a local run falls back to it when no upstream or remote default branch exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function headBase(layout: GitLayout): NativeBase  // throws: ask Git
/// ```
fn head_base(layout: &GitLayout, git_starts: u32) -> Result<NativeBase, String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    // Typed binding: `commit: Result<String, String>`.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let commit: Result<String, String> = match read_head(layout)? {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Head::Detached(object) => confirmed(layout, object)?,
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&name`: lends the value without copying or giving it away.
        Head::Branch(name) => match lookup_ref(layout, &name)? {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `Some`: the present or the absent value of an `Option`.
            Some(object) => confirmed(layout, object)?,
            // An unborn branch names no commit, which Git reports as an error too.
            None => Err(crate::git_base::not_a_commit_message("HEAD")),
        },
    };
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    return Ok(NativeBase { spelling: "HEAD".to_string(), commit, git_starts });
}

/// What:
/// `pub fn native_base(root, in_ci, environment) -> Result<NativeBase, String>` chooses the trust base without Git:
/// in CI `HEAD`;
/// locally `@{upstream}`,
/// else `origin/main`,
/// else `origin/master`,
/// else `HEAD`.
/// `Err(reason)` means Git must answer.
/// Why:
/// This is the choice `git_base::resolve_trust_base` makes with `git rev-parse`,
/// made from the files (delegated pick of
/// 2026-11-03,
/// with the owner's rule "Invoke real git as rarely as possible."
/// from 2026-11-01).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeBase(root: string, inCi: boolean, environment: NativeEnvironment): NativeBase  // throws: ask Git
/// ```
pub fn native_base(root: &Path, in_ci: bool, environment: &NativeEnvironment) -> Result<NativeBase, String> {
    return native_base_counting(root, in_ci, environment, &std::cell::Cell::new(0));
}

/// What:
/// `pub fn native_base_counting(root, in_ci, environment, starts) -> Result<NativeBase, String>` is `native_base` with the count of
/// Git starts kept by the caller.
/// Why:
/// A declined attempt may already have started Git (to ask for its system configuration); the caller adds its own starts to the
/// same count, so a declined choice reports every start it cost (round 6, 5a F6).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeBaseCounting(root: string, inCi: boolean, environment: NativeEnvironment, starts: { count: number }): NativeBase  // throws to decline
/// ```
pub fn native_base_counting(root: &Path, in_ci: bool, environment: &NativeEnvironment, starts: &std::cell::Cell<u32>) -> Result<NativeBase, String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Typed binding: `layout: GitLayout`.
    let layout: GitLayout = find_layout(root, environment)?;
    // The configuration is read even for CI's `HEAD`, because a reftable store or an include changes what `HEAD` means.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Typed binding: `config: ParsedConfig`.
    let config: ParsedConfig = read_config(&layout, environment, root, starts)?;
    if in_ci {
        // Borrow `&layout`: lends the value without copying or giving it away.
        return head_base(&layout, starts.get());
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // Borrow `&layout`: lends the value without copying or giving it away.
    // Typed binding: `Head: :Branch(full)`.
    if let Head::Branch(full) = read_head(&layout)?
        // `Some`: the present or the absent value of an `Option`.
        && let Some(branch) = full.strip_prefix("refs/heads/")
        // `Some`: the present or the absent value of an `Option`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&config`: lends the value without copying or giving it away.
        && let Some(tracking) = tracking_ref(&config, &layout, branch)?
        // `Some`: the present or the absent value of an `Option`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&layout`: lends the value without copying or giving it away.
        && let Some(object) = lookup_tracked(&config, &layout, branch, &tracking)?
    {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Ok(NativeBase { spelling: "@{upstream}".to_string(), commit: confirmed(&layout, object)?, git_starts: starts.get() });
    }
    for candidate in ["origin/main", "origin/master"] {
        // `Some`: the present or the absent value of an `Option`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        // Borrow `&layout`: lends the value without copying or giving it away.
        if let Some(object) = lookup_short(&layout, candidate)? {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Ok(NativeBase { spelling: candidate.to_string(), commit: confirmed(&layout, object)?, git_starts: starts.get() });
        }
    }
    // Borrow `&layout`: lends the value without copying or giving it away.
    return head_base(&layout, starts.get());
}
