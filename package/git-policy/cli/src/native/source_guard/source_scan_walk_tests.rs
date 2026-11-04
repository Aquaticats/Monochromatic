//! The strict walk of a source tree: every `.rs` file under a root, or an error saying why the walk cannot be trusted.
//! A listing that skips an unreadable entry, a link, or a whole empty tree would let a scan pass over sources it never
//! read, so each of those is an error here instead.

use std::path::{Path, PathBuf};

/// What:
///  `fn relative_name(root: &Path, path: &Path) -> Result<String, String>` turns an absolute `path` under `root` into
///  the `/`-separated text every scan uses as the identity of a source file. `Path` is Rust's operating-system path
///  type (sibling: `PathBuf`, the owned form); `String` is an owned text buffer (sibling: `&str`, a borrowed view);
///  `Result<String, String>` is "success text or failure text" (sibling: `Option<String>`, which cannot say why).
/// Why:
///  A path joined with `/` reads the same on every host, and spells the identity an exception names, so a nested file
///  never collides with a file of the same name elsewhere. `String` and not `&str` because the caller keeps the text
///  after this function returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativeName(root: string, path: string): string  // throws Error
/// ```
fn relative_name(root: &Path, path: &Path) -> Result<String, String> {
    // What:
    //  `path.strip_prefix(root)` removes the `root` front from `path`, giving `Err` when `path` is not under `root`.
    //  `.map_err(|error| ...)` rewrites that error into our own text; `|error| ...` is a closure (an arrow function
    //  in TS). `?` returns early with the `Err` and otherwise unwraps the `Ok` value.
    // Why:
    //  A file outside the root would be an identity that no exception can name, so the walk stops.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const below = stripPrefix(path, root) ?? throwError(`${path} is not under ${root}`)
    // ```
    let below: &Path = path
        .strip_prefix(root)
        .map_err(|error| return format!("{} is not under {}: {error}", path.display(), root.display()))?;
    // What:
    //  `Vec<String>` is a growable list of owned text (sibling: `&[String]`, a borrowed view; `[String; N]`, a
    //  fixed-size array). `Vec::new()` is an empty one. `mut` allows the list to change.
    // Why:
    //  Each path component is collected before joining.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const parts: string[] = []
    // ```
    let mut parts: Vec<String> = Vec::new();
    for component in below.components() {
        // What:
        //  `component.as_os_str().to_str()` asks for the component as UTF-8 text; it gives `Some(text)` or `None`
        //  (`Option` is Rust's "maybe a value", where TS would use `string | undefined`). `match` picks a branch by
        //  which one it is. `.to_owned()` copies the borrowed text into an owned `String`.
        // Why:
        //  A name that is not UTF-8 cannot be written in an exception, so it stops the walk.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const text = component.toText()
        // if (text === undefined) throw new Error(`${path} has a name that is not UTF-8`)
        // parts.push(text)
        // ```
        match component.as_os_str().to_str() {
            Some(text) => parts.push(text.to_owned()),
            None => return Err(format!("{} has a name that is not UTF-8", path.display())),
        }
    }
    // What:
    //  `Ok(parts.join("/"))` wraps the joined text in the success variant of `Result`; with no trailing `;` it is
    //  the function's return value.
    // Why:
    //  Hand the identity to the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return parts.join('/')
    // ```
    return Ok(parts.join("/"));
}

/// What:
///  `pub(crate) fn discover(root: &Path) -> Result<Vec<String>, String>` lists every `.rs` file under `root`,
///  sorted, as `/`-separated names relative to `root`. `pub(crate)` means other modules of this crate may call it,
///  outside code may not (TS: an export that is not in the package's public entry).
/// Why:
///  Every scan needs the same complete listing. It fails, instead of returning less, when a directory cannot be
///  listed, an entry cannot be inspected, any entry is a symbolic link (its target may be outside the tree, or
///  absent), or no source file is found at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function discover(root: string): string[]  // throws Error
/// ```
pub(crate) fn discover(root: &Path) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    // What:
    //  `vec![root.to_path_buf()]` makes a list holding one owned copy of the root path (`vec![...]` is a macro,
    //  Rust's compile-time code expansion, here only a list literal; `PathBuf` is the owned sibling of `&Path`).
    // Why:
    //  A work list of directories still to read replaces recursion, so a deep tree cannot overflow the call stack.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pending: string[] = [root]
    // ```
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    // What:
    //  `while let Some(directory) = pending.pop()` loops while `pop()` yields `Some`; `pop()` removes the last item.
    // Why:
    //  Visit each directory exactly once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let directory
    // while ((directory = pending.pop()) !== undefined) { ... }
    // ```
    while let Some(directory) = pending.pop() {
        let entries: std::fs::ReadDir = std::fs::read_dir(&directory)
            .map_err(|error| return format!("cannot list {}: {error}", directory.display()))?;
        for item in entries {
            // The entry itself can fail to read, which a plain `.flatten()` would skip silently.
            let entry: std::fs::DirEntry =
                item.map_err(|error| return format!("cannot read an entry of {}: {error}", directory.display()))?;
            let path: PathBuf = entry.path();
            // `file_type()` does not follow links, so a link is seen as a link.
            let kind: std::fs::FileType =
                entry.file_type().map_err(|error| return format!("cannot inspect {}: {error}", path.display()))?;
            if kind.is_symlink() {
                return Err(format!("{} is a symbolic link; the scan cannot prove what it reads", path.display()));
            }
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().is_some_and(|extension| return extension == "rs") {
                found.push(relative_name(root, &path)?);
            }
        }
    }
    if found.is_empty() {
        return Err(format!("no .rs file under {}; an empty listing proves nothing", root.display()));
    }
    found.sort();
    // What:
    //  `Ok(found)` is the success variant of `Result` carrying the sorted list; it is the tail expression, so it is
    //  returned.
    // Why:
    //  Sorted output makes failure messages and comparisons stable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return found
    // ```
    return Ok(found);
}
