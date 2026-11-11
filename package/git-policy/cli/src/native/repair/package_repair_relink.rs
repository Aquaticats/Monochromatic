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
///  The search path a record carries.
/// Why:
///   The record must say which programs the add resolved,
/// and "the add held no search path" is not "this record is too old to carry one".
/// With no path,
/// the add selects the worktree's own `.cache/toolchain-bin/mpm-shadow`,
/// while a record that names none makes the service search its own path and start another program;
/// so the three answers travel as three,
/// not as the two an optional field could carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchPathOfTheAdd = { kind: 'not-recorded' } | { kind: 'unset' } | { kind: 'set'; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchPathOfTheAdd {
    /// A record written before this release carried the add's environment:
    /// the service searches its own path.
    NotRecorded,
    /// The add's environment held no search path,
    /// so the service's own must not stand in for one.
    Unset,
    /// The directories the add resolved each tool's program on.
    Set(OsString),
}

/// What:
///  The work one relink record asks for:
/// the new worktree,
/// the search path the add resolved its programs on,
/// and the tools whose state needs relinking there.
/// Why:
///   The first two facts are what the add already read from the worktree and the person's environment,
/// so the service repeats no decision the add made,
/// and the record says exactly what the add would have run.
/// The search path travels because the service is a resident process of another session:
/// its own path may hold no pnpm or another one,
/// and the owner decided the repair starts whatever pnpm is first on the person's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RelinkEntry = { root: string; searchPath: SearchPathOfTheAdd; tools: RelinkTool[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelinkEntry {
    /// The new worktree.
    pub root: PathBuf,
    /// The search path the add resolved each tool's program on,
    /// and that the tools' own children inherit.
    pub search_path: SearchPathOfTheAdd,
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
/// A path or a search path that is not UTF-8,
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
    // The three answers are three spellings: no field for a record written before the add's environment travelled,
    // a null for an add that held no search path, and the text for the one it held.
    match entry.search_path {
        SearchPathOfTheAdd::NotRecorded => {}
        SearchPathOfTheAdd::Unset => {
            fields.insert(String::from("searchPath"), serde_json::Value::Null);
        }
        SearchPathOfTheAdd::Set(ref search_path) => {
            let Some(text) = search_path.to_str() else {
                return Err(String::from("the search path of the relink is not valid UTF-8, so no record can carry it"));
            };
            fields.insert(String::from("searchPath"), serde_json::Value::from(text));
        }
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
/// so such a record still relinks the worktree it was written for.
/// A record whose search path is null names an add that held none,
/// so the service searches nowhere rather than searching its own.
///
/// # Errors
///
/// Text that is not the object,
/// a root that is not named,
/// a tool that cannot be read,
/// and a search path that is neither text nor null.
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
    let search_path: SearchPathOfTheAdd = match parsed.get("searchPath") {
        None => SearchPathOfTheAdd::NotRecorded,
        Some(serde_json::Value::Null) => SearchPathOfTheAdd::Unset,
        Some(serde_json::Value::String(text)) => SearchPathOfTheAdd::Set(OsString::from(text)),
        Some(_) => return Err(String::from("the search path of the relink is neither text nor null")),
    };
    return Ok(RelinkEntry { root: PathBuf::from(root), search_path, tools });
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
///  The entry with every relative directory of its search path made absolute against the add's directory.
/// Why:
///   `PATH=bin` names the add's own `bin`,
/// and the service is a resident process of another session,
/// started from another directory,
/// so a record carrying `bin` would make it search its own `bin`,
/// and the program the add resolved may never run.
/// The tools' own children read the same path from the same record,
/// so they resolve on the add's directories too.
/// An empty entry is left as it is:
/// it means the current directory of whoever runs,
/// which no record can name for another process.
///
/// # Errors
///
/// A working directory this process cannot read,
/// so no record could say what `bin` meant,
/// and a search path that cannot be written back.
fn entry_with_an_absolute_search_path(entry: &RelinkEntry) -> Result<RelinkEntry, String> {
    let SearchPathOfTheAdd::Set(search_path) = &entry.search_path else { return Ok(entry.clone()) };
    if !names_a_relative_directory(search_path) {
        return Ok(entry.clone());
    }
    let add: PathBuf = std::env::current_dir().map_err(no_working_directory)?;
    let mut absolute: Vec<PathBuf> = Vec::new();
    for directory in std::env::split_paths(search_path) {
        if directory.as_os_str().is_empty() || directory.is_absolute() {
            absolute.push(directory);
            continue;
        }
        absolute.push(add.join(directory));
    }
    let joined: OsString = std::env::join_paths(absolute).map_err(unrecordable_search_path)?;
    return Ok(RelinkEntry { search_path: SearchPathOfTheAdd::Set(joined), ..entry.clone() });
}

/// What:
///  Whether any entry of `search_path` is relative to whatever directory runs.
/// Why:
///   Only a relative entry needs the add's directory,
/// and a path of only absolute entries and empty entries is recorded as the add held it,
/// which is the same text the person wrote.
/// An empty entry is not relative:
/// a shell reads it as the current directory of whatever runs,
/// which no record can name for another process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const relative = splitPath(searchPath).some((directory) => directory !== '' && !isAbsolute(directory));
/// ```
fn names_a_relative_directory(search_path: &std::ffi::OsStr) -> bool {
    for directory in std::env::split_paths(search_path) {
        if !directory.as_os_str().is_empty() && !directory.is_absolute() {
            return true;
        }
    }
    return false;
}

/// What:
///  The words a working directory that cannot be read is reported with.
/// Why:
///   `bin` means the add's own `bin`,
/// so a command whose own directory cannot be read cannot say what its search path named,
/// and no record could carry an answer it did not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noWorkingDirectory = (error) => `the directory this command runs in cannot be read (${error}), so a relative search path names no program it could resolve`;
/// ```
fn no_working_directory(error: std::io::Error) -> String {
    return format!("the directory this command runs in cannot be read ({error}), so a relative search path names no program it could resolve");
}

/// What:
///  The words a search path that cannot be written back are reported with.
/// Why:
///   A joined path holds a separator between entries,
/// so an entry holding that separator cannot be recorded,
/// and the add says so rather than writing a path that means another set of directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unrecordableSearchPath = (error) => `the search path of this command cannot be recorded (${error})`;
/// ```
fn unrecordable_search_path(error: std::env::JoinPathsError) -> String {
    return format!("the search path of this command cannot be recorded ({error})");
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
    let recorded: RelinkEntry = entry_with_an_absolute_search_path(entry)?;
    return Ok(WorkItem {
        kind: WorkKind::Relink,
        repository: destination.to_path_buf(),
        git_dir: common_dir.to_path_buf(),
        branch_ref: String::from(RELINK_REFERENCE),
        oid: object_name_of(destination, created.as_str()),
        parent: encode(&recorded)?,
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
/// the answer says why,
/// and the record it leaves is taken back after that run finishes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverToTheService({ context, commonDir, destination, entry });
/// ```
pub fn hand_over_to_the_service(context: &GitContext, common_dir: &Path, destination: &Path, entry: &RelinkEntry) -> Result<(), NotTaken> {
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
pub fn hand_over_with(context: &GitContext, common_dir: &Path, destination: &Path, entry: &RelinkEntry, reach: &Reach<'_>) -> Result<(), NotTaken> {
    let item: WorkItem = item_of(common_dir, destination, entry).map_err(not_written)?;
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
                Err(NotTaken { reason: format!("the record {id} of this relink was already taken by a service, which holds it"), queued: None })
            };
        }
        Err(reason) => {
            debug(TAG, format!("the work item could not be written: {reason}").as_str());
            return Err(not_written(format!("the work item could not be written: {reason}")));
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
    // The record stays queued until the add's own run has finished: a record taken back first would leave nothing for a
    //  service to recover the repair from after a kill, and a record left queued only names a path the proof above
    //  confines to the worktree the record was written for.
    return Err(NotTaken { reason, queued: Some(QueuedRelink { work_dir, id }) });
}

/// The answer of a hand-over whose record was never written.
fn not_written(reason: String) -> NotTaken {
    return NotTaken { reason, queued: None };
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
///  What a hand-over that no service took leaves the add:
///  the words the person reads,
///  and the record still queued.
/// Why:
///   The add relinks the worktree itself,
/// but the record may only be taken back once that run has finished:
/// a record removed first leaves nothing for a service to recover the repair from after a kill,
/// and the worktree then keeps copied links that name the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NotTaken = { reason: string; queued?: QueuedRelink };
/// ```
#[derive(Debug)]
pub struct NotTaken {
    /// Why no service took the record.
    pub reason: String,
    /// The record still queued when one was written,
    /// taken back by the add after its own run.
    pub queued: Option<QueuedRelink>,
}

/// What:
///  A record still queued for a service,
///  which the add's own run makes superfluous when it ends.
/// Why:
///   Taking the record back is one unlink by its own id,
/// and the claim a service makes moves the file out of `pending` in one rename,
/// so this either wins and the claim finds nothing,
/// or loses and the service owns the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type QueuedRelink = { workDir: string; id: string };
/// ```
#[derive(Debug)]
pub struct QueuedRelink {
    /// The work directory the record sits in.
    work_dir: PathBuf,
    /// The record's id.
    id: String,
}

impl QueuedRelink {
    /// What:
    ///  Take the record back,
    ///  now that the add's own run has finished.
    /// Why:
    ///   The record owes nothing from here on,
    /// and a record that cannot be removed stays while the service may relink this worktree again,
    /// which is said here rather than failing the add:
    /// a worktree whose copied links were relinked twice is right either way.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const takeBack = ({ workDir, id }) => { try { unlink(pending(workDir, id)) } catch (error) { log(`the record ${id} stays: ${error.message}`) } };
    /// ```
    pub fn take_back(&self) {
        match std::fs::remove_file(pending_record(self.work_dir.as_path(), self.id.as_str()).as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                debug(TAG, format!("the record {} was taken by a service, whose run this add's own run stood beside", self.id).as_str())
            }
            Err(error) => debug(TAG, format!("the record {} stays ({error}), so the commit service may relink this worktree again", self.id).as_str()),
        }
    }
}

/// The pending record of one id.
fn pending_record(work_dir: &Path, id: &str) -> PathBuf {
    return state_directory(work_dir, PENDING).join(id);
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
pub(super) fn hand_over_in_a_job(state: TheHandOver) -> Result<(), NotTaken> {
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
