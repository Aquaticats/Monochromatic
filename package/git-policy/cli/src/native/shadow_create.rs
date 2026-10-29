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
//!  so a real `git prune` cannot delete a pending commit
//!  (`src/shadow-repository/shadow-repository.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await createShadowRepository({ gitPath, cwd, capture, transactionId, transactionDirectory });
//! ```

/// Shadow path, redirect variables and the shadow request.
use super::shadow_git::{REPOSITORY_REDIRECT_VARIABLES, shadow_repository_path};
/// Invocation capture.
use super::capture_invocation::InvocationCapture;
/// Format copy and config text.
use super::shadow_config::{format_shadow_config, read_repository_format, repository_format_from_snapshot};
/// Entry links.
use super::shadow_links::link_shadow_entries;
/// Conclusion copy and ref snapshot.
use super::conclusion_copy::copy_conclusion_state;
use super::shadow_refs::write_shadow_refs;
/// Private directory creation and file writes.
use super::private_storage::create_private_directory;
/// Process context and one required Git start.
use super::transaction_git::{GitContext, GitFailure, GitRequest, run_git_checked};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// Strings for the redirect variable list.
use std::ffi::OsString;

/// What:
///  Overwrites a file the shadow's `git init` already created, with private mode.
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
///   The shadow config names the real hooks directory and worktree; it stays private to the account.
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

/// Windows has no mode bits for this file; the incumbent's ACL work is not ported.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(windows)]
fn set_private_file(_path: &Path) -> std::io::Result<()> {
    return Ok(());
}

/// What:
///  Creates the shadow's parent directories as private directories.
/// Why:
///   The incumbent creates `cli-git/shadow` with mode 0700.
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
///   Concurrent commits of one repository race to create the shared `cli-git/shadow` parents; whoever loses has
///   nothing left to do. A path that exists as anything but a directory is still a failure.
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

/// What:
///  Creates the shadow repository of one published transaction.
/// Why:
///   Every step runs before any native Git command reads the shadow,
///  so the first read already sees the real config, objects, links and refs.
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
    let layout = &capture.layout;
    let shadow: PathBuf = shadow_repository_path(layout.common_dir.as_path(), transaction_id);
    create_parent(shadow.as_path())
        .map_err(|error| { return GitFailure(format!("creating {} failed: {error}", shadow.display())); })?;
    let format_argument: String = format!("--object-format={}", layout.object_format);
    let mut arguments: Vec<String> = vec![
        String::from("init"),
        String::from("--bare"),
        String::from("--quiet"),
        String::from("--template="),
        format_argument,
    ];
    if layout.ref_format == super::transaction_journal::RefFormat::Reftable {
        arguments.push(String::from("--ref-format=reftable"));
    }
    arguments.push(shadow.display().to_string());
    let mut init: GitRequest = GitRequest::new(layout.common_dir.as_path(), &arguments);
    init.without_prefix = true;
    init.unset = REPOSITORY_REDIRECT_VARIABLES.iter().map(|name| { return OsString::from(*name); }).collect();
    run_git_checked(context, &init)?;
    set_private_directory_mode(shadow.as_path())
        .map_err(|error| { return GitFailure(format!("protecting {} failed: {error}", shadow.display())); })?;
    let format = match layout.config_snapshot.as_ref().and_then(repository_format_from_snapshot) {
        Some(from_snapshot) => from_snapshot,
        None => read_repository_format(context, layout.common_dir.as_path())?,
    };
    let config_text: String =
        format_shadow_config(&format, layout.common_dir.as_path(), layout.repository_root.as_path());
    overwrite_private(shadow.join("config").as_path(), config_text.as_bytes())
        .map_err(|error| { return GitFailure(format!("writing the shadow config failed: {error}")); })?;
    overwrite_private(
        shadow.join("objects").join("info").join("alternates").as_path(),
        format!("{}\n", layout.object_directory.display()).as_bytes(),
    )
    .map_err(|error| { return GitFailure(format!("writing the shadow alternates failed: {error}")); })?;
    link_shadow_entries(shadow.as_path(), layout.common_dir.as_path(), layout.git_dir.as_path())
        .map_err(|error| { return GitFailure(format!("linking the shadow entries failed: {error}")); })?;
    copy_conclusion_state(
        context,
        cwd,
        layout.git_dir.as_path(),
        shadow.as_path(),
        transaction_directory,
        layout.ref_format,
    )?;
    write_shadow_refs(
        context,
        cwd,
        shadow.as_path(),
        layout.ref_format,
        layout.target_ref.as_str(),
        &layout.symbolic_head,
        &capture.base,
    )?;
    return Ok(shadow);
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
