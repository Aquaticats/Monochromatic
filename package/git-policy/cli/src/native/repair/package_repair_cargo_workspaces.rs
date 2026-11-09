//! What:
//!  The Cargo workspaces of a new worktree:
//! what `cargo metadata --no-deps` says about one,
//! the tracked manifests that
//!  lead to them,
//! and the order and uniqueness of the workspaces found.
//! Why:
//!  `package_repair_cargo.rs` decides what each workspace's build state needs and stays under the file-size limit;
//! the
//!  discovery of workspaces needs nothing from it,
//! so it lives in a file of its own that the repair imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const workspaces = distinctWorkspaces(await Promise.all(manifestDirectories(request).map(readWorkspace)));
//! ```

/// The blocking budget and its refusal of a step,
/// for the reading that must never run on a task.
use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
/// The tool runner and its result words,
/// for the face that awaits and the one that enters the runtime.
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolLimits, ToolRun, run_tool_awaiting};
/// Entering the runtime,
/// for the synchronous face beside each awaited one.
use crate::lazy_runtime::run_on_runtime;
/// Git runs for the tracked manifest listing.
use crate::transaction_git::{GitContext, GitRequest};
/// The Git child awaited on the runtime,
/// for the repair's tasks.
use crate::transaction_git_awaiting::run_git_awaiting;
/// `OsString` is owned operating-system text.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// The manifest file that makes a directory a Cargo package or workspace root.
pub const MANIFEST: &str = "Cargo.toml";

/// Whether one byte is the NUL byte that ends each path of `git ls-files -z`.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Order two workspaces by their root directory.
/// `sort_by` takes this function by path;
/// the sort stays stable.
fn compare_by_root(left: &Workspace, right: &Workspace) -> std::cmp::Ordering {
    return left.root.cmp(&right.root);
}

/// Whether two workspaces have the same root.
/// `dedup_by` hands the later one first,
/// and drops it when this is true.
fn same_root(later: &mut Workspace, earlier: &mut Workspace) -> bool {
    return later.root == earlier.root;
}

/// One text field of the metadata as a path;
/// nothing when the field is missing or is not text.
fn text_field(metadata: &serde_json::Value, key: &str) -> Option<PathBuf> {
    return metadata.get(key).and_then(serde_json::Value::as_str).map(PathBuf::from);
}

/// What:
///  One Cargo workspace as `cargo metadata --no-deps` describes it.
/// Why:
///   The root identifies the workspace,
/// the two directories say where its state lives,
/// and the members are what gets cleaned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Workspace = { root: string; buildDirectory: string; targetDirectory: string; members: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    /// The directory holding the workspace's root manifest.
    pub root: PathBuf,
    /// Where intermediate build state lives.
    pub build_directory: PathBuf,
    /// Where final outputs live.
    pub target_directory: PathBuf,
    /// The package names of the workspace members.
    pub members: Vec<String>,
}

/// What:
///  Read the workspace out of `cargo metadata --no-deps --format-version 1` output.
/// Why:
///   Only these four facts are used,
/// and any other shape is an error that names what is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMetadata(text: string): Workspace; // throws
/// ```
pub fn parse_metadata(text: &str) -> Result<Workspace, String> {
    let metadata: serde_json::Value = match serde_json::from_str(text) {
        Ok(parsed) => parsed,
        Err(error) => return Err(format!("the output of cargo metadata is not JSON ({error})")),
    };
    let Some(root) = text_field(&metadata, "workspace_root") else {
        return Err(String::from("cargo metadata names no workspace_root"));
    };
    // A Cargo that predates `build-dir` keeps intermediate state in the target directory.
    let Some(target_directory) = text_field(&metadata, "target_directory") else {
        return Err(String::from("cargo metadata names no target_directory"));
    };
    let build_directory: PathBuf = match text_field(&metadata, "build_directory") {
        Some(directory) => directory,
        None => target_directory.clone(),
    };
    let Some(packages) = metadata.get("packages").and_then(serde_json::Value::as_array) else {
        return Err(String::from("cargo metadata names no packages"));
    };
    let mut members: Vec<String> = Vec::new();
    for package in packages {
        if let Some(name) = package.get("name").and_then(serde_json::Value::as_str) {
            members.push(name.to_owned());
        }
    }
    return Ok(Workspace { root, build_directory, target_directory, members });
}

/// What:
///  The workspaces once each,
/// in root order.
/// Why:
///   Cargo describes a workspace the same way from any of its members' manifests,
/// so several manifests of one workspace
///  must not be repaired several times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function distinctWorkspaces(found: Workspace[]): Workspace[];
/// ```
pub fn distinct_workspaces(mut found: Vec<Workspace>) -> Vec<Workspace> {
    found.sort_by(compare_by_root);
    found.dedup_by(same_root);
    return found;
}

/// What:
///  The directories of the tracked `Cargo.toml` files in one `git ls-files -z` listing.
/// Why:
///   The two faces of the listing differ only in how they wait for Git,
/// so the reading of its output is one code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directoriesOfTheListing(listing: Uint8Array, destination: string): string[];
/// ```
fn directories_of_the_listing(listing: &[u8], destination: &Path) -> Vec<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let mut directories: Vec<PathBuf> = Vec::new();
    for entry in listing.split(is_nul_byte) {
        if entry.is_empty() {
            continue;
        }
        let path: PathBuf = destination.join(std::ffi::OsStr::from_bytes(entry));
        let is_manifest: bool = match path.file_name() {
            Some(name) => name == MANIFEST,
            None => false,
        };
        if !(is_manifest && path.is_file()) {
            continue;
        }
        if let Some(parent) = path.parent() {
            directories.push(parent.to_path_buf());
        }
    }
    directories.sort();
    directories.dedup();
    return directories;
}

/// What:
///  `directories_of_the_listing` for the state a job of the blocking pool owns.
/// Why:
///  A job reads only what it was handed,
/// so the listing and the worktree travel together as owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directoriesInAJob([listing, destination]: [Uint8Array, string]): string[];
/// ```
fn directories_in_a_job((listing, destination): (Vec<u8>, PathBuf)) -> Vec<PathBuf> {
    return directories_of_the_listing(listing.as_slice(), destination.as_path());
}

/// What:
///  The directories of the tracked manifests,
/// as a job of the process's blocking pool.
/// Why:
///  Whether a listed path is a regular file is a storage question,
/// and the task that polls the three repairs must never ask it:
/// a filesystem that stopped answering would then stop the pnpm repair's quiet window and its ceiling with it.
/// A refusal from the system runs the same reading in turn,
/// under a place of the process's bound on blocking jobs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directories = await directoriesOfTheListingWithinTheBudget(listing, destination);
/// ```
async fn directories_of_the_listing_within_the_budget(listing: Vec<u8>, destination: &Path) -> Result<Vec<PathBuf>, BlockingJobError> {
    return run_a_job_or_in_turn((listing, destination.to_path_buf()), directories_in_a_job).await;
}

/// What:
///  The directories of the worktree's tracked `Cargo.toml` files,
/// with the Git child awaited on the runtime.
/// Why:
///   Git knows which manifests belong to the checkout,
/// so no walk reads `node_modules` or build output,
/// and a task of the
///  runtime cannot enter that runtime to run one (`RuntimeError::Nested`).
/// The question each listed path answers (is it a regular file)
///  is asked as a job of the blocking pool,
/// so no filesystem of the worktree can hold the task's other waits open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function manifestDirectoriesAwaiting(git: GitContext, destination: string): Promise<string[]>;
/// ```
pub async fn manifest_directories_awaiting(git: &GitContext, destination: &Path) -> Result<Vec<PathBuf>, String> {
    let listing = GitRequest::new(destination, &["ls-files", "-z", "--", "*Cargo.toml"]);
    let output = match run_git_awaiting(git, &listing).await {
        Ok(done) => done,
        Err(error) => return Err(format!("git could not list the tracked manifests ({error})")),
    };
    if !output.succeeded() {
        return Err(format!("git could not list the tracked manifests: {}", output.error_text()));
    }
    return match directories_of_the_listing_within_the_budget(output.stdout, destination).await {
        Ok(directories) => Ok(directories),
        // No directory was learned, and the reason is the person's to read.
        Err(refusal) => Err(format!("the tracked manifests could not be read ({refusal})")),
    };
}

/// What:
///  `cargo metadata` in a directory,
/// with the tool awaited on the runtime,
/// as a workspace.
/// Why:
///   `--no-deps` resolves nothing,
/// so it needs no network and writes no lock;
/// `--locked` makes that a promise,
///  and a task of the runtime awaits the child rather than entering the runtime to block on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readWorkspaceAwaiting(cargo: string, directory: string, limit: number): Promise<Workspace>;
/// ```
pub async fn read_workspace_awaiting(cargo: &Path, directory: &Path, limit: ToolLimits) -> Result<Workspace, String> {
    let arguments: Vec<OsString> = ["metadata", "--locked", "--offline", "--no-deps", "--format-version", "1"].iter().map(OsString::from).collect();
    let command: ToolCommand = ToolCommand { arguments, directory: PathBuf::from(directory), set: Vec::new() };
    let run: ToolRun = run_tool_awaiting(cargo, &command, limit).await;
    return workspace_of_the_run(run, directory);
}

/// What:
///  `read_workspace` for a caller that is not a task of the runtime.
/// Why:
///   The callers of this face are the controls and the synchronous repair,
/// and both faces must read a run the same way;
/// from inside the
///  runtime it refuses,
/// and `read_workspace_awaiting` is the face such a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readWorkspace(cargo: string, directory: string, limit: number): Workspace;
/// ```
pub fn read_workspace(cargo: &Path, directory: &Path, limit: ToolLimits) -> Result<Workspace, String> {
    return match run_on_runtime(read_workspace_awaiting(cargo, directory, limit)) {
        Ok(read) => read,
        Err(refusal) => Err(format!("cargo metadata in {} could not be run: {refusal}", directory.display())),
    };
}

/// What:
///  What a finished `cargo metadata` run says about the workspace of `directory`.
/// Why:
///   Both faces of the read make the same answer of the same run,
/// so neither can parse a run the other refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const workspaceOfTheRun = (run: ToolRun, directory: string) => { ... };
/// ```
fn workspace_of_the_run(run: ToolRun, directory: &Path) -> Result<Workspace, String> {
    return match run.end {
        ToolEnd::Succeeded if !run.notes.is_empty() => Err(format!("cargo metadata in {} wrote output that could not be taken whole ({}), so it was not read", directory.display(), run.notes.join("; "))),
        ToolEnd::Succeeded => parse_metadata(String::from_utf8_lossy(run.stdout.as_slice()).as_ref()),
        ToolEnd::Failed(reason) => Err(format!("cargo metadata in {} failed: {reason}", directory.display())),
        ToolEnd::TimedOut => Err(format!("cargo metadata in {} showed no sign of work for the quiet window and was stopped", directory.display())),
        ToolEnd::CeilingReached(ceiling) => Err(format!("cargo metadata in {} ran for {} seconds, the longest the wrapper lets one repair tool run, and was stopped with its whole process group", directory.display(), ceiling.as_secs())),
    };
}
