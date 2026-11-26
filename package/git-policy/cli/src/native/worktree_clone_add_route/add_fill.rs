//! What:
//!  The fill: the frozen proof, the proof's phase marker, the clone's leave-outs,
//!  the private index, the pointer's exchange, and the check of the result.
//! Why:
//!  The order is the designs' and nothing under a real name is published before
//!  the finished tree is proven
//!  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
//!  "The restore": steps 2 to 8).
//!  Step 10's clean source wrote nothing through Git, so the private index was
//!  the captured index normalized and published. Step 11 adds the restore: the
//!  private index is the index the btrfs rung's own `restore_dirty` reads and
//!  writes, so the clone is reset to the commit the caller asked for and the
//!  paths Git must write go through Git's own conversion.
//!  A resume after any kill reads the states those writes leave: the marker says
//!  the proof passed, an existing trash is the leave-out's own, the private
//!  name's absence with the real name present says the publication ran, and the
//!  pointer file with the trash's manifest says the exchange happened.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const trash = fillTheClone({ source, plan, cloneRoot, capture, pointer, admin });
//! ```

use super::add_capture::Capture;
use super::add_capture_store::{
    marker_is_ours,
    publish_marker,
};
use super::add_exchange::{
    PreparedExchange,
    exchange_the_git_entry,
    prepare_the_exchange,
};
use super::add_frozen_proof::prove_the_clone;
use super::add_index::{
    PENDING_INDEX_NAME,
    index_for_the_fill,
    pending_stands,
    publish_the_private_index,
    published_stands,
    write_the_private_index,
};
use super::add_route_git::RouteGit;
use crate::fs_beneath::{
    entry_metadata_at,
    open_root,
};
use crate::private_storage::read_regular_file;
use crate::transaction_git::GitContext;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_clean_content::ConversionEnvironment;
use crate::worktree_clean_content_attribute_sources::for_context;
use crate::worktree_dirty_restore::restore_dirty;
use crate::worktree_ladder_target::Target;
use crate::worktree_snapshot_placeholders::{
    HandOver,
    remove_snapshot_placeholders,
};
use crate::worktree_snapshot_source::SnapshotSource;
use crate::worktree_trash::{
    PendingTrash,
    Pruned,
    TRASH_PARENT,
    TrashManifest,
    is_trash_name,
    prune_snapshot,
};
use crate::worktree_trash_own_manifest;
use std::ffi::{
    OsStr,
    OsString,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Everything one fill reads.
/// Why:
///  The fill touches the clone, the registration and the records, and each of
///  them is named once by the caller that proved it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FillInput = { source; plan; cloneRoot; slot; capture; pointer; nonce; adminPath; quiet };
/// ```
pub struct FillInput<'a> {
    /// The proven clean source.
    pub source: &'a SnapshotSource,
    /// The add's plan,
    /// which names the destination the stand-ins compare with.
    pub plan: &'a AddPlan,
    /// The slot,
    /// which the trash's record names.
    pub slot: &'a str,
    /// The mounted clone's root,
    /// which is the slot.
    pub clone_root: &'a Path,
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The capture the proof compares with.
    pub capture: &'a Capture,
    /// The repository's object name length in bytes.
    pub hash_length: usize,
    /// The bytes Git wrote into the slot's `.git`.
    pub pointer: &'a [u8],
    /// This add's nonce,
    /// which the marker and the record home are keyed by.
    pub nonce: &'a crate::worktree_clone_mount_protocol::RequestNonce,
    /// The wrapper's record home.
    pub home: &'a Path,
    /// The registration's directory,
    /// which receives the private index.
    pub admin_path: &'a Path,
    /// The repository's Git context,
    /// which the restore's own Git runs start through.
    pub context: &'a GitContext,
    /// The real Git this command resolved.
    pub real_git: &'a Path,
    /// The variables every Git child of this command receives.
    pub overlay: &'a [(OsString, OsString)],
    /// Whether the person asked for no lines.
    pub quiet: bool,
    /// The test-only switch that ends a control run at one step.
    pub stops: &'a super::add_route_plan::Stops,
}

/// What:
///  What a filled clone hands back.
/// Why:
///  The trash the leave-out made and the exchange extended is the disposal's
///  work, and the add hands it over after the fill.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Filled = { trash?: PendingTrash };
/// ```
#[derive(Debug)]
pub struct Filled {
    /// The trash the add leaves for the service.
    pub trash: Option<PendingTrash>,
}

/// What:
///  Prove the mounted clone, leave out what must not be carried, publish the
///  index, exchange the pointer, and check the result.
/// Why:
///  Each step reads the state a kill at the step before it leaves, so a resume
///  continues instead of repeating a write.
///
/// # Errors
/// The proof refused,
/// a leave-out refused,
/// the index could not be written or published,
/// the exchange refused,
/// or the result is not the worktree the record names.
pub fn fill_the_clone(input: &FillInput<'_>) -> Result<Filled, String> {
    let root: std::fs::File = open_root(input.clone_root).map_err(|error| {
        return format!(
            "the clone root {} cannot be opened without a link ({error})",
            input.clone_root.display()
        );
    })?;
    let already_exchanged: bool = entry_metadata_at(&root, OsStr::new(".git"))
        .map(|found| return found.is_file())
        .unwrap_or(false);
    if !already_exchanged {
        if marker_is_ours(input.home, input.slot, input.nonce)? {
            // The proof passed before the first write; the tree has been written
            // since, so proving it again would refuse the add's own work.
        } else {
            prove_the_clone(input.capture, input.clone_root)?;
            input.stops.after("proof");
            publish_marker(
                &crate::fs_beneath::open_root(input.home).map_err(|error| {
                    return format!(
                        "the record home {} cannot be opened ({error})",
                        input.home.display()
                    );
                })?,
                input.slot,
                input.nonce,
            )
            .map_err(|error| return format!("the proof's marker cannot be published ({error})"))?;
            input.nonce.as_str();
        }
    }
    let manifest: Option<TrashManifest> = match already_exchanged {
        true => the_existing_trash(input.clone_root)?,
        false => {
            let left_out: Option<TrashManifest> = leave_out_the_clone(input)?;
            input.stops.after("leave-outs");
            left_out
        }
    };
    if !published_stands(input.admin_path) && !pending_stands(input.admin_path) {
        // The seed is the clone's copied index, which D1 just proved equal to the
        // capture: a fresh read of the source's index could hold a later write.
        let seed: Vec<u8> = read_regular_file(input.clone_root.join(".git").join("index").as_path(), input.capture.index.size.saturating_add(1)).map_err(|refusal| {
            return format!("the clone's copied index cannot be read for the fill ({refusal:?}); Ways forward: run the add again with nothing writing the main checkout");
        })?;
        let bytes: Vec<u8> = index_for_the_fill(seed.as_slice(), input.hash_length)?;
        // A killed Git leaves its waiting index and its lock behind, and the write
        // below refuses a name that stands: the wrapper's own step clears both.
        crate::worktree_copy_interrupted_add_settlement::clear_the_waiting_index(input.admin_path);
        let admin: std::fs::File = open_root(input.admin_path).map_err(|error| {
            return format!(
                "the registration {} cannot be opened ({error})",
                input.admin_path.display()
            );
        })?;
        write_the_private_index(&admin, bytes.as_slice())?;
        input.stops.after("index");
    }
    if the_restore_still_runs(input.admin_path, input.clone_root, already_exchanged) {
        restore_the_clone(input)?;
        input.stops.after("restore");
    }
    let prepared: PreparedExchange =
        prepare_the_exchange(input.clone_root, input.pointer, manifest)?;
    input.stops.after("trash-entry");
    let admin: std::fs::File = open_root(input.admin_path).map_err(|error| {
        return format!(
            "the registration {} cannot be opened ({error})",
            input.admin_path.display()
        );
    })?;
    if !published_stands(input.admin_path) {
        publish_the_private_index(&admin, input.admin_path)?;
    }
    exchange_the_git_entry(input.clone_root, &prepared)?;
    input.stops.after("exchange");
    check_the_result(input)?;
    // The filled worktree is no resumable state, so the proof's records go in
    // the design's order: the marker first, then the capture. A kill between
    // them leaves a capture whose nonce the next add clears.
    super::add_capture_store::remove_capture(input.home, input.slot);
    return Ok(Filled {
        trash: Some(PendingTrash {
            destination: input.clone_root.to_path_buf(),
            common_dir: input.common_dir.to_path_buf(),
            manifest: prepared.manifest,
            taken_over: 0,
            quiet: input.quiet,
        }),
    });
}

/// What:
///  Whether the fill's restore still has to run.
/// Why:
///  The restore edits the private index and the tree,
///  and the publication takes that index's name away,
///  so a resume that finds the worktree's own index already standing was
///  past the restore and must not run it again:
///  a restored tree needs no second restore, and a Git run pointed at the
///  gone private name would read an empty index
///  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
///  "What a resume finds",
///  the rows for a mounted clone with the real-name index and for the saved
///  pointer).
///  A run resumed after the exchange is past the restore for the same reason.
pub(super) fn the_restore_still_runs(
    admin_path: &Path,
    clone_root: &Path,
    already_exchanged: bool,
) -> bool {
    if already_exchanged || published_stands(admin_path) {
        return false;
    }
    // The restore only runs against a private name that stands.
    return pending_stands(admin_path) || clone_root.join(".git").is_dir();
}

/// What:
///  Reset the clone to the commit the caller asked for,
///  through the btrfs rung's own restore against the private index.
/// Why:
///  The clone is a snapshot of a checkout that may hold uncommitted edits and
///  whose `HEAD` may not be the commit this add checks out, so the tree must be
///  made equal to Git's own checkout of that commit before any name is published
///  (`doc/planning/cli-git-zfs-dirty-main-restore-on-rung-1-design.md`,
///  "The restore" step 6).
///  The restore runs only once the private index stands, and it reads and writes
///  that file rather than the worktree's own, so no run touches a name the
///  worktree does not hold yet.
///
/// # Errors
/// The restore refused: the clean, the reset or its own proof.
fn restore_the_clone(input: &FillInput<'_>) -> Result<(), String> {
    let target: Target = Target {
        admin: input.admin_path.to_path_buf(),
        index: Some(input.admin_path.join(PENDING_INDEX_NAME)),
        pointer: input.pointer.to_vec(),
        commit: input.capture.head.clone(),
        differs: false,
        line: String::new(),
    };
    let conversion: ConversionEnvironment = for_context(input.context);
    let mut runner: RouteGit<'_> = RouteGit {
        real_git: input.real_git,
        overlay: input.overlay,
    };
    return restore_dirty(
        input.source,
        &target,
        input.clone_root,
        &mut runner,
        input.context,
        &conversion,
    );
}

/// The trash the leave-out makes for the clone, written so a kill after it
/// leaves a record of what it set aside.
fn leave_out_the_clone(input: &FillInput<'_>) -> Result<Option<TrashManifest>, String> {
    let pruned: Pruned = prune_snapshot(input.clone_root, input.source)?;
    let mut hand_over: HandOver = HandOver::starting(pruned.trash);
    let root: std::fs::File = open_root(input.clone_root).map_err(|error| {
        return format!(
            "the clone root {} cannot be opened without a link ({error})",
            input.clone_root.display()
        );
    })?;
    remove_snapshot_placeholders(&root, input.source, input.plan, &mut hand_over)?;
    let manifest: Option<TrashManifest> = hand_over.finish();
    if let Some(written) = manifest.as_ref() {
        worktree_trash_own_manifest::write(input.clone_root, written)?;
    }
    return Ok(manifest);
}

/// The trash an earlier run of this add left, when one stands.
fn the_existing_trash(clone_root: &Path) -> Result<Option<TrashManifest>, String> {
    let parent: PathBuf = clone_root.join(TRASH_PARENT);
    let listing: std::fs::ReadDir = match std::fs::read_dir(parent.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "the trash place {} cannot be listed ({error})",
                parent.display()
            ));
        }
    };
    let mut found: Vec<TrashManifest> = Vec::new();
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| {
            return format!(
                "the trash place {} cannot be listed ({error})",
                parent.display()
            );
        })?;
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if !is_trash_name(name.as_str()) {
            continue;
        }
        let Some(manifest) = worktree_trash_own_manifest::read(clone_root, name.as_str()) else {
            return Err(format!(
                "the trash {name} under {} holds no record this add wrote, so nothing proves what it is; Ways forward: inspect it and remove it if it is not the add's",
                parent.display()
            ));
        };
        found.push(manifest);
    }
    return match found.len() {
        0 => Ok(None),
        1 => Ok(Some(found.remove(0))),
        _several => Err(format!(
            "the clone at {} holds more than one trash, and one add leaves one; Ways forward: inspect them and remove the ones this add did not make",
            clone_root.display()
        )),
    };
}

/// What:
///  Check the filled worktree: the pointer, its registration, the common
///  directory, the index and the head.
/// Why:
///  "Mark the binding usable only after the fill is verified",
///  and the verification is the facts every later reader relies on.
fn check_the_result(input: &FillInput<'_>) -> Result<(), String> {
    let git: PathBuf = input.clone_root.join(".git");
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(git.as_path()).map_err(|error| {
            return format!("the filled worktree's .git cannot be examined ({error})");
        })?;
    if !metadata.is_file() {
        return Err(format!(
            "the filled worktree {} holds a .git that is no pointer file, so the exchange did not land",
            input.clone_root.display()
        ));
    }
    // The pointer is read as a plain file and to a bound, so a name swapped for a named pipe after the look above is
    //  refused at once and the fill is reported as not landed (`add_fill_tests`).
    let pointer: Vec<u8> = crate::plain_file::read_plain_file(
        git.as_path(),
        crate::worktree_convert_inspection::GIT_FILE_LIMIT,
    )
    .map_err(|error| return format!("the filled worktree's .git cannot be read ({error})"))?;
    if pointer != input.pointer {
        return Err(String::from(
            "the filled worktree's .git is not the pointer this add saved, so the fill is not the worktree the record names",
        ));
    }
    let registration: PathBuf =
        crate::worktree_add_binding::pointer_target(input.clone_root, pointer.as_slice())?;
    if registration != input.admin_path {
        return Err(format!(
            "the filled worktree's pointer names {}, and this add's registration is {}",
            registration.display(),
            input.admin_path.display()
        ));
    }
    // The one reader of `commondir` (`foundation/common_directory_of_a_git_directory.rs`) opens the file as a plain one, so a
    //  named pipe there is a fill that cannot be verified rather than a wait.
    let commondir: Vec<u8> =
        crate::common_directory_of_a_git_directory::bytes_named(registration.as_path())
            .map_err(|error| {
                return format!("the registration's commondir cannot be read ({error})");
            })?
            .ok_or_else(|| {
                return String::from(
                    "the registration's commondir is not there, so the fill is not whole",
                );
            })?;
    let resolved: PathBuf = crate::worktree_add_binding::resolve_lexically(
        registration.as_path(),
        commondir
            .strip_suffix(b"\n")
            .unwrap_or(commondir.as_slice()),
    )
    .map_err(|()| {
        return String::from("the registration's commondir leaves the file system root");
    })?;
    if resolved != input.common_dir
        && std::fs::canonicalize(resolved.as_path()).ok()
            != std::fs::canonicalize(input.common_dir).ok()
    {
        return Err(format!(
            "the registration's commondir names {}, and this repository's common directory is {}",
            resolved.display(),
            input.common_dir.display()
        ));
    }
    if !input.admin_path.join("index").is_file() {
        return Err(String::from(
            "the filled worktree's registration holds no index, so the fill is not whole",
        ));
    }
    return Ok(());
}

/// The trash a filled clone holds,
/// for the resume that finds the exchange already done.
pub fn the_trash_of_a_filled_clone(clone_root: &Path) -> Result<Option<TrashManifest>, String> {
    return the_existing_trash(clone_root);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_fill_tests.rs"]
mod tests;
