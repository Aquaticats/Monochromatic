//! What:
//!  A mirror of one workspace,
//!  which is the only place this repair lets Cargo read and write a `Cargo.lock`.
//! Why:
//!   The repair has to resolve a workspace whose lock a `--locked` clean refused,
//! and Cargo's only lock file is the workspace's own,
//! written by the resolution and read by the clean:
//! there is no option that moves it (`cargo clean --help`,
//! and this toolchain's `-Z help` has no lockfile path),
//! so every step that lets Cargo run in the worktree writes over bytes a person may be editing.
//! Seven rounds of review found seven ways for that to lose or mislabel their file,
//! each in a different step of moving it.
//! A mirror removes the class:
//! the repair reads the person's lock and never renames, exchanges or writes it.
//! The mirror keeps the workspace's place below the worktree root,
//! because a path dependency is written relative to the manifest that names it,
//! every directory on the way is this job's,
//! every other entry is a link to the real one,
//! and the workspace's own manifest and lock are copies.
//! Cargo then resolves and cleans the same packages with the same inputs,
//! measured on this host on 2026-11-14:
//! a clean in a mirror of a two-member workspace with a build script removed every artifact the
//! workspace's own clean removed (29 files, 4.4 MiB),
//! and the workspace recompiled its member afterwards
//! (`${HOME}/temp/agent/add-never-skips-the-cargo-repair-because-of-a-marker-it-carried-scratch/experiments/does-the-mirror-shape-clean-the-same.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mirrored = mirrorTheWorkspace(worktree, workspace, jobRoot); // links for every sibling, copies at the leaf
//! ```

use std::path::{Path, PathBuf};

/// The directory,
/// under the Git directory,
/// that holds every mirror of this repair.
/// Why:
///   The Git directory is outside the worktree,
/// so a mirror of it never shows in a person's `git status` and is never committed,
/// and one name holds them all so a run's leftovers are in one place.
const MIRRORS_DIRECTORY: &str = "monochromatic-git-policy-cli-lock-resolutions";

/// The manifest a workspace is read from,
/// which the mirror copies rather than links.
const MANIFEST_FILE: &str = "Cargo.toml";

/// The lock file Cargo writes in the mirror,
/// which is the workspace's own bytes in a place this repair owns,
/// and never the person's file.
const LOCK_FILE: &str = "Cargo.lock";

/// The most bytes of one workspace path's hexadecimal form that go into one directory name.
/// Why:
///   A file system refuses a name longer than 255 bytes,
/// and a workspace path of ordinary depth is longer than that once it is written in hexadecimal,
///  so the text is split into names that each fit and that read back as one path when they are joined.
const NAME_BYTES: usize = 200;

/// What:
///  This job's own name,
///  as its process and the moment it asked.
/// Why:
///   One mirror for one job is what keeps two jobs of one workspace from deleting each other's folder while a Cargo
/// runs in it,
/// and a name no other job has is proof enough that the folder is this one's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aNameOfThisJob = () => `${process.pid}-${nanos().toString(16)}`;
/// ```
pub(crate) fn a_name_of_this_job() -> String {
    let since_the_epoch: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap_or_default();
    return format!("{}-{:x}", std::process::id(), since_the_epoch.as_nanos());
}

/// What:
///  The directory under the Git directory that holds one job's mirror of one workspace.
/// Why:
///   The workspace's own path decides the path above it now,
/// so two workspaces of one worktree never write over each other,
/// and this job's name decides the last part,
/// so two jobs never do either.
/// The path survives as bytes whatever the text encoding of a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = chunk(hexOfPath(workspace), 200).reduce((path, piece) => join(path, piece), join(keptIn, MIRRORS_DIRECTORY)).join(job)
/// ```
pub(crate) fn the_mirror_root(kept_in: &Path, workspace: &Path, job: &str) -> PathBuf {
    let hex: String = crate::path_bytes_in_records::hex_of_path(workspace);
    let mut directory: PathBuf = kept_in.join(MIRRORS_DIRECTORY);
    for piece in hex.as_bytes().chunks(NAME_BYTES) {
        directory = directory.join(String::from_utf8_lossy(piece).into_owned());
    }
    return directory.join(job);
}

/// What:
///  Lay out a mirror of one workspace below a root of this job's own,
///  and answer the mirror's own directory.
/// Why:
///   A path dependency is written relative to the manifest that names it,
/// and workspaces of this repository climb out of their own root,
/// so the mirror keeps the same place below the worktree root:
/// every directory on the way is this job's and every other entry is a link to the real one.
/// The workspace's own manifest and lock are copies,
/// which is what makes Cargo resolve and rewrite the mirror's own lock.
/// A path dependency that leaves the worktree root cannot be mirrored,
/// and the answer says so rather than guessing:
/// a link to it would let Cargo write outside the mirror.
///
/// # Errors
///
/// A workspace outside the worktree,
/// and a directory or a file of the workspace that cannot be read or linked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mirrored = mirrorTheWorkspace(worktree, workspace, root); // links for every sibling, copies at the leaf
/// ```
pub(crate) fn mirror_the_workspace(worktree: &Path, workspace: &Path, root: &Path) -> Result<PathBuf, String> {
    let Some(relative) = workspace.strip_prefix(worktree).ok() else {
        return Err(format!("{} is not inside the worktree {}", workspace.display(), worktree.display()));
    };
    let steps: Vec<std::ffi::OsString> = relative.iter().map(std::ffi::OsStr::to_os_string).collect();
    let mut real: PathBuf = worktree.to_path_buf();
    let mut mirrored: PathBuf = root.to_path_buf();
    for depth in 0..=steps.len() {
        let leaf: bool = depth == steps.len();
        if let Err(error) = std::fs::create_dir_all(mirrored.as_path()) {
            return Err(format!("{} could not be made ({error})", mirrored.display()));
        }
        let listing = match std::fs::read_dir(real.as_path()) {
            Ok(listing) => listing,
            Err(error) => return Err(format!("{} could not be read ({error})", real.display())),
        };
        for entry in listing.flatten() {
            let name: std::ffi::OsString = entry.file_name();
            // The entry the mirror walks into is the directory this step made, and the leaf's manifest and lock are copies.
            if !leaf && Some(name.as_os_str()) == Some(steps[depth].as_os_str()) {
                continue;
            }
            if name == std::ffi::OsStr::new(".git") {
                continue;
            }
            if leaf && (name == std::ffi::OsStr::new(MANIFEST_FILE) || name == std::ffi::OsStr::new(LOCK_FILE)) {
                continue;
            }
            let named: PathBuf = mirrored.join(name.as_os_str());
            if let Err(error) = std::os::unix::fs::symlink(entry.path(), named.as_path()) {
                return Err(format!("{} could not be linked into the mirror ({error})", named.display()));
            }
        }
        if leaf {
            for name in [MANIFEST_FILE, LOCK_FILE] {
                let from: PathBuf = real.join(name);
                if !from.is_file() {
                    continue;
                }
                let to: PathBuf = mirrored.join(name);
                if let Err(error) = std::fs::copy(from.as_path(), to.as_path()) {
                    return Err(format!("{} could not be copied into the mirror ({error})", to.display()));
                }
            }
            break;
        }
        real = real.join(steps[depth].as_os_str());
        mirrored = mirrored.join(steps[depth].as_os_str());
    }
    return Ok(mirrored);
}

/// What:
///  Remove a mirror this job made,
///  and say why when it cannot.
/// Why:
///   The mirror is this job's own and holds nothing of the person's but copies,
/// so it goes whatever the steps in it came to,
/// and a mirror that cannot be removed is only a line: the next job of the same workspace makes its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removeTheMirror = (root) => rmSync(root, { recursive: true, force: true })
/// ```
pub(crate) fn remove_the_mirror(root: &Path) {
    if let Err(error) = std::fs::remove_dir_all(root) {
        if error.kind() != std::io::ErrorKind::NotFound {
            crate::diagnostic_log::debug("packageRepairWorkspaceMirror", format!("{} could not be removed ({error})", root.display()).as_str());
        }
    }
}

#[cfg(test)]
#[path = "package_repair_workspace_mirror_tests.rs"]
mod tests;
