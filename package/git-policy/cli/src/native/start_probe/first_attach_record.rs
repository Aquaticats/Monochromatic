//! What:
//!  The `first-attach` file the start-probe design gives plan step 8:
//!  this boot's first real clone attach as an operation outcome,
//!  beside the probe's record in the same root-owned `/run` directory,
//!  mode `0644`,
//!  claimed by one exclusive creation and published by one atomic rename.
//! Why:
//!  The project answered "Write first-attach" and on 2026-11-22 chose that step
//!  8 builds the writer and the call its design gives it
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "The `first-attach` writer";
//!  `doc/decision/cli-git-zfs-start-probe.md`,
//!  Q7 and Q-b;
//!  `doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The first real attach's record").
//!  The file keeps the outcome no probe can produce:
//!  a real clone,
//!  at a real slot,
//!  in the host mount namespace.
//!  `first` means first to claim the name,
//!  not first to attach,
//!  so the claim is one exclusive creation that writes the started record,
//!  and a process that finds the name answers `EEXIST` and writes nothing.
//!  The outcome is staged beside the name and renamed over it in one step,
//!  so a reader sees the started record or the whole outcome.
//!  The claim is per boot because the directory is on `/run`;
//!  the outcome also carries the boot identifier,
//!  and a reader whose boot differs reads the file as stale.
//!  Nothing gates on this file:
//!  its `kind` line says `operation-outcome`,
//!  it carries no verdict,
//!  and `doctor` prints it under its own label once plan step 18 adds the
//!  reader.
//!  The record's words and its key order are in `first_attach_record_text.rs`,
//!  beside this file,
//!  and every caller keeps naming them through this module.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = FirstAttachPlace.live().claim(fingerprint, time); // { state: 'started' }
//! place.record(claim, outcomeText); // one rename over `first-attach`
//! ```

use super::probe_fingerprint::ProbeFingerprint;
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

/// What:
///  The record's words and its text,
///  re-exported so the writer and the controls keep naming this module.
/// Why:
///  A split of a file may not move a caller's path to what it names,
///  so the words this file's records are written with are reachable here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './first_attach_record_text';
/// ```
pub use super::first_attach_record_text::{
    ATTACHED,
    FINISHED,
    FIRST_ATTACH_KIND,
    FIRST_ATTACH_OPERATION,
    FIRST_ATTACH_VERSION,
    FirstAttachOutcome,
    NONE,
    REFUSED,
    STARTED,
    outcome_record,
    started_record,
};

/// The file beside the probe's record that keeps this boot's first real attach.
pub const FIRST_ATTACH_NAME: &str = "first-attach";

/// The file's mode:
/// readable by everyone,
/// written by root only.
pub const FIRST_ATTACH_MODE: u32 = 0o644;

/// The directory's mode,
/// which is the probe's own.
pub const DIRECTORY_MODE: u32 = 0o755;

/// The bound the whole record is held to,
/// so a value of a length this route does not expect cannot fill it.
pub const RECORD_BOUND: usize = 4_096;
/// What:
///  Why the claim or the outcome could not be written.
/// Why:
///  A boot that cannot keep its first attach must say so in the handler's
///  journal rather than write half a record a reader would take as whole,
///  and a name another process already claimed is not an error at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FirstAttachRefusal = { kind: 'already-claimed' | 'directory' | 'publish'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FirstAttachRefusal {
    /// Another process claimed this boot's name first,
    /// so this attempt writes nothing.
    AlreadyClaimed {
        /// The file's path.
        path: PathBuf,
    },
    /// The directory could not be made or given its mode.
    Directory {
        /// The path.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
    /// The claim or the outcome could not be written.
    Publish {
        /// The path.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
}

impl FirstAttachRefusal {
    /// The refusal in words,
    /// naming the path and the way forward.
    pub fn message(&self) -> String {
        return match self {
            FirstAttachRefusal::AlreadyClaimed {
                path,
            } => {
                format!(
                    "another process of this boot claimed the first attach's record {} before this one, so this attempt's outcome is in the handler's journal alone",
                    path.display()
                )
            }
            FirstAttachRefusal::Directory {
                path,
                reason,
            } => format!(
                "the first attach's record directory {} could not be made at its own mode ({reason}), so this boot keeps no first attach. Ways forward: make the directory writable by root and run the sweep again.",
                path.display()
            ),
            FirstAttachRefusal::Publish {
                path,
                reason,
            } => {
                format!(
                    "the first attach's record at {} could not be written ({reason}), so this boot keeps a started record or none. Ways forward: make the directory writable by root and run the sweep again.",
                    path.display()
                )
            }
        };
    }
}

/// What:
///  One claimed `first-attach` name,
///  held by the process that won it.
/// Why:
///  The claim is the exclusive creation,
///  so one value names the file the winner may replace and no other process
///  writes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FirstAttachClaim = { path: string; fingerprint: ProbeFingerprint };
/// ```
#[derive(Debug)]
pub struct FirstAttachClaim {
    /// The claimed file.
    path: PathBuf,
    /// The build and machine facts the started record carried.
    fingerprint: ProbeFingerprint,
}

impl FirstAttachClaim {
    /// The claimed file's path.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// The fingerprint the started record carried.
    pub fn fingerprint(&self) -> &ProbeFingerprint {
        return &self.fingerprint;
    }
}

/// What:
///  Where this boot's `first-attach` file lives.
/// Why:
///  The live place is beside the probe's record,
///  and a control hands in a directory of its own,
///  so the claim and the rename are exercised without `/run` or root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = FirstAttachPlace.live();
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirstAttachPlace {
    /// The directory that holds the record.
    directory: PathBuf,
}

impl FirstAttachPlace {
    /// The place the handler writes at boot.
    pub fn live() -> Self {
        return FirstAttachPlace {
            directory: PathBuf::from(super::probe_record_files::RECORD_DIRECTORY),
        };
    }

    /// A place at the given directory,
    /// which a control names.
    pub fn at(directory: PathBuf) -> Self {
        return FirstAttachPlace {
            directory,
        };
    }

    /// The file's own path.
    pub fn path(&self) -> PathBuf {
        return self.directory.join(FIRST_ATTACH_NAME);
    }

    /// This writer's own staging path,
    /// named for its process so nothing truncates another file.
    pub fn staging(&self) -> PathBuf {
        return self.directory.join(format!(
            "{FIRST_ATTACH_NAME}.staging.{}",
            std::process::id()
        ));
    }

    /// What:
    ///  Make the directory at the probe record's own mode when it is absent,
    ///  then claim the name with one exclusive creation that writes the started
    ///  record.
    /// Why:
    ///  The claim is the kernel's own exclusive creation,
    ///  so exactly one process of this boot wins it,
    ///  and the started record is already there when the winner goes on to the
    ///  walk.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const claim = place.claim(fingerprint, time); // EEXIST answers already-claimed
    /// ```
    ///
    /// # Errors
    /// The directory could not be made,
    /// or the name was already claimed or could not be created.
    pub fn claim(
        &self,
        fingerprint: &ProbeFingerprint,
        time: &str,
    ) -> Result<FirstAttachClaim, FirstAttachRefusal> {
        if let Err(error) = std::fs::create_dir_all(self.directory.as_path()) {
            return Err(FirstAttachRefusal::Directory {
                path: self.directory.clone(),
                reason: error.to_string(),
            });
        }
        if let Err(error) = std::fs::set_permissions(
            self.directory.as_path(),
            std::fs::Permissions::from_mode(DIRECTORY_MODE),
        ) {
            return Err(FirstAttachRefusal::Directory {
                path: self.directory.clone(),
                reason: error.to_string(),
            });
        }
        let path: PathBuf = self.path();
        let mut file: File = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(FIRST_ATTACH_MODE)
            .open(path.as_path())
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err(FirstAttachRefusal::AlreadyClaimed {
                    path,
                });
            }
            Err(error) => {
                return Err(FirstAttachRefusal::Publish {
                    path,
                    reason: error.to_string(),
                });
            }
        };
        let written: Result<(), String> = (|| {
            file.set_permissions(std::fs::Permissions::from_mode(FIRST_ATTACH_MODE))
                .map_err(|error| return error.to_string())?;
            file.write_all(started_record(fingerprint, time).as_bytes())
                .map_err(|error| return error.to_string())?;
            file.sync_all().map_err(|error| return error.to_string())?;
            return sync_the_directory(self.directory.as_path());
        })();
        return match written {
            Ok(()) => Ok(FirstAttachClaim {
                path,
                fingerprint: fingerprint.clone(),
            }),
            Err(reason) => Err(FirstAttachRefusal::Publish {
                path,
                reason,
            }),
        };
    }

    /// What:
    ///  Publish one outcome over the claimed name:
    ///  stage it at this process's own name and mode `0644`,
    ///  sync it,
    ///  rename it over the claim,
    ///  and sync the directory.
    /// Why:
    ///  A reader sees the started record or the whole outcome,
    ///  never a mixture,
    ///  and the rename is the one step that makes the outcome visible.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// place.record(claim, outcomeText);
    /// ```
    ///
    /// # Errors
    /// The staging write,
    /// its sync or the rename answered an error.
    pub fn record(
        &self,
        claim: &FirstAttachClaim,
        text: &str,
    ) -> Result<(), FirstAttachRefusal> {
        if text.len() > RECORD_BOUND {
            return Err(FirstAttachRefusal::Publish {
                path: claim.path.clone(),
                reason: format!(
                    "the record is longer than the {RECORD_BOUND} bytes a reader takes"
                ),
            });
        }
        let staging: PathBuf = self.staging();
        let staged: Result<(), String> = (|| {
            let mut file: File = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(FIRST_ATTACH_MODE)
                .open(staging.as_path())
                .map_err(|error| return error.to_string())?;
            file.set_permissions(std::fs::Permissions::from_mode(FIRST_ATTACH_MODE))
                .map_err(|error| return error.to_string())?;
            file.write_all(text.as_bytes())
                .map_err(|error| return error.to_string())?;
            file.sync_all().map_err(|error| return error.to_string())?;
            std::fs::rename(staging.as_path(), claim.path.as_path())
                .map_err(|error| return error.to_string())?;
            return sync_the_directory(self.directory.as_path());
        })();
        return match staged {
            Ok(()) => Ok(()),
            Err(reason) => Err(FirstAttachRefusal::Publish {
                path: claim.path.clone(),
                reason,
            }),
        };
    }
}

/// The directory's own entry,
/// synced so the rename survives a crash.
fn sync_the_directory(directory: &Path) -> Result<(), String> {
    let folder: File = File::open(directory).map_err(|error| return error.to_string())?;
    return folder.sync_all().map_err(|error| return error.to_string());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "first_attach_record_tests.rs"]
mod tests;
