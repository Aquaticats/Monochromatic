//! Pure relative-path display:
//!  strip the longest common directory prefix from
//! the queue's track paths so the UI can show each track's path relative to the
//! loaded root (e.g. `Artist/Album/01.flac` rather than the full absolute path,
//! or just `01.flac` when the whole queue lives in one folder).
//!  No Slint,
//!  audio,
//! or I/O,
//!  so it is fully unit-tested;
//!  `queue::Queue::display_paths` calls this
//! to build the list handed to the UI,
//!  and `pagination` then groups the result
//! (by folder for subfolder tracks,
//!  by first letter for root-level ones).

/// What:
///      `use std::path::{Component, Path, PathBuf};`.
///  `PathBuf` is the OWNED,
///           growable path (sibling:
///  the borrowed `&Path`);
///  `Path` is the borrowed
///           view;
///  `Component` is one piece of a split path (a root `/`,
///  a `..`,
///  a
///           normal name segment,
///  etc.).
/// Why:
///       We borrow each track path (`&Path`),
///  split it into `Component`s,
///  and
///           keep only the named segments;
///  the input list is owned `PathBuf`s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // all three are just strings in TS; Component ~ one element of path.split("/")
/// ```
use std::path::{Component, Path, PathBuf};

/// What:
///      `const SEPARATOR: &str = "/";`.
///  `&str` is a BORROWED string slice
///           (sibling:
///  the owned `String`);
///  the separator we re-join segments with.
/// Why:
///       The queue is Linux-only and the UI/pagination expect a single `/`
///           separator;
///  naming it avoids a bare `"/"` literal scattered around.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SEPARATOR = "/";
/// ```
const SEPARATOR: &str = "/";

/// What:
///      `pub fn relative_display_paths(tracks: &[PathBuf]) -> Vec<String>`.
///  Turn
///           each track's full path into its path relative to the queue's common
///           root.
///  `&[PathBuf]` is a borrowed slice of owned paths (read-only);
///  the
///           result is one owned relative string per track,
///  in order.
/// Why:
///       The UI shows folders,
///  not just filenames,
///  but the absolute prefix (e.g.
///           `/home/user/Music`) is noise;
///  this strips it once per queue.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativeDisplayPaths(tracks: string[]): string[] {
///   if (tracks.length === 0) return [];
///   const lists = tracks.map(normalComponents);
///   const prefix = commonPrefixLen(lists);
///   return lists.map((list, i) => {
///     const rel = list.slice(prefix).join("/");
///     return rel === "" ? tracks[i] : rel;
///   });
/// }
/// ```
pub fn relative_display_paths(tracks: &[PathBuf]) -> Vec<String> {
    return relative_display_paths_in(tracks, None);
}

/// What:
///      `pub fn relative_display_paths_in(tracks: &[PathBuf], root: Option<&Path>) -> Vec<String>`.
///  Like `relative_display_paths`, but relative to the opened folder `root` when every track lies under it, so a
///  library with one top-level folder keeps that folder in its names.
/// Why:
///       Stripping the longest shared directory made `Music/Artist/*.flac` look like loose files and hid the folder
///  (the polish review finding LS1). The folder walk's own batch names are already relative to the root, so Search and
///  the queue now agree for every library, not only for those with several top-level entries. Without a root, or when
///  a track lies outside it, the shared directory is stripped as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const relativeDisplayPathsIn = (tracks: string[], root?: string) => tracks.map((t) => relativeTo(root ?? commonDir(tracks), t));
/// ```
pub fn relative_display_paths_in(tracks: &[PathBuf], root: Option<&Path>) -> Vec<String> {
    if tracks.is_empty() {
        return Vec::new();
    }
    let prefix_len: usize = display_prefix_len(tracks, root);
    // Join the components after the stripped prefix.
    return tracks
        .iter()
        .map(|path| {
            let mut relative: String = String::new();
            for (position, part) in normal_parts(path).skip(prefix_len).enumerate() {
                if position > 0 {
                    relative.push_str(SEPARATOR);
                }
                relative.push_str(&part);
            }
            if relative.is_empty() {
                return path.to_string_lossy().into_owned();
            }
            return relative;
        })
        .collect();
}

/// What:
///      `fn display_prefix_len(tracks: &[PathBuf], root: Option<&Path>) -> usize`.
///  How many leading components every display name drops: the components of `root` when every track lies strictly
///  under it, otherwise the shared directory of the tracks.
/// Why:
///       One rule picks the prefix for the names and for the page keys, so a track's page never disagrees with its name.
fn display_prefix_len(tracks: &[PathBuf], root: Option<&Path>) -> usize {
    let Some(root_path) = root else {
        return shared_prefix_len(tracks);
    };
    let root_parts: Vec<std::borrow::Cow<'_, str>> = normal_parts(root_path).collect();
    if root_parts.is_empty() {
        tracing::debug!(root = %root_path.display(), "the opened folder has no named component; the shared directory is stripped");
        return shared_prefix_len(tracks);
    }
    let all_under_root: bool = tracks.iter().all(|path| {
        let mut parts = normal_parts(path);
        for root_part in &root_parts {
            if parts.next().is_none_or(|part| return part != *root_part) {
                return false;
            }
        }
        return parts.next().is_some();
    });
    if all_under_root {
        return root_parts.len();
    }
    tracing::debug!(root = %root_path.display(), "a track lies outside the opened folder; the shared directory is stripped");
    return shared_prefix_len(tracks);
}

/// What:
///      `pub fn shared_prefix_len(tracks: &[PathBuf]) -> usize`.
///  How many leading components every path shares,
///  never counting a path's last component.
/// Why:
///       The display name of a track is its path without this prefix;
///  counting it takes one pass that allocates nothing,
///  where building a component list per path took hundreds of megabytes at one million tracks.
fn shared_prefix_len(tracks: &[PathBuf]) -> usize {
    let Some(first_path) = tracks.first() else {
        return 0;
    };
    let first_parts: Vec<std::borrow::Cow<'_, str>> = normal_parts(first_path).collect();
    let mut shortest: usize = first_parts.len();
    let mut shared: usize = first_parts.len();
    for path in tracks {
        let mut length: usize = 0;
        let mut matching: usize = 0;
        let mut still_matching: bool = true;
        for part in normal_parts(path) {
            if still_matching && matching < shared && first_parts.get(length).is_some_and(|first| return *first == part) {
                matching += 1;
            } else {
                still_matching = false;
            }
            length += 1;
        }
        shortest = shortest.min(length);
        shared = shared.min(matching);
    }
    return if shortest == 0 { 0 } else { shared.min(shortest - 1) };
}

/// What:
///      `enum PageKey<'a> { Folder(Cow<'a, str>), Letter(char), CatchAll }`.
///  Which page of the queue a display name belongs to.
/// Why:
///       `pagination` puts a name with a folder in that folder's page,
///  a root-level name starting with an ASCII letter in that letter's page (a virtual folder named by the letter),
///  and any other root-level name in the `#` page.
#[derive(PartialEq, Eq)]
enum PageKey<'a> {
    /// The page of the folder with this name.
    Folder(std::borrow::Cow<'a, str>),
    /// The page of this upper-case ASCII letter.
    Letter(char),
    /// The `#` page.
    CatchAll,
}

/// What:
///      `fn page_key_of(path: &Path, prefix_len: usize) -> PageKey<'_>`.
///  The page of one track,
///  from its first two components after the dropped prefix.
/// Why:
///       It gives the same page as `paginate` gives the display name,
///  without building the name.
fn page_key_of(path: &Path, prefix_len: usize) -> PageKey<'_> {
    let mut parts = normal_parts(path).skip(prefix_len);
    let Some(first) = parts.next() else {
        return PageKey::CatchAll;
    };
    if parts.next().is_some() {
        return PageKey::Folder(first);
    }
    return match first.chars().next() {
        Some(letter) if letter.is_ascii_alphabetic() => PageKey::Letter(letter.to_ascii_uppercase()),
        _ => PageKey::CatchAll,
    };
}

/// What:
///      `pub fn page_scope_for_anchor(tracks: &[PathBuf], anchor: usize) -> Option<Vec<usize>>`.
///  The positions of every track on the same page as `anchor`,
///  or `None` when `anchor` is past the end.
/// Why:
///       Playback scope follows the page of the playing track.
///  Grouping every name into pages to find one page cost over a second at one million tracks;
///  comparing page keys does not allocate per track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pageScopeForAnchor = (tracks: string[], anchor: number) => tracks.flatMap((_, i) => pageKey(i) === pageKey(anchor) ? [i] : []);
/// ```
pub fn page_scope_for_anchor(tracks: &[PathBuf], anchor: usize) -> Option<Vec<usize>> {
    return page_scope_for_anchor_in(tracks, anchor, None);
}

/// What:
///      `pub fn page_scope_for_anchor_in(tracks, anchor, root) -> Option<Vec<usize>>`.
///  `page_scope_for_anchor` for names that are relative to the opened folder `root` (see `relative_display_paths_in`).
/// Why:
///       The engine's playback scope must be the page the window shows, and the window groups the names
///  `relative_display_paths_in` makes, so both must drop the same prefix.
pub fn page_scope_for_anchor_in(tracks: &[PathBuf], anchor: usize, root: Option<&Path>) -> Option<Vec<usize>> {
    let anchor_path: &PathBuf = tracks.get(anchor)?;
    let prefix_len: usize = display_prefix_len(tracks, root);
    let wanted: PageKey<'_> = page_key_of(anchor_path, prefix_len);
    return Some(
        tracks
            .iter()
            .enumerate()
            .filter(|(_, path)| return page_key_of(path, prefix_len) == wanted)
            .map(|(position, _)| return position)
            .collect(),
    );
}

/// What:
///      `fn normal_parts(path: &Path) -> impl Iterator<Item = Cow<'_, str>>`.
///  The normal components of a path as text,
///  borrowed when the name is valid UTF-8.
/// Why:
///       `relative_display_paths` walks every track twice;
///  borrowing keeps that to the cost of reading the components,
///  with no allocation per component.
fn normal_parts(path: &Path) -> impl Iterator<Item = std::borrow::Cow<'_, str>> {
    return path.components().filter_map(|component| {
        return match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        };
    });
}

#[cfg(test)]
#[path = "relpath_tests.rs"]
mod tests;
