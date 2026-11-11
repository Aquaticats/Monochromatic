//! What:
//!  Hand the filling of a new worktree's Cargo build directories to the commit service instead of
//!  waiting for it inside the add.
//! Why:
//!  Cargo keys its build directory by the workspace's path,
//! so a new worktree starts with none of the dependency builds its source checkout already
//!  has.
//! The repair therefore cloned the source's build directory and then cleaned the worktree's own
//!  crates out of the copy.
//! That clone is the heaviest step of the repair:
//! a traced `git worktree add` on 2026-11-10 spent 111 s of its 160 s repair phase inside one
//!  workspace's clone (`strace -f` of the installed wrapper,
//! `doc/planning/cli-git-worktree-add-package-repair.md`).
//! The clone only ever fills a directory the worktree does not have yet,
// so it can happen after the add returns:
//! the record names each workspace's source directory,
//! its destination and its members,
//! and the service clones into a staging directory,
//! cleans the staging directory,
//! and publishes it under the destination's name with a rename that never replaces.
//! A worktree whose first build got there first keeps its own build directory,
//! and the staging directory is dropped:
//! the prewarm only ever adds state a build would otherwise compile.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! handOverPrewarm({ context, commonDir, destination, entries, notices }); // enqueue one `prewarm` item, wake the service
//! ```

use crate::diagnostic_log::debug;
use crate::hooks_path_values::{path_answer, without_the_terminator};
use crate::iso_time::now_iso;
use crate::service_client::{START_WAIT, wake};
use crate::service_commit_input_notes_and_embedded_commit::managed_client_config;
use crate::service_messages::{ClientFailure, render};
use crate::service_provision::provision;
use crate::service_route::route_from_environment;
use crate::transaction_git::{GitContext, GitRequest};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::service_work::work_directory;
use crate::work_item::{WorkItem, WorkKind, enqueue};
use crate::worktree_trash_hand_over::Reach;

use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairPrewarm";

/// The reference a prewarm record carries.
/// It names no branch:
/// the record is about a directory,
/// and a Git reference is only what the queue's record format asks every kind for.
pub const PREWARM_REFERENCE: &str = "refs/monochromatic/cargo-prewarm";

/// The environment variable that points a `cargo clean` at a build directory other than the one the
///  workspace's configuration resolves.
/// `doc/decision/cargo-build-dir-host-cache.md` records that a literal path given this way is used
///  verbatim.
pub const BUILD_DIRECTORY_VARIABLE: &str = "CARGO_BUILD_BUILD_DIR";

/// The suffix the service clones into beside a workspace's build directory.
pub const STAGING_SUFFIX: &str = ".monochromatic-prewarm";

/// The environment variable that names the target directory a `cargo clean` deletes from,
/// so the filling deletes nothing from the worktree's own.
pub const TARGET_DIRECTORY_VARIABLE: &str = "CARGO_TARGET_DIR";

/// What:
///  One workspace whose build directory the service fills:
/// the workspace to ask Cargo about,
/// the source checkout's build directory,
/// the new worktree's build directory,
/// and the members whose own artifacts must not be carried.
/// Why:
///   The four facts are what the add already read from `cargo metadata`,
/// so the service runs no metadata read of its own,
/// and the record says exactly what the add would have done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PrewarmEntry = { root: string; from: string; to: string; members: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrewarmEntry {
    /// The workspace root in the new worktree,
    /// where the clean runs.
    pub root: PathBuf,
    /// The source workspace's build directory,
    /// which holds the dependency builds worth keeping.
    pub from: PathBuf,
    /// The new worktree's build directory,
    /// which does not exist yet.
    pub to: PathBuf,
    /// The package names the clean removes from the copy.
    pub members: Vec<String>,
}

/// What:
///  One text field of a prewarm record's payload,
/// or nothing when it is missing or is not text.
/// Why:
///   A record written by another release,
/// or damaged,
/// must be refused rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const textField = (value, name) => typeof value?.[name] === 'string' ? value[name] : undefined;
/// ```
fn text_field(entry: &serde_json::Value, name: &str) -> Option<PathBuf> {
    return entry.get(name).and_then(serde_json::Value::as_str).map(PathBuf::from);
}

/// What:
///  The entries as the one text a record can carry.
/// Why:
///   The queue's record holds one text field,
/// so the entries travel as one JSON array,
/// and every path is refused rather than written lossily.
///
/// # Errors
///
/// A path that is not UTF-8,
/// which no record could name back.
pub fn encode(entries: &[PrewarmEntry]) -> Result<String, String> {
    let mut written: Vec<serde_json::Value> = Vec::with_capacity(entries.len());
    for entry in entries {
        let (Some(root), Some(from), Some(to)) = (entry.root.to_str(), entry.from.to_str(), entry.to.to_str()) else {
            return Err(String::from("a path of the prewarm is not valid UTF-8, so no record can name it"));
        };
        let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        for (name, value) in [("root", root), ("from", from), ("to", to)] {
            fields.insert(String::from(name), serde_json::Value::from(value));
        }
        fields.insert(String::from("members"), serde_json::Value::from(entry.members.clone()));
        written.push(serde_json::Value::Object(fields));
    }
    return Ok(serde_json::Value::Array(written).to_string());
}

/// What:
///  The entries a prewarm record carries.
/// Why:
///   A payload that is not the JSON array `encode` writes is refused,
/// and so is an entry missing a field or carrying no member:
/// there would be nothing to clean,
/// and cleaning nothing would leave the source's own artifacts in the copy.
///
/// # Errors
///
/// Text that is not the array,
/// and an entry whose fields do not say where and what to clean.
pub fn decode(payload: &str) -> Result<Vec<PrewarmEntry>, String> {
    let parsed: serde_json::Value = match serde_json::from_str(payload) {
        Ok(value) => value,
        Err(error) => return Err(format!("the prewarm payload is not JSON ({error})")),
    };
    let Some(list) = parsed.as_array() else {
        return Err(String::from("the prewarm payload is not a list of workspaces"));
    };
    let mut entries: Vec<PrewarmEntry> = Vec::with_capacity(list.len());
    for value in list {
        let (Some(root), Some(from), Some(to)) = (text_field(value, "root"), text_field(value, "from"), text_field(value, "to")) else {
            return Err(String::from("a prewarm entry names no workspace, no source directory or no destination"));
        };
        let Some(members) = value.get("members").and_then(serde_json::Value::as_array) else {
            return Err(String::from("a prewarm entry names no members"));
        };
        let names: Vec<String> = members.iter().filter_map(serde_json::Value::as_str).map(String::from).collect();
        if names.len() != members.len() || names.is_empty() {
            return Err(String::from("a prewarm entry names no members"));
        }
        entries.push(PrewarmEntry { root, from, to, members: names });
    }
    if entries.is_empty() {
        return Err(String::from("the prewarm payload names no workspaces"));
    }
    return Ok(entries);
}

/// What:
///  The record's object name for one hand-over:
/// a digest of the worktree's path and the moment the add asked for the filling.
/// Why:
///   A work item's id is its idempotency key,
/// and the queue answers an id it already knows as known in every state.
/// With the path alone,
/// a worktree removed and added again at the same path would be answered by the earlier record,
/// and a filling that failed could never be asked for again.
/// The moment tells two hand-overs of one path apart,
/// and the path keeps two worktrees apart.
/// Forty hexadecimal digits is what a work item's object name must be,
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
///  The record a worktree's prewarm is queued as.
/// Why:
///   The producer and its controls both write exactly this,
/// so a control can compare a record without reaching a service.
/// The stamp is read once and is both the record's `created` and part of its object name,
/// so a control can tell which record it is looking at from the record alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itemOf = ({ context, commonDir, destination, entries }) => { const created = nowISO(); return { kind: 'prewarm', repository: destination, gitDir: commonDir, ref: PREWARM_REFERENCE, oid: objectNameOf(destination, created), parent: encode(entries), created, tries: 0 }; };
/// ```
pub fn item_of(common_dir: &Path, destination: &Path, entries: &[PrewarmEntry]) -> Result<WorkItem, String> {
    let created: String = now_iso();
    return Ok(WorkItem {
        kind: WorkKind::Prewarm,
        repository: destination.to_path_buf(),
        git_dir: common_dir.to_path_buf(),
        branch_ref: String::from(PREWARM_REFERENCE),
        oid: object_name_of(destination, created.as_str()),
        parent: encode(entries)?,
        created,
        tries: 0,
    });
}

/// What:
///  Write the prewarm's record and wake the service that runs it.
/// Why:
///   The record is written first,
/// so a wake that cannot reach the service leaves the work named rather than lost,
/// and the add never waits for the filling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverToTheService({ context, commonDir, destination, entries });
/// ```
pub fn hand_over_to_the_service(context: &GitContext, common_dir: &Path, destination: &Path, entries: &[PrewarmEntry]) -> Result<(), String> {
    return hand_over_with(context, common_dir, destination, entries, &Reach { router: &route_from_environment, provisioner: &provision, start_wait: START_WAIT });
}

/// What:
///  `hand_over_to_the_service` with the route finder and the provisioner injected,
/// so a control reaches no service and no session bus.
/// Why:
///   The same reach a trash is handed over with,
/// so a prewarm is queued and woken exactly as that work is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handOverWith({ context, commonDir, destination, entries, reach });
/// ```
pub fn hand_over_with(context: &GitContext, common_dir: &Path, destination: &Path, entries: &[PrewarmEntry], reach: &Reach<'_>) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }
    let item: WorkItem = item_of(common_dir, destination, entries)?;
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
    // The wake's lines are not the add's: it says once what became of the filling.
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
///  The repository's common Git directory,
/// as Git names it for a worktree,
/// with the child awaited on the runtime.
/// Why:
///  The prewarm's record lives where the repository's service drains its work,
/// and `is_acceptable` refuses a record whose Git directory is not the one it was read from,
/// so the common directory is asked of Git rather than assembled from paths.
/// `--path-format=absolute` answers an absolute path even when Git's answer is relative.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commonDirectoryAwaiting(git, destination): Promise<string>;
/// ```
pub async fn common_directory_awaiting(git: &GitContext, destination: &Path) -> Result<PathBuf, String> {
    let request: GitRequest = GitRequest::new(destination, &["rev-parse", "--path-format=absolute", "--git-common-dir"]);
    let output = match run_git_awaiting(git, &request).await {
        Ok(done) => done,
        Err(error) => return Err(format!("git could not name the repository's common directory ({error})")),
    };
    if !output.succeeded() {
        return Err(format!("git could not name the repository's common directory: {}", output.error_text()));
    }
    // Git ends the path with one line feed; every other byte is the path's own, so only that terminator goes.
    //  A directory whose name ends in a space is that directory, and the record is queued there.
    let named: &[u8] = without_the_terminator(output.stdout.as_slice());
    return if named.is_empty() {
        Err(String::from("git named no common directory for the worktree"))
    } else {
        // The answer is a pathname, and a pathname is bytes: decoding it replaced a byte that is not UTF-8.
        Ok(path_answer(named))
    };
}

/// What:
///  The hand-over of a prewarm record as a job of the process's blocking pool.
/// Why:
///  Writing a record and waking a service that may have to be started are storage and socket waits,
/// and the task that polls the three repairs must never do either:
/// a service that never answered would stop the pnpm repair's quiet window and its ceiling with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handOverInAJob = ([context, commonDir, destination, entries]) => handOverToTheService(context, commonDir, destination, entries);
/// ```
pub(super) fn hand_over_in_a_job(state: TheHandOver) -> Result<(), String> {
    let (context, common_dir, destination, entries): TheHandOver = state;
    return hand_over_to_the_service(&context, common_dir.as_path(), destination.as_path(), entries.as_slice());
}

/// What a hand-over job reads:
/// the Git context,
/// the repository's common directory,
/// the new worktree,
/// and the workspaces.
/// Why:
///   A job reads only what it was handed,
/// so the four travel together as owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheHandOver = [GitContext, string, string, PrewarmEntry[]];
/// ```
pub(super) type TheHandOver = (GitContext, PathBuf, PathBuf, Vec<PrewarmEntry>);

/// Prewarm controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_prewarm_tests.rs"]
mod tests;
