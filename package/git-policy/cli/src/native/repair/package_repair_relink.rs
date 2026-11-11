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
use crate::service_client::wake;
use crate::wait_ceilings::HAND_OVER_GREETING_WAIT;
use crate::service_commit_input_notes_and_embedded_commit::managed_client_config;
use crate::service_messages::{ClientFailure, render};
use crate::service_provision::provision;
use crate::service_route::route_from_environment;
use crate::service_server::Bind;
use crate::service_work::work_directory;
use crate::transaction_git::GitContext;
use crate::work_item::{Enqueued, PENDING, WorkItem, WorkKind, enqueue, state_directory};
use crate::worktree_trash_hand_over::Reach;

use std::ffi::OsString;
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
/// the search path the add resolved its programs on,
/// the directory that search path was resolved from,
/// and the tools whose state needs relinking there.
/// Why:
///   The first three facts are what the add already read from the worktree and the person's environment,
/// so the service repeats no decision the add made,
/// and the record says exactly what the add would have run.
/// The search path travels because the service is a resident process of another session:
/// its own path may hold no pnpm or another one,
/// and the owner decided the repair starts whatever pnpm is first on the person's path.
/// The directory travels with it because an entry of a search path without a leading slash means
/// a directory of the working directory the programs were found in,
/// and a service works in its own:
/// without that fact the record's relative entries name other directories there,
/// the program is not found,
/// and the worktree keeps the links the copy left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RelinkEntry = { root: string; searchPath?: string; searchFrom?: string; tools: RelinkTool[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelinkEntry {
    /// The new worktree.
    pub root: PathBuf,
    /// The search path the add resolved each tool's program on,
    /// and that the tools' own children inherit;
    /// absent only in a record written before this release carried it,
    /// where the service searches its own path.
    pub search_path: Option<OsString>,
    /// The working directory the add resolved that search path in,
    /// so a relative entry of it names the same directory for the service;
    /// absent only in a record written before this release carried it,
    /// where a relative entry was read against the service's own working directory.
    pub search_from: Option<PathBuf>,
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
/// and a path or a search path is refused rather than written lossily.
///
/// # Errors
///
/// A path,
/// a search path,
/// or the directory a search path was resolved in that is not UTF-8,
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
    if let Some(search_path) = entry.search_path.as_deref() {
        let Some(text) = search_path.to_str() else {
            return Err(String::from("the search path of the relink is not valid UTF-8, so no record can carry it"));
        };
        fields.insert(String::from("searchPath"), serde_json::Value::from(text));
    }
    if let Some(search_from) = entry.search_from.as_deref() {
        let Some(text) = search_from.to_str() else {
            return Err(String::from("the directory the relink resolved its search path in is not valid UTF-8, so no record can carry it"));
        };
        fields.insert(String::from("searchFrom"), serde_json::Value::from(text));
    }
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
/// A record written before this release carried a search path names none,
/// and the service then searches its own path,
/// so such a record still relinks the worktree it was written for;
/// a record of that release also names no directory its search path was resolved in,
/// and a relative entry of it therefore keeps the service's working directory.
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
    let search_path: Option<OsString> = parsed.get("searchPath").and_then(serde_json::Value::as_str).map(OsString::from);
    let search_from: Option<PathBuf> = parsed.get("searchFrom").and_then(serde_json::Value::as_str).map(PathBuf::from);
    return Ok(RelinkEntry { root: PathBuf::from(root), search_path, search_from, tools });
}

/// What:
///  The record's object name for one hand-over:
/// a digest of the worktree's path and the moment the add asked for the relink.
/// Why:
///   A work item's id is its idempotency key,
/// and the queue answers an id it already knows as known in every state.
/// With the path alone,
/// a worktree removed and added again at the same path would be answered by the earlier record,
/// and its copied links would stay exactly as the copy left them while the add reported a relink
/// that can never arrive.
/// A previous failure would keep every later add at that path from ever asking again.
/// The moment tells two hand-overs of one path apart,
/// and the path keeps two worktrees apart.
/// Forty hexadecimal digits is what a work item's object name must have,
/// and a SHA-256 slice is cheap to compute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const objectNameOf = (destination, queuedAt) => sha256(utf8(`${destination}\n${queuedAt}`)).toString('hex').slice(0, 40);
/// ```
pub fn object_name_of(destination: &Path, queued_at: &str) -> String {
    let mut text: Vec<u8> = Vec::new();
    text.extend_from_slice(destination.as_os_str().as_bytes());
    text.push(b'\n');
    text.extend_from_slice(queued_at.as_bytes());
    let digest: [u8; 32] = crate::subvolume_sha::sha256(text.as_slice());
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(&digest[..20]);
}

/// What:
///  The record a worktree's relink is queued as.
/// Why:
///   The producer and its controls both write exactly this,
/// so a control can compare a record without reaching a service.
/// The stamp is read once and is both the record's `created` and part of its object name,
/// so a control can tell which record it is looking at from the record alone,
/// and two adds at one path are two records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itemOf = ({ commonDir, destination, entry }) => { const created = new Date().toISOString(); return { kind: 'relink', repository: destination, gitDir: commonDir, ref: RELINK_REFERENCE, oid: objectNameOf(destination, created), parent: encode(entry), created, tries: 0 }; };
/// ```
pub fn item_of(common_dir: &Path, destination: &Path, entry: &RelinkEntry) -> Result<WorkItem, String> {
    let created: String = now_iso();
    return Ok(WorkItem {
        kind: WorkKind::Relink,
        repository: destination.to_path_buf(),
        git_dir: common_dir.to_path_buf(),
        branch_ref: String::from(RELINK_REFERENCE),
        oid: object_name_of(destination, created.as_str()),
        parent: encode(entry)?,
        created,
        tries: 0,
    });
}

/// What:
///  Write the relink's record and wake the service that runs it,
/// or take the record back when no service holds it.
/// Why:
///   The record is written first,
/// so a wake that cannot reach the service leaves the work named rather than lost,
/// and the add never waits for the relinking.
/// A record nobody took owes nothing,
/// because the add relinks the worktree itself,
/// and it must go:
/// a record names a path and the tools for it,
/// and a service that drained it later would run those tools in whatever directory holds that path by then,
/// which need not be the worktree the add relinked.
/// Taking the pending record back is one unlink,
/// which the queue's claim answers by finding nothing to take,
/// so the two never both do the work.
///
/// Ok:
///  A service holds the record,
/// so the relinking is its work.
/// Err:
///  No service holds the record and none can be given it,
/// so the add relinks the worktree itself;
/// the text says why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverToTheService({ context, commonDir, destination, entry });
/// ```
pub fn hand_over_to_the_service(context: &GitContext, common_dir: &Path, destination: &Path, entry: &RelinkEntry) -> Result<(), String> {
    return hand_over_with(context, common_dir, destination, entry, &Reach { router: &route_from_environment, provisioner: &provision, start_wait: HAND_OVER_GREETING_WAIT });
}

/// What:
///  `hand_over_to_the_service` with the route finder and the provisioner injected,
/// so a control reaches no service and no session bus.
/// Why:
///   The same reach a prewarm is handed over with,
/// so a relink is queued and woken exactly as that work is,
/// and a hand-over that reaches nothing answers as `hand_over_to_the_service` documents.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ context, commonDir, destination, entry, reach });
/// ```
pub fn hand_over_with(context: &GitContext, common_dir: &Path, destination: &Path, entry: &RelinkEntry, reach: &Reach<'_>) -> Result<(), String> {
    let item: WorkItem = item_of(common_dir, destination, entry)?;
    let work_dir: PathBuf = work_directory(common_dir);
    let id: String = item.id();
    match enqueue(work_dir.as_path(), &item) {
        Ok(Enqueued::Added) => {}
        Ok(Enqueued::Known) => {
            // The id says this add's own record is already queued: the moment is part of the name, so no earlier add's
            //  record answers for this one. A record of this add that is not pending any more was claimed by a service,
            //  which knows what became of it.
            return if pending_record(&work_dir, id.as_str()).exists() {
                Ok(())
            } else {
                Err(format!("the record {id} of this relink was already taken by a service, which holds it"))
            };
        }
        Err(reason) => {
            debug(TAG, format!("the work item could not be written: {reason}").as_str());
            return Err(format!("the work item could not be written: {reason}"));
        }
    }
    let reason: String = match (reach.router)(context.real_git.as_path(), common_dir) {
        None => String::from("no commit service can be started from this session"),
        Some(route) => {
            let crate::service_route::Route { address, provision: host } = route;
            let reached: Result<(), String> = match host.as_ref() {
                Some(setup) => match (reach.provisioner)(setup) {
                    Ok(()) => wake_the_service(address, host.as_ref(), common_dir, reach.start_wait),
                    Err(failure) => Err(render(&failure)),
                },
                None => wake_the_service(address, host.as_ref(), common_dir, reach.start_wait),
            };
            match reached {
                Ok(()) => return Ok(()),
                Err(reason) => reason,
            }
        }
    };
    debug(TAG, format!("{}: no service took the relink ({reason})", destination.display()).as_str());
    return Err(match withdraw_pending(&work_dir, id.as_str()) {
        Withdrawal::Withdrew => reason,
        // Another party took the record between the write and the unlink: a service holds it, so its work is not lost
        //  and the add must not run the same install beside it.
        Withdrawal::Taken => {
            debug(TAG, format!("{}: a service took the record {id} before it could be withdrawn", destination.display()).as_str());
            return Ok(());
        }
        // The record could not be removed, so it stays and the service may run the same install again: the add relinks
        //  the worktree too, because a worktree whose copied links were never relinked is wrong either way.
        Withdrawal::Unremoved(error) => format!("{reason}; the record {id} stays ({error}), so the commit service may relink this worktree again"),
    });
}

/// What:
///  Tell the service at `address` that the repository has work to drain,
///  waiting `start_wait` for its greeting and no longer than that for a start.
/// Why:
///   The wake's lines are not the add's:
/// it says once what became of the relinking,
/// so the client's output is dropped here,
/// and the wait is the reach's,
/// which a control sets short enough that a service which never answers cannot hold it.
/// The record is written before this wake,
/// so the bound is the hand-over's own greeting wait and never the commit's start wait:
/// a greeting that does not come within it means this command does the relinking itself
/// (`wait_ceilings.rs`,
/// `HAND_OVER_GREETING_WAIT`),
/// and the ceiling moves with it so no other bound lengthens the wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const wakeTheService = (address, host, commonDir, startWait) => { const config = { ...managedClientConfig(address, host), startWait, startCeiling: startWait }; wake(config, commonDir, undefined, silent) };
/// ```
fn wake_the_service(address: Bind, host: Option<&crate::service_provision::ProvisionHost>, common_dir: &Path, start_wait: std::time::Duration) -> Result<(), String> {
    let config: ClientConfig = ClientConfig { start_wait, start_ceiling: start_wait, ..managed_client_config(address, host) };
    let mut silent: std::io::Sink = std::io::sink();
    return wake(&config, common_dir, None, &mut silent).map_err(render_failure);
}

/// What:
///  What a hand-over's attempt to take its pending record back found.
/// Why:
///   The three answers differ in what the add does next:
/// nothing took the record,
/// another party took it,
/// or it could not be removed and both may run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Withdrawal = 'withdrew' | 'taken' | { unremoved: string };
/// ```
enum Withdrawal {
    /// The pending record was removed,
    /// so no service can claim it.
    Withdrew,
    /// The pending record was gone already,
    /// so a claim or a settlement took it.
    Taken,
    /// The record is still queued and says why it could not be removed.
    Unremoved(String),
}

/// The pending record of one id.
fn pending_record(work_dir: &Path, id: &str) -> PathBuf {
    return state_directory(work_dir, PENDING).join(id);
}

/// What:
///  Remove the pending record of one id,
///  so no service drains work the add did itself.
/// Why:
///   The claim moves the file out of `pending` in one rename,
/// so this unlink either wins and the claim finds nothing,
/// or loses and the service owns the work;
/// neither party runs the install for nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withdrawPending = (workDir, id) => { try { unlink(pending(workDir, id)); return 'withdrew' } catch (error) { return error.code === 'ENOENT' ? 'taken' : { unremoved: error.message } } };
/// ```
fn withdraw_pending(work_dir: &Path, id: &str) -> Withdrawal {
    return match std::fs::remove_file(pending_record(work_dir, id).as_path()) {
        Ok(()) => Withdrawal::Withdrew,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Withdrawal::Taken,
        Err(error) => Withdrawal::Unremoved(error.to_string()),
    };
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
