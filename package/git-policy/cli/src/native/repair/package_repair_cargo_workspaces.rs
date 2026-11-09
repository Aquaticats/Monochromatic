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

/// The request of one repair,
/// whose Git lists the tracked manifests.
use crate::package_repair_request::RepairRequest;
/// The tool runner and its result words.
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolLimits, ToolRun, run_tool};
/// Git runs for the tracked manifest listing.
use crate::transaction_git::{GitRequest, run_git};
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
///  The directories of the worktree's tracked `Cargo.toml` files.
/// Why:
///   Git knows which manifests belong to the checkout,
/// so no walk reads `node_modules` or build output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function manifestDirectories(git: GitContext, destination: string): Promise<string[]>;
/// ```
pub fn manifest_directories(request: &RepairRequest<'_>) -> Result<Vec<PathBuf>, String> {
    use std::os::unix::ffi::OsStrExt;
    // `--full-name` names every manifest from the top of the worktree, which is how each name is joined to the
    // destination; and the pattern is the wrapper's own pathspec, so no mode the caller set reaches it: under
    // literal mode it would match nothing, case-insensitive matching would widen it, and two glob modes at once
    // make Git refuse the listing.
    let listing: GitRequest =
        GitRequest::new(request.destination, &["ls-files", "-z", "--full-name", "--", "*Cargo.toml"]).own_pathspecs();
    let output = match run_git(request.git, &listing) {
        Ok(done) => done,
        Err(error) => return Err(format!("git could not list the tracked manifests ({error})")),
    };
    if !output.succeeded() {
        return Err(format!("git could not list the tracked manifests: {}", output.error_text()));
    }
    let mut directories: Vec<PathBuf> = Vec::new();
    for entry in output.stdout.split(is_nul_byte) {
        if entry.is_empty() {
            continue;
        }
        let path: PathBuf = request.destination.join(std::ffi::OsStr::from_bytes(entry));
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
    return Ok(directories);
}

/// What:
///  `cargo metadata` in a directory,
/// as a workspace.
/// Why:
///   `--no-deps` resolves nothing,
/// so it needs no network and writes no lock;
/// `--locked` makes that a promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readWorkspace(cargo: string, directory: string, limit: number): Promise<Workspace>;
/// ```
pub fn read_workspace(cargo: &Path, directory: &Path, limit: ToolLimits) -> Result<Workspace, String> {
    let arguments: Vec<OsString> = ["metadata", "--locked", "--offline", "--no-deps", "--format-version", "1"].iter().map(OsString::from).collect();
    let command: ToolCommand = ToolCommand { arguments, directory: PathBuf::from(directory), set: Vec::new() };
    let run: ToolRun = run_tool(cargo, &command, limit);
    return match run.end {
        ToolEnd::Succeeded if !run.notes.is_empty() => Err(format!("cargo metadata in {} wrote output that could not be taken whole ({}), so it was not read", directory.display(), run.notes.join("; "))),
        ToolEnd::Succeeded => parse_metadata(String::from_utf8_lossy(run.stdout.as_slice()).as_ref()),
        ToolEnd::Failed(reason) => Err(format!("cargo metadata in {} failed: {reason}", directory.display())),
        ToolEnd::TimedOut => Err(format!("cargo metadata in {} showed no sign of work for the quiet window and was stopped", directory.display())),
        ToolEnd::CeilingReached(ceiling) => Err(format!("cargo metadata in {} ran for {} seconds, the longest the wrapper lets one repair tool run, and was stopped with its whole process group", directory.display(), ceiling.as_secs())),
    };
}
