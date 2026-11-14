//! What:
//! The record of the postinstall links `install` made,
//! and the retirement of a recorded link once the lock no longer declares it.
//! Why:
//! A link dropped from the plan (or renamed,
//! or moved to another directory) used to stay where it was,
//! and a stale `cargo` link in a directory early on `PATH` kept running a tool the plan had stopped linking.
//! The lock only says what is declared now,
//! so the record is the only memory of what an earlier run made.
//! A link is removed only when it is still the symlink the record names;
//! anything else at that path belongs to a person and stays.
//! A new worktree starts as a snapshot of the checkout it was made from,
//! `.cache` included,
//! so its record can name the links of that other checkout;
//! a repository scope retires only links inside its own checkout,
//! and drops every other recorded path from its record without touching what is there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // postinstall-links.ts
//! export function syncPostinstallLinks(ctx: Context, tools: LockedTool[]): LinkNotices  // throws
//! ```

/// Imports `BTreeMap` from `std::collections`.
use std::collections::{BTreeMap, BTreeSet};
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `Map` and `Value` from `serde_json`.
use serde_json::{Map, Value};

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::exposure`.
use super::exposure::{MadeLink, link_directory, make_postinstall_links};
/// Imports `LockedTool` and `Postinstall` from `super::model`.
use super::model::{LockedTool, Postinstall};
/// Imports `ScopeKind` from `super::scope`.
use super::scope::{Scope, ScopeKind};
/// Imports `write_atomically` from `super::publish`.
use super::publish::write_atomically;
/// Imports what this module uses from `super::removal`.
use super::removal::{Judged, Removal, judged_of_metadata, remove_if_still};

/// The file name of the record,
/// inside the scope's launcher directory.
const RECORD_FILE_NAME: &str = ".postinstall-links.json";

/// How many times a write of the record merges and checks before it gives up and says the next install records them.
/// Delegated pick under the owner's rule that nothing waits without end;
/// a lost write costs one pass's bookkeeping, never a link.
const RECORD_WRITE_ATTEMPTS: usize = 8;

/// The JSON key holding the links,
/// as an object of link path to link target.
const LINKS_KEY: &str = "links";

/// What a pass of the cleanup did:
/// one line per link retired,
/// and one warning per recorded path left alone.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LinkNotices {
    /// One sentence per link removed.
    pub retired: Vec<String>,
    /// One sentence per recorded path that holds something this tool did not make.
    pub warnings: Vec<String>,
}

/// The record file of a scope.
fn record_file(ctx: &Context) -> PathBuf {
    return ctx.scope.launcher_dir.join(RECORD_FILE_NAME);
}

/// The recorded links (path to target),
/// and a warning when the record exists but cannot be read,
/// in which case nothing is retired this run.
fn read_record(file: &Path) -> (BTreeMap<PathBuf, PathBuf>, Option<String>) {
    let text: String = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (BTreeMap::new(), None),
        Err(error) => return (BTreeMap::new(), Some(format!("cannot read {}, so no stale postinstall link is retired: {error}", file.display()))),
    };
    let mut links: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
    let parsed: Option<Value> = serde_json::from_str(&text).ok();
    let links_value: Option<&Value> = match &parsed {
        Some(value) => value.get(LINKS_KEY),
        None => None,
    };
    let Some(entries) = links_value.and_then(Value::as_object) else {
        return (links, Some(format!("{} is not a record of postinstall links, so no stale postinstall link is retired", file.display())));
    };
    for (path, target) in entries {
        if let Some(target_text) = target.as_str() {
            links.insert(PathBuf::from(path), PathBuf::from(target_text));
        }
    }
    return (links, None);
}

/// The record as JSON text.
fn render_record(links: &BTreeMap<PathBuf, PathBuf>) -> Result<String, ToolchainError> {
    let mut entries: Map<String, Value> = Map::new();
    for (path, target) in links {
        entries.insert(path.to_string_lossy().to_string(), Value::String(target.to_string_lossy().to_string()));
    }
    let mut root: Map<String, Value> = Map::new();
    root.insert(LINKS_KEY.to_string(), Value::Object(entries));
    return match serde_json::to_string_pretty(&Value::Object(root)) {
        Ok(text) => Ok(format!("{text}\n")),
        Err(error) => Err(ToolchainError::failure(format!("cannot render the record of postinstall links: {error}"))),
    };
}

/// What:
/// The full path of every postinstall link any locked tool declares,
/// whatever its operating system.
/// Why:
/// The retirement of a launcher asks which links point at it,
/// and a link the lock still declares is not a leftover whatever its target is now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function declaredLinkPaths(ctx: Context): string[]
/// ```
pub fn declared_link_paths(ctx: &Context) -> BTreeSet<PathBuf> {
    return declared_link_paths_of(ctx.scope, ctx.tools);
}

/// What:
/// The full path of every postinstall link `tools` declares,
/// for a caller that has read the lock it wants the answer for.
/// Why:
/// The retirement asks this of the lock on disk at the instant it removes a link,
/// and `reconcile` of the lock this run loaded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function declaredLinkPathsOf(scope: Scope, tools: LockedTool[]): string[]
/// ```
pub fn declared_link_paths_of(scope: &Scope, tools: &[LockedTool]) -> BTreeSet<PathBuf> {
    let mut declared: BTreeSet<PathBuf> = BTreeSet::new();
    for tool in tools {
        if let Some(Postinstall::Link(link)) = &tool.declaration.options.postinstall {
            declared.insert(link_directory(scope, &link.link_dir).join(&link.link_name));
        }
    }
    return declared;
}

/// What:
/// Every path a postinstall link of this scope can stand at:
/// the paths the record names,
/// and the paths the lock's postinstall links declare.
/// Why:
/// The retirement of a launcher asks which links point at it,
/// and a postinstall link of a name the lock dropped can outlive its declaration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recordedLinkPaths(ctx: Context): string[]
/// ```
pub fn recorded_link_paths(ctx: &Context) -> Vec<PathBuf> {
    let (recorded, _unreadable) = read_record(&record_file(ctx));
    let mut paths: Vec<PathBuf> = recorded.into_keys().collect();
    paths.extend(declared_link_paths(ctx));
    return paths;
}

/// The path with symbolic links resolved when it exists,
/// otherwise as given.
fn resolved(path: &Path) -> PathBuf {
    return match fs::canonicalize(path) {
        Ok(found) => found,
        Err(_) => path.to_path_buf(),
    };
}

/// The path of a link with its directory resolved and the link itself left as a link
/// (resolving the link would name its target).
fn resolved_beside(link: &Path) -> PathBuf {
    return match (link.parent(), link.file_name()) {
        (Some(directory), Some(name)) => resolved(directory).join(name),
        _ => link.to_path_buf(),
    };
}

/// What:
/// Whether the scope made the link at `path` and may retire it:
/// a repository scope owns the links inside its own checkout and none inside a checkout nested in it;
/// the global scope owns its recorded paths wherever they are,
/// because its record is never copied from another place.
/// Why:
/// The record stores absolute paths,
/// and a worktree made as a snapshot of another checkout starts with that checkout's record.
/// Such a path was never made by this scope,
/// so its link is not this scope's to remove,
/// whatever it points at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isOwnLink(ctx: Context, path: string): boolean
/// ```
fn is_own_link(ctx: &Context, path: &Path) -> bool {
    if ctx.scope.kind == ScopeKind::Global {
        return true;
    }
    let root: PathBuf = resolved(&ctx.scope.root);
    let place: PathBuf = resolved_beside(path);
    if !place.starts_with(&root) {
        return false;
    }
    for directory in place.ancestors().skip(1) {
        if directory == root {
            return true;
        }
        // A `.git` entry (a directory, or the file a linked worktree has) marks another checkout between the root and the link.
        if fs::symlink_metadata(directory.join(".git")).is_ok() {
            return false;
        }
    }
    return false;
}

/// What a recorded path holds now.
enum Held {
    /// Nothing.
    Nothing,
    /// The symlink the record names,
    /// with the entry that was read,
    /// so its removal can tell it from a file a person put there afterwards.
    OwnLink(Judged),
    /// Something else,
    /// described for the warning.
    Foreign(
        /// What stands at the path in place of the link.
        String,
    ),
}

/// What is at `path`,
/// judged against the target the record names.
fn inspect(path: &Path, recorded_target: &Path) -> Held {
    let metadata: fs::Metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Held::Nothing,
        Err(error) => return Held::Foreign(format!("it cannot be inspected: {error}")),
    };
    if !metadata.file_type().is_symlink() {
        return Held::Foreign("it is not a symlink".to_string());
    }
    let judged: Judged = judged_of_metadata(&metadata);
    return match fs::read_link(path) {
        Ok(target) if target == recorded_target => Held::OwnLink(judged),
        Ok(target) => Held::Foreign(format!("it points at {}", target.display())),
        Err(error) => Held::Foreign(format!("its target cannot be read: {error}")),
    };
}

/// What:
/// Retires every recorded link the lock no longer declares,
/// then writes the record of the links that remain.
/// Why:
/// A recorded link that is still the symlink the record names is removed;
/// a path that now holds a file of a person's,
/// or a link to somewhere else,
/// is left untouched and named in a warning,
/// and leaves the record so the warning appears once.
/// A recorded path outside this scope's own checkout is neither inspected nor removed,
/// and leaves the record:
/// the record was copied from the checkout that made it.
fn reconcile(ctx: &Context, made: &[MadeLink]) -> Result<LinkNotices, ToolchainError> {
    let file: PathBuf = record_file(ctx);
    let (recorded, unreadable) = read_record(&file);
    let mut notices: LinkNotices = LinkNotices::default();
    if let Some(warning) = unreadable {
        notices.warnings.push(warning);
        return Ok(notices);
    }
    let declared: BTreeSet<PathBuf> = declared_link_paths(ctx);
    // The paths that leave the record: retired, gone, replaced by a person's file, or outside this scope's own checkout.
    let mut leaving: BTreeSet<PathBuf> = BTreeSet::new();
    for (path, target) in &recorded {
        if declared.contains(path) {
            continue;
        }
        if !is_own_link(ctx, path) {
            leaving.insert(path.clone());
            continue;
        }
        match inspect(path, target) {
            Held::Nothing => {
                leaving.insert(path.clone());
            }
            Held::OwnLink(judged) => match remove_if_still(path, judged) {
                Ok(Removal::Retired) => {
                    leaving.insert(path.clone());
                    notices.retired.push(format!(
                        "retired the postinstall link {} (it pointed at {}) because the lock no longer declares it",
                        path.display(),
                        target.display()
                    ));
                }
                Ok(Removal::Gone) => {
                    leaving.insert(path.clone());
                }
                Ok(Removal::Replaced) => {
                    leaving.insert(path.clone());
                    notices.warnings.push(format!(
                        "left {} alone: the lock no longer declares a postinstall link there, but something replaced it between reading it \
                         and removing it, so it was put back",
                        path.display()
                    ));
                }
                // A link that could not be removed stays in the record, so the next install tries it once more.
                Err(reason) => notices.warnings.push(format!("cannot remove the stale postinstall link {}: {reason}", path.display())),
            },
            Held::Foreign(why) => {
                leaving.insert(path.clone());
                notices.warnings.push(format!(
                    "left {} alone: the lock no longer declares a postinstall link there, but {why}, so mpm-shadow did not make what is there now; \
                     remove it yourself if it is stale",
                    path.display()
                ));
            }
        }
    }
    let additions: BTreeMap<PathBuf, PathBuf> = made.iter().map(|link| return (link.path.clone(), link.target.clone())).collect();
    match publish_record(&file, &additions, &leaving) {
        Ok(RecordWrite::Settled) => {}
        Ok(RecordWrite::Given) | Err(_) => notices.warnings.push(format!(
            "could not record the postinstall links this run made in {} because another install kept changing it; the links stay where they \
             are, and the next install records them",
            file.display()
        )),
    }
    return Ok(notices);
}

/// What a write of the record settled to.
enum RecordWrite {
    /// The file holds this run's changes.
    Settled,
    /// The file kept changing under the write; this run's changes may be missing and the next install records them.
    Given,
}

/// What:
/// Writes the record of postinstall links by merging this run's changes into what the file holds now,
/// and checks that they survived;
/// after `RECORD_WRITE_ATTEMPTS` it gives up and says so.
/// Why:
/// The record is a map two installs can write at once,
/// and a read-modify-write would let the later writer drop the other's entries.
/// Each run adds only what it made and removes only what it retired,
/// so merging is what the record means,
/// and the check that the additions are still there catches a write that landed between the merge and the check.
/// No lock is taken: the check and the retry are the whole mechanism,
/// and what a lost write costs is one pass's bookkeeping, because the links themselves are what the retirement proves by.
fn publish_record(file: &Path, additions: &BTreeMap<PathBuf, PathBuf>, leaving: &BTreeSet<PathBuf>) -> Result<RecordWrite, ToolchainError> {
    for _attempt in 0..RECORD_WRITE_ATTEMPTS {
        let (current, unreadable) = read_record(file);
        if unreadable.is_some() {
            return Ok(RecordWrite::Given);
        }
        let mut merged: BTreeMap<PathBuf, PathBuf> = current;
        for path in leaving {
            merged.remove(path);
        }
        for (path, target) in additions {
            merged.insert(path.clone(), target.clone());
        }
        if let Some(parent) = file.parent()
            && let Err(error) = fs::create_dir_all(parent)
        {
            return Err(ToolchainError::failure(format!("cannot create {}: {error}", parent.display())));
        }
        write_atomically(file, &render_record(&merged)?)?;
        let (after, _) = read_record(file);
        let mine: bool = additions.iter().all(|(path, target)| return after.get(path) == Some(target))
            && leaving.iter().all(|path| return !after.contains_key(path));
        if mine {
            return Ok(RecordWrite::Settled);
        }
    }
    return Ok(RecordWrite::Given);
}

/// What:
/// Makes the postinstall links of `tools`,
/// retires the recorded links the lock no longer declares,
/// and records what was made.
/// Why:
/// This is the one place `install` handles postinstall links,
/// so a link made by an earlier run and dropped from the lock since is removed by the next run.
/// A partial run (`tools` is a subset) keeps the recorded links of the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncPostinstallLinks(ctx: Context, tools: LockedTool[]): LinkNotices  // throws
/// ```
pub fn sync_postinstall_links(ctx: &Context, tools: &[&LockedTool]) -> Result<LinkNotices, ToolchainError> {
    let outcome = make_postinstall_links(ctx, tools);
    let notices: Result<LinkNotices, ToolchainError> = reconcile(ctx, &outcome.made);
    if let Some(error) = outcome.error {
        return Err(error);
    }
    return notices;
}
