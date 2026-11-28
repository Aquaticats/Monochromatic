//! What:
//!  The withdrawal of one boot's readiness record:
//!  what stood under the record's name when it began,
//!  the removal of that name through the proved directory's descriptor,
//!  the directory sync,
//!  and the read that proves this route's own reader now answers nothing.
//! Why:
//!  The shutdown unit's `ExecStopPost` withdraws readiness before it drains
//!  in-flight slot work and before it unmounts anything
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit":
//!  "Withdraw readiness and close admission to new ZED attaches before
//!  draining in-flight slot operations").
//!  Admission reads this record and nothing else
//!  (`crate::worktree_clone_mount_protocol::clone_mount_readiness_admission`),
//!  and no reader admits an attach when no record stands,
//!  so the absence this call proves is what closes admission.
//!  The name is removed and never replaced with a whole second record:
//!  a record says a boot verified a pool and a checkout,
//!  and its fields carry no word for "this boot is ending".
//!  A record this route's reader refuses is removed too,
//!  because the name lies in a directory only root may write,
//!  and an unreadable record already admits nothing:
//!  refusing here would leave every mount of a machine that is going down
//!  standing over a cache file.
//!  What this call refuses is the directory itself,
//!  whose owner and mode are what make a name under it the route's:
//!  a folder another account owns is never the place this record lives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const withdrawal = withdrawTheRecord('/run/git-worktree-clone-mount-readiness', 0);
//! withdrawal.stood; // 'nothing' | the record | the reader's refusal
//! ```

use super::ReadinessFile;
use super::ReadinessFileRefusal;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    READINESS_DIRECTORY_MODE,
    READINESS_NAME,
    ReadinessRecord,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_calls::{
    sync_directory,
    unlink_at,
};
use crate::worktree_clone_mount_protocol::root_owned_directory::{
    RootDirectoryRefusal,
    make_and_prove,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  What stood under the record's name when the withdrawal began.
/// Why:
///   A person reading a shutdown pass has to tell a boot that published a
///  record from one that published none,
///  and a record the route's reader refuses is a third answer rather than
///  either of those
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Requests,
/// results and ordering").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordStood = { kind: 'nothing' } | { kind: 'record'; record } | { kind: 'unreadable'; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RecordStood {
    /// No record was there:
    /// the boot published none,
    /// or an earlier withdrawal took it.
    Nothing,
    /// The record this route wrote,
    /// read whole.
    Record(Box<ReadinessRecord>),
    /// A name that is there and that this route's own reader refuses,
    /// with the reader's own sentence.
    Unreadable {
        /// Why the record's reader refused the file.
        said: String,
    },
}

/// What:
///  What one withdrawal found and where the record lived.
/// Why:
///   The pass decides which mounts to unmount from the record it withdrew,
///  so the record the removal read travels back with the removal rather than
///  from a second read that a concurrent publish could have answered
///  differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Withdrawal = { stood: RecordStood; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Withdrawal {
    /// What stood under the record's name before it was removed.
    pub(crate) stood: RecordStood,
    /// The record's own path.
    pub(crate) path: PathBuf,
}

/// What:
///  Why a withdrawal changed nothing.
/// Why:
///   The record's directory is the one place the route's own root-owned file
///  lives,
///  so a directory another account owns,
///  a mode nobody set,
///  or a name that survives the removal is a state a person repairs,
///  never one this call works around.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WithdrawRefusal = { kind: 'directory'; refusal } | { kind: 'removal'; path; reason } | { kind: 'still-there'; path; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WithdrawRefusal {
    /// The directory is not the designed root-owned folder,
    /// or could not be opened.
    Directory {
        /// Why the directory was not proved,
        /// in the record's own words.
        refusal: Box<ReadinessFileRefusal>,
    },
    /// The record's name could not be removed.
    Removal {
        /// The record's own path.
        path: PathBuf,
        /// What the kernel answered.
        reason: String,
    },
    /// This route's own reader still answers something under the record's name
    /// after the removal,
    /// so admission is not closed.
    StillThere {
        /// The record's own path.
        path: PathBuf,
        /// What the reader answered instead of nothing.
        said: String,
    },
}

impl WithdrawRefusal {
    /// The refusal in words,
    /// naming the path and the way forward.
    pub(crate) fn message(&self) -> String {
        return match self {
            WithdrawRefusal::Directory {
                refusal,
            } => format!(
                "{}so no readiness record was withdrawn and no attach was refused on this pass",
                refusal.message()
            ),
            WithdrawRefusal::Removal {
                path,
                reason,
            } => format!(
                "the readiness record {} could not be removed ({reason}), so this boot's readiness still stands and admission is not closed. Ways forward: remove the name as root and stop the machine again, and read `journalctl -u reconcile-git-worktree-clone-mounts.service` for what the pass reached before it stopped",
                path.display()
            ),
            WithdrawRefusal::StillThere {
                path,
                said,
            } => format!(
                "the readiness record {} still answers after it was removed ({said}), so admission is not closed. Ways forward: remove the name as root and stop the machine again",
                path.display()
            ),
        };
    }
}

/// What:
///  Withdraw one boot's readiness record,
///  answering what stood there.
/// Why:
///   The removal is relative to the proved directory's descriptor,
///  so the name that is removed is the one inside the folder this route
///  proved and never one a replaced path holds;
///  the directory is synced so the absence survives a crash,
///  and the route's own reader is asked again so the absence is a fact and not
///  an assumption
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Requests,
/// results and ordering").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withdrawal = withdrawTheRecord(directory, 0); // throws when the directory is not the route's
/// ```
///
/// # Errors
/// The directory is not the designed root-owned folder,
/// its name cannot be removed,
/// or a record still answers after the removal.
pub(crate) fn withdraw_the_record(
    directory: &Path,
    owner: u32,
) -> Result<Withdrawal, WithdrawRefusal> {
    let record_path: PathBuf = directory.join(READINESS_NAME);
    let record_file: ReadinessFile = ReadinessFile::at(directory.to_path_buf(), owner);
    let stood: RecordStood = read_what_stood(&record_file, record_path.as_path())?;
    // A boot that published nothing has no name to remove, and the proof makes
    // the directory it is asked about: a machine whose handler never ran learns
    // no path from a shutdown that had nothing to withdraw. A name that arrived
    // between the read and here is removed like any other, so the window this
    // leaves is one publish and not one pass.
    let is_nothing: bool = matches!(stood, RecordStood::Nothing);
    let name_is_there: bool = std::fs::symlink_metadata(record_path.as_path()).is_ok();
    if is_nothing && !name_is_there {
        return Ok(Withdrawal {
            stood,
            path: record_path,
        });
    }
    let held: std::fs::File = prove_the_directory(directory, owner)?;
    match unlink_at(&held, READINESS_NAME) {
        Ok(()) => {}
        // A name that is already gone is the state a second withdrawal of one
        // boot finds, and it is the same absence the caller asked for.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(WithdrawRefusal::Removal {
                path: record_path,
                reason: error.to_string(),
            });
        }
    }
    sync_directory(&held).map_err(|error| {
        return WithdrawRefusal::Removal {
            path: record_path.clone(),
            reason: format!("the directory could not be synced after the removal ({error})"),
        };
    })?;
    drop(held);
    return match record_file.read() {
        Ok(None) => Ok(Withdrawal {
            stood,
            path: record_path,
        }),
        Err(ReadinessFileRefusal::DirectoryAbsent {
            ..
        }) => Ok(Withdrawal {
            stood,
            path: record_path,
        }),
        Ok(Some(record)) => Err(WithdrawRefusal::StillThere {
            path: record_path,
            said: format!(
                "this route's reader read the record of boot {} back",
                record.boot
            ),
        }),
        Err(refusal) => Err(WithdrawRefusal::StillThere {
            path: record_path,
            said: refusal.message(),
        }),
    };
}

/// What:
///  What stood under the record's name,
///  read before anything is removed.
/// Why:
///   The pass unmounts only what the record it withdrew proved,
///  so the values travel back from this one read;
///  a file the reader refuses is an answer rather than a stop,
///  because a cache the route cannot read admits nothing already,
///  while the directory's own terms stay refusals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stood = readWhatStood(directory, 0, path); // 'nothing' | record | reader refusal
/// ```
fn read_what_stood(
    record_file: &ReadinessFile,
    path: &Path,
) -> Result<RecordStood, WithdrawRefusal> {
    return match record_file.read() {
        Ok(None) => Ok(RecordStood::Nothing),
        Ok(Some(record)) => Ok(RecordStood::Record(Box::new(record))),
        // A directory that is not there holds no record, which is a boot whose
        // handler never published readiness.
        Err(ReadinessFileRefusal::DirectoryAbsent {
            ..
        }) => Ok(RecordStood::Nothing),
        // Every refusal the directory itself answers is the route's to repair,
        // and no name below it is removed on one.
        Err(refusal) if refusal_is_about_the_directory(&refusal) => {
            Err(WithdrawRefusal::Directory {
                refusal: Box::new(refusal),
            })
        }
        // A file the reader refuses is removed like any other name here: the
        // folder is root's and the content is a cache.
        Err(refusal) => Ok(RecordStood::Unreadable {
            said: format!("{}: {}", path.display(), refusal.message()),
        }),
    };
}

/// Whether one refusal is about the record's directory rather than its file.
///
/// Why:
///  The directory's owner and mode are what make a name under it the route's,
///  so those are the answers no removal may pass.
fn refusal_is_about_the_directory(refusal: &ReadinessFileRefusal) -> bool {
    return matches!(
        refusal,
        ReadinessFileRefusal::Directory { .. }
            | ReadinessFileRefusal::DirectoryAbsent { .. }
            | ReadinessFileRefusal::DirectoryOwner { .. }
            | ReadinessFileRefusal::DirectoryMode { .. }
    );
}

/// What:
///  The record's directory opened and proved as the designed root-owned folder.
/// Why:
///   The removal is named relative to this descriptor,
///  so a path replaced between the proof and the removal cannot make another
///  folder's file go.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = proveTheDirectory('/run/git-worktree-clone-mount-readiness', 0);
/// ```
fn prove_the_directory(
    directory: &Path,
    owner: u32,
) -> Result<std::fs::File, WithdrawRefusal> {
    return make_and_prove(directory, owner, READINESS_DIRECTORY_MODE).map_err(|refusal| {
        return WithdrawRefusal::Directory {
            refusal: Box::new(directory_refusal(directory, refusal)),
        };
    });
}

/// One directory proof's refusal in the record family's own words.
///
/// Why:
///  The proof and the read name the same three deviations,
///  so a person reads one sentence whichever call answered.
fn directory_refusal(
    directory: &Path,
    refusal: RootDirectoryRefusal,
) -> ReadinessFileRefusal {
    return match refusal {
        RootDirectoryRefusal::Open {
            reason,
        } => ReadinessFileRefusal::Directory {
            path: directory.to_path_buf(),
            reason,
        },
        RootDirectoryRefusal::Owner {
            found,
            wanted,
        } => ReadinessFileRefusal::DirectoryOwner {
            path: directory.to_path_buf(),
            found,
            wanted,
        },
        RootDirectoryRefusal::Mode {
            found,
            wanted: _wanted,
        } => ReadinessFileRefusal::DirectoryMode {
            path: directory.to_path_buf(),
            found,
        },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_readiness_file_withdraw_tests.rs"]
mod tests;
