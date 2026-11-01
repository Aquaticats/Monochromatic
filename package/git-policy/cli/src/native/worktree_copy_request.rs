//! What:
//!  The durable request to copy ignored state into one new worktree: a small file in the repository's copy directory
//!  that the wrapper writes after real Git returned and the resident service runs and removes.
//! Why:
//!  The copy is the service's work, so a wrapper that is killed after Git created the worktree, a service that is killed
//!  in the middle of a copy, and a copy that failed all leave the same thing behind: the request. The next wake of the
//!  service runs it again, from the start, so no copy is ever lost for lack of someone to remember it, and no process has
//!  to prove that another one is dead (`doc/worktree-copy-claim.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeRequest(commonDir, request); const pending = await readRequests(commonDir); await removeRequest(commonDir, id);
//! ```

use super::json_record::quote;
use super::private_storage::{create_private_directory_all, sync_directory};
use super::random_id::random_uuid;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_journal::{
    TEMPORARY_SUFFIX, ensure_journal_root, journal_root, remove_file_if_present, write_atomic,
};
use super::worktree_copy_selection::{CopySelection, STAGE_PREFIX, assert_safe_repository_path, js_order};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The directory of pending requests under the journal root.
pub const REQUESTS_NAME: &str = "requests";

/// The directory of requests given up on under the journal root.
pub const GIVEN_UP_NAME: &str = "given-up";

/// The suffix of a request file.
const REQUEST_SUFFIX: &str = ".json";

/// How many runs a request gets before the service gives up on it and tells the person.
pub const MAX_TRIES: u32 = 3;

/// How old a leftover temporary of a request write must be before the service removes it.
///  A write takes milliseconds, so a temporary this old belongs to a writer that was killed.
pub const STALE_TEMPORARY: Duration = Duration::from_secs(3600);

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
        let selection: String = match &self.selection {
            CopySelection::Ignored => String::from("{\"kind\":\"ignored\"}"),
            CopySelection::Paths(paths) => {
                let quoted: Vec<String> = paths.iter().map(|path| return quote(path.as_str())).collect();
                format!("{{\"kind\":\"paths\",\"paths\":[{}]}}", quoted.join(","))
            }
        };
        return format!(
            "{{\"adminId\":{},\"destinationRoot\":{},\"id\":{},\"registration\":{},\"selection\":{selection},\"sourceRoot\":{},\"tries\":{},\"version\":1}}\n",
            quote(self.admin_id.as_str()),
            quote(self.destination_root.to_string_lossy().as_ref()),
            quote(self.id.as_str()),
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
        if value.get("version").and_then(serde_json::Value::as_u64) != Some(1) {
            return Err(String::from("unsupported version"));
        }
        let selection_value = value.get("selection").ok_or("selection is missing")?;
        let selection: CopySelection = match selection_value.get("kind").and_then(serde_json::Value::as_str) {
            Some("ignored") => CopySelection::Ignored,
            Some("paths") => {
                let mut paths: Vec<String> = Vec::new();
                for item in selection_value.get("paths").and_then(serde_json::Value::as_array).ok_or("paths is missing")? {
                    let path: &str = item.as_str().ok_or("a path is not text")?;
                    assert_safe_repository_path(path).map_err(|failure| return failure.0)?;
                    paths.push(path.to_string());
                }
                CopySelection::Paths(paths)
            }
            _ => return Err(String::from("selection is unknown")),
        };
        let request: CopyRequest = CopyRequest {
            id: text_field("id")?,
            admin_id: text_field("adminId")?,
            registration: text_field("registration")?,
            destination_root: PathBuf::from(text_field("destinationRoot")?),
            source_root: PathBuf::from(text_field("sourceRoot")?),
            selection,
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
    return write_atomic(
        directory.join(format!("{}{REQUEST_SUFFIX}", request.id)).as_path(),
        request.encode().as_bytes(),
        "request",
    );
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
            Ok(_) => refused.push(refuse(common_dir, path.as_path(), "its name is not its identity")?),
            Err(reason) => refused.push(refuse(common_dir, path.as_path(), reason.as_str())?),
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
///  Moves a request that failed `MAX_TRIES` times to the given-up directory.
/// Why:
///   A copy that can never succeed (a destination entry that differs from the source's) must not run on every wake.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rename(join(requestsDirectory, `${id}.json`), join(givenUpDirectory, `${id}.json`));
/// ```
pub fn give_up(common_dir: &Path, request: &CopyRequest) -> Result<(), WorktreeCopyError> {
    let given_up: PathBuf = given_up_directory(common_dir);
    create_private_directory_all(given_up.as_path()).map_err(|error| return raw_failure(&error))?;
    let name: String = format!("{}{REQUEST_SUFFIX}", request.id);
    std::fs::rename(requests_directory(common_dir).join(name.as_str()), given_up.join(name.as_str()))
        .map_err(|error| return raw_failure(&error))?;
    return Ok(());
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
    if requests_exist(common_dir)? {
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
///  Removes the temporaries that killed writers left in the requests directory.
/// Why:
///   A write takes milliseconds and is renamed into place, so a temporary an hour old will never be; nothing else reads
///  these files, so no process has to be judged alive or dead to remove them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await readdir(dir)) if (name.endsWith('.tmp') && olderThan(name, hour)) await rm(join(dir, name));
/// ```
pub fn remove_stale_temporaries(common_dir: &Path, now: SystemTime) -> Result<usize, WorktreeCopyError> {
    let directory: PathBuf = requests_directory(common_dir);
    // `match` treats an absent directory as having nothing to remove, and every other failure as fatal.
    let listing = match std::fs::read_dir(directory.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut removed: usize = 0;
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
        if !entry.file_name().to_string_lossy().ends_with(TEMPORARY_SUFFIX) {
            continue;
        }
        let modified: SystemTime = entry
            .metadata()
            .and_then(|metadata| return metadata.modified())
            .map_err(|error| return failure_with_cause("cli-git: could not examine a copy request temporary.", Some(&error)))?;
        if now.duration_since(modified).is_ok_and(|age| return age > STALE_TEMPORARY) {
            remove_file_if_present(entry.path().as_path())?;
            removed += 1;
        }
    }
    return Ok(removed);
}

/// Request controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_request_tests.rs"]
mod tests;
