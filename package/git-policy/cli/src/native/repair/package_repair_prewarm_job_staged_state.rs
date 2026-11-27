//! What:
//!  The add's own move of a worktree's copied build state:
//!  the staging name it is moved to in one rename,
//!  the marker that proves the name is the wrapper's,
//!  and the patterns that keep the name out of the person's `git status`.
//! Why:
//!  `package_repair_prewarm_job.rs` would otherwise pass the file-size limit,
//!  and the add cannot wait for the clean,
//!  and must not leave state a build would trust,
//!  so the move is its own step with its own proof,
//!  in a file that reads the names and changes no filling.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const staged = stageTheWorktreesOwnState(destination, gitDir);
//! ```

/// Import the three names and what a writing of the marker came to.
use super::names::{
    TheMarkersAnswer,
    TheNames,
};
/// Import the names the move works in and the proof a marker is this job's own.
use super::names::{
    marker_claims,
    marker_of,
    staging_of,
    target_of,
    write_marker,
};
/// Import native paths.
use std::path::{
    Path,
    PathBuf,
};
/// What a staging step came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Staged {
    /// The worktree's own copied state now sits at this staging path,
    /// under this job's marker.
    Staged(PathBuf),
    /// The worktree held no build state of its own to move.
    Nothing,
}

/// What:
///  The names a filling of the worktree's own staged state works in,
/// or nothing when the add staged none.
/// Why:
///   The add moves the copied build state to the staging name in one rename before the add answers,
/// so a build that starts at once finds no build directory and compiles everything itself.
/// This run cleans that state out of the way it always does,
/// and publishes it back only where the worktree still has no build directory of its own.
/// The marker the add wrote is the proof the directory is the wrapper's own,
/// so anything without it is somebody's work and is never removed.
///
/// # Errors
///
/// A staging name this job's own marker does not claim.
pub(super) fn staged_names(staging: &Path) -> Result<Option<TheNames>, String> {
    if !staging.exists() {
        return Ok(None);
    }
    let marker: PathBuf = marker_of(staging);
    if !marker_claims(marker.as_path()) {
        return Err(format!(
            "{} holds what no filling of this job staged, so it is left exactly as it is found",
            staging.display()
        ));
    }
    let target: PathBuf = target_of(staging);
    return Ok(Some(TheNames {
        staging: staging.to_path_buf(),
        target,
        marker: Some(marker),
    }));
}

/// What:
///  Move the worktree's own copied build state into the filling's staging name,
/// under this job's marker,
/// in one rename;
/// or say that there was nothing to move.
/// Why:
///   Cargo treats the copied artifacts of the worktree's own crates as fresh,
/// so a build that started before a clean would reuse them at the wrong paths.
/// The add cannot wait for the clean (`REPAIR_TOOL_CEILING` is eighteen minutes and the quiet window is six hundred
///  seconds),
/// and it must not leave that state where a build can trust it,
/// so it moves the state aside in one rename before it answers:
/// a build that starts at once finds no build directory and compiles everything itself,
/// which is correct and only slower,
/// and this job cleans the state and publishes it back where nothing has rebuilt yet.
/// The marker is written first,
/// so a kill between the two leaves a name this job proves is its own.
///
/// # Errors
///
/// A staging name another file or directory holds,
/// and a state that cannot be moved.
pub(crate) fn stage_the_worktrees_own_state(
    destination: &Path,
    git_dir: Option<&Path>,
) -> Result<Staged, String> {
    if !destination.exists() {
        return Ok(Staged::Nothing);
    }
    // The staging name sits beside the build directory it takes, so the rename stays inside the worktree's own subvolume:
    // btrfs refuses to rename a directory out of a subvolume, and a worktree of this repository is one.
    let staging: PathBuf = staging_of(destination);
    let marker: PathBuf = marker_of(staging.as_path());
    if marker_claims(marker.as_path()) {
        // A killed add left the names it claimed; the state is already out of the worktree.
        return Ok(if staging.exists() {
            Staged::Staged(staging)
        } else {
            Staged::Nothing
        });
    }
    if staging.exists() || marker.exists() {
        return Err(format!(
            "{} already holds something this wrapper did not stage, so the copied build state is left where it is; the worktree's first build would reuse it, so run `cargo clean --offline --package <member>` in {} instead",
            staging.display(),
            destination.display()
        ));
    }
    // The staging name and the target directory its clean uses stand inside the worktree, so the repository's own exclude file
    // names them and no `git status` or `git add -A` of the person's sees them.
    if let Some(directory) = git_dir
        && let Err(reason) = exclude_the_staging_names(directory)
    {
        return Err(format!(
            "{reason}, so the copied build state stays where it is; run `cargo clean --offline --package <member>` in the worktree instead"
        ));
    }
    match write_marker(marker.as_path())? {
        TheMarkersAnswer::WrittenByThisRun => {}
        TheMarkersAnswer::WrittenByAnother => {
            return Err(format!(
                "{} was taken by another writer, so the copied build state is left where it is; run `cargo clean --offline --package <member>` in {} instead",
                marker.display(),
                destination.display()
            ));
        }
    }
    return match std::fs::rename(destination, staging.as_path()) {
        Ok(()) => Ok(Staged::Staged(staging)),
        Err(error) => {
            // Nothing moved, so the marker claims a name nothing is in; take it back and say the state stays where it is.
            let _taken_back = std::fs::remove_file(marker.as_path());
            Err(format!(
                "the copied build state at {} could not be moved aside ({error}), so it stays where it is; run `cargo clean --offline --package <member>` in that worktree instead",
                destination.display()
            ))
        }
    };
}

/// The patterns that keep this job's staging names out of the person's `git status` and `git add -A`.
///
/// They are written into the repository's own exclude file (`<git-dir>/info/exclude`),
/// which applies to every worktree of it,
/// because the names stand beside the build directory they take,
/// inside the worktree.
pub(crate) const EXCLUDED_STAGING_PATTERNS: &[&str] = &[
    "*.monochromatic-prewarm/",
    "*.monochromatic-prewarm.target/",
    "*.monochromatic-prewarm.owner",
];

/// What:
///  Name this job's staging patterns in the repository's exclude file,
/// once.
/// Why:
///   The staging name sits inside the worktree beside the build directory it takes,
/// because btrfs refuses a rename of a directory out of a worktree that is a subvolume,
/// so without this the person's `git status` would show it and a `git add -A` would commit a copy of `target/`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const excludeTheStagingNames = (gitDir) => { const path = join(gitDir, 'info/exclude'); if (!read(path, '').includes(PATTERNS[0])) appendFileSync(path, PATTERNS.join('\n') + '\n'); };
/// ```
pub(crate) fn exclude_the_staging_names(git_dir: &Path) -> Result<(), String> {
    let info: PathBuf = git_dir.join("info");
    let exclude: PathBuf = info.join("exclude");
    let already: String = std::fs::read_to_string(exclude.as_path()).unwrap_or_default();
    if already.contains(EXCLUDED_STAGING_PATTERNS[0]) {
        return Ok(());
    }
    if let Err(error) = std::fs::create_dir_all(info.as_path()) {
        return Err(format!("{} could not be made ({error})", info.display()));
    }
    let named_before: usize = already.len();
    let mut text: String = already;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    for pattern in EXCLUDED_STAGING_PATTERNS {
        text.push_str(pattern);
        text.push('\n');
    }
    // The file is the person's own and Git made it when the repository was, so this appends to it and never rewrites it:
    // a kill in the middle leaves their lines whole and at most adds this job's patterns twice.
    let opened = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(exclude.as_path());
    let mut file = match opened {
        Ok(file) => file,
        Err(error) => {
            return Err(format!(
                "{} could not be opened ({error})",
                exclude.display()
            ));
        }
    };
    return match std::io::Write::write_all(&mut file, &text.as_bytes()[named_before..]) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!(
            "{} could not be appended to ({error})",
            exclude.display()
        )),
    };
}
