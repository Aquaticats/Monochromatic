//! What:
//!  The linked worktree a `git worktree add` registered at the destination its command line names:
//!  whether that destination was a registered worktree before real Git ran,
//! and whether it is one registered by this
//!  command after Git returned,
//! with the canonical root of every registered worktree.
//! Why:
//!  The destination is read from the command line and its registration is found by following the pointer in its `.git`
//!  file,
//! so a concurrent add in the same repository can neither be mistaken for this one's worktree nor make this one wait
//!  (the earlier comparison of the whole set of registrations needed a lock to keep other adds out of its window).
//!  A worktree that Git registered before failing still receives the ignored files,
//!  so the check runs after Git returns whatever its status was.
//!  The main worktree is observed only through `observe_copy_source`,
//! which the caller reaches only for a command that
//!  creates a worktree,
//! so its other invocations cost no filesystem reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const observation = await observeWorktreeRepository({ args, gitPath });
//! const { created, registeredRoots } = await findCreatedWorktree({ observation, destination, registeredBefore, gitPath });
//! ```

use crate::git_metadata::path_from_git_bytes;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::worktree_add_binding::{Binding, Bound, RegistrationOwner, bind_registration, claim_registration};
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of};
use crate::json_record::quote;
use crate::worktree_copy_selection::CopySelection;
use crate::worktree_copy_stage_claim::StageClaim;
use crate::worktree_identity::WorktreeIdentity;
use std::path::{Path, PathBuf};

/// What:
///  The facts captured before real Git runs for an applicable invocation.
/// Why:
///   The copy after Git needs the common directory that owns the journals,
/// the source worktree and which of its ignored
///  paths are taken;
/// nothing of the repository's other worktrees is read,
/// so a concurrent add is none of its business.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyObservation = { commonDir; effectiveCwd; sourceRoot?; selection };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeCopyObservation {
    /// Canonical common Git directory,
    /// which owns the journals.
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
///  One linked worktree this command registered.
/// Why:
///   Its root is where the copy lands,
/// and the binding is the registration the copy verifies before every
///  write,
/// so a worktree that another command removed or replaced is never filled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CreatedWorktree = { root: string; binding: Binding };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedWorktree {
    /// Canonical created worktree root.
    pub root: PathBuf,
    /// The registration Git made for it,
    /// as it was when real Git returned.
    pub binding: Binding,
}

/// What:
///  The worktree this command created,
/// if any,
/// and every registered root that recursive copying must exclude.
/// Why:
///   A nested registered worktree is never copied into its parent's copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { created: CreatedWorktree[]; registeredRoots: string[] }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedWorktrees {
    /// The linked worktree registered at the destination,
    /// absent when this command registered none.
    pub created: Vec<CreatedWorktree>,
    /// Every registered root that exists on the filesystem.
    pub registered_roots: Vec<PathBuf>,
    /// The token of the command that proved the destination's registration is its own,
    /// when that command is not this one.
    pub other_commands: Option<String>,
}

/// What:
///  Captures what the copy after Git needs for one repository identity.
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
    // `match` decides applicability from the identity alone; only the two repository kinds are observed.
    match identity {
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::MainWorktree { .. } => {
            return Ok(None);
        }
        WorktreeIdentity::BareRepository { common_dir, .. } => {
            return Ok(Some(observation(common_dir.as_path(), None, effective_cwd, CopySelection::Ignored)));
        }
        WorktreeIdentity::LinkedWorktree {
            common_dir,
            worktree_root,
            ..
        } => {
            return Ok(Some(observation(
                common_dir.as_path(),
                Some(worktree_root.as_path()),
                effective_cwd,
                CopySelection::Ignored,
            )));
        }
    }
}

/// What:
///  The observation that also covers a main checkout,
/// which takes part only when the caller's command creates a worktree.
/// Why:
///   A main checkout hands over its ignored state under its leave-out rule,
/// so a worktree added from it works at once;
///  the caller observes it only for a creating command,
/// so the main checkout's other invocations cost no filesystem reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// observeCopySource({ identity, effectiveCwd }): Promise<WorktreeCopyObservation | typeof WORKTREE_COPY_NOT_APPLICABLE>;
/// ```
pub fn observe_copy_source(
    identity: &WorktreeIdentity,
    effective_cwd: &Path,
) -> Result<Option<WorktreeCopyObservation>, WorktreeCopyError> {
    // `match`: a main checkout takes the main-checkout selection; every other shape keeps the full listing.
    match identity {
        WorktreeIdentity::MainWorktree {
            common_dir,
            worktree_root,
            ..
        } => {
            return Ok(Some(observation(
                common_dir.as_path(),
                Some(worktree_root.as_path()),
                effective_cwd,
                CopySelection::MainCheckout,
            )));
        }
        _ => return observe_worktree_repository(identity, effective_cwd),
    }
}

/// What:
///  The observation of one repository with its optional source worktree.
/// Why:
///   Both repository kinds share it;
/// only the source differs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { commonDir, effectiveCwd, sourceRoot, selection }
/// ```
fn observation(
    common_dir: &Path,
    source_root: Option<&Path>,
    effective_cwd: &Path,
    selection: CopySelection,
) -> WorktreeCopyObservation {
    return WorktreeCopyObservation {
        common_dir: common_dir.to_path_buf(),
        effective_cwd: effective_cwd.to_path_buf(),
        source_root: source_root.map(Path::to_path_buf),
        selection,
    };
}

/// What:
///  Whether `destination` is a registered worktree of the repository right now.
/// Why:
///   It is asked before real Git runs:
/// a destination already registered then is not this command's creation,
/// whatever Git
///  does with it.
/// The answer follows the pointer in the destination's `.git` file and checks that the registration it names
///  names the destination back,
/// so no other worktree is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function destinationIsRegistered(commonDir: string, destination: string): boolean;
/// ```
pub fn destination_is_registered(common_dir: &Path, destination: &Path) -> bool {
    return bind_registration(common_dir, destination).is_ok();
}

/// What:
///  Whether a byte is NUL,
/// the end of each field of Git's `-z` output.
/// Why:
///   Git's worktree listing is split on it,
/// and a path may hold any other byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// byte === 0
/// ```
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The canonical root of a Git-reported worktree path,
///  or `None` when Git still lists a root that no longer exists.
/// Why:
///   Git keeps a registration after its directory is deleted;
///  such a root cannot be copied into or excluded by identity,
/// so it is skipped.
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
///  The roots of the worktrees Git lists right now,
/// those that exist.
/// Why:
///   Git's porcelain listing with NUL separators is the one authority for registered roots,
///  so the copy never guesses a root from the administrative directory name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readRegisteredRoots({ observation, gitPath }): Promise<string[]>;
/// ```
fn read_registered_roots(
    context: &GitContext,
    observation: &WorktreeCopyObservation,
) -> Result<Vec<PathBuf>, WorktreeCopyError> {
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
    let output = run_git(context, &request).map_err(raw_failure_of)?;
    if !output.succeeded() {
        return Err(raw_failure(&String::from_utf8_lossy(
            output.stderr.as_slice(),
        )));
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    // `filter_map` keeps only fields of the `worktree <path>` form.
    for field in output.stdout.split(is_nul_byte) {
        let Some(named) = field.strip_prefix(b"worktree ") else {
            continue;
        };
        let Some(path) = path_from_git_bytes(named) else {
            continue;
        };
        if let Some(canonical) = canonical_registered_root(path.as_path())? {
            roots.push(canonical);
        }
    }
    return Ok(roots);
}

/// What:
///  The worktree this command registered at `destination`,
/// with every registered root.
/// Why:
///   Runs after real Git returned,
/// whatever its status was,
/// so a worktree Git registered before failing still counts.
///  It is this command's when the destination was not registered before Git ran and is registered now:
/// the destination is
///  the one the command line named,
/// so another add's worktree is never taken for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function findCreatedWorktree({ observation, destination, registeredBefore, gitPath }): Promise<{ created; registeredRoots }>;
/// ```
pub fn find_created_worktree(
    context: &GitContext,
    observation: &WorktreeCopyObservation,
    destination: &Path,
    registered_before: bool,
    mine: &str,
    expected: Option<crate::fs_beneath::ObjectId>,
) -> Result<CreatedWorktrees, WorktreeCopyError> {
    if registered_before {
        return Ok(nothing_created(None));
    }
    // The binding is proved here, when Git has returned, so the copy and a later recovery check the registration this
    // command saw and not merely whatever stands at the path now.
    let Ok(bound) = bind_registration(observation.common_dir.as_path(), destination) else {
        return Ok(nothing_created(None));
    };
    // The claim is taken only for the registration this command made: a destination that is another object now is a
    // replacement, and claiming its registration would keep the command that made it from copying its own ignored state.
    if let Some(identity) = expected
        && bound.binding.destination_id != identity
    {
        return Ok(nothing_created(None));
    }
    let owner: RegistrationOwner = claim_registration(&bound.admin, bound.binding.admin.as_path(), mine).map_err(raw_failure_of)?;
    let Bound { binding, .. } = bound;
    if let RegistrationOwner::Another(other) = owner {
        return Ok(nothing_created(Some(other)));
    }
    return Ok(CreatedWorktrees {
        created: vec![CreatedWorktree { root: destination.to_path_buf(), binding }],
        registered_roots: read_registered_roots(context, observation)?,
        other_commands: None,
    });
}

/// What:
///  The answer that this command created no linked worktree,
/// with the token of the command that owns the one at the destination when there is one.
/// Why:
///   Both refusals return the same shape,
/// and the caller tells them apart only by the token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nothingCreated(otherCommands: string | undefined): CreatedWorktrees;
/// ```
fn nothing_created(other_commands: Option<String>) -> CreatedWorktrees {
    return CreatedWorktrees {
        created: Vec::new(),
        registered_roots: Vec::new(),
        other_commands,
    };
}

/// What:
///  Whether the registration at `destination` carries another command's proof.
/// Why:
///   A command whose Git failed may find a registration another command made in the directory it reserved,
/// and that
///  command's worktree is whole once its proof is there;
/// the failed command must not keep a record describing that
///  worktree as unfinished,
/// because every other command would then refuse to take it as a source.
/// This only reads the proof and claims nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function registeredByAnother({ commonDir, destination, mine }): Promise<boolean>;
/// ```
pub fn registered_by_another(common_dir: &Path, destination: &Path, mine: &str) -> Result<bool, WorktreeCopyError> {
    let Ok(bound) = bind_registration(common_dir, destination) else {
        return Ok(false);
    };
    let Ok(mut proof) = crate::fs_beneath::open_file_at(&bound.admin, std::ffi::OsStr::new(crate::worktree_add_binding::REGISTRATION_OWNER_NAME)) else {
        return Ok(false);
    };
    let mut recorded: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut proof, &mut recorded).map_err(raw_failure_of)?;
    let owner: String = String::from_utf8_lossy(recorded.as_slice()).trim().to_string();
    return Ok(!owner.is_empty() && owner != mine);
}

/// Registry controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_registry_tests.rs"]
mod tests;

/// What:
///  The text this command proves its own registration with:
/// the stage claim's random name on the first line and the owning process on the second.
/// Why:
///   The name tells two commands of one process apart,
/// and the process lets the person's line name who made the worktree.
/// A command whose claim could not be made still proves a registration of its own,
/// with a
///  name made here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${claim?.identifier() ?? randomUuid()}\n${currentCopyOwner().token()}`
/// ```
pub(crate) fn command_token(recorded: Option<&StageClaim>, destination: &Path) -> Result<String, WorktreeCopyError> {
    let name: String = match recorded {
        Some(claim) => claim.identifier(),
        None => match crate::random_id::random_uuid() {
            Ok(made) => made,
            Err(error) => {
                let message: String = format!(
                    "monochromatic-git-policy-cli: could not name the worktree add of {}.",
                    quote(destination.to_string_lossy().as_ref())
                );
                return Err(crate::worktree_copy_error::failure_with_cause(message.as_str(), Some(&error.0)));
            }
        },
    };
    let owner: String = crate::worktree_copy_owner::current_copy_owner()?.token();
    return Ok(format!("{name}\n{owner}"));
}
