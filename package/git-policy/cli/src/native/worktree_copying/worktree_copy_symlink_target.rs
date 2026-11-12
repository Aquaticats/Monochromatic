//! What:
//!  The target a copied symbolic link must hold so that it names,
//!  at the new place the copy is installed at, what it named where it was read,
//!  and the rule that tells two links naming one place from two that do not.
//! Why:
//!  A link's text is a path read from the directory the link sits in,
//!  and a relative text counts directory levels from there.
//!  A link copied into a worktree at another depth therefore climbs to a directory the new worktree does not have at that
//!  distance;
//!  pnpm's `node_modules` links reach its store by exactly such a count,
//!  and a fresh worktree's package state named a directory that does not exist
//!  (`doc/troubleshooting/pnpm-symlinked-store-path.md`).
//!  The copied tree keeps the shape of the tree it was read from,
//!  so a text that stays inside that tree names the same place at any depth and needs no change;
//!  a text that leaves it is written as the absolute path it resolved to,
//!  which names that place from anywhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = targetForTheNewPlace(readLink(sourceLink), sourceLink, sourceRoot); symlink(text, linkInTheCopy);
//! ```

use crate::lexical_path::normalized_path;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

/// What:
///  The place one link's text names,
///  seen from the tree the link sits in.
/// Why:
///   Two links name one place when both stay inside their own tree at the same path relative to its root,
///   or when both leave it and name the same absolute path;
///   a copy is faithful when every link of it names the place the source's link named,
///   which is not the same as holding the same text once the text counts levels.
///  The two answers are kept apart because one of them is relative to a root and the other is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Place = { kind: 'inside'; path: string } | { kind: 'outside'; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Place {
    /// A path inside the tree, relative to its root.
    Inside(PathBuf),
    /// The absolute path a text that leaves the tree resolved to.
    Outside(PathBuf),
}

/// What:
///  The place the text `target` names when it is read from the link `link` of the tree rooted at `root`.
/// Why:
///   A relative text is climbed from the link's own directory,
///   one component at a time,
///   so the climb is followed here rather than resolved by the kernel:
///   the destination of a link may not exist yet,
///   and a copy must be judged before anything runs in it.
///  A climb that leaves the root is resolved against the directory the link sits in,
///  which is where the kernel would look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function placeOf(target: string, link: string, root: string): Place;
/// ```
pub fn place_of(target: &Path, link: &Path, root: &Path) -> Place {
    let parent: &Path = link.parent().unwrap_or_else(|| Path::new(""));
    if target.is_absolute() {
        return match target.strip_prefix(root) {
            Ok(inside) => Place::Inside(normalized_path(inside)),
            Err(_outside) => Place::Outside(normalized_path(target)),
        };
    }
    let inside: Option<Vec<OsString>> = match parent.strip_prefix(root) {
        Ok(from_root) => walked_from_the_root(from_root, target),
        // A link outside the tree it is copied with is not a shape any selection makes; the text is taken as it stands.
        Err(_elsewhere) => None,
    };
    return match inside {
        Some(parts) => Place::Inside(parts.iter().collect()),
        None => Place::Outside(normalized_path(&parent.join(target))),
    };
}

/// What:
///  The components of the path `target` reaches when it is climbed from `from_root`,
///  or nothing when the climb leaves the root.
/// Why:
///   A `..` that would leave the root leaves the tree,
///   and one that is answered by a directory the root itself holds stays inside,
///   which is the difference between a link into the checkout and a link into pnpm's store.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const walkedFromTheRoot = (fromRoot, target) => { const parts = [...]; for (const part of segments(target)) { if (part === '..') { if (!parts.pop()) return undefined } else parts.push(part) } return parts };
/// ```
fn walked_from_the_root(from_root: &Path, target: &Path) -> Option<Vec<OsString>> {
    let mut parts: Vec<OsString> = Vec::new();
    for component in from_root.components() {
        match component {
            Component::Normal(name) => parts.push(name.to_os_string()),
            // The link's own directory is inside the root, so no other component can appear here.
            Component::CurDir | Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    for component in target.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let climbed: Option<OsString> = parts.pop();
                if climbed.is_none() {
                    return None;
                }
            }
            Component::Normal(name) => parts.push(name.to_os_string()),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    return Some(parts);
}

/// What:
///  The text a link at `link` of the tree rooted at `root` needs so that it names `inside`,
///  a path relative to that same root.
/// Why:
///   A text that is already relative and stays inside the tree names the same place in the copy,
///   so it is kept as the source wrote it;
///   an absolute text into the tree would name the tree it was read from,
///   so it is turned into the climb that reaches the same path of whichever tree holds the link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function textFromTheLinkTo(inside: string, link: string, root: string): string;
/// ```
fn text_from_the_link_to(inside: &Path, link: &Path, root: &Path) -> PathBuf {
    let from_root: &Path = link.parent().unwrap_or_else(|| Path::new("")).strip_prefix(root).unwrap_or_else(|_elsewhere| Path::new(""));
    let mut from: Vec<Component<'_>> = from_root.components().collect();
    let mut to: Vec<Component<'_>> = inside.components().collect();
    let mut shared: usize = 0;
    while shared < from.len() && shared < to.len() && from[shared] == to[shared] {
        shared += 1;
    }
    from.drain(..shared);
    to.drain(..shared);
    let mut text: PathBuf = PathBuf::new();
    for _step in &from {
        text.push("..");
    }
    for component in &to {
        text.push(component.as_os_str());
    }
    if text.as_os_str().is_empty() {
        return PathBuf::from(".");
    }
    return text;
}

/// What:
///  The text the copy's link must hold,
///  read from the source's text and the place the source's link sits in.
/// Why:
///   The copy keeps the source tree's shape below its root,
///   so the only texts that need a new spelling are the ones that leave the tree,
///   and those need an absolute path because their climb was counted from another depth.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function targetForTheNewPlace(target: string, sourceLink: string, sourceRoot: string): string;
/// ```
pub fn target_for_the_new_place(target: &Path, source_link: &Path, source_root: &Path) -> PathBuf {
    return match place_of(target, source_link, source_root) {
        Place::Inside(_inside) if target.is_relative() => target.to_path_buf(),
        Place::Inside(inside) => text_from_the_link_to(inside.as_path(), source_link, source_root),
        Place::Outside(absolute) => absolute,
    };
}

/// What:
///  Whether the text `expected` of the link `expected_link` in the tree rooted at `expected_root`,
///  and the text `actual` of the link `actual_link` in the tree rooted at `actual_root`,
///  name one place.
/// Why:
///   A stage's link and the source's link hold the same text only while the copy keeps the source's depth;
///   a faithful copy is one whose links name the same places,
///   and that is what the check that the source still matches the stage has to ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const samePlace = (expected, expectedLink, expectedRoot, actual, actualLink, actualRoot) => placeOf(...) === placeOf(...);
/// ```
pub fn names_the_same_place(expected: &Path, expected_link: &Path, expected_root: &Path, actual: &Path, actual_link: &Path, actual_root: &Path) -> bool {
    return place_of(expected, expected_link, expected_root) == place_of(actual, actual_link, actual_root);
}

/// Target controls stay out of the release executable;
/// they build links, which only a unix filesystem has here.
#[cfg(all(test, unix))]
#[path = "worktree_copy_symlink_target_tests.rs"]
mod tests;
