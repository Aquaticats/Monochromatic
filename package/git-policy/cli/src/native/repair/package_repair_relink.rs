//! What:
//!  Hand the relinking of a new worktree's copied package state to the commit service instead of
//!  waiting for it inside the add.
//! Why:
//!  pnpm and mpm-shadow both rewrite state that a copy left naming another location:
//! pnpm's `node_modules` for the new depth,
//! and the toolchain launchers for the new checkout.
//! A traced `git worktree add` of the installed wrapper spent 49.6 s in `pnpm install` and 11.8 s in
//!  `mpm-shadow install`
//! (`doc/planning/cli-git-worktree-add-package-repair.md`),
//! and the second of them started `pnpm list --global` eleven times.
//! Neither run needs the worktree's files:
//! both rewrite link layouts in place,
//! so a person's first command in the worktree can come before or after them,
//! and the add does not have to wait while dozens of adds queue on one disk.
//! The record names the worktree and the tools it needs,
//! so the service runs each tool's own frozen,
//! offline install there,
//! and the add returns after one record and one wake.
//! A record that cannot reach a service stays,
//! and the line the add prints names the command that does the same work by hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! handOverRelink({ context, commonDir, destination, entry }); // enqueue one `relink` item, wake the service
//! ```

use crate::diagnostic_log::debug;
use crate::iso_time::now_iso;
use crate::service_client::{START_WAIT, wake};
use crate::service_commit_input_notes_and_embedded_commit::managed_client_config;
use crate::service_messages::{ClientFailure, render};
use crate::service_provision::provision;
use crate::service_route::route_from_environment;
use crate::service_work::work_directory;
use crate::transaction_git::GitContext;
use crate::work_item::{WorkItem, WorkKind, enqueue};
use crate::worktree_trash_hand_over::Reach;

use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairRelink";

/// The reference a relink record carries.
/// It names no branch:
/// the record is about working state,
/// and a Git reference is only what the queue's record format asks every kind for.
pub const RELINK_REFERENCE: &str = "refs/monochromatic/relink";

/// What:
///  One tool whose copied state the service relinks in the worktree.
/// Why:
///   The record says which tools the worktree needs,
/// so a worktree with only a pnpm layout pays for no toolchain run,
/// and the name is what a record carries,
/// so a record another release wrote cannot ask for a tool this one does not know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RelinkTool = 'pnpm' | 'toolchain';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelinkTool {
    /// pnpm relinks the copied `node_modules` for the new depth.
    Pnpm,
    /// mpm-shadow rebuilds the toolchain links and launchers for the new checkout.
    Toolchain,
}

impl RelinkTool {
    /// The name a record carries for this tool.
    pub fn name(self) -> &'static str {
        return match self {
            RelinkTool::Pnpm => "pnpm",
            RelinkTool::Toolchain => "toolchain",
        };
    }

    /// The tool a name spells,
    /// or nothing for a name this release does not know.
    pub fn from_name(name: &str) -> Option<RelinkTool> {
        return match name {
            "pnpm" => Some(RelinkTool::Pnpm),
            "toolchain" => Some(RelinkTool::Toolchain),
            _ => None,
        };
    }
}

/// What:
///  The work one relink record asks for:
/// the new worktree,
/// and the tools whose state needs relinking there.
/// Why:
///   Both facts are what the add already read from the worktree,
/// so the service repeats no decision the add made,
/// and the record says exactly what the add would have run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RelinkEntry = { root: string; tools: RelinkTool[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelinkEntry {
    /// The new worktree.
    pub root: PathBuf,
    /// The tools the service runs there,
    /// in the order the report has always named them.
    pub tools: Vec<RelinkTool>,
}

/// What:
///  The name of one tool as the payload carries it.
/// Why:
///   The record's list is one JSON string per tool,
/// and a named function stands where a closure that only forwarded its argument stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nameOfTool = (tool) => tool.name();
/// ```
fn name_of_tool(tool: &RelinkTool) -> serde_json::Value {
    return serde_json::Value::from(tool.name());
}

/// What:
///  The entry as the one text a record can carry.
/// Why:
///   The queue's record holds one text field,
/// so the entry travels as one JSON object,
/// and the path is refused rather than written lossily.
///
/// # Errors
///
/// A path that is not UTF-8,
/// which no record could name back,
/// and an entry that asks for no tool,
/// which would be a record of nothing.
pub fn encode(entry: &RelinkEntry) -> Result<String, String> {
    if entry.tools.is_empty() {
        return Err(String::from("a relink of no tool is no work"));
    }
    let Some(root) = entry.root.to_str() else {
        return Err(String::from("the path of the relink is not valid UTF-8, so no record can name it"));
    };
    let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    fields.insert(String::from("root"), serde_json::Value::from(root));
    let names: Vec<serde_json::Value> = entry.tools.iter().map(name_of_tool).collect();
    fields.insert(String::from("tools"), serde_json::Value::Array(names));
    return Ok(serde_json::Value::Object(fields).to_string());
}

/// What:
///  The entry a relink record carries.
/// Why:
///   A payload that is not the object `encode` writes is refused,
/// and so is a tool name this release does not know or a list of no tool:
/// the service must run exactly the tools the add decided on.
///
/// # Errors
///
/// Text that is not the object,
/// a root that is not named,
/// and a tool that cannot be read.
pub fn decode(payload: &str) -> Result<RelinkEntry, String> {
    let parsed: serde_json::Value = match serde_json::from_str(payload) {
        Ok(value) => value,
        Err(error) => return Err(format!("the relink payload is not JSON ({error})")),
    };
    let Some(root) = parsed.get("root").and_then(serde_json::Value::as_str) else {
        return Err(String::from("the relink names no worktree"));
    };
    let Some(names) = parsed.get("tools").and_then(serde_json::Value::as_array) else {
        return Err(String::from("the relink names no tools"));
    };
    let mut tools: Vec<RelinkTool> = Vec::with_capacity(names.len());
    for name in names {
        let Some(named) = name.as_str() else {
            return Err(String::from("a tool of the relink is not text"));
        };
        let Some(tool) = RelinkTool::from_name(named) else {
            return Err(format!("this release knows no tool named {named}"));
        };
        tools.push(tool);
    }
    if tools.is_empty() {
        return Err(String::from("the relink names no tools"));
    }
    return Ok(RelinkEntry { root: PathBuf::from(root), tools });
}

/// What:
///  The record's object name for a worktree:
/// a digest of its path,
/// which no commit and no other worktree shares.
/// Why:
///   The record's id is its idempotency key,
/// and two worktrees added from the same commit must each get their own relink,
/// while a record a killed add wrote and the next add's record are one item.
/// Forty hexadecimal digits is what a work item's object name must have,
/// and a SHA-256 slice is cheap to compute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const objectNameOf = (destination) => sha256(utf8(destination)).toString('hex').slice(0, 40);
/// ```
pub fn object_name_of(destination: &Path) -> String {
    let digest: [u8; 32] = crate::subvolume_sha::sha256(destination.as_os_str().as_bytes());
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(&digest[..20]);
}

/// What:
///  The record a worktree's relink is queued as.
/// Why:
///   The producer and its controls both write exactly this,
/// so a control can compare a record without reaching a service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itemOf = ({ commonDir, destination, entry }) => ({ kind: 'relink', repository: destination, gitDir: commonDir, ref: RELINK_REFERENCE, oid: objectNameOf(destination), parent: encode(entry), created: nowISO(), tries: 0 });
/// ```
pub fn item_of(common_dir: &Path, destination: &Path, entry: &RelinkEntry) -> Result<WorkItem, String> {
    return Ok(WorkItem {
        kind: WorkKind::Relink,
        repository: destination.to_path_buf(),
        git_dir: common_dir.to_path_buf(),
        branch_ref: String::from(RELINK_REFERENCE),
        oid: object_name_of(destination),
        parent: encode(entry)?,
        created: now_iso(),
        tries: 0,
    });
}

/// What:
///  Write the relink's record and wake the service that runs it.
/// Why:
///   The record is written first,
/// so a wake that cannot reach the service leaves the work named rather than lost,
/// and the add never waits for the relinking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverToTheService({ context, commonDir, destination, entry });
/// ```
pub fn hand_over_to_the_service(context: &GitContext, common_dir: &Path, destination: &Path, entry: &RelinkEntry) -> Result<(), String> {
    return hand_over_with(context, common_dir, destination, entry, &Reach { router: &route_from_environment, provisioner: &provision, start_wait: START_WAIT });
}

/// What:
///  `hand_over_to_the_service` with the route finder and the provisioner injected,
/// so a control reaches no service and no session bus.
/// Why:
///   The same reach a prewarm is handed over with,
/// so a relink is queued and woken exactly as that work is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ context, commonDir, destination, entry, reach });
/// ```
pub fn hand_over_with(context: &GitContext, common_dir: &Path, destination: &Path, entry: &RelinkEntry, reach: &Reach<'_>) -> Result<(), String> {
    let item: WorkItem = item_of(common_dir, destination, entry)?;
    if let Err(reason) = enqueue(work_directory(common_dir).as_path(), &item) {
        debug(TAG, format!("the work item could not be written: {reason}").as_str());
        return Err(format!("the work item could not be written: {reason}"));
    }
    let Some(route) = (reach.router)(context.real_git.as_path(), common_dir) else {
        return Err(String::from("no commit service can be started from this session"));
    };
    let crate::service_route::Route { address, provision: host } = route;
    if let Some(setup) = host.as_ref()
        && let Err(failure) = (reach.provisioner)(setup)
    {
        return Err(render(&failure));
    }
    let config: ClientConfig = managed_client_config(address, host.as_ref());
    // The wake's lines are not the add's: it says once what became of the relinking.
    let mut silent: std::io::Sink = std::io::sink();
    return wake(&config, common_dir, None, &mut silent).map_err(render_failure);
}

/// What:
///  The words a client failure is reported with.
/// Why:
///   The wake's failure is the person's,
/// and a named function stands where a closure that only forwarded its argument stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const renderFailure = (failure) => render(failure);
/// ```
fn render_failure(failure: ClientFailure) -> String {
    return render(&failure);
}

/// The client configuration type the wake takes,
/// named here so the import list stays one word per line.
type ClientConfig = crate::service_client::ClientConfig;

/// What:
///  The hand-over of a relink record as a job of the process's blocking pool.
/// Why:
///  Writing a record and waking a service that may have to be started are storage and socket waits,
/// and the task that runs the Cargo repair beside it must never do either:
/// a service that never answered would stop that repair's waits with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handOverInAJob = ([context, commonDir, destination, entry]) => handOverToTheService(context, commonDir, destination, entry);
/// ```
pub(super) fn hand_over_in_a_job(state: TheHandOver) -> Result<(), String> {
    let (context, common_dir, destination, entry): TheHandOver = state;
    return hand_over_to_the_service(&context, common_dir.as_path(), destination.as_path(), &entry);
}

/// What a hand-over job reads:
/// the Git context,
/// the repository's common directory,
/// the new worktree,
/// and the entry.
/// Why:
///   A job reads only what it was handed,
/// so the four travel together as owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheHandOver = [GitContext, string, string, RelinkEntry];
/// ```
pub(super) type TheHandOver = (GitContext, PathBuf, PathBuf, RelinkEntry);

/// Relink controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_tests.rs"]
mod tests;
