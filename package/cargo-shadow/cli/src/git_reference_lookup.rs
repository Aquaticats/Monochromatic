//! What:
//! What a reference names:
//! the object name stored in a loose or packed reference file,
//! the short name Git would resolve for a remote-tracking branch,
//! and the reference a branch's upstream names.
//! Why:
//! `git_refs.rs` reads where a repository's files are and what `HEAD` holds;
//! the reference lookup was split out of it to keep each file under the code-line limit.
//! Every function answers `Err(reason)` for a layout or setting it cannot read exactly,
//! and the caller then asks Git once.
//! The items are re-exported from `git_refs`,
//! so their paths there are unchanged.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-reference-lookup.ts
//! export function lookupRef(layout: GitLayout, name: string): string | undefined  // throws: ask Git
//! export function lookupShort(layout: GitLayout, name: string): string | undefined  // throws: ask Git
//! export function trackingRef(config: ParsedConfig, layout: GitLayout, branch: string): string | undefined  // throws: ask Git
//! ```

/// Imports `fs` to read the loose reference files.
use std::fs;
/// Imports the owned path type of the loose reference file it builds.
use std::path::PathBuf;

/// Imports the parsed configuration and the lexer entry point.
use crate::git_config::ParsedConfig;
/// Imports the refspec checks and the mapping through them.
use crate::git_refspec::{
    is_plain_ref_name,
    map_through_refspecs,
};
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports the read helpers and the reference-name checks from `git_refs`,
/// which holds them beside `HEAD`'s own reading.
use crate::git_refs::{
    GitLayout,
    SHARED_NAMESPACES,
    cannot_read,
    is_object_name,
    is_symbolic_link,
    is_unsafe_component,
    lacks_text,
    read_optional,
    trim_space,
};
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
    let escapes: bool =
        name.is_empty() || name.starts_with('/') || name.split('/').any(is_unsafe_component);
    if escapes {
        return Err(format!(
            "{name:?} is not a reference name this reader will turn into a path"
        ));
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
fn read_reference(
    layout: &GitLayout,
    name: &str,
) -> Result<Option<String>, String> {
    check_ref_name(name)?;
    let loose: PathBuf = layout.common_dir.join(name);
    // Borrow `&loose`: lends the value without copying or giving it away.
    if is_symbolic_link(&loose) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!(
            "{} is a symbolic link, which Git reads as a symbolic reference",
            loose.display()
        ));
    }
    match fs::read_to_string(&loose) {
        Ok(text) => {
            // Borrow `&text`: lends the value without copying or giving it away.
            // Typed binding: `trimmed: &str`.
            let trimmed: &str = trim_space(&text);
            if is_object_name(trimmed) {
                return Ok(Some(trimmed.to_string()));
            }
            return Err(format!(
                "{} holds something other than an object name (a symbolic reference?)",
                loose.display()
            ));
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
pub fn lookup_ref(
    layout: &GitLayout,
    name: &str,
) -> Result<Option<String>, String> {
    // Closure `|namespace|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    let mut in_shared_namespace: bool = false;
    for namespace in SHARED_NAMESPACES {
        if name.starts_with(namespace) {
            in_shared_namespace = true;
            break;
        }
    }
    if !in_shared_namespace {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!(
            "{name:?} is outside refs/heads, refs/remotes and refs/tags, so only Git reads it"
        ));
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
pub fn lookup_short(
    layout: &GitLayout,
    name: &str,
) -> Result<Option<String>, String> {
    check_ref_name(name)?;
    // Borrow `&layout.common_dir`: lends the value without copying or giving it away.
    for directory in [&layout.common_dir, &layout.git_dir] {
        if directory.join(name).exists() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!(
                "{name:?} also names a file in the repository directory"
            ));
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
        return Err(format!(
            "{name:?} resolves through refs/remotes/{name}/HEAD, which only Git reads"
        ));
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
fn single_value<'a>(
    values: &[Option<&'a str>],
    key: &str,
) -> Result<Option<&'a str>, String> {
    // Closure `|value|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    if values.iter().any(lacks_text) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!(
            "{key} is written without a value, which Git rejects"
        ));
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
pub fn tracking_ref(
    config: &ParsedConfig,
    layout: &GitLayout,
    branch: &str,
) -> Result<Option<String>, String> {
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
        return Err(format!(
            "branch.{branch}.merge is written without a value, which Git rejects"
        ));
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
        return Err(format!(
            "branch.{branch}.merge is {merge:?}, not a plain full reference name"
        ));
    }
    if remote != "." && !is_plain_ref_name(remote) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!(
            "branch.{branch}.remote is {remote:?}, which is not a plain remote name"
        ));
    }
    for legacy in ["remotes", "branches"] {
        if layout.common_dir.join(legacy).join(remote).exists() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!(
                "the remote {remote:?} is also described by a legacy {legacy}/ file, which only Git reads"
            ));
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
            None => {
                return Err(format!(
                    "remote.{remote}.fetch is written without a value, which Git rejects"
                ));
            }
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
pub fn tracks_locally(
    config: &ParsedConfig,
    branch: &str,
) -> bool {
    // `Some`: the present or the absent value of an `Option`.
    return config.last("branch", Some(branch), "remote") == Some(".");
}
