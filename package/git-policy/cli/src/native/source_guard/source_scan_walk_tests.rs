//! The strict walk of a source tree:
//! every `.rs` file under a root,
//! or an error saying why the walk cannot be trusted.
//! A listing that skips an unreadable entry,
//! a link,
//! or a whole empty tree would let a scan pass over sources it never
//! read,
//! so each of those is an error here instead.

use std::path::{
    Path,
    PathBuf,
};

// What:
//  `write_tree` is the shared fixture writer of the parent module: it makes each `(name, text)` pair a file.
// Why:
//  The controls below build their trees the same way every other control of these scans does.
//
// In TS you'd write (pseudocode):
// ```ts
// import { writeTree } from './shared'
// ```
use super::write_tree;

/// What:
///  `fn relative_name(root: &Path, path: &Path) -> Result<String, String>` turns an absolute `path` under `root` into
///  the `/`-separated text every scan uses as the identity of a source file.
/// `Path` is Rust's operating-system path
///  type (sibling:
/// `PathBuf`,
/// the owned form);
/// `String` is an owned text buffer (sibling:
/// `&str`,
/// a borrowed view);
///  `Result<String, String>` is "success text or failure text" (sibling:
/// `Option<String>`,
/// which cannot say why).
/// Why:
///  A path joined with `/` reads the same on every host,
/// and spells the identity an exception names,
/// so a nested file
///  never collides with a file of the same name elsewhere.
/// `String` and not `&str` because the caller keeps the text
///  after this function returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativeName(root: string, path: string): string  // throws Error
/// ```
fn relative_name(
    root: &Path,
    path: &Path,
) -> Result<String, String> {
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
    let below: &Path = path.strip_prefix(root).map_err(|error| {
        return format!(
            "{} is not under {}: {error}",
            path.display(),
            root.display()
        );
    })?;
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

/// The name of the folder whose entries are the repository's links to the source folders its code uses:
/// every source folder inside a crate holds one,
/// and the layout lint compares it with the folder's `use` lines
/// (`AGENTS.md`,
/// rules `LYD` and `LYL`).
const FOLDER_USES_LINKS: &str = "uses";

/// Whether `directory` is the folder whose entries are the repository's links between source folders.
fn is_uses_folder(directory: &Path) -> bool {
    return directory
        .file_name()
        .is_some_and(|name| return name == FOLDER_USES_LINKS);
}

/// What:
///  `pub(crate) fn discover(root: &Path) -> Result<Vec<String>, String>` lists every `.rs` file under `root`,
///  sorted,
/// as `/`-separated names relative to `root`.
/// `pub(crate)` means other modules of this crate may call it,
///  outside code may not (TS:
/// an export that is not in the package's public entry).
/// Why:
///  Every scan needs the same complete listing.
/// It fails,
/// instead of returning less,
/// when a directory cannot be
///  listed,
/// an entry cannot be inspected,
/// any entry is a symbolic link (its target may be outside the tree,
/// or
///  absent),
/// or no source file is found at all.
/// The one link it reads nothing through is the one a `uses` folder holds
/// when that link names a folder below `root`:
/// that folder is the repository's own record of which source folders a folder's code uses
/// (`AGENTS.md`,
/// rule `LYD`),
/// the lint that checks those links owns them,
/// and such a link is a folder of sources this walk already reaches under its own name.
/// Any other link in a `uses` folder is refused like every other link,
/// because a link to a file,
/// to a folder outside the tree or to nothing is not that record and could hide a source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function discover(root: string): string[]  // throws Error
/// ```
pub(crate) fn discover(root: &Path) -> Result<Vec<String>, String> {
    let mut found: Vec<String> = Vec::new();
    // The walk's own root, resolved once, so a `uses` link is judged against the tree this listing covers.
    let canonical_root: PathBuf = std::fs::canonicalize(root)
        .map_err(|error| return format!("cannot resolve {}: {error}", root.display()))?;
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
            let entry: std::fs::DirEntry = item.map_err(|error| {
                return format!("cannot read an entry of {}: {error}", directory.display());
            })?;
            let path: PathBuf = entry.path();
            // `file_type()` does not follow links, so a link is seen as a link.
            let kind: std::fs::FileType = entry
                .file_type()
                .map_err(|error| return format!("cannot inspect {}: {error}", path.display()))?;
            if kind.is_symlink() {
                // A `uses` folder is the repository's own record of which source folders a folder's code uses, written
                // as one link per folder (`AGENTS.md`, rule LYD). Only a link to a folder below this walk's root is that
                // record: reading it reaches sources the walk already lists under their own names. A link to a file, to
                // a folder outside the tree or to nothing is refused, because it is not the layout's record and it could
                // hide a source. The lint that compares those links with the code owns them.
                if is_uses_folder(&directory) {
                    let target: PathBuf = std::fs::canonicalize(&path).map_err(|error| {
                        return format!("cannot resolve the link {}: {error}", path.display());
                    })?;
                    if target.starts_with(canonical_root.as_path()) && target.is_dir() {
                        continue;
                    }
                    return Err(format!(
                        "{} is a symbolic link in a `uses` folder whose target is not a folder below {}; a `uses` link names one source folder of this tree, and the scan cannot prove what this one reads",
                        path.display(),
                        root.display()
                    ));
                }
                return Err(format!(
                    "{} is a symbolic link; the scan cannot prove what it reads",
                    path.display()
                ));
            }
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if path
                .extension()
                .is_some_and(|extension| return extension == "rs")
            {
                found.push(relative_name(root, &path)?);
            }
        }
    }
    if found.is_empty() {
        return Err(format!(
            "no .rs file under {}; an empty listing proves nothing",
            root.display()
        ));
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

/// What:
///  The one line the fixture's source file holds,
///  so a control counts how many times a walk reaches that file.
/// Why:
///  The subject of this file's controls is how many times a walk reaches a file,
/// and the file's own text is the evidence, never its name.
const THE_LINE_ONE_FILE_HOLDS: &str = "// the needle: a walk reaches this file once\n";

/// What:
///  `fn a_tree_a_link_points_back_into(name: &str) -> PathBuf` makes a fixture tree of one source file in `folder`
///  and one link in `folder/uses`,
///  naming the tree's own root,
///  which is the shape this repository's source folders have
///  (`AGENTS.md`,
///  rules `LYD` and `LYL`).
/// `&str` is borrowed text (sibling: `String`,
/// the owned form) and `PathBuf` is an owned file-system path (sibling: `&Path`,
/// the borrowed view).
/// Why:
///  A read that descends a link reaches that source file once per chain of links above it,
///  and this tree shows it once and through the link,
///  which is the read the shared walk replaces (issue 6598).
///  The owned path comes back because the caller removes the folder at its end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aTreeALinkPointsBackInto(name: string): string  // throws Error
/// ```
#[cfg(unix)]
fn a_tree_a_link_points_back_into(name: &str) -> PathBuf {
    // What:
    //  `fixture(name)` makes a folder of this call's own under the fixture root and hands back its path.
    // Why:
    //  The tree this control walks is written by the control alone,
    //  so no other test thread writes or removes it.
    let tree: PathBuf = crate::test_support::fixture(name);
    write_tree(
        tree.as_path(),
        &[("folder/mod.rs", THE_LINE_ONE_FILE_HOLDS)],
    );
    // What:
    //  `create_dir_all` makes the `uses` folder, and `join` builds a path from its parts.
    // Why:
    //  The link below needs the folder the layout rules put it in;
    //  `write_tree` wrote the file's folder only.
    std::fs::create_dir_all(tree.join("folder").join("uses")).expect("the uses folder");
    // What:
    //  `std::os::unix::fs::symlink(target, link)` makes `link` a symbolic link naming `target`.
    //  `target` here is `../..`, the tree's own root, written relative as the repository's links are.
    // Why:
    //  A read that descends it walks the tree again from that root,
    //  so the one source file is reachable twice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // symlinkSync('../..', join(tree, 'folder', 'uses', 'again')); // throws Error
    // ```
    std::os::unix::fs::symlink("../..", tree.join("folder").join("uses").join("again"))
        .expect("a link back to the tree's own root");
    return tree;
}

/// What:
///  `fn every_name_the_read_this_walk_replaces_finds(root: &Path) -> Vec<String>` lists every `.rs` file under `root`
///  as `/`-separated names relative to it,
///  the way this crate's guards listed their sources before the shared walk:
///  `read_dir` at every folder,
///  and `path.is_dir()` to decide whether an entry is a folder,
///  which follows a symbolic link.
/// Why:
///  The control beside this function has to show that its tree is one the earlier read read twice,
///  or it would also pass for a walk that had not changed.
///  Nothing else calls this:
///  the read it stands for is the defect the shared walk replaced (issue 6598).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function everyNameTheReadThisWalkReplacesFinds(root: string): string[]  // throws Error
/// ```
fn every_name_the_read_this_walk_replaces_finds(root: &Path) -> Vec<String> {
    // What:
    //  `Vec<String>` is a growable list of owned text (siblings: `&[String]`, a borrowed view, and
    //  `[String; N]`, a fixed length) and `Vec::new()` an empty one.
    // Why:
    //  The list is the evidence the control asserts on, so it keeps every name the read reaches.
    let mut found: Vec<String> = Vec::new();
    // What:
    //  `vec![root.to_path_buf()]` is a work list holding one owned copy of the root (`to_path_buf()` copies a borrowed
    //  `&Path` into the owned `PathBuf`), and `while let Some(x) = ...pop()` repeats while the list holds one.
    // Why:
    //  A list of folders still to read replaces recursion here, so a deep tree cannot exhaust the stack.
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)
            .expect("a readable folder")
            .flatten()
        {
            let path: PathBuf = entry.path();
            // What: `path.is_dir()` asks the operating system whether the path names a folder, and it follows a link.
            // Why:  This is exactly the decision the shared walk makes on the entry's own type, so the two reads differ
            //       only in whether a link is descended.
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path
                .extension()
                .is_some_and(|extension| return extension == "rs")
            {
                let relative: &Path = path.strip_prefix(root).expect("a path below the root");
                found.push(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    return found;
}

/// What:
///  A control of the shared walk on a tree whose `uses` link names the tree's own root:
///  the walk lists the one source file once,
///  reading every listed file reads it once,
///  and the recursive read the walk replaces reaches it again for each link above it.
/// Why:
///  A walk that descends a link returns the same file once per chain of links (issue 6598),
///  so a guard that counts a needle or a file reads a tree this size as one many times larger,
///  and this control fails the moment the shared walk starts reading through a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('the walk reaches a linked-back source once', () => {
///   const tree = aTreeALinkPointsBackInto('scan-walk-linked-back')
///   expect(discover(tree)).toEqual(['folder/mod.rs'])
///   expect(needleReadings(discover(tree))).toBe(1)
///   expect(everyNameTheReadThisWalkReplacesFinds(tree).length).toBeGreaterThan(1)
///   remove(tree)
/// })
/// ```
#[cfg(unix)]
#[test]
fn the_walk_reaches_a_linked_back_source_once_where_a_recursive_read_reaches_it_again() {
    let tree: PathBuf = a_tree_a_link_points_back_into("scan-walk-linked-back");
    let listed: Vec<String> = discover(&tree).expect("a listing of the fixture tree");
    assert_eq!(
        listed,
        vec![String::from("folder/mod.rs")],
        "the walk lists the one source file once, and the link names no second file"
    );
    // What:
    //  The number of times the one line the file holds is read: `read_to_string` gives one file's text as an owned
    //  `String`, and `matches(needle).count()` counts non-overlapping occurrences of the needle in it.
    // Why:
    //  Reading every listed file once is what a guard of the crate's sources does,
    //  and the count is 1 only while the listing holds the file once.
    let mut readings: usize = 0;
    for name in &listed {
        let text: String =
            std::fs::read_to_string(tree.join(name.as_str())).expect("a listed source file");
        readings += text.matches(THE_LINE_ONE_FILE_HOLDS).count();
    }
    assert_eq!(
        readings, 1,
        "the walk reaches the file once, so its text is read once"
    );
    let recursive: Vec<String> = every_name_the_read_this_walk_replaces_finds(&tree);
    assert!(
        recursive.len() > 1,
        "the tree is one the read this walk replaces reads again, and it read: {recursive:?}"
    );
    crate::test_support::remove(tree.as_path());
}
