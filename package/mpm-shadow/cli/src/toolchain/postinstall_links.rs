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
/// Imports `write_atomically` from `super::publish`.
use super::publish::write_atomically;

/// The file name of the record,
/// inside the scope's launcher directory.
const RECORD_FILE_NAME: &str = ".postinstall-links.json";

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
    let Some(entries) = parsed.as_ref().and_then(|value| return value.get(LINKS_KEY)).and_then(Value::as_object) else {
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

/// What a recorded path holds now.
enum Held {
    /// Nothing.
    Nothing,
    /// The symlink the record names.
    OwnLink,
    /// Something else,
    /// described for the warning.
    Foreign(String),
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
/// Retires every recorded link the lock no longer declares,
/// then writes the record of the links that remain.
/// Why:
/// A recorded link that is still the symlink the record names is removed;
/// a path that now holds a file of a person's,
/// or a link to somewhere else,
/// is left untouched and named in a warning,
/// and leaves the record so the warning appears once.
fn reconcile(ctx: &Context, made: &[MadeLink]) -> Result<LinkNotices, ToolchainError> {
    let file: PathBuf = record_file(ctx);
    let (recorded, unreadable) = read_record(&file);
    let mut notices: LinkNotices = LinkNotices::default();
    if let Some(warning) = unreadable {
        notices.warnings.push(warning);
        return Ok(notices);
    }
    let declared: BTreeSet<PathBuf> = declared_link_paths(ctx);
    let mut kept: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
    for (path, target) in &recorded {
        if declared.contains(path) {
            kept.insert(path.clone(), target.clone());
            continue;
        }
        match inspect(path, target) {
            Held::Nothing => {}
            Held::OwnLink => match fs::remove_file(path) {
                Ok(()) => notices.retired.push(format!(
                    "retired the postinstall link {} (it pointed at {}) because the lock no longer declares it",
                    path.display(),
                    target.display()
                )),
                Err(error) => {
                    notices.warnings.push(format!("cannot remove the stale postinstall link {}: {error}", path.display()));
                    kept.insert(path.clone(), target.clone());
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
        kept.insert(link.path.clone(), link.target.clone());
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
