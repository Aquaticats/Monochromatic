//! What:
//!  Where the probe's record lives and how it is written:
//!  a root-owned directory of mode `0755` under `/run`,
//!  the record file at mode `0644`,
//!  the started record before the fork,
//!  and the atomic rename that replaces it.
//! Why:
//!  The add preflight and `doctor` read the file,
//!  and a half-written record would be read as an answer,
//!  so every write goes to a sibling name and is renamed over the record in
//!  one step
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "Where the verdict is enforced,
//! and where the record lives").
//!  The directory is the mode's own rather than the unit's
//!  `RuntimeDirectory=`,
//!  which systemd removes when the unit dies and which would take the record
//!  with it exactly when a failed arm's verdict is what a reader needs.
//!  `scratch` and `holder` sit beside the record as the two mount points the
//!  children use;
//!  both mounts live in a child namespace,
//!  so the host sees empty directories.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const place = RecordPlace.live(); place.prepare(); place.publish(resultText);
//! ```

use std::fs::{
    File,
    OpenOptions,
};
use std::io::Write;
use std::os::unix::fs::{
    OpenOptionsExt,
    PermissionsExt,
};
use std::path::{
    Path,
    PathBuf,
};

/// The directory the probe's record and the first real attach's outcome live in.
pub const RECORD_DIRECTORY: &str = "/run/git-worktree-clone-mount-probe";
/// The record's file name inside that directory.
pub const RESULT_NAME: &str = "result";
/// The sibling the record is staged under before its rename,
/// so a reader never sees half a record.
/// The process's own id follows it,
/// because two probes at once (a manual run beside the unit's) must not
/// truncate each other's staging file.
pub const STAGING_NAME: &str = "result.staging";
/// The mount point the worker mounts its tmpfs on.
pub const SCRATCH_NAME: &str = "scratch";
/// The directory the holder attaches its own mount at.
pub const HOLDER_NAME: &str = "holder";

/// The directory's mode:
/// readable by everyone so an account-side reader can list it,
/// writable by root only.
pub const DIRECTORY_MODE: u32 = 0o755;
/// The record's mode:
/// readable by everyone,
/// written by root only.
pub const RECORD_MODE: u32 = 0o644;

/// What:
///  Why the record could not be written.
/// Why:
///   A boot that cannot write its record must say so rather than leave a
///  silent absence,
///  because an absent record already refuses the managed route and a person
///  needs to know which of the two happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordFileRefusal = { kind: 'directory' | 'publish'; path: string; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordFileRefusal {
    /// The record's directory could not be made or given its mode.
    Directory {
        /// The path.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
    /// The record could not be staged or renamed.
    Publish {
        /// The path.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
}

impl RecordFileRefusal {
    /// The refusal in words,
    /// naming the path and the way forward.
    pub fn message(&self) -> String {
        return match self {
            RecordFileRefusal::Directory {
                path,
                reason,
            } => format!(
                "the probe could not make its record directory {} ({reason}), so this boot has no record and every managed attach stays on the reflink or ordinary Git route. Ways forward: make {} writable by root, then run the probe again.",
                path.display(),
                path.parent().unwrap_or(path).display()
            ),
            RecordFileRefusal::Publish {
                path,
                reason,
            } => format!(
                "the probe could not write its record at {} ({reason}), so this boot has no record and every managed attach stays on the reflink or ordinary Git route. Ways forward: make the record directory writable by root and start the probe again.",
                path.display()
            ),
        };
    }
}

/// What:
///  Where one boot's record and its two mount points live.
/// Why:
///   The live place is one path,
///  and a control hands a directory of its own in the same shape,
///  so the mode's writes are exercised without `/run` or root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RecordPlace { constructor(readonly directory: string) {} result(): string { return join(this.directory, 'result'); } }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordPlace {
    /// The directory that holds the record.
    directory: PathBuf,
}

impl RecordPlace {
    /// The place a probe at boot writes.
    pub fn live() -> Self {
        return RecordPlace {
            directory: PathBuf::from(RECORD_DIRECTORY),
        };
    }

    /// A place at the given directory,
    /// which a control names.
    pub fn at(directory: PathBuf) -> Self {
        return RecordPlace {
            directory,
        };
    }

    /// The directory.
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// The record's own path.
    pub fn result(&self) -> PathBuf {
        return self.directory.join(RESULT_NAME);
    }

    /// This writer's own staging path,
    /// named for its process so two probes at once keep separate files.
    pub fn staging(&self) -> PathBuf {
        return self
            .directory
            .join(format!("{STAGING_NAME}.{}", std::process::id()));
    }

    /// The worker's scratch mount point.
    pub fn scratch(&self) -> PathBuf {
        return self.directory.join(SCRATCH_NAME);
    }

    /// The holder's mount point.
    pub fn holder(&self) -> PathBuf {
        return self.directory.join(HOLDER_NAME);
    }

    /// What:
    ///  Make the directory and its two mount points,
    ///  each at mode `0755`,
    ///  keeping one that is already there.
    /// Why:
    ///   The probe begins by creating what its children mount inside,
    ///  and a second run in the same boot keeps the record it finds until the
    ///  moment it replaces it.
    ///
    /// # Errors
    /// A directory could not be made or given its mode.
    pub fn prepare(&self) -> Result<(), RecordFileRefusal> {
        for path in [self.directory.clone(), self.scratch(), self.holder()] {
            if let Err(error) = std::fs::create_dir_all(path.as_path()) {
                return Err(RecordFileRefusal::Directory {
                    path,
                    reason: error.to_string(),
                });
            }
            if let Err(error) = std::fs::set_permissions(
                path.as_path(),
                std::fs::Permissions::from_mode(DIRECTORY_MODE),
            ) {
                return Err(RecordFileRefusal::Directory {
                    path,
                    reason: error.to_string(),
                });
            }
        }
        return Ok(());
    }

    /// What:
    ///  Publish one record:
    ///  write it at the staging name and mode `0644`,
    ///  sync it,
    ///  rename it over the record,
    ///  and sync the directory.
    /// Why:
    ///   A reader sees the old record or the whole new one,
    ///  never a mixture,
    ///  and the rename is the one step that makes the new one visible.
    ///
    /// # Errors
    /// The staging write,
    /// its sync or the rename answered an error.
    pub fn publish(
        &self,
        text: &str,
    ) -> Result<(), RecordFileRefusal> {
        let staging: PathBuf = self.staging();
        let staged: Result<(), String> = (|| {
            let mut file: File = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(RECORD_MODE)
                .open(staging.as_path())
                .map_err(|error| return error.to_string())?;
            file.set_permissions(std::fs::Permissions::from_mode(RECORD_MODE))
                .map_err(|error| return error.to_string())?;
            file.write_all(text.as_bytes())
                .map_err(|error| return error.to_string())?;
            file.sync_all().map_err(|error| return error.to_string())?;
            std::fs::rename(staging.as_path(), self.result().as_path())
                .map_err(|error| return error.to_string())?;
            let directory: File =
                File::open(self.directory.as_path()).map_err(|error| return error.to_string())?;
            return directory
                .sync_all()
                .map_err(|error| return error.to_string());
        })();
        return match staged {
            Ok(()) => Ok(()),
            Err(reason) => Err(RecordFileRefusal::Publish {
                path: self.result(),
                reason,
            }),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_record_files_tests.rs"]
mod tests;
