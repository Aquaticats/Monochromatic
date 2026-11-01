//! What:
//!  The durable request to copy ignored state into one new worktree: a small file in the repository's copy directory
//!  that the wrapper writes after real Git returned and the resident service runs and removes.
//! Why:
//!  The copy is the service's work, so a service that is killed in the middle of a copy and a copy that failed both leave
//!  the same thing behind: the request. The next wake of the service runs it again, from the start, and no process has to
//!  prove that another one is dead. A wrapper that is killed after Git created the worktree and before it wrote the
//!  request leaves an intent instead (`worktree_copy_intent.rs`), from which the service writes the request itself
//!  (`doc/worktree-copy-service.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeRequest(commonDir, request); const pending = await readRequests(commonDir); await removeRequest(commonDir, id);
//! ```

use super::json_record::quote;
use super::private_storage::{create_private_directory_all, sync_directory};
use super::random_id::random_uuid;
use super::worktree_copy_error::{WorktreeCopyError, raw_failure};
use super::worktree_copy_journal::{
    TEMPORARY_SUFFIX, ensure_journal_root, journal_root, remove_file_if_present, write_atomic, write_atomic_unsynced,
};
use super::worktree_copy_selection::{CopySelection, STAGE_PREFIX, assert_safe_repository_path, js_order};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The directory of pending requests under the journal root.
pub const REQUESTS_NAME: &str = "requests";

/// The directory of requests given up on under the journal root.
pub const GIVEN_UP_NAME: &str = "given-up";

/// The suffix of a request file.
const REQUEST_SUFFIX: &str = ".json";

/// How many runs a request gets before the service gives up on it and tells the person.
pub const MAX_TRIES: u32 = 3;

/// The directory of finished requests under the journal root: the durable answer for a client that asks after the fact.
pub const DONE_NAME: &str = "done";

/// How many finished-request records are kept; the oldest go first. A record only answers a client that was not told.
pub const KEPT_DONE_RECORDS: usize = 256;

/// The version of the request text.
const REQUEST_VERSION: u64 = 2;

/// The text of a selection, inside a record.
pub fn encode_selection(selection: &CopySelection) -> String {
    return match selection {
        CopySelection::Ignored => String::from("{\"kind\":\"ignored\"}"),
        CopySelection::Paths(paths) => {
            let quoted: Vec<String> = paths.iter().map(|path| return quote(path.as_str())).collect();
            format!("{{\"kind\":\"paths\",\"paths\":[{}]}}", quoted.join(","))
        }
    };
}

/// Reads a selection from a record, refusing a path that could reach outside the repository.
pub fn decode_selection(value: &serde_json::Value) -> Result<CopySelection, String> {
    return match value.get("kind").and_then(serde_json::Value::as_str) {
        Some("ignored") => Ok(CopySelection::Ignored),
        Some("paths") => {
            let mut paths: Vec<String> = Vec::new();
            for item in value.get("paths").and_then(serde_json::Value::as_array).ok_or("paths is missing")? {
                let path: &str = item.as_str().ok_or("a path is not text")?;
                assert_safe_repository_path(path).map_err(|failure| return failure.0)?;
                paths.push(path.to_string());
            }
            Ok(CopySelection::Paths(paths))
        }
        _ => Err(String::from("selection is unknown")),
    };
}

/// What:
///  One request: which registration to fill, from where, and how often it was tried.
/// Why:
///   The service needs nothing from the wrapper that wrote it: the registration's identity is what tells a worktree from
///  a later one of the same name, and the source and the selection are what the wrapper decided.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyRequest = { id; adminId; registration; destinationRoot; sourceRoot; selection; tries };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyRequest {
    /// Time-ordered unique name: milliseconds since the epoch in hexadecimal, a hyphen, a random identifier.
    pub id: String,
    /// The administrative name of the registration to fill.
    pub admin_id: String,
    /// The identity of that registration, from `worktrees/<name>/cli-git-registration`.
    pub registration: String,
    /// The canonical root of the new worktree.
    pub destination_root: PathBuf,
    /// The canonical root of the worktree the ignored state comes from.
    pub source_root: PathBuf,
    /// Which of the source's ignored paths are copied.
    pub selection: CopySelection,
    /// Runs that failed so far.
    pub tries: u32,
    /// The identity of the administrative directory when the wrapper observed it (`worktree_copy_identity.rs`).
    pub identity: String,
    /// Whether the worktree may be locked when it is filled: its creator asked for `--lock`.
    pub allow_locked: bool,
}

impl CopyRequest {
    /// The stage container this request's copy uses: beside the destination, named after the request, so a service
    ///  that restarts finds and removes what an interrupted run of it left.
    pub fn stage_container(&self) -> PathBuf {
        let parent: &Path = self.destination_root.parent().unwrap_or(self.destination_root.as_path());
        return parent.join(format!("{STAGE_PREFIX}{}", self.id));
    }

    /// The request's text: sorted keys, one line.
    pub fn encode(&self) -> String {
        let selection: String = encode_selection(&self.selection);
        return format!(
            "{{\"adminId\":{},\"allowLocked\":{},\"destinationRoot\":{},\"id\":{},\"identity\":{},\"registration\":{},\"selection\":{selection},\"sourceRoot\":{},\"tries\":{},\"version\":{REQUEST_VERSION}}}\n",
            quote(self.admin_id.as_str()),
            self.allow_locked,
            quote(self.destination_root.to_string_lossy().as_ref()),
            quote(self.id.as_str()),
            quote(self.identity.as_str()),
            quote(self.registration.as_str()),
            quote(self.source_root.to_string_lossy().as_ref()),
            self.tries
        );
    }

    /// Reads a request, or says why the text is not one this release wrote.
    pub fn decode(text: &str) -> Result<CopyRequest, String> {
        let value: serde_json::Value = serde_json::from_str(text).map_err(|error| return format!("not JSON ({error})"))?;
        let text_field = |name: &str| -> Result<String, String> {
            return value
                .get(name)
                .and_then(serde_json::Value::as_str)
                .map(String::from)
                .ok_or_else(|| return format!("{name} is missing or not text"));
        };
        if value.get("version").and_then(serde_json::Value::as_u64) != Some(REQUEST_VERSION) {
            return Err(String::from("unsupported version"));
        }
        let selection: CopySelection = decode_selection(value.get("selection").ok_or("selection is missing")?)?;
        let request: CopyRequest = CopyRequest {
            id: text_field("id")?,
            admin_id: text_field("adminId")?,
            registration: text_field("registration")?,
            destination_root: PathBuf::from(text_field("destinationRoot")?),
            source_root: PathBuf::from(text_field("sourceRoot")?),
            selection,
            identity: text_field("identity")?,
            allow_locked: value
                .get("allowLocked")
                .and_then(serde_json::Value::as_bool)
                .ok_or("allowLocked is missing")?,
            tries: value
                .get("tries")
                .and_then(serde_json::Value::as_u64)
                .and_then(|count| return u32::try_from(count).ok())
                .ok_or("tries is missing")?,
        };
        let plain = |name: &str| -> bool {
            return !name.is_empty() && !name.contains(['/', '\0']) && name != "." && name != "..";
        };
        let well_formed: bool = plain(request.id.as_str())
            && request.id.bytes().all(|byte| return byte.is_ascii_alphanumeric() || byte == b'-')
            && plain(request.admin_id.as_str())
            && !request.registration.is_empty()
            && !request.identity.is_empty()
            && request.destination_root.is_absolute()
            && request.source_root.is_absolute()
            && !request.destination_root.to_string_lossy().contains('\0')
            && !request.source_root.to_string_lossy().contains('\0');
        if !well_formed {
            return Err(String::from("a name or a path is not acceptable"));
        }
        return Ok(request);
    }
}

/// What:
///  A new request identity that sorts by creation time.
/// Why:
///   The service runs requests in the order they were written, which is the order the worktrees were created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${Date.now().toString(16).padStart(16, '0')}-${randomUUID()}`
/// ```
pub fn new_request_id() -> Result<String, WorktreeCopyError> {
    let milliseconds: u128 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |elapsed| return elapsed.as_millis());
    let unique: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    return Ok(format!("{milliseconds:016x}-{unique}"));
}

/// The directory of pending requests of a repository.
pub fn requests_directory(common_dir: &Path) -> PathBuf {
    return journal_root(common_dir).join(REQUESTS_NAME);
}

/// The directory of given-up requests of a repository.
pub fn given_up_directory(common_dir: &Path) -> PathBuf {
    return journal_root(common_dir).join(GIVEN_UP_NAME);
}

/// What:
///  Writes a request whole and durably, replacing an earlier version of the same request.
/// Why:
///   A reader sees the old request or the new one, never a fragment, and a request that was written is not lost to a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFileAtomically(join(requestsDirectory, `${id}.json`), request.encode());
/// ```
pub fn write_request(common_dir: &Path, request: &CopyRequest) -> Result<(), WorktreeCopyError> {
    ensure_journal_root(common_dir)?;
    let directory: PathBuf = requests_directory(common_dir);
    create_private_directory_all(directory.as_path()).map_err(|error| return raw_failure(&error))?;
    let published: PathBuf = directory.join(format!("{}{REQUEST_SUFFIX}", request.id));
    let written: Result<(), WorktreeCopyError> = write_atomic(published.as_path(), request.encode().as_bytes(), "request");
    // The service adopts a complete temporary it finds, so a rename that finds its temporary gone is fine when the request
    // stands in its place with the very same text.
    if written.is_err() && std::fs::read_to_string(published.as_path()).is_ok_and(|text| return text == request.encode()) {
        return Ok(());
    }
    return written;
}

/// What:
///  Reads the pending requests of a repository in the order they were written, and moves the ones that do not parse to
///  the given-up directory with the reason.
/// Why:
///   One damaged file must not stop every copy of the repository, and a request that vanishes between the listing and
///  the read was finished by a run that overlapped this one, so it is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { requests, refused } = await readRequests(commonDir);
/// ```
pub fn read_requests(common_dir: &Path) -> Result<(Vec<CopyRequest>, Vec<String>), WorktreeCopyError> {
    return read_requests_with(common_dir, true);
}

/// Reads the pending requests without setting aside the ones that cannot be read, for lookups that only look.
pub fn read_requests_unfiled(common_dir: &Path) -> Result<(Vec<CopyRequest>, Vec<String>), WorktreeCopyError> {
    return read_requests_with(common_dir, false);
}

fn read_requests_with(common_dir: &Path, set_aside: bool) -> Result<(Vec<CopyRequest>, Vec<String>), WorktreeCopyError> {
    let directory: PathBuf = requests_directory(common_dir);
    // `match` treats an absent directory as having no requests, and every other failure as fatal.
    let listing = match std::fs::read_dir(directory.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), Vec::new())),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed.map_err(|error| return raw_failure(&error))?.file_name().to_string_lossy().into_owned();
        if name.ends_with(REQUEST_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(|left, right| return js_order(left, right));
    let mut requests: Vec<CopyRequest> = Vec::new();
    let mut refused: Vec<String> = Vec::new();
    for name in names {
        let path: PathBuf = directory.join(name.as_str());
        let text: String = match std::fs::read_to_string(path.as_path()) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(raw_failure(&error)),
        };
        match CopyRequest::decode(text.as_str()) {
            Ok(request) if format!("{}{REQUEST_SUFFIX}", request.id) == name => requests.push(request),
            Ok(_) if set_aside => refused.push(refuse(common_dir, path.as_path(), "its name is not its identity")?),
            Err(reason) if set_aside => refused.push(refuse(common_dir, path.as_path(), reason.as_str())?),
            Ok(_) | Err(_) => {}
        }
    }
    return Ok((requests, refused));
}

/// Moves a request file that cannot be run to the given-up directory and says which file and why.
fn refuse(common_dir: &Path, path: &Path, reason: &str) -> Result<String, WorktreeCopyError> {
    let given_up: PathBuf = given_up_directory(common_dir);
    create_private_directory_all(given_up.as_path()).map_err(|error| return raw_failure(&error))?;
    let name: String = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
    std::fs::rename(path, given_up.join(name.as_str())).map_err(|error| return raw_failure(&error))?;
    return Ok(format!(
        "cli-git: set aside the copy request {} because {reason}; it is kept in {}.",
        quote(name.as_str()),
        quote(given_up.to_string_lossy().as_ref())
    ));
}

/// What:
///  Removes a finished request and makes the removal durable.
/// Why:
///   A request is removed only after its copy is marked done, so a crash before this point runs the request again and
///  finds it done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(join(requestsDirectory, `${id}.json`), { force: true });
/// ```
pub fn remove_request(common_dir: &Path, request: &CopyRequest) -> Result<(), WorktreeCopyError> {
    let directory: PathBuf = requests_directory(common_dir);
    remove_file_if_present(directory.join(format!("{}{REQUEST_SUFFIX}", request.id)).as_path())?;
    return sync_directory(directory.as_path()).map_err(|error| return raw_failure(&error));
}

/// What:
///  Sets a request aside for good: its record, with the tries it took, is written into the given-up directory and only
///  then removed from the pending ones.
/// Why:
///   A service killed between the two steps finds the request still pending and runs it once more; a service killed after
///  them finds the durable record, so a client that asks later is told the copy was given up on and never that it settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeAtomic(givenUp/id, tried.encode()); await removeRequest(commonDir, tried);
/// ```
pub fn give_up(common_dir: &Path, request: &CopyRequest) -> Result<(), WorktreeCopyError> {
    let given_up: PathBuf = given_up_directory(common_dir);
    create_private_directory_all(given_up.as_path()).map_err(|error| return raw_failure(&error))?;
    write_atomic(
        given_up.join(format!("{}{REQUEST_SUFFIX}", request.id)).as_path(),
        request.encode().as_bytes(),
        "given-up request",
    )?;
    return remove_request(common_dir, request);
}

/// How a request that ended well came to be over.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoneKind {
    /// The copy ran and created this many entries.
    Copied(usize),
    /// The registration was marked copied already.
    Already,
    /// The worktree was removed or replaced, so nothing was created.
    Removed,
}

/// How a request ended for good, as the service's disk says it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Terminal {
    /// It ended well.
    Done(DoneKind),
    /// It was given up on; the record is the request as it stood.
    GivenUp(CopyRequest),
    /// It is still pending.
    Pending,
    /// Nothing on disk knows this identity.
    Unknown,
}

fn encode_done(kind: DoneKind) -> String {
    return match kind {
        DoneKind::Copied(count) => format!("{{\"entries\":{count},\"outcome\":\"copied\"}}\n"),
        DoneKind::Already => String::from("{\"outcome\":\"already\"}\n"),
        DoneKind::Removed => String::from("{\"outcome\":\"removed\"}\n"),
    };
}

fn decode_done(text: &str) -> Option<DoneKind> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    return match value.get("outcome")?.as_str()? {
        "copied" => Some(DoneKind::Copied(usize::try_from(value.get("entries")?.as_u64()?).ok()?)),
        "already" => Some(DoneKind::Already),
        "removed" => Some(DoneKind::Removed),
        _ => None,
    };
}

/// What:
///  Ends a request that ended well: its outcome is written into the done directory and only then is the request removed.
/// Why:
///   Absence from the pending directory is not an answer; this record is, so a service that is killed after the removal
///  and asked again still knows how the copy ended. The oldest records beyond a bound are dropped: a record only answers a
///  client that was not told.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeAtomic(done/id, outcome); await removeRequest(commonDir, request); await pruneDone(commonDir);
/// ```
pub fn finish_request(common_dir: &Path, request: &CopyRequest, kind: DoneKind) -> Result<(), WorktreeCopyError> {
    let done: PathBuf = done_directory(common_dir);
    create_private_directory_all(done.as_path()).map_err(|error| return raw_failure(&error))?;
    // Unsynchronized: it answers a client that was not told, and a power failure that loses it costs that client a failure
    // it can retry; a killed process leaves it whole.
    write_atomic_unsynced(done.join(format!("{}{REQUEST_SUFFIX}", request.id)).as_path(), encode_done(kind).as_bytes(), "finished request")?;
    remove_request(common_dir, request)?;
    return prune_done(done.as_path(), KEPT_DONE_RECORDS);
}

/// The directory of finished requests.
pub fn done_directory(common_dir: &Path) -> PathBuf {
    return journal_root(common_dir).join(DONE_NAME);
}

/// Drops the oldest finished-request records beyond `keep`.
pub fn prune_done(done: &Path, keep: usize) -> Result<(), WorktreeCopyError> {
    let mut names: Vec<String> = std::fs::read_dir(done)
        .map_err(|error| return raw_failure(&error))?
        .flatten()
        .map(|entry| return entry.file_name().to_string_lossy().into_owned())
        .filter(|name| return name.ends_with(REQUEST_SUFFIX))
        .collect();
    if names.len() <= keep {
        return Ok(());
    }
    names.sort_by(|left, right| return js_order(left, right));
    let excess: usize = names.len() - keep;
    for name in names.into_iter().take(excess) {
        remove_file_if_present(done.join(name).as_path())?;
    }
    return Ok(());
}

/// What:
///  How the request with this identity ended, from the disk alone.
/// Why:
///   The answer to a client that connects after the run that finished its request, or after the service was killed and
///  started again: pending first, then given up, then finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await terminalOf(commonDir, id)
/// ```
pub fn terminal_of(common_dir: &Path, id: &str) -> Result<Terminal, WorktreeCopyError> {
    let name: String = format!("{id}{REQUEST_SUFFIX}");
    let read = |path: PathBuf| -> Result<Option<String>, WorktreeCopyError> {
        return match std::fs::read_to_string(path.as_path()) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(raw_failure(&error)),
        };
    };
    if read(requests_directory(common_dir).join(name.as_str()))?.is_some() {
        return Ok(Terminal::Pending);
    }
    if let Some(text) = read(given_up_directory(common_dir).join(name.as_str()))? {
        return Ok(CopyRequest::decode(text.as_str()).map_or(Terminal::Unknown, Terminal::GivenUp));
    }
    if let Some(text) = read(done_directory(common_dir).join(name.as_str()))? {
        return Ok(decode_done(text.as_str()).map_or(Terminal::Unknown, Terminal::Done));
    }
    return Ok(Terminal::Unknown);
}

/// The identity of a pending request for this registration, if the wrapper or a recovery already wrote one.
pub fn request_for_registration(
    common_dir: &Path,
    admin_id: &str,
    registration: &str,
) -> Result<Option<String>, WorktreeCopyError> {
    let (requests, _refused) = read_requests_unfiled(common_dir)?;
    return Ok(requests
        .into_iter()
        .find(|request| return request.admin_id == admin_id && request.registration == registration)
        .map(|request| return request.id));
}

/// Whether any request of the repository is pending.
pub fn requests_exist(common_dir: &Path) -> Result<bool, WorktreeCopyError> {
    // `match` treats an absent directory as having no requests, and every other failure as fatal.
    return match std::fs::read_dir(requests_directory(common_dir)) {
        Ok(mut listing) => Ok(listing.any(|listed| {
            return listed.is_ok_and(|entry| return entry.file_name().to_string_lossy().ends_with(REQUEST_SUFFIX));
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(raw_failure(&error)),
    };
}

/// What:
///  Whether a copy is pending in the repository: a request to run, or a journal a killed service left.
/// Why:
///   A command that creates no worktree nudges the service only when this is true, so the common case reads one
///  directory and one more, and a copy in progress is not a reason to start anything: the nudge is harmless to repeat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await requestsExist(commonDir)) || (await journalsExist(commonDir))
/// ```
pub fn copy_pending(common_dir: &Path) -> Result<bool, WorktreeCopyError> {
    if requests_exist(common_dir)? || super::worktree_copy_intent::intents_exist(common_dir)? {
        return Ok(true);
    }
    // `match` treats an absent journal root as having no journals, and every other failure as fatal.
    return match std::fs::read_dir(journal_root(common_dir)) {
        Ok(mut listing) => Ok(listing.any(|listed| {
            return listed.is_ok_and(|entry| return entry.file_name().to_string_lossy().ends_with(".json"));
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(raw_failure(&error)),
    };
}

/// What:
///  Adopts the complete temporaries of request writes that stand without their request.
/// Why:
///   A writer that was suspended or killed between its synchronized write and its rename leaves a complete request under a
///  temporary name. Deleting it by age could delete a writer that is only slow; publishing it, which is what the writer
///  was about to do, cannot. A temporary that does not parse is left alone: its writer is mid-write or was killed, and the
///  intent record the wrapper wrote before Git ran is what recovers the request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const temporary of temporaries) if (parses(temporary) && !exists(final)) await rename(temporary, final);
/// ```
pub fn adopt_temporaries(common_dir: &Path) -> Result<usize, WorktreeCopyError> {
    let directory: PathBuf = requests_directory(common_dir);
    // `match` treats an absent directory as having nothing to adopt, and every other failure as fatal.
    let listing = match std::fs::read_dir(directory.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut adopted: usize = 0;
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(published) = name.strip_suffix(TEMPORARY_SUFFIX).and_then(|stem| return stem.split_once(REQUEST_SUFFIX)) else {
            continue;
        };
        let id: &str = published.0;
        let Ok(text) = std::fs::read_to_string(entry.path()) else { continue };
        let complete: bool = CopyRequest::decode(text.as_str()).is_ok_and(|request| return request.id == id);
        let final_path: PathBuf = directory.join(format!("{id}{REQUEST_SUFFIX}"));
        let ended: bool = !matches!(terminal_of(common_dir, id)?, Terminal::Unknown);
        if complete && !ended {
            // A request that stands already is the newer word of its writer; the temporary is not published over it.
            if std::fs::symlink_metadata(final_path.as_path()).is_err() {
                std::fs::rename(entry.path(), final_path.as_path()).map_err(|error| return raw_failure(&error))?;
                adopted += 1;
            }
        }
    }
    if adopted > 0 {
        sync_directory(directory.as_path()).map_err(|error| return raw_failure(&error))?;
    }
    return Ok(adopted);
}

#[cfg(test)]
#[path = "worktree_copy_request_tests.rs"]
mod tests;
