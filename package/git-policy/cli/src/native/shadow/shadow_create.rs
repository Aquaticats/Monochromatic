//! What:
//!  Creates the shadow repository of one published transaction:
//!       an empty bare repository,
//!       its config and alternates,
//!       the linked shared entries,
//!       the conclusion state,
//!       and the ref snapshot.
//! Why:
//!  The shadow is a separate repository rather than a registered worktree,
//!  so `git worktree list` never shows it and preparation creates no ref in the real repository.
//!  Its object store receives the objects native `git commit` writes,
//!  and its alternates name the real object directory,
//!  so a real `git prune` cannot delete a pending commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await createShadowRepository({ gitPath, cwd, capture, transactionId, transactionDirectory });
//! ```

/// Shadow path,
/// redirect variables and the shadow request.
use crate::shadow_git::{REPOSITORY_REDIRECT_VARIABLES, shadow_repository_path};
/// Invocation capture.
use crate::capture_invocation::{InvocationCapture, InvocationLayout};
/// The in-process entries of an empty bare repository.
use crate::shadow_skeleton::create_shadow_skeleton;
/// Format copy and config text.
use crate::shadow_config::{format_shadow_config, read_repository_format, repository_format_from_snapshot};
/// Entry links.
use crate::shadow_links::link_shadow_entries;
/// Conclusion copy and ref snapshot.
use crate::conclusion_copy::copy_conclusion_state;
use crate::shadow_refs::write_shadow_refs;
/// Private directory creation and file writes.
use crate::private_storage::create_private_directory;
/// Where Git's words go now that the `init` carries no `--quiet`.
use crate::inner_git_words::record_words;
/// Process context and one required Git start.
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, checked_from, refused_by_the_system, run_git};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// Strings for the redirect variable list.
use std::ffi::OsString;
/// The build phase's stretches,
/// which the creation marks as it goes.
use crate::commit_build_part_durations::BuildPartClock;

/// What:
///  Overwrites a file the shadow's `git init` already created,
/// with private mode.
/// Why:
///   The incumbent's `writeFile` replaces the template `config` and `alternates`,
///  while the exclusive writer used for new files would refuse them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(path, bytes, { mode: 0o600 });
/// ```
fn overwrite_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    std::fs::write(path, bytes)?;
    return set_private_file(path);
}

/// What:
///  Gives a file the private file mode on Unix.
/// Why:
///   The shadow config names the worktree root,
/// includes the real common config and switches
///  every hook off;
/// it stays private to the account.
///  It names no hooks directory:
/// every shadow command passes its own `core.hooksPath`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await chmod(path, 0o600);
/// ```
#[cfg(unix)]
fn set_private_file(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

/// Windows has no mode bits for this file;
/// the incumbent's ACL work is not ported.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn set_private_file(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}

/// What:
///  Creates the shadow's parent directories as private directories.
/// Why:
///   The incumbent creates `monochromatic-git-policy-cli/shadow` with mode 0700.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(dirname(shadowPath), { recursive: true, mode: 0o700 });
/// ```
fn create_parent(shadow: &Path) -> std::io::Result<()> {
    let parent: &Path = shadow.parent().unwrap_or(shadow);
    if let Some(grandparent) = parent.parent() {
        ensure_private_directory(grandparent)?;
    }
    return ensure_private_directory(parent);
}

/// What:
///  Create a private directory unless another process made it first.
/// Why:
///   Concurrent commits of one repository race to create the shared `monochromatic-git-policy-cli/shadow` parents;
/// whoever loses has
///   nothing left to do.
/// A path that exists as anything but a directory is still a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory, { mode: 0o700 }).catch((error) => { if (error.code !== 'EEXIST') throw error; });
/// ```
pub fn ensure_private_directory(directory: &Path) -> std::io::Result<()> {
    match create_private_directory(directory) {
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && directory.is_dir() => return Ok(()),
        other => return other,
    }
}

/// How many times the shadow's `git init` is started,
/// each in a fresh directory,
/// when a start is cut at its ceiling or refused by the
///  system.
/// Two:
/// the start and one more,
/// the same "redone once" the quiet-window rule reads (`doc/planning/cli-git-shipped-limits-audit.md`,
///  pick 4,
/// the coordinator's reading,
/// open to veto);
/// a shadow whose `init` is cut twice fails its transaction.
pub(crate) const SHADOW_CREATION_ATTEMPTS: u32 = 2;

/// What:
///  Creates an empty bare repository with `git init`,
/// for the ref backend whose tables this crate does not write.
/// Why:
///   Reftable tables carry their own format,
/// so only Git creates them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGit({ args: ['init', '--bare', '--template=', `--object-format=${format}`, '--ref-format=reftable', shadow] });
/// ```
fn init_with_git(
    context: &GitContext,
    layout: &InvocationLayout,
    shadow: &Path,
) -> Result<(), GitFailure> {
    let arguments: Vec<String> = vec![
        String::from("init"),
        String::from("--bare"),
        String::from("--template="),
        format!("--object-format={}", layout.object_format),
        String::from("--ref-format=reftable"),
        shadow.display().to_string(),
    ];
    let mut init: GitRequest = GitRequest::new(layout.common_dir.as_path(), &arguments);
    init.without_prefix = true;
    init.unset = REPOSITORY_REDIRECT_VARIABLES.iter().copied().map(OsString::from).collect();
    let mut attempt: u32 = 1;
    loop {
        let outcome = run_git(context, &init);
        // A start ended at its ceiling, or refused a process or a thread, leaves a directory that a second `init` in place does not bring to
        //  the state of an uncut run (the cut experiment: 50 of 104 second runs differed in the reftable tables). It is discarded and the
        //  start is made again in a fresh directory, which the experiment found equal to an uncut run (104 of 104).
        let cut: bool = start_was_cut_or_refused(&outcome);
        if cut && attempt < SHADOW_CREATION_ATTEMPTS {
            let note: String = format!("the shadow's init was cut or refused (attempt {attempt} of {SHADOW_CREATION_ATTEMPTS}); its directory {} is discarded and the step starts again in a fresh one", shadow.display());
            crate::client_journal::record_entry(crate::journald_native::Priority::Warning, note.as_str(), &[]);
            discard_partial_shadow(shadow)?;
            attempt += 1;
            continue;
        }
        if cut {
            discard_partial_shadow(shadow)?;
        }
        let made = checked_from(outcome, init.arguments.as_slice())?;
        // The `init` carries no `--quiet`; what Git said of it is kept in the journal.
        record_words(&["init", "--bare", "--template=", "--ref-format=reftable"], &made);
        return Ok(());
    }
}

/// What:
///  Whether a start of `init` was ended at its ceiling or refused a process or a thread.
/// Why:
///   Only these leave a partial directory that a fresh start replaces.
/// An I/O error of another kind (no such program,
/// no permission)
///  is not retried:
/// a fresh start would fail the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcome.error?.kind === 'timed-out' || outcome.output?.stderr.includes('Resource temporarily unavailable')
/// ```
fn start_was_cut_or_refused(outcome: &std::io::Result<GitOutput>) -> bool {
    return match outcome {
        // `WouldBlock` is `EAGAIN`: the system refused the process or the thread before the command ran.
        Err(error) => matches!(error.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock),
        Ok(finished) => refused_by_the_system(finished),
    };
}

/// What:
///  Removes what a cut `git init` left at the shadow's path.
/// Why:
///   A shadow is private to its transaction and holds nothing but what `init` made,
/// so a partial one is discarded whole and never
///  completed in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(shadow, { recursive: true, force: true });
/// ```
fn discard_partial_shadow(shadow: &Path) -> Result<(), GitFailure> {
    return match std::fs::remove_dir_all(shadow) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(GitFailure(format!("the partial shadow {} cannot be discarded: {error}", shadow.display()))),
    };
}

/// What:
///  Turn the result of a filesystem step on the shadow into a Git failure that names the verb and the shadow:
///  `<verb> <shadow> failed: <error>`.
///  `std::io::Result<T>` is "the value,
/// or an input/output error".
/// Why:
///   Several steps of the creation report their failure this way,
/// and a named function stands where a closure that built
///  that text stood.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stepFailure<T>(attempt: () => T, verb: string, shadow: string): T; // throws GitFailure(`${verb} ${shadow} failed: ${error}`)
/// ```
fn step_failure<T>(result: std::io::Result<T>, verb: &str, shadow: &Path) -> Result<T, GitFailure> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(GitFailure(format!("{verb} {} failed: {error}", shadow.display()))),
    };
}

/// The same for a step that is named by a phrase and not by a verb and the shadow:
/// `<phrase> failed: <error>`.
fn named_step_failure<T>(result: std::io::Result<T>, phrase: &str) -> Result<T, GitFailure> {
    return match result {
        Ok(value) => Ok(value),
        Err(error) => Err(GitFailure(format!("{phrase} failed: {error}"))),
    };
}

/// What:
///  Creates the shadow repository of one published transaction.
/// Why:
///   Every step runs before any native Git command reads the shadow,
///  so the first read already sees the real config,
/// objects,
/// links and refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepository({ gitPath, cwd, capture, transactionId, transactionDirectory });
/// ```
pub fn create_shadow_repository(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    transaction_id: &str,
    transaction_directory: &Path,
) -> Result<PathBuf, GitFailure> {
    let shadow: PathBuf = shadow_repository_path(capture.layout.common_dir.as_path(), transaction_id);
    // No Git child runs here, so the stretch is named for the record a commit writes.
    let mut clock: BuildPartClock = BuildPartClock::begin("shadow parent");
    step_failure(create_parent(shadow.as_path()), "creating", shadow.as_path())?;
    create_shadow_repository_at(context, cwd, capture, shadow.as_path(), transaction_directory, &mut clock)?;
    return Ok(shadow);
}

/// What:
///  Creates a shadow repository at a given path,
/// whose parent directory exists.
/// Why:
///   The commit service builds the commits Git must build (an editor,
/// a signature) in a scratch repository inside its own
///  scratch directory,
/// away from the registry directory that recovery scans,
/// so no other process mistakes it for a dead
///  transaction's shadow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepositoryAt({ gitPath, cwd, capture, shadowPath, transactionDirectory });
/// ```
pub fn create_shadow_repository_at(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    return create_shadow_repository_carrying(context, cwd, capture, shadow, transaction_directory, true, clock);
}

/// What:
///  `create_shadow_repository_at`,
/// with the merge,
/// cherry-pick and revert conclusion state of the worktree either copied into
///  the scratch repository or left out of it.
/// Why:
///   A commit of named paths while another session's merge is unfinished must build as an ordinary commit of those paths on
///  `HEAD` (real Git refuses it as a partial commit during a merge);
/// a scratch repository that holds no `MERGE_HEAD` builds
///  exactly that,
/// and the landing then leaves the worktree's own conclusion state alone because the scratch repository
///  never held it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepositoryCarrying({ gitPath, cwd, capture, shadowPath, transactionDirectory, carryConclusion });
/// ```
pub fn create_shadow_repository_carrying(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    carry_conclusion: bool,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    let layout = &capture.layout;
    clock.next("shadow skeleton");
    if layout.ref_format == crate::transaction_journal::RefFormat::Reftable {
        // The reftable backend's tables are Git's to create, so this one case still starts `git init`.
        init_with_git(context, layout, shadow)?;
    } else {
        step_failure(create_shadow_skeleton(shadow), "creating", shadow)?;
    }
    step_failure(set_private_directory_mode(shadow), "protecting", shadow)?;
    clock.next("shadow config");
    let format = match layout.config_snapshot.as_ref().and_then(repository_format_from_snapshot) {
        Some(from_snapshot) => from_snapshot,
        None => read_repository_format(context, layout.common_dir.as_path())?,
    };
    let config_text: String =
        format_shadow_config(&format, layout.common_dir.as_path(), layout.repository_root.as_path());
    named_step_failure(
        overwrite_private(shadow.join("config").as_path(), config_text.as_bytes()),
        "writing the shadow config",
    )?;
    named_step_failure(
        overwrite_private(
            shadow.join("objects").join("info").join("alternates").as_path(),
            format!("{}\n", layout.object_directory.display()).as_bytes(),
        ),
        "writing the shadow alternates",
    )?;
    clock.next("shadow links");
    named_step_failure(
        link_shadow_entries(shadow, layout.common_dir.as_path(), layout.git_dir.as_path()),
        "linking the shadow entries",
    )?;
    clock.next("shadow conclusion");
    if carry_conclusion {
        copy_conclusion_state(
            context,
            cwd,
            layout.git_dir.as_path(),
            shadow,
            transaction_directory,
            layout.ref_format,
        )?;
    }
    clock.next("shadow refs");
    write_shadow_refs(
        context,
        cwd,
        shadow,
        layout.ref_format,
        layout.target_ref.as_str(),
        &layout.symbolic_head,
        &capture.base,
    )?;
    return Ok(());
}

/// What:
///  Gives the shadow directory the private mode.
/// Why:
///   The shadow holds the transaction's objects and refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await chmod(shadowPath, 0o700);
/// ```
#[cfg(unix)]
fn set_private_directory_mode(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    return std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
}

/// Windows has no mode bits for the shadow directory.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn set_private_directory_mode(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}

/// Shadow creation controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_create_tests.rs"]
mod tests;
