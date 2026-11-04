//! What:
//!  The Cargo repair: for each Cargo workspace of a new worktree, clone the source workspace's build state when Cargo keeps
//!  it outside the worktree, then `cargo clean --package <member>` for every member so only the worktree's own crates
//!  rebuild.
//! Why:
//!  Cargo bakes absolute paths into what it builds (`CARGO_MANIFEST_DIR`, build script output), so build state copied from
//!  another location is wrong for the worktree's own crates, but registry and git dependencies hold nothing path-specific
//!  and can be reused. The owner chose this on 2026-11-03: "Reuse deps, rebuild own", done with "Run cargo clean -p".
//!  Where `build-dir` is outside the worktree (the person's `~/.cargo/config.toml` keys it by workspace path) a copy of
//!  ignored files does not carry it, so it is cloned; under Cargo's default layout it is inside `target/`, which is already
//!  copied. `cargo metadata` names both directories, so no configuration is read by the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const findings = await repairCargo(request); // one per workspace that failed, one for the rest that worked
//! ```

use crate::add_timing::work;
use crate::diagnostic_log::debug;
use crate::package_repair::RepairRequest;
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolRun, find_program, run_tool};
use crate::package_repair_report::{Finding, Outcome, Tool, missing_tool};
use crate::package_repair_tree_clone::clone_tree;
use crate::shell_quote::shell_quote;
use crate::transaction_git::{GitRequest, run_git};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairCargo";

/// The program the person's environment resolves.
const PROGRAM: &str = "cargo";

/// The manifest file that makes a directory a Cargo package or workspace root.
const MANIFEST: &str = "Cargo.toml";

/// The lock file beside a workspace's root manifest.
const LOCK_FILE: &str = "Cargo.lock";

/// What:
///  One Cargo workspace as `cargo metadata --no-deps` describes it.
/// Why:
///   The root identifies the workspace, the two directories say where its state lives, and the members are what gets cleaned.
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
///  What a workspace's build state needs in the new worktree.
/// Why:
///   The decision is pure so each layout is tested without Cargo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BuildStatePlan = { kind: 'nothing'; reason: string } | { kind: 'clean-only' } | { kind: 'clone-then-clean'; from: string; to: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuildStatePlan {
    /// Nothing is cloned or cleaned; the text says why, for the log.
    Nothing(String),
    /// The state is already in the worktree (copied with `target/`): only the members are cleaned.
    CleanOnly,
    /// The source's external build directory is cloned to the worktree's, then the members are cleaned.
    CloneThenClean {
        /// The source workspace's build directory.
        from: PathBuf,
        /// The worktree's build directory, which does not exist yet.
        to: PathBuf,
    },
}

/// What:
///  Facts about the disk that decide a plan.
/// Why:
///   They are read once, so the planner stays pure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StateFacts = { sourceBuild?: string; sourceBuildExists: boolean; destinationBuildExists: boolean; destinationTargetExists: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateFacts {
    /// The build directory of the source's counterpart workspace; none when the source has no such workspace.
    pub source_build: Option<PathBuf>,
    /// Whether that directory exists.
    pub source_build_exists: bool,
    /// Whether the worktree's build directory exists.
    pub destination_build_exists: bool,
    /// Whether the worktree's target directory exists.
    pub destination_target_exists: bool,
}

/// What:
///  Read the workspace out of `cargo metadata --no-deps --format-version 1` output.
/// Why:
///   Only these four facts are used, and any other shape is an error that names what is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMetadata(text: string): Workspace; // throws
/// ```
pub fn parse_metadata(text: &str) -> Result<Workspace, String> {
    let metadata: serde_json::Value = serde_json::from_str(text).map_err(|error| { return format!("the output of cargo metadata is not JSON ({error})"); })?;
    let text_field = |key: &str| -> Option<PathBuf> { return metadata.get(key).and_then(serde_json::Value::as_str).map(PathBuf::from); };
    let root: PathBuf = text_field("workspace_root").ok_or_else(|| { return String::from("cargo metadata names no workspace_root"); })?;
    // A Cargo that predates `build-dir` keeps intermediate state in the target directory.
    let target_directory: PathBuf = text_field("target_directory").ok_or_else(|| { return String::from("cargo metadata names no target_directory"); })?;
    let build_directory: PathBuf = text_field("build_directory").unwrap_or_else(|| { return target_directory.clone(); });
    let members: Vec<String> = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .map(|packages: &Vec<serde_json::Value>| { return packages.iter().filter_map(|package: &serde_json::Value| { return package.get("name").and_then(serde_json::Value::as_str).map(str::to_owned); }).collect(); })
        .ok_or_else(|| { return String::from("cargo metadata names no packages"); })?;
    return Ok(Workspace { root, build_directory, target_directory, members });
}

/// What:
///  The workspaces once each, in root order.
/// Why:
///   Cargo describes a workspace the same way from any of its members' manifests, so several manifests of one workspace
///  must not be repaired several times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function distinctWorkspaces(found: Workspace[]): Workspace[];
/// ```
pub fn distinct_workspaces(mut found: Vec<Workspace>) -> Vec<Workspace> {
    found.sort_by(|left: &Workspace, right: &Workspace| { return left.root.cmp(&right.root); });
    found.dedup_by(|later: &mut Workspace, earlier: &mut Workspace| { return later.root == earlier.root; });
    return found;
}

/// What:
///  Decide what the worktree's build state needs.
/// Why:
///   Reusing state that is shared with the source, or cleaning where there is none, would do harm or nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planBuildState(destination: Workspace, destinationRoot: string, facts: StateFacts): BuildStatePlan;
/// ```
pub fn plan_build_state(destination: &Workspace, destination_root: &Path, facts: &StateFacts) -> BuildStatePlan {
    let Some(source_build) = facts.source_build.as_ref() else {
        return BuildStatePlan::Nothing(String::from("the source has no such workspace, so no state was carried"));
    };
    if *source_build == destination.build_directory {
        return BuildStatePlan::Nothing(String::from("the source and the worktree share one build directory, which a clean would damage"));
    }
    if destination.build_directory.starts_with(destination_root) {
        return if facts.destination_build_exists || facts.destination_target_exists {
            BuildStatePlan::CleanOnly
        } else {
            BuildStatePlan::Nothing(String::from("the build directory is inside the worktree and was not carried"))
        };
    }
    if facts.destination_build_exists {
        return BuildStatePlan::Nothing(String::from("the worktree's build directory already holds state of its own"));
    }
    if facts.source_build_exists {
        return BuildStatePlan::CloneThenClean { from: source_build.clone(), to: destination.build_directory.clone() };
    }
    return if facts.destination_target_exists { BuildStatePlan::CleanOnly } else { BuildStatePlan::Nothing(String::from("neither side holds build state")) };
}

/// What:
///  The arguments of the clean that removes exactly the members' own artifacts.
/// Why:
///   `--offline` keeps the clean from reaching the network; one `--package` per member keeps registry and git dependencies.
///  `--locked` is not used: a committed `Cargo.lock` that no longer matches its manifest (several do here) would make the clean
///  fail, and the person's own clean would not; the file is put back as checked out instead (`restore_lock`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanArguments(members: string[]): string[];
/// ```
pub fn clean_arguments(members: &[String]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = ["clean", "--offline"].iter().map(OsString::from).collect();
    for member in members {
        arguments.push(OsString::from("--package"));
        arguments.push(OsString::from(member));
    }
    return arguments;
}

/// What:
///  The command a person runs in a workspace to do what the repair could not.
/// Why:
///   A printed line must work as pasted, so the directory change is part of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function manualCommand(root: string, members: string[]): string;
/// ```
fn manual_command(root: &Path, members: &[String]) -> String {
    let words: Vec<String> = clean_arguments(members).iter().map(|word: &OsString| { return shell_quote(word.as_encoded_bytes()); }).collect();
    return format!("cd {} && {PROGRAM} {}", shell_quote(root.as_os_str().as_encoded_bytes()), words.join(" "));
}

/// What:
///  The directories of the worktree's tracked `Cargo.toml` files.
/// Why:
///   Git knows which manifests belong to the checkout, so no walk reads `node_modules` or build output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function manifestDirectories(git: GitContext, destination: string): Promise<string[]>;
/// ```
fn manifest_directories(request: &RepairRequest<'_>) -> Result<Vec<PathBuf>, String> {
    use std::os::unix::ffi::OsStrExt;
    let listing = GitRequest::new(request.destination, &["ls-files", "-z", "--", "*Cargo.toml"]);
    let output = run_git(request.git, &listing).map_err(|error| { return format!("git could not list the tracked manifests ({error})"); })?;
    if !output.succeeded() {
        return Err(format!("git could not list the tracked manifests: {}", output.error_text()));
    }
    let mut directories: Vec<PathBuf> = output
        .stdout
        .split(|byte: &u8| { return *byte == 0; })
        .filter(|entry: &&[u8]| { return !entry.is_empty(); })
        .map(|entry: &[u8]| { return request.destination.join(std::ffi::OsStr::from_bytes(entry)); })
        .filter(|path: &PathBuf| { return path.file_name().is_some_and(|name| { return name == MANIFEST; }) && path.is_file(); })
        .filter_map(|path: PathBuf| { return path.parent().map(Path::to_path_buf); })
        .collect();
    directories.sort();
    directories.dedup();
    return Ok(directories);
}

/// What:
///  `cargo metadata` in a directory, as a workspace.
/// Why:
///   `--no-deps` resolves nothing, so it needs no network and writes no lock; `--locked` makes that a promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readWorkspace(cargo: string, directory: string, limit: number): Promise<Workspace>;
/// ```
fn read_workspace(cargo: &Path, directory: &Path, limit: std::time::Duration) -> Result<Workspace, String> {
    let arguments: Vec<OsString> = ["metadata", "--locked", "--offline", "--no-deps", "--format-version", "1"].iter().map(OsString::from).collect();
    let command: ToolCommand = ToolCommand { arguments, directory: PathBuf::from(directory), set: Vec::new() };
    let run: ToolRun = run_tool(cargo, &command, limit);
    return match run.end {
        ToolEnd::Succeeded => parse_metadata(String::from_utf8_lossy(run.stdout.as_slice()).as_ref()),
        ToolEnd::Failed(reason) => Err(format!("cargo metadata in {} failed: {reason}", directory.display())),
        ToolEnd::TimedOut => Err(format!("cargo metadata in {} showed no sign of work for the quiet window and was stopped", directory.display())),
    };
}

/// What:
///  The bytes of the workspace's `Cargo.lock`, or none when it has none.
/// Why:
///   `cargo clean --package` resolves the workspace, and Cargo rewrites a lock that no longer matches its manifest; the
///  new worktree must stay as checked out, so the file is put back afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = readLock(root); // Uint8Array | undefined
/// ```
fn read_lock(root: &Path) -> Option<Vec<u8>> {
    return std::fs::read(root.join(LOCK_FILE)).ok();
}

/// What:
///  Put `Cargo.lock` back as `before` read it, removing a lock that did not exist before.
/// Why:
///   A write is made only when the content changed, so an untouched lock keeps its file time. A lock that cannot be put back
///  is logged, and left for the person: it is the worktree's own file and the checkout shows the difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// restoreLock(root, before);
/// ```
fn restore_lock(root: &Path, before: Option<&[u8]>) {
    let path: PathBuf = root.join(LOCK_FILE);
    let after: Option<Vec<u8>> = std::fs::read(&path).ok();
    if after.as_deref() == before {
        return;
    }
    let outcome: std::io::Result<()> = match before {
        Some(bytes) => std::fs::write(&path, bytes),
        None => std::fs::remove_file(&path),
    };
    match outcome {
        Ok(()) => debug(TAG, format!("{} was put back as checked out after the clean", path.display()).as_str()),
        Err(error) => debug(TAG, format!("{} changed during the clean and could not be put back ({error})", path.display()).as_str()),
    }
}

/// What a repair of one workspace did.
enum Done {
    /// Nothing was needed; the text says why.
    Skipped,
    /// The members were cleaned, and the build directory was cloned first when `cloned` is set.
    Cleaned {
        /// Whether a build directory was cloned.
        cloned: bool,
        /// How many entries of the source's build directory could not be read and were left out of the clone.
        left_out: usize,
    },
    /// The workspace could not be repaired.
    Failed(Finding),
}

/// What:
///  Repair one workspace: plan, clone, clean.
/// Why:
///   Each workspace's state is in a directory of its own, so workspaces never touch each other's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairWorkspace(cargo: string, workspace: Workspace, request: RepairRequest, cloneWorkers: number): Promise<Done>;
/// ```
fn repair_workspace(cargo: &Path, workspace: &Workspace, destination_root: &Path, request: &RepairRequest<'_>, clone_workers: usize) -> Done {
    let manual: String = manual_command(workspace.root.as_path(), workspace.members.as_slice());
    let failed = |reason: String| -> Done { return Done::Failed(Finding { tool: Tool::Cargo, outcome: Outcome::Failed { reason, run_instead: manual.clone() } }); };
    let Ok(relative) = workspace.root.strip_prefix(destination_root) else {
        debug(TAG, format!("{} lies outside {} and is left alone", workspace.root.display(), destination_root.display()).as_str());
        return Done::Skipped;
    };
    let source_workspace: PathBuf = request.source.join(relative);
    let source_build: Option<PathBuf> = if source_workspace.join(MANIFEST).is_file() {
        match work("cargoMetadataSource", || { return read_workspace(cargo, source_workspace.as_path(), request.environment.quiet_window); }) {
            Ok(found) => Some(found.build_directory),
            Err(reason) => {
                debug(TAG, format!("the source workspace {} cannot be read: {reason}", source_workspace.display()).as_str());
                None
            }
        }
    } else {
        None
    };
    let facts: StateFacts = StateFacts {
        source_build_exists: source_build.as_ref().is_some_and(|path: &PathBuf| { return path.exists(); }),
        source_build,
        destination_build_exists: workspace.build_directory.exists(),
        destination_target_exists: workspace.target_directory.exists(),
    };
    let plan: BuildStatePlan = plan_build_state(workspace, destination_root, &facts);
    debug(TAG, format!("{}: {plan:?}", workspace.root.display()).as_str());
    let (cloned, left_out): (bool, usize) = match &plan {
        BuildStatePlan::Nothing(_) => return Done::Skipped,
        BuildStatePlan::CleanOnly => (false, 0),
        BuildStatePlan::CloneThenClean { from, to } => {
            match work("cargoCloneTree", || { return clone_tree(from.as_path(), to.as_path(), clone_workers); }) {
                Ok(made) => {
                    debug(TAG, format!("cloned {} entries from {} to {}", made.made, from.display(), to.display()).as_str());
                    for path in &made.left_out {
                        debug(TAG, format!("{} could not be read and is not in the clone", from.join(path).display()).as_str());
                    }
                    (true, made.left_out.len())
                }
                Err(reason) => return failed(format!("the build state could not be cloned: {reason}")),
            }
        }
    };
    let command: ToolCommand = ToolCommand { arguments: clean_arguments(workspace.members.as_slice()), directory: workspace.root.clone(), set: Vec::new() };
    let lock_before: Option<Vec<u8>> = read_lock(workspace.root.as_path());
    let run: ToolRun = work("cargoClean", || { return run_tool(cargo, &command, request.environment.quiet_window); });
    restore_lock(workspace.root.as_path(), lock_before.as_deref());
    return match run.end {
        ToolEnd::Succeeded => Done::Cleaned { cloned, left_out },
        ToolEnd::Failed(reason) => failed(reason),
        ToolEnd::TimedOut => failed(String::from("it showed no sign of work for the quiet window and was stopped")),
    };
}

/// What:
///  Run `work` for every item on up to `workers` threads, each item once, one result per item in item order; an item whose work
///  panicked has an error that names it and says what the panic said.
/// Why:
///   The repairs of different workspaces share no state, so a queue of items keeps every thread busy. Each item is run inside its
///  own catch, so a panic costs that item and never the results its thread had already produced or the items it had still to take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const results = await mapConcurrent(items, workers, label, work); // Array<{ ok: R } | { error: string }>
/// ```
fn map_concurrent<T: Sync, R: Send>(items: &[T], workers: usize, label: &(dyn Fn(&T) -> String + Sync), work: &(dyn Fn(&T) -> R + Sync)) -> Vec<Result<R, String>> {
    let next: AtomicUsize = AtomicUsize::new(0);
    let mut results: Vec<(usize, Result<R, String>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers.max(1).min(items.len().max(1)))
            .map(|_worker: usize| {
                return scope.spawn(|| {
                    let mut done: Vec<(usize, Result<R, String>)> = Vec::new();
                    loop {
                        let index: usize = next.fetch_add(1, Ordering::Relaxed);
                        let Some(item) = items.get(index) else { break };
                        let outcome: Result<R, String> = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { return work(item); }))
                            .map_err(|payload: Box<dyn std::any::Any + Send>| { return format!("{} stopped because its worker panicked: {}", label(item), panic_words(payload.as_ref())); });
                        done.push((index, outcome));
                    }
                    return done;
                });
            })
            .collect();
        // A worker cannot panic outside the catch around each item, so a thread that did not return its list is a bug to raise.
        return handles.into_iter().flat_map(|handle| { return handle.join().unwrap_or_else(|payload| { std::panic::resume_unwind(payload) }); }).collect();
    });
    results.sort_by_key(|(index, _result): &(usize, Result<R, String>)| { return *index; });
    return results.into_iter().map(|(_index, result): (usize, Result<R, String>)| { return result; }).collect();
}

/// What a panic said: its text when it carried a string, else a plain statement that it carried none.
fn panic_words(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        return (*text).to_string();
    }
    if let Some(text) = payload.downcast_ref::<String>() {
        return text.clone();
    }
    return String::from("no message");
}

/// What:
///  The words that say how many entries of the cloned build state could not be read, or nothing when all could.
/// Why:
///   The person is told what the clone left out: the entries are the source's, unreadable to everyone but a chmod the repair
///  never performs, and Cargo rebuilds whatever of them it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leftOutWords(2) // '; 2 unreadable entries were left out'
/// ```
fn left_out_words(left_out: usize) -> String {
    return match left_out {
        0 => String::new(),
        1 => String::from("; 1 unreadable entry was left out"),
        many => format!("; {many} unreadable entries were left out"),
    };
}

/// What:
///  Repair every Cargo workspace of the new worktree.
/// Why:
///   Workspaces are independent (each has its own build directory), so they are repaired side by side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairCargo(request: RepairRequest): Promise<Finding[]>;
/// ```
pub fn repair_cargo(request: &RepairRequest<'_>) -> Vec<Finding> {
    let started: Instant = Instant::now();
    let directories: Vec<PathBuf> = match manifest_directories(request) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("{}: {reason}", request.destination.display()).as_str());
            return Vec::new();
        }
    };
    if directories.is_empty() {
        return Vec::new();
    }
    let Some(cargo) = find_program(PROGRAM, request.environment.path()) else {
        return vec![missing_tool(Tool::Cargo, PROGRAM, "cargo clean --offline --package <member> in each Cargo workspace of the worktree")];
    };
    let workers: usize = request.environment.workers;
    let read: Vec<Result<Workspace, String>> = map_concurrent(directories.as_slice(), workers, &|directory: &PathBuf| { return format!("reading the Cargo workspace at {}", directory.display()); }, &|directory: &PathBuf| { return work("cargoMetadataWorktree", || { return read_workspace(cargo.as_path(), directory.as_path(), request.environment.quiet_window); }); })
        .into_iter()
        .map(|outcome: Result<Result<Workspace, String>, String>| { return outcome.and_then(|inner: Result<Workspace, String>| { return inner; }); })
        .collect();
    let mut findings: Vec<Finding> = Vec::new();
    let mut found: Vec<Workspace> = Vec::new();
    for (directory, outcome) in directories.iter().zip(read) {
        match outcome {
            Ok(workspace) => found.push(workspace),
            Err(reason) => findings.push(Finding { tool: Tool::Cargo, outcome: Outcome::Failed { reason, run_instead: manual_command(directory.as_path(), &[String::from("<member>")]) } }),
        }
    }
    let workspaces: Vec<Workspace> = distinct_workspaces(found);
    // Real paths: Cargo names the workspace by its real path, and the worktree may be reached through a link.
    let destination_root: PathBuf = std::fs::canonicalize(request.destination).unwrap_or_else(|_error| { return request.destination.to_path_buf(); });
    let clone_workers: usize = if workspaces.len() > 1 { 1 } else { workers };
    let done: Vec<Done> = map_concurrent(workspaces.as_slice(), workers, &|workspace: &Workspace| { return format!("the repair of the Cargo workspace at {}", workspace.root.display()); }, &|workspace: &Workspace| { return work("cargoRepairWorkspace", || { return repair_workspace(cargo.as_path(), workspace, destination_root.as_path(), request, clone_workers); }); })
        .into_iter()
        .zip(workspaces.iter())
        .map(|(outcome, workspace): (Result<Done, String>, &Workspace)| {
            return outcome.unwrap_or_else(|reason: String| { return Done::Failed(Finding { tool: Tool::Cargo, outcome: Outcome::Failed { reason, run_instead: manual_command(workspace.root.as_path(), &[String::from("<member>")]) } }); });
        })
        .collect();
    let mut cleaned: usize = 0;
    let mut cloned: usize = 0;
    let mut left_out: usize = 0;
    for outcome in done {
        match outcome {
            Done::Skipped => {}
            Done::Cleaned { cloned: was_cloned, left_out: unreadable } => {
                cleaned += 1;
                cloned += usize::from(was_cloned);
                left_out += unreadable;
            }
            Done::Failed(finding) => findings.push(finding),
        }
    }
    if cleaned > 0 {
        let workspace_noun: &str = if cleaned == 1 { "workspace" } else { "workspaces" };
        let directory_noun: &str = if cloned == 1 { "build directory" } else { "build directories" };
        findings.push(Finding { tool: Tool::Cargo, outcome: Outcome::Repaired { detail: format!("Cargo kept dependency builds and cleaned the worktree's own crates in {cleaned} {workspace_noun} ({cloned} {directory_noun} cloned{})", left_out_words(left_out)), millis: started.elapsed().as_millis() } });
    }
    return findings;
}

/// Cargo repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_cargo_tests.rs"]
mod tests;
