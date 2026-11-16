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
//! The record names the tool that made each link,
//! because a command that converges a subset of the plan retires nothing but its own:
//! the retirement of a stale link waits for the command that converges the whole plan.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // postinstall-links.ts
//! export function syncPostinstallLinks(ctx: Context, tools: LockedTool[], pass: LinkPass): LinkNotices  // throws
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
use super::scope::ScopeKind;
/// Imports `write_atomically` from `super::publish`.
use super::publish::write_atomically;

/// The file name of the record,
/// inside the scope's launcher directory.
const RECORD_FILE_NAME: &str = ".postinstall-links.json";

/// The JSON key holding the links,
/// as an object of link path to that link's entry.
const LINKS_KEY: &str = "links";

/// The key of a link's target inside its record entry.
const TARGET_KEY: &str = "target";

/// The key of the tool that made a link inside its record entry.
const TOOL_KEY: &str = "tool";

/// What:
/// Which tools one pass of the postinstall links covers.
/// Why:
/// Making a link is always narrowed to the tools the pass was given;
/// retiring one must be narrowed the same way,
/// or a command that converges a subset retires the stale link of a tool it never looked at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LinkPass {
    /// Every tool of the lock (`install` converges the whole plan):
    /// a recorded link the lock no longer declares is retired whoever declared it.
    WholePlan,
    /// Only the tools this pass was given (`update` installs a few before looking them up again):
    /// a recorded link another tool made stays where it is and stays in the record,
    /// for the next pass over the whole plan to retire.
    Subset,
}

/// One link as the record holds it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RecordedLink {
    /// What the link pointed at when this tool made it.
    target: PathBuf,
    /// The id of the tool that declared and made the link,
    /// or `None` for a record written before the record named its tools.
    tool: Option<String>,
}

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

/// What:
/// One record entry as the record holds it:
/// the link target it names and the tool that made the link.
/// Why:
/// A record written before the record named its tools holds the target alone;
/// such an entry names no tool, so a pass over a subset leaves it alone.
fn recorded_link(value: &Value) -> Option<RecordedLink> {
    if let Some(target) = value.as_str() {
        return Some(RecordedLink { target: PathBuf::from(target), tool: None });
    }
    let entry: &Map<String, Value> = value.as_object()?;
    let target: &str = entry.get(TARGET_KEY)?.as_str()?;
    let tool: Option<String> = entry.get(TOOL_KEY).and_then(Value::as_str).map(str::to_string);
    return Some(RecordedLink { target: PathBuf::from(target), tool });
}

/// The recorded links (path to its entry),
/// and a warning when the record exists but cannot be read,
/// in which case nothing is retired this run.
fn read_record(file: &Path) -> (BTreeMap<PathBuf, RecordedLink>, Option<String>) {
    let text: String = match fs::read_to_string(file) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (BTreeMap::new(), None),
        Err(error) => return (BTreeMap::new(), Some(format!("cannot read {}, so no stale postinstall link is retired: {error}", file.display()))),
    };
    let mut links: BTreeMap<PathBuf, RecordedLink> = BTreeMap::new();
    let parsed: Option<Value> = serde_json::from_str(&text).ok();
    let links_value: Option<&Value> = match &parsed {
        Some(value) => value.get(LINKS_KEY),
        None => None,
    };
    let Some(entries) = links_value.and_then(Value::as_object) else {
        return (links, Some(format!("{} is not a record of postinstall links, so no stale postinstall link is retired", file.display())));
    };
    for (path, value) in entries {
        if let Some(recorded) = recorded_link(value) {
            links.insert(PathBuf::from(path), recorded);
        }
    }
    return (links, None);
}

/// The record as JSON text.
fn render_record(links: &BTreeMap<PathBuf, RecordedLink>) -> Result<String, ToolchainError> {
    let mut entries: Map<String, Value> = Map::new();
    for (path, recorded) in links {
        let mut entry: Map<String, Value> = Map::new();
        entry.insert(TARGET_KEY.to_string(), Value::String(recorded.target.to_string_lossy().to_string()));
        if let Some(tool) = &recorded.tool {
            entry.insert(TOOL_KEY.to_string(), Value::String(tool.clone()));
        }
        entries.insert(path.to_string_lossy().to_string(), Value::Object(entry));
    }
    let mut root: Map<String, Value> = Map::new();
    root.insert(LINKS_KEY.to_string(), Value::Object(entries));
    return match serde_json::to_string_pretty(&Value::Object(root)) {
        Ok(text) => Ok(format!("{text}\n")),
        Err(error) => Err(ToolchainError::failure(format!("cannot render the record of postinstall links: {error}"))),
    };
}

/// The full path of every postinstall link any locked tool declares,
/// whatever its operating system.
fn declared_link_paths(ctx: &Context) -> BTreeSet<PathBuf> {
    let mut declared: BTreeSet<PathBuf> = BTreeSet::new();
    for tool in ctx.tools {
        if let Some(Postinstall::Link(link)) = &tool.declaration.options.postinstall {
            declared.insert(link_directory(ctx.scope, &link.link_dir).join(&link.link_name));
        }
    }
    return declared;
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
    /// The symlink the record names.
    OwnLink,
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
    return match fs::read_link(path) {
        Ok(target) if target == recorded_target => Held::OwnLink,
        Ok(target) => Held::Foreign(format!("it points at {}", target.display())),
        Err(error) => Held::Foreign(format!("its target cannot be read: {error}")),
    };
}

/// What:
/// Whether this pass may retire the link this record entry names,
/// given the tools a pass over a subset covers (`None` for a pass over the whole plan).
/// Why:
/// A pass over a subset may retire only what its own tools declared.
/// An entry written before the record named its tools cannot prove that it belongs to the pass,
/// so it waits for the pass over the whole plan.
fn may_retire(recorded: &RecordedLink, subset: Option<&BTreeSet<&str>>) -> bool {
    let Some(covered) = subset else {
        return true;
    };
    return match recorded.tool.as_deref() {
        Some(tool) => covered.contains(tool),
        None => false,
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
/// A pass over a subset (`covered` names its tools) retires only links those tools made;
/// a link another tool made stays where it is and stays fully in the record,
/// so the pass that converges the whole plan is the one that retires it.
fn reconcile(ctx: &Context, made: &[MadeLink], covered: Option<&BTreeSet<&str>>) -> Result<LinkNotices, ToolchainError> {
    let file: PathBuf = record_file(ctx);
    let (recorded, unreadable) = read_record(&file);
    let mut notices: LinkNotices = LinkNotices::default();
    if let Some(warning) = unreadable {
        notices.warnings.push(warning);
        return Ok(notices);
    }
    let declared: BTreeSet<PathBuf> = declared_link_paths(ctx);
    let mut kept: BTreeMap<PathBuf, RecordedLink> = BTreeMap::new();
    for (path, recorded_link) in &recorded {
        if declared.contains(path) {
            kept.insert(path.clone(), recorded_link.clone());
            continue;
        }
        if !is_own_link(ctx, path) {
            continue;
        }
        if !may_retire(recorded_link, covered) {
            kept.insert(path.clone(), recorded_link.clone());
            continue;
        }
        match inspect(path, &recorded_link.target) {
            Held::Nothing => {}
            Held::OwnLink => match fs::remove_file(path) {
                Ok(()) => notices.retired.push(format!(
                    "retired the postinstall link {} (it pointed at {}) because the lock no longer declares it",
                    path.display(),
                    recorded_link.target.display()
                )),
                Err(error) => {
                    notices.warnings.push(format!("cannot remove the stale postinstall link {}: {error}", path.display()));
                    kept.insert(path.clone(), recorded_link.clone());
                }
            },
            Held::Foreign(why) => notices.warnings.push(format!(
                "left {} alone: the lock no longer declares a postinstall link there, but {why}, so mpm-shadow did not make what is there now; \
                 remove it yourself if it is stale",
                path.display()
            )),
        }
    }
    for link in made {
        kept.insert(link.path.clone(), RecordedLink { target: link.target.clone(), tool: Some(link.tool.clone()) });
    }
    if kept == recorded {
        return Ok(notices);
    }
    if let Some(parent) = file.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", parent.display())));
    }
    write_atomically(&file, &render_record(&kept)?)?;
    return Ok(notices);
}

/// What:
/// Makes the postinstall links of `tools`,
/// retires the recorded links the lock no longer declares,
/// and records what was made.
/// Why:
/// This is the one place `install` handles postinstall links,
/// so a link made by an earlier run and dropped from the lock since is removed by the next run.
/// A pass over a subset keeps the recorded links of the others,
/// including the stale ones: only a tool of the pass loses a link it stopped declaring.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncPostinstallLinks(ctx: Context, tools: LockedTool[], pass: LinkPass): LinkNotices  // throws
/// ```
pub fn sync_postinstall_links(ctx: &Context, tools: &[&LockedTool], pass: LinkPass) -> Result<LinkNotices, ToolchainError> {
    let outcome = make_postinstall_links(ctx, tools);
    let mut covered: Option<BTreeSet<&str>> = None;
    if pass == LinkPass::Subset {
        covered = Some(tools.iter().map(|tool| return tool.declaration.id.as_str()).collect());
    }
    let notices: Result<LinkNotices, ToolchainError> = reconcile(ctx, &outcome.made, covered.as_ref());
    if let Some(error) = outcome.error {
        return Err(error);
    }
    return notices;
}
