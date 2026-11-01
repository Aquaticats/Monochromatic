//! What:
//!  Pure classification of image destinations for `markdown/lfs-image-url`.
//! Why:
//!  The rule (which decides fixes) and the context builder (which resolves every candidate path
//! before the rule runs) must agree on which repository path a destination names.
//! Nothing here reads the filesystem;
//!  paths are resolved lexically.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // relativeTargetPath({ url, filePath, repoRoot }); objectUrlParts({ url, objectBase });
//! ```

/// Import the object-id spelling check.
use crate::markdown_lfs_oid::is_lfs_oid;
/// Import native path components;
///  `PathBuf` owns a path,
///  `Path` borrows one.
use std::path::{Component, Path, PathBuf};

/// What:
///  Whether a destination carries a URL scheme such as `https:`,
///  `mailto:` or `data:`.
/// Why:
///  A scheme-qualified destination never names a working-tree file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasScheme(url: string): boolean;
/// ```
fn has_scheme(url: &str) -> bool {
    let Some(colon): Option<usize> = url.find(':') else {
        return false;
    };
    if colon == 0 {
        return false;
    }
    for byte in url[..colon].bytes() {
        let allowed: bool =
            byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'-' || byte == b'.';
        if !allowed {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether a destination is a path inside the working tree.
/// Why:
///  Fragments,
///  site-absolute paths and scheme-qualified URLs are left to other rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isRelativePath(url: string): boolean;
/// ```
pub(crate) fn is_relative_path(url: &str) -> bool {
    return !url.is_empty() && !url.starts_with('#') && !url.starts_with('/') && !has_scheme(url);
}

/// What:
///  A destination without its query and fragment.
/// Why:
///  Neither part names a file;
///  the cut is at whichever delimiter comes first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathPart(url: string): string;
/// ```
pub(crate) fn path_part(url: &str) -> &str {
    let mut cut: usize = url.len();
    if let Some(query) = url.find('?') {
        cut = cut.min(query);
    }
    if let Some(fragment) = url.find('#') {
        cut = cut.min(fragment);
    }
    return &url[..cut];
}

/// What:
///  Remove `.` components and resolve `..` components without touching the filesystem.
/// Why:
///  A path such as `cwd/../doc/a.md` must name the same directory as its resolved form before
/// a relative destination is applied to it.
///  `..` at the root stays at the root,
///  as `path.resolve` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lexicalNormal(path: Path): Path { return resolve(path); }
/// ```
pub(crate) fn lexical_normal(path: &Path) -> PathBuf {
    let mut normal: PathBuf = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                // pop() removes the last ordinary component and refuses to remove a root.
                normal.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normal.push(component.as_os_str());
            }
        }
    }
    return normal;
}

/// What:
///  Apply forward-slash destination segments to a directory,
///  lexically.
/// Why:
///  Markdown destinations use `/` on every platform;
///  empty and `.` segments name nothing new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applySegments(directory: Path, segments: string): Path { return resolve(directory, ...segments.split('/')); }
/// ```
pub(crate) fn apply_segments(directory: &Path, segments: &str) -> PathBuf {
    let mut resolved: PathBuf = directory.to_path_buf();
    for segment in segments.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            resolved.pop();
            continue;
        }
        resolved.push(segment);
    }
    return resolved;
}

/// What:
///  The forward-slash path of `path` below `root`,
///  or `None` when it is the root itself,
///  lies
/// outside it,
///  or has a component that is not valid UTF-8.
/// Why:
///  Object URLs and gitignore patterns both address files by this repository-relative spelling,
/// and a destination that escapes the repository names no repository file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repoRelative({ repoRoot, path }): string | undefined;
/// ```
pub(crate) fn repo_relative(root: &Path, path: &Path) -> Option<String> {
    let Ok(below): Result<&Path, std::path::StripPrefixError> = path.strip_prefix(root) else {
        return None;
    };
    let mut parts: Vec<&str> = Vec::<&str>::new();
    for component in below.components() {
        // to_str() is None for bytes that cannot be written in a URL or matched as text.
        let text: &str = component.as_os_str().to_str()?;
        parts.push(text);
    }
    if parts.is_empty() {
        return None;
    }
    return Some(parts.join("/"));
}

/// What:
///  The repository-relative path a relative destination names,
///  or `None` when the destination
/// is not a relative path or leaves the repository.
/// Why:
///  This is the key the context builder resolves and the path an object URL embeds.
/// Both `file_path` and `repo_root` must already be absolute and lexically normal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativeTargetPath({ url, filePath, repoRoot }): string | undefined;
/// ```
pub(crate) fn relative_target_path(
    url: &str,
    file_path: &Path,
    repo_root: &Path,
) -> Option<String> {
    if !is_relative_path(url) {
        return None;
    }
    let directory: &Path = file_path.parent()?;
    let absolute: PathBuf = apply_segments(directory, path_part(url));
    return repo_relative(repo_root, &absolute);
}

/// What:
///  The object id and repository path embedded in an object URL under the repository's base.
/// Why:
///  Only a complete `<base>/<oid>/<path>` is kept in step with the file it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ObjectUrlParts = { oid: string; repoRelativePath: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObjectUrlParts {
    /// Embedded object id,
    ///  64 lowercase hexadecimal characters.
    pub(crate) oid: String,
    /// Embedded forward-slash path exactly as written,
    ///  without query or fragment.
    pub(crate) repo_relative_path: String,
}

/// What:
///  Parse `<objectBase>/<oid>/<path>`,
///  or `None` when the destination is not under the base or
/// lacks a well-formed id or a path.
/// Why:
///  The base match is an exact string prefix,
///  as in the incumbent;
///  no URL is re-parsed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function objectUrlParts({ url, objectBase }): ObjectUrlParts | undefined;
/// ```
pub(crate) fn object_url_parts(url: &str, object_base: &str) -> Option<ObjectUrlParts> {
    let after_base: &str = url.strip_prefix(object_base)?;
    if !after_base.starts_with('/') {
        return None;
    }
    // The base holds no query or fragment, so the cut can only shorten the part after it.
    let rest: &str = path_part(&after_base[1..]);
    let slash: usize = rest.find('/')?;
    let oid: &str = &rest[..slash];
    let repo_relative_path: &str = &rest[slash + 1..];
    if !is_lfs_oid(oid) || repo_relative_path.is_empty() {
        return None;
    }
    return Some(ObjectUrlParts {
        oid: String::from(oid),
        repo_relative_path: String::from(repo_relative_path),
    });
}

/// What:
///  The forward-slash link from one directory to a file,
///  both absolute and lexically normal.
/// Why:
///  A target that left LFS returns to a relative link from the Markdown file's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativeLink(fromDirectory: Path, to: Path): string { return relative(fromDirectory, to).split(sep).join('/'); }
/// ```
pub(crate) fn relative_link(from_directory: &Path, to: &Path) -> String {
    let from: Vec<Component<'_>> = from_directory.components().collect::<Vec<Component<'_>>>();
    let target: Vec<Component<'_>> = to.components().collect::<Vec<Component<'_>>>();
    // Count the leading components both paths share.
    let mut shared: usize = 0;
    // What: `zip` walks both component lists in step, as pairs, and stops at the shorter list by itself.
    // Why: The pair walk needs no bounds checks or hand-stepped index, so no mutation of the count can make it spin.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let i = 0; i < Math.min(from.length, target.length) && from[i] === target[i]; i++) shared++;
    // ```
    for (left, right) in from.iter().zip(target.iter()) {
        if left != right {
            break;
        }
        shared += 1;
    }
    let mut parts: Vec<String> = Vec::<String>::new();
    for _ in shared..from.len() {
        parts.push(String::from(".."));
    }
    for component in &target[shared..] {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }
    return parts.join("/");
}

/// Destination grammar and traversal controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_lfs_target_tests.rs"]
mod tests;
