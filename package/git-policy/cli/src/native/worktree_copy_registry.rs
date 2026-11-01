//! What:
//!  The linked worktrees a worktree-creating Git command registered:
//!  the administrative identities before real Git ran,
//!  the identities after it returned,
//!  and the canonical root of each new one.
//! Why:
//!  A worktree that Git registered before failing still receives the ignored files,
//!  so the comparison runs after Git returns whatever its status was.
//!  The main worktree is observed only when the caller names repository paths to copy from it
//!  (`observe_copy_source`), so its other invocations cost no filesystem reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const observation = await observeWorktreeRepository({ args, gitPath });
//! const { created, registeredRoots } = await findCreatedWorktrees({ observation, gitPath });
//! ```

use super::git_metadata::{path_from_git_bytes, strip_git_line};
use super::private_storage::{sync_directory, write_exclusive_synced};
pub use super::worktree_copy_identity::{admin_identity, ensure_registration_token};
use super::random_id::random_uuid;
use super::transaction_git::{GitContext, GitRequest, run_git};
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_selection::CopySelection;
use super::worktree_identity::WorktreeIdentity;
use crate::json_record::quote;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// What:
///  The Git-file prefix that introduces a linked worktree's administrative path.
/// Why:
///   A `.git` file that does not start with it marks the main worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GITDIR_PREFIX = 'gitdir: ';
/// ```
const GITDIR_PREFIX: &str = "gitdir: ";

/// What:
///  The file inside a worktree's administrative directory that Git uses to lock it.
/// Why:
///   Git writes it with the text `initializing` while it creates a worktree and removes it when the checkout is done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOCKED_NAME = 'locked';
/// ```
const LOCKED_NAME: &str = "locked";

/// What:
///  The text Git puts in `locked` while it creates a worktree (English; a localized Git writes a translation).
/// Why:
///   A worktree that still says so belongs to the Git run that creates it, and no other copy may fill it yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INITIALIZING_TEXT = 'initializing';
/// ```
const INITIALIZING_TEXT: &str = "initializing";

/// What:
///  The file inside a worktree's administrative directory that holds the identity of its registration.
/// Why:
///   Git deletes the directory, identity included, when it removes or prunes the worktree, and a later worktree of the
///  same name starts without one, so the identity says which registration a copy was made for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REGISTRATION_NAME = 'cli-git-registration';
/// ```
pub const REGISTRATION_NAME: &str = "cli-git-registration";

/// What:
///  The marker file a finished copy leaves inside the worktree's administrative directory.
/// Why:
///   Git deletes the directory, marker included, when it removes or prunes the worktree, so the marker lasts exactly as
///  long as the registration it describes and an overlapping command that arrives later skips the destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const COPIED_MARKER_NAME = 'cli-git-copied';
/// ```
pub const COPIED_MARKER_NAME: &str = "cli-git-copied";

/// What:
///  The facts captured before real Git runs for an applicable invocation.
/// Why:
///   The comparison after Git needs the administrative directory, the identities that
///  already existed and the roots that decide where copies land.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyObservation = { adminRoot; beforeAdminIds; commonDir; effectiveCwd; sourceRoot? };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeCopyObservation {
    /// Common-directory location holding linked-worktree administration.
    pub admin_root: PathBuf,
    /// Linked-worktree administrative identities present before real Git ran.
    pub before_admin_ids: BTreeSet<String>,
    /// Canonical common Git directory, which owns the journals.
    pub common_dir: PathBuf,
    /// Working directory the Git queries run in.
    pub effective_cwd: PathBuf,
    /// Canonical source worktree root;
    ///  absent for a bare repository.
    pub source_root: Option<PathBuf>,
    /// Which of the source's ignored paths the copy takes.
    pub selection: CopySelection,
}

/// What:
///  One linked worktree registered while real Git ran.
/// Why:
///   The administrative identity is stable across renames of the destination path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CreatedWorktree = { adminId: string; root: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedWorktree {
    /// Basename of the linked administrative directory.
    pub admin_id: String,
    /// Canonical created worktree root.
    pub root: PathBuf,
    /// Whether Git has a lock file on the registration: it is still being created, or it was locked by its creator.
    pub locked: bool,
    /// The identity of the administrative directory when it was observed (`worktree_copy_identity.rs`).
    pub identity: String,
}

/// What:
///  The created worktrees and every registered root that recursive copying must exclude.
/// Why:
///   A nested registered worktree is never copied into its parent's copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { created: CreatedWorktree[]; registeredRoots: string[] }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedWorktrees {
    /// Newly registered linked worktrees, sorted by administrative identity.
    pub created: Vec<CreatedWorktree>,
    /// Every registered root that exists on the filesystem.
    pub registered_roots: Vec<PathBuf>,
}

/// What:
///  The linked worktrees Git currently lists, keyed by administrative identity.
/// Why:
///   The after-Git lookup maps each new identity to its root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { rootsByAdminId: Map<string, string>; roots: string[] }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct RegisteredWorktrees {
    /// Root of each linked worktree by administrative identity.
    roots_by_admin_id: BTreeMap<String, PathBuf>,
    /// Every existing registered root.
    roots: Vec<PathBuf>,
}

/// What:
///  The administrative directory names under a common worktree-administration root.
/// Why:
///   A missing directory means no linked worktree has ever been registered,
///  so it is an empty set rather than a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readAdminIds(adminRoot: string): Promise<ReadonlySet<string>>;
/// ```
pub fn read_admin_ids(admin_root: &Path) -> Result<BTreeSet<String>, WorktreeCopyError> {
    // `match` separates a missing directory (empty) from every other failure (fatal).
    match std::fs::read_dir(admin_root) {
        Ok(entries) => {
            let mut names: BTreeSet<String> = BTreeSet::new();
            for entry in entries {
                let listing: std::fs::DirEntry =
                    entry.map_err(|error| return inspect_failure(admin_root, &error))?;
                // `file_type` does not follow symbolic links, as the incumbent's `Dirent` does not.
                let is_directory: bool = listing
                    .file_type()
                    .map_err(|error| return inspect_failure(admin_root, &error))?
                    .is_dir();
                if is_directory {
                    names.insert(listing.file_name().to_string_lossy().into_owned());
                }
            }
            return Ok(names);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BTreeSet::new());
        }
        Err(error) => {
            return Err(inspect_failure(admin_root, &error));
        }
    }
}

/// What:
///  The failure of reading linked-worktree administration.
/// Why:
///   Names the directory so the user can see which repository state could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`cli-git: could not inspect linked-worktree administration at ${JSON.stringify(adminRoot)}.`, error)
/// ```
fn inspect_failure(admin_root: &Path, error: &std::io::Error) -> WorktreeCopyError {
    let message: String = format!(
        "cli-git: could not inspect linked-worktree administration at {}.",
        quote(admin_root.to_string_lossy().as_ref())
    );
    return failure_with_cause(message.as_str(), Some(error));
}

/// What:
///  Captures what the post-Git comparison needs for one repository identity.
/// Why:
///   The main worktree and a repository outside any worktree return `None` before any read,
///  so their invocations stay on the plain forwarding path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function observeWorktreeRepository({ args, gitPath }): Promise<WorktreeCopyObservation | typeof WORKTREE_COPY_NOT_APPLICABLE>;
/// ```
pub fn observe_worktree_repository(
    identity: &WorktreeIdentity,
    effective_cwd: &Path,
) -> Result<Option<WorktreeCopyObservation>, WorktreeCopyError> {
    // `match` decides applicability from the identity alone; only the two repository kinds reach a read.
    match identity {
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::MainWorktree { .. } => {
            return Ok(None);
        }
        WorktreeIdentity::BareRepository { common_dir, .. } => {
            return observation(
                common_dir.as_path(),
                None,
                effective_cwd,
                CopySelection::Ignored,
            )
            .map(Some);
        }
        WorktreeIdentity::LinkedWorktree {
            common_dir,
            worktree_root,
            ..
        } => {
            return observation(
                common_dir.as_path(),
                Some(worktree_root.as_path()),
                effective_cwd,
                CopySelection::Ignored,
            )
            .map(Some);
        }
    }
}

/// What:
///  The observation that also covers a main checkout, when the caller names repository paths to copy from it.
/// Why:
///   A main checkout is not a bypass any more for the named paths (the generated rules file a fresh worktree needs),
///  but it is never copied as a whole: its ignored trees are the repository's heaviest.
///  With no named paths the main checkout stays unobserved, so its invocations cost no filesystem reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// observeCopySource({ identity, effectiveCwd, mainCheckoutPaths }): Promise<WorktreeCopyObservation | typeof WORKTREE_COPY_NOT_APPLICABLE>;
/// ```
pub fn observe_copy_source(
    identity: &WorktreeIdentity,
    effective_cwd: &Path,
    main_checkout_paths: &[String],
) -> Result<Option<WorktreeCopyObservation>, WorktreeCopyError> {
    // `match` with a guard: only a main checkout with named paths takes the restricted selection.
    match identity {
        WorktreeIdentity::MainWorktree {
            common_dir,
            worktree_root,
            ..
        } if !main_checkout_paths.is_empty() => {
            return observation(
                common_dir.as_path(),
                Some(worktree_root.as_path()),
                effective_cwd,
                CopySelection::Paths(main_checkout_paths.to_vec()),
            )
            .map(Some);
        }
        _ => return observe_worktree_repository(identity, effective_cwd),
    }
}

/// What:
///  The observation of one repository with its optional source worktree.
/// Why:
///   Both repository kinds share the administrative read; only the source differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { adminRoot: join(commonDir, 'worktrees'), beforeAdminIds: await readAdminIds(adminRoot), commonDir, effectiveCwd, sourceRoot }
/// ```
fn observation(
    common_dir: &Path,
    source_root: Option<&Path>,
    effective_cwd: &Path,
    selection: CopySelection,
) -> Result<WorktreeCopyObservation, WorktreeCopyError> {
    let admin_root: PathBuf = common_dir.join("worktrees");
    let before_admin_ids: BTreeSet<String> = read_admin_ids(admin_root.as_path())?;
    return Ok(WorktreeCopyObservation {
        admin_root,
        before_admin_ids,
        common_dir: common_dir.to_path_buf(),
        effective_cwd: effective_cwd.to_path_buf(),
        source_root: source_root.map(Path::to_path_buf),
        selection,
    });
}

/// What:
///  The linked-worktree administrative identity named by a root's `.git` file,
///  or `None` for the main worktree.
/// Why:
///   Only a linked worktree's `.git` is a file that names an administrative directory;
///  a `.git` directory is the main worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readLinkedAdminId(root: string): Promise<string | typeof MAIN_WORKTREE_ADMIN_ID>;
/// ```
fn read_linked_admin_id(root: &Path) -> Result<Option<String>, WorktreeCopyError> {
    let pointer_path: PathBuf = root.join(".git");
    // `read` fails with `IsADirectory` for a main worktree's `.git` directory, which is not an error here.
    match std::fs::read(pointer_path.as_path()) {
        Ok(bytes) => {
            let line: &[u8] = strip_git_line(bytes.as_slice());
            let Some(gitdir) = line.strip_prefix(GITDIR_PREFIX.as_bytes()) else {
                return Ok(None);
            };
            let raw_admin: PathBuf =
                path_from_git_bytes(gitdir).ok_or_else(|| return identity_failure(root, None))?;
            let admin_path: PathBuf = if raw_admin.is_absolute() {
                raw_admin
            } else {
                root.join(raw_admin)
            };
            let canonical: PathBuf = std::fs::canonicalize(admin_path.as_path())
                .map_err(|error| return identity_failure(root, Some(&error)))?;
            return Ok(canonical
                .file_name()
                .map(|name| return name.to_string_lossy().into_owned()));
        }
        Err(error) if error.kind() == std::io::ErrorKind::IsADirectory => {
            return Ok(None);
        }
        Err(error) => {
            return Err(identity_failure(root, Some(&error)));
        }
    }
}

/// What:
///  The failure of resolving a registered root's linked identity.
/// Why:
///   Names the root so the user can find the worktree whose pointer could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`cli-git: could not resolve worktree identity for ${JSON.stringify(root)}.`, error)
/// ```
fn identity_failure(root: &Path, cause: Option<&dyn std::fmt::Display>) -> WorktreeCopyError {
    let message: String = format!(
        "cli-git: could not resolve worktree identity for {}.",
        quote(root.to_string_lossy().as_ref())
    );
    return failure_with_cause(message.as_str(), cause);
}

/// What:
///  The canonical root of a Git-reported worktree path,
///  or `None` when Git still lists a root that no longer exists.
/// Why:
///   Git keeps a registration after its directory is deleted;
///  such a root cannot be copied into or excluded by identity, so it is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function canonicalRegisteredRoot(root: string): Promise<string | typeof REGISTERED_ROOT_MISSING>;
/// ```
fn canonical_registered_root(root: &Path) -> Result<Option<PathBuf>, WorktreeCopyError> {
    match std::fs::canonicalize(root) {
        Ok(canonical) => return Ok(Some(canonical)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  The linked worktrees Git lists right now, keyed by administrative identity.
/// Why:
///   Git's porcelain listing with NUL separators is the one authority for registered roots,
///  so the copy never guesses a root from the administrative directory name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readRegisteredWorktrees({ observation, gitPath }): Promise<{ rootsByAdminId; roots }>;
/// ```
fn read_registered_worktrees(
    context: &GitContext,
    observation: &WorktreeCopyObservation,
) -> Result<RegisteredWorktrees, WorktreeCopyError> {
    let request: GitRequest = GitRequest {
        cwd: observation.effective_cwd.clone(),
        arguments: vec![
            "--git-dir".into(),
            observation.common_dir.as_os_str().to_os_string(),
            "worktree".into(),
            "list".into(),
            "--porcelain".into(),
            "-z".into(),
        ],
        without_prefix: true,
        ..GitRequest::default()
    };
    let output = run_git(context, &request).map_err(|error| return raw_failure(&error))?;
    if !output.succeeded() {
        return Err(raw_failure(&String::from_utf8_lossy(
            output.stderr.as_slice(),
        )));
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut listed: Vec<PathBuf> = Vec::new();
    // `filter_map` keeps only fields of the `worktree <path>` form.
    for path in output
        .stdout
        .split(|byte| return *byte == 0)
        .filter_map(|field| return field.strip_prefix(b"worktree "))
        .filter_map(path_from_git_bytes)
    {
        listed.push(path);
    }
    for path in listed {
        if let Some(canonical) = canonical_registered_root(path.as_path())? {
            roots.push(canonical);
        }
    }
    let mut roots_by_admin_id: BTreeMap<String, PathBuf> = BTreeMap::new();
    for root in &roots {
        // The main worktree is recognized by its root being the common directory of a non-bare repository.
        let is_main: bool = observation.source_root.is_none() && root == &observation.common_dir;
        if is_main {
            continue;
        }
        if let Some(admin_id) = read_linked_admin_id(root.as_path())? {
            roots_by_admin_id.insert(admin_id, root.clone());
        }
    }
    return Ok(RegisteredWorktrees {
        roots_by_admin_id,
        roots,
    });
}

/// What:
///  Whether Git has a lock file on a registered worktree.
/// Why:
///   Git writes it while it creates a worktree and removes it when the checkout is done; `--lock` keeps it. The text in
///  it depends on the language and on `--reason`, so only its presence is evidence, and it is evidence of a worktree
///  that must not be copied into by a command that did not ask for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await exists(join(adminRoot, adminId, 'locked'))
/// ```
pub fn is_locked(admin_root: &Path, admin_id: &str) -> Result<bool, WorktreeCopyError> {
    // `match` on the stat: no `locked` file means no creation in progress; any other failure is reported.
    match std::fs::symlink_metadata(admin_root.join(admin_id).join(LOCKED_NAME)) {
        Ok(_) => return Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => return Ok(false),
        Err(error) => return Err(inspect_failure(admin_root, &error)),
    }
}

/// What:
///  Whether Git says in English that it is still creating a registered worktree.
/// Why:
///   A creating command run from inside a checkout (a filter or hook of an enclosing `git worktree add`) has a source
///  that is not filled yet. This is a heuristic for that one case: a localized Git or a lock reason that spells the
///  word defeats it, and nothing else depends on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readFile(join(adminRoot, adminId, 'locked'), 'utf8').catch(() => '')).trim() === 'initializing'
/// ```
pub fn is_initializing(admin_root: &Path, admin_id: &str) -> Result<bool, WorktreeCopyError> {
    // `match` on the read: no `locked` file means no creation in progress; any other failure is reported.
    match std::fs::read_to_string(admin_root.join(admin_id).join(LOCKED_NAME)) {
        Ok(text) => return Ok(text.trim() == INITIALIZING_TEXT),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => return Ok(false),
        Err(error) => return Err(inspect_failure(admin_root, &error)),
    }
}

/// What:
///  The identity of a registration: the text of its registration file, or nothing when it has none or is gone.
/// Why:
///   Git reuses an administrative name after a worktree is removed, so the name does not say which registration a copy
///  was made for; a token written once into the directory does, because Git deletes it with the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readFile(join(adminRoot, adminId, 'cli-git-registration'), 'utf8').catch(() => undefined)
/// ```
pub fn registration_token(admin_root: &Path, admin_id: &str) -> Result<Option<String>, WorktreeCopyError> {
    // `match` on the read: no file and no directory both mean no identity yet.
    match std::fs::read_to_string(admin_root.join(admin_id).join(REGISTRATION_NAME)) {
        Ok(text) => return Ok(Some(text.trim().to_string()).filter(|token| return !token.is_empty())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => return Ok(None),
        Err(error) => return Err(inspect_failure(admin_root, &error)),
    }
}

/// What:
///  Whether a finished copy marked this very registration.
/// Why:
///   A marker left by a copy of an earlier registration of the same name carries another token and says nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readFile(join(adminRoot, adminId, 'cli-git-copied'), 'utf8').catch(() => '')).trim() === token
/// ```
pub fn copied_for(admin_root: &Path, admin_id: &str, token: &str) -> Result<bool, WorktreeCopyError> {
    // `match` on the read: absence means unmarked; any other failure is reported.
    match std::fs::read_to_string(admin_root.join(admin_id).join(COPIED_MARKER_NAME)) {
        Ok(text) => return Ok(text.trim() == token),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => return Ok(false),
        Err(error) => return Err(inspect_failure(admin_root, &error)),
    }
}

/// What:
///  Marks a registration as copied, durably: the marker is written whole, synchronized, renamed into place, and its
///  directory synchronized. A registration that is gone needs no marker.
/// Why:
///   The marker is what tells recovery and later requests that the copy is over, and the journal is removed only after
///  it is durable, so no crash can forget a finished copy. A marker of another token that stands in the way is replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFileAtomically(join(adminRoot, adminId, 'cli-git-copied'), token);
/// ```
pub fn write_copied_marker(admin_root: &Path, admin_id: &str, token: &str) -> Result<(), WorktreeCopyError> {
    let directory: PathBuf = admin_root.join(admin_id);
    let unique: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    let temporary: PathBuf = directory.join(format!("{COPIED_MARKER_NAME}.{unique}.tmp"));
    let published: std::io::Result<()> = write_exclusive_synced(temporary.as_path(), token.as_bytes())
        .and_then(|()| return std::fs::rename(temporary.as_path(), directory.join(COPIED_MARKER_NAME)))
        .and_then(|()| return sync_directory(directory.as_path()));
    // `match` on the publication: a registration removed meanwhile has nothing to mark, and a leftover temporary goes.
    match published {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            let _ignored_removal: std::io::Result<()> = std::fs::remove_file(temporary.as_path());
            return Err(inspect_failure(admin_root, &error));
        }
    }
}

/// What:
///  The observation with its before-set read again.
/// Why:
///   Recovery may run between the first observation and real Git; the before-set must be as close to Git as possible so
///  worktrees other commands registered meanwhile are not taken for this command's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { ...observation, beforeAdminIds: await readAdminIds(observation.adminRoot) }
/// ```
pub fn refresh_before_admin_ids(
    observation: &WorktreeCopyObservation,
) -> Result<WorktreeCopyObservation, WorktreeCopyError> {
    return Ok(WorktreeCopyObservation {
        before_admin_ids: read_admin_ids(observation.admin_root.as_path())?,
        ..observation.clone()
    });
}

/// What:
///  Every registered worktree root that exists, which a copy excludes from the source it stages.
/// Why:
///   The service stages long after the command that created the worktree returned, so it reads the roots itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readRegisteredWorktrees({ observation, gitPath })).roots
/// ```
pub fn registered_roots_of(
    context: &GitContext,
    observation: &WorktreeCopyObservation,
) -> Result<Vec<PathBuf>, WorktreeCopyError> {
    return Ok(read_registered_worktrees(context, observation)?.roots);
}

/// What:
///  The worktrees newly registered since the observation, with every registered root.
/// Why:
///   Runs after real Git returned, whatever its status,
///  so a worktree Git registered before failing still counts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function findCreatedWorktrees({ observation, gitPath }): Promise<{ created; registeredRoots }>;
/// ```
pub fn find_created_worktrees(
    context: &GitContext,
    observation: &WorktreeCopyObservation,
) -> Result<CreatedWorktrees, WorktreeCopyError> {
    let after_admin_ids: BTreeSet<String> = read_admin_ids(observation.admin_root.as_path())?;
    let created_ids: Vec<String> = after_admin_ids
        .difference(&observation.before_admin_ids)
        .cloned()
        .collect();
    if created_ids.is_empty() {
        return Ok(CreatedWorktrees {
            created: Vec::new(),
            registered_roots: Vec::new(),
        });
    }
    let registered: RegisteredWorktrees = read_registered_worktrees(context, observation)?;
    let mut created: Vec<CreatedWorktree> = Vec::new();
    for admin_id in created_ids {
        // A new identity whose root Git no longer lists cannot receive a copy, so the copy fails.
        let Some(root) = registered.roots_by_admin_id.get(&admin_id) else {
            return Err(failure_with_cause(
                format!(
                    "cli-git: linked worktree {} was registered but its root could not be resolved.",
                    quote(admin_id.as_str())
                )
                .as_str(),
                None,
            ));
        };
        // The identity is read with the lock, before anything else is done for this worktree, so that a token written
        // later is refused if the directory was replaced meanwhile.
        let Some(identity) = admin_identity(observation.admin_root.as_path(), admin_id.as_str())? else {
            continue;
        };
        let locked: bool = is_locked(observation.admin_root.as_path(), admin_id.as_str())?;
        created.push(CreatedWorktree {
            admin_id,
            root: root.clone(),
            locked,
            identity,
        });
    }
    return Ok(CreatedWorktrees {
        created,
        registered_roots: registered.roots,
    });
}

/// Registry controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_registry_tests.rs"]
mod tests;
