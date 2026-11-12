//! What:
//!  What the filesystem proves about a Git directory the service can no longer resolve:
//!  that the repository is gone,
//!  or that something may still put it back.
//! Why:
//!  A drain whose directory cannot be resolved keeps its push plans forever
//! (`keep_plans_while_unreachable`),
//! because that
//!  state covers two cases the service could not tell apart:
//! a drive that is not mounted yet,
//! whose repository returns when it is,
//! and a
//!  repository that was deleted,
//! whose plans can never run and whose records went with the directory.
//! The same question is asked about the units of the worktree a service serves
//! (`service_units_of_removed_worktrees::worktree_is_gone`),
//! so a service of a deleted repository,
//! its socket unit and its sweep agree with its retry plans about what is gone.
//! The owner's
//!  "nothing should ever wait forever or indefinitely" is what the second case breaks:
//! on 2026-11-10 a service kept
//!  retrying the pushes of two deleted throwaway repositories every 600 s for hours,
//! at try 48 and climbing
//! (`doc/planning/cli-git-unreachable-repository-ends-its-plans.md`).
//!  The two are told apart by what a path that cannot be resolved still shows:
//! the nearest directory above it that does exist,
//! and whether
//!  that directory is the root of a mounted filesystem,
//! is an empty directory (what a mount point looks like while its drive is not
//!  mounted),
//! or is an ordinary directory with entries in it.
//!  Nothing is missing but the repository itself when the nearest existing directory is a directory the
//!  missing part would have lived in:
//! a mounted filesystem's root,
//! or an ordinary directory that holds other entries.
//! A directory
//!  with nothing in it may be waiting for a drive,
//! so the plan stays where it is,
//! as the owner asked.
//!  The proof's limit is stated where it is made:
//! a repository deleted under a directory that is left empty,
//! and a mount
//!  point whose directory does not exist,
//! are the two states this reading can still confuse,
//! and a question the filesystem does not answer proves nothing,
//! so the plans it is asked about stay.
//!  Only a name that is not there is absence:
//! a directory this process may not search answers every question below it with an access error,
//! and the repository behind it is then invisible,
//! never deleted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = prove(gitDir, mounts); // { gone: { nearest, because } } | { mayReturn: { nearest } }
//! ```

#[cfg(target_os = "linux")]
use crate::fs_mount_options::{MOUNTINFO_PATH, MountFacts, mount_holding, parse_mount_table};
use std::path::{Path, PathBuf};

/// What:
///  What a path that cannot be resolved shows about the repository it named.
/// Why:
///   The drain asks one question of it:
/// whether keeping a plan can ever make the push run,
/// and the two answers
///  differ in what the person is told and in whether the service speaks again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proof = { gone: Gone } | { mayReturn: MayReturn };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Proof {
    /// The repository itself is gone:
    /// nothing is missing but it.
    Gone(Gone),
    /// Something may still put the directory back,
    /// so a plan is kept.
    MayReturn(MayReturn),
}

/// Why the repository is proven gone,
/// which is what the line says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GoneBecause = 'mounted-filesystem-root' | 'ordinary-directory';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoneBecause {
    /// The nearest existing directory is the root of a mounted filesystem,
    /// so every name below it would live on a
    ///  filesystem that is mounted and answering,
    /// and only someone writing the repository again brings it back.
    MountedFilesystemRoot,
    /// The nearest existing directory is an ordinary directory that holds other entries,
    /// so the repository was one
    ///  entry among them and was deleted.
    OrdinaryDirectory,
}

impl GoneBecause {
    /// The words the reason is said with.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// because.name // 'is the root of a mounted filesystem that is answering'
    /// ```
    pub fn words(self) -> &'static str {
        return match self {
            GoneBecause::MountedFilesystemRoot => "is the root of a mounted filesystem that is answering",
            GoneBecause::OrdinaryDirectory => "is a directory that holds other entries",
        };
    }
}

/// What:
///  The proof that the repository is gone:
/// the nearest directory that exists above it,
/// and which reading of that directory it is.
/// Why:
///   The line names the directory,
/// so the person can see for themselves what the service looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gone = { nearest: string; because: GoneBecause };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Gone {
    /// The nearest directory above the Git directory that exists.
    pub nearest: PathBuf,
    /// Which reading of it the proof is.
    pub because: GoneBecause,
}

/// What:
///  The proof that something may put the Git directory back:
/// the nearest directory that exists above it,
/// which holds nothing at all,
/// or which could not be read,
/// or the path itself,
/// which may be there behind a question the filesystem did not answer.
/// Why:
///   A mount point is an empty directory while its drive is not mounted,
/// so an empty directory is the look of a
///  drive that is not there yet;
/// a directory this process may not search or list is one whose contents are unknown,
/// and a path no question about it could be answered may be anything;
/// the plan is kept for each of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MayReturn = { nearest: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MayReturn {
    /// The nearest directory above the Git directory that exists.
    pub nearest: PathBuf,
}

/// What:
///  Whether a directory is the root of a mounted filesystem.
/// Why:
///   The proof is a question about the running system,
/// and a control of the proof must be able to answer it without
///  a mount of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Mounts { isMountPoint(path: string): boolean }
/// ```
pub trait Mounts {
    /// Whether `path` is the root of a mounted filesystem.
    fn is_mount_point(&self, path: &Path) -> bool;
}

/// What:
///  One look at the mount table of the running system,
/// asked of the resolved path.
/// Why:
///   The kernel's own table is the only statement of what is mounted,
/// and a path that is a mount point there is a
///  filesystem's root,
/// not a directory on its parent's filesystem.
///  A platform with no table at this path answers that nothing is a mount point,
/// which keeps an empty directory's plan and
///  ends an ordinary one's:
/// the reading that loses nothing.
///  One table is one look at the system:
/// it is read when it is made,
/// and a caller asks again by making another,
/// because a drive can be mounted between two questions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TheMountTable().isMountPoint(path) // the table's mount points as they are now
/// ```
#[derive(Debug)]
pub struct TheMountTable {
    /// The table,
    /// read when this table is made.
    #[cfg(target_os = "linux")]
    table: Vec<MountFacts>,
}

impl TheMountTable {
    /// What:
    ///  The mount table of the running system,
    ///  read now.
    /// Why:
    ///   A table is one look at the system,
    /// and a question it answered may be asked again after a drive was mounted;
    /// the caller that asks again makes another table,
    /// which is what `the_mounts` is for.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// TheMountTable.readNow() // the table as it is now; on a platform without one, nothing is a mount point
    /// ```
    fn read_now() -> Self {
        #[cfg(target_os = "linux")]
        {
            return Self { table: the_mount_table() };
        }
        #[cfg(not(target_os = "linux"))]
        {
            return Self {};
        }
    }
}

impl Mounts for TheMountTable {
    fn is_mount_point(&self, path: &Path) -> bool {
        #[cfg(target_os = "linux")]
        {
            return match mount_holding(self.table.as_slice(), path) {
                Some(mount) => mount.mount_point == path,
                None => false,
            };
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _unused: &Path = path;
            return false;
        }
    }
}

/// What:
///  The mount table of the running system,
///  read for this question.
/// Why:
///   A serving service asks whether its worktree is still there every minute
/// (`service_server::OWN_WORKTREE_CHECK`),
/// and a drive may be mounted,
/// or the only repository on it deleted,
/// between one of those questions and the next.
/// A table kept from the first question would answer the second about a system that is no longer there,
/// and would keep a service of a deleted repository serving for ever;
/// a read of the kernel's own file is a few kilobytes once a minute for every service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theMounts().isMountPoint('/') // the table as it is now
/// ```
pub fn the_mounts() -> TheMountTable {
    return TheMountTable::read_now();
}

/// The mount table this process reads,
/// and nothing when it cannot be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const table = () => parseMountTable(readFileSync('/proc/self/mountinfo')) ?? [];
/// ```
#[cfg(target_os = "linux")]
fn the_mount_table() -> Vec<MountFacts> {
    let Ok(text) = std::fs::read(MOUNTINFO_PATH) else {
        return Vec::new();
    };
    return parse_mount_table(text.as_slice());
}

/// What:
///  That nothing is a mount point,
/// which is a control's answer and a platform without a mount table's.
/// Why:
///   A control states what a directory is,
/// and the reading without a table is the one that ends the fewest plans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingMounted = { isMountPoint: () => false };
/// ```
pub struct NothingIsMounted;

impl Mounts for NothingIsMounted {
    fn is_mount_point(&self, _path: &Path) -> bool {
        return false;
    }
}

/// What:
///  What one question about a path answered.
/// Why:
///   Only a name that is not there says the path is gone.
/// A directory this process may not search answers every question below it with an access error,
/// and the repository behind it is then invisible,
/// not absent:
/// reading that answer as absence would delete a live repository's plans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Look = { there: { directory: boolean } } | { absent: true } | { unknown: true };
/// ```
enum Look {
    /// The path is there,
    /// and whether it is a directory.
    There {
        /// Whether the filesystem said the path is a directory.
        directory: bool,
    },
    /// Nothing is there under that name.
    Absent,
    /// The filesystem answered with nothing about whether the name is there.
    Unknown,
}

/// What one question about `path` answered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = (p) => { try { return { there: { directory: statSync(p).isDirectory() } }; } catch (e) { return e.code === 'ENOENT' ? { absent: true } : { unknown: true }; } };
/// ```
fn look(path: &Path) -> Look {
    return match std::fs::metadata(path) {
        Ok(found) => Look::There { directory: found.is_dir() },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Look::Absent,
        Err(_unanswered) => Look::Unknown,
    };
}

/// What:
///  What the walk up from a path found:
///  the nearest directory that is there,
///  that nothing above the path is,
///  or that a question on the way was not answered.
/// Why:
///   The proof reads the first directory above the missing part that is there,
/// and which one it is decides the answer;
/// a walk that was not answered at one step has not seen the whole path,
/// so it says nothing about what the path was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Nearest = { at: string } | { nothing: true } | { unknown: true };
/// ```
enum Nearest {
    /// The nearest directory that is there.
    At(PathBuf),
    /// Nothing above the path is there.
    Nothing,
    /// A question on the way was not answered.
    Unknown,
}

/// What:
///  The nearest directory above `path`,
///  and `path` itself,
///  that is there.
/// Why:
///   A question that was not answered stops the walk:
/// a directory above it that refused the search hides everything below it,
/// and reading past that refusal would name a directory the path is not under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nearestLookedAt = (p) => [p, ...ancestors(p)].reduce(...) // { at } | { nothing } | { unknown }
/// ```
fn nearest_looked_at(path: &Path) -> Nearest {
    for candidate in path.ancestors() {
        match look(candidate) {
            Look::There { directory: true } => return Nearest::At(candidate.to_path_buf()),
            Look::Unknown => return Nearest::Unknown,
            Look::There { directory: false } | Look::Absent => {}
        }
    }
    return Nearest::Nothing;
}

/// What:
///  Whether a directory holds nothing at all,
///  and why the question was not answered.
/// Why:
///   An empty directory may be a mount point waiting for its drive,
/// and one whose entries could not be listed is not known to hold the other entries of a directory a repository
///  was deleted from.
///   The error is given back rather than taken for "holds something".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isEmpty = (dir) => readdirSync(dir).length === 0; // throws when it cannot be read
/// ```
fn is_empty_directory(path: &Path) -> std::io::Result<bool> {
    let mut listing: std::fs::ReadDir = std::fs::read_dir(path)?;
    return Ok(listing.next().is_none());
}

/// What:
///  Whether a Git directory that cannot be resolved is proven gone,
/// and what the proof rests on.
/// Why:
///   The drain keeps its plans for a directory that may return and ends them for one that cannot;
/// this is the
///  question that tells them apart,
/// asked only after the directory failed to resolve.
/// Four answers keep a plan,
/// because each of them leaves
///  something that can still put the directory back:
/// a worktree that is still there,
/// a directory with nothing in it,
/// a path
///  nothing is known about,
/// and a question the filesystem did not answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proof = prove(gitDir, worktree, mounts);
/// ```
pub fn prove(git_dir: &Path, worktree: Option<&Path>, mounts: &dyn Mounts) -> Proof {
    if let Some(root) = worktree {
        match look(root) {
            // A worktree that is still there is a repository whose own Git directory moved or is away: nothing of the repository was
            //  deleted, and the plan waits for the directory as it always has.
            Look::There { directory: true } => return Proof::MayReturn(MayReturn { nearest: root.to_path_buf() }),
            // A worktree this process may not look at may be there, and then nothing of the repository was deleted either.
            Look::Unknown => return Proof::MayReturn(MayReturn { nearest: root.to_path_buf() }),
            Look::There { directory: false } | Look::Absent => {}
        }
    }
    let nearest: PathBuf = match nearest_looked_at(git_dir) {
        Nearest::At(nearest) => nearest,
        // No ancestor at all exists, so nothing is known about the path: it is read as one that may return, which loses nothing.
        Nearest::Nothing => return Proof::MayReturn(MayReturn { nearest: git_dir.to_path_buf() }),
        // Something above the path did not answer, so what the path is was not seen: it may be there behind that answer.
        Nearest::Unknown => return Proof::MayReturn(MayReturn { nearest: git_dir.to_path_buf() }),
    };
    if mounts.is_mount_point(nearest.as_path()) {
        return Proof::Gone(Gone { nearest, because: GoneBecause::MountedFilesystemRoot });
    }
    return match is_empty_directory(nearest.as_path()) {
        // The directory holds nothing, so it may be a mount point waiting for its drive.
        Ok(true) => Proof::MayReturn(MayReturn { nearest }),
        // It holds other entries, so the repository was one of them and was deleted.
        Ok(false) => Proof::Gone(Gone { nearest, because: GoneBecause::OrdinaryDirectory }),
        // Whether it holds anything was not answered, so it is not read as holding the entries of a directory a repository was deleted from.
        Err(_unanswered) => Proof::MayReturn(MayReturn { nearest }),
    };
}

/// What:
///  The one sentence the journal keeps when a deleted repository's plans end.
/// Why:
///   The line has to say which path lost its plans and what the service looked at instead,
/// so a person reading the journal
///  can tell an unmounted drive the service kept from a repository the service gave up on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// goneLine(gitDir, gone, branches) // 'the pushes of ... end: the repository at ... is gone ...'
/// ```
pub fn gone_line(git_dir: &Path, gone: &Gone, branches: &[String]) -> String {
    let named: String = if branches.is_empty() {
        String::from("its push")
    } else {
        format!("the push of {}", branches.join(", "))
    };
    return format!(
        "{named} ends without another try: the repository at {} is gone, because the nearest directory above it that exists, {}, {}. Nothing is left there for a try to reach, and its work records went with it. A commit made there again is pushed as any other.",
        git_dir.display(),
        gone.nearest.display(),
        gone.because.words()
    );
}

/// Controls of the proof stay out of the release executable.
#[cfg(test)]
#[path = "unreachable_git_dir_proven_gone_tests.rs"]
mod tests;
