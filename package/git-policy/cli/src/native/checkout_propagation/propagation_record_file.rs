//! What:
//!  Where one run's propagation record lives and how it is written:
//!  a root-owned `0755` directory under `/run`,
//!  its `result` file at `0644`,
//!  one whole record published by a staging sibling and a single rename,
//!  and the read that proves the directory and the file before either is
//!  believed.
//! Why:
//!  The record is one answer for one checkout,
//!  so it is replaced whole rather than appended to,
//!  and a reader sees the whole earlier answer or the whole new one
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "What it writes",
//!  which takes this shape from the readiness record the existing route
//!  publishes).
//!  Two runs write the same facts about the same instance and the last rename
//!  wins whole,
//!  so there is no lock file anywhere (rule `NLK`).
//!  A refusal is a record too,
//!  which is why it outlives the readiness gate instead of living inside it
//!  (choice V1).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const file = PropagationFile.live(); file.publish(record); const read = file.read();
//! ```

use crate::checkout_propagation::propagation_record::PropagationRecord;
use crate::checkout_propagation::propagation_record_file_refusals::PropagationFileRefusal;
use crate::checkout_propagation::propagation_record_parse::parse_record;
use crate::checkout_propagation::propagation_words::{
    MAXIMUM_PROPAGATION_BYTES,
    PROPAGATION_DIRECTORY,
    PROPAGATION_DIRECTORY_MODE,
    PROPAGATION_MODE,
    PROPAGATION_NAME,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_calls::{
    c_string_of,
    create_at,
    open_receipt_at,
    rename_at,
    sync_directory,
    unlink_at,
};
use crate::worktree_clone_mount_protocol::root_owned_directory::{
    RootDirectoryRefusal,
    make_and_prove,
};
use std::ffi::CString;
use std::io::{
    Read,
    Write,
};
use std::os::fd::{
    FromRawFd,
    OwnedFd,
};
use std::os::unix::fs::{
    MetadataExt,
    PermissionsExt,
};
use std::path::PathBuf;
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

/// The sibling a record is staged under before its rename.
const STAGING_NAME: &str = "result.staging";

/// What:
///  The staging name one publish writes under.
/// Why:
///  The name carries this process's id and a counter of this process's own
///  attempts,
///  so two publishes in one process never share a staging name,
///  and a writer killed before its rename leaves a name no later attempt
///  reuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staging = `result.staging.${process.pid}.${nextAttempt()}`;
/// ```
fn staging_name() -> String {
    let attempt: u64 = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
    return format!("{STAGING_NAME}.{}.{attempt}", std::process::id());
}

/// Distinguishes the staging names of two publishes one process makes.
static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// What:
///  Where one run's record lives and which account must own it.
/// Why:
///  The live place requires root,
///  and a control hands in a fixture directory it owns,
///  so the publish and the read are exercised without `/run`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PropagationFile { constructor(readonly directory: string, readonly owner: number) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropagationFile {
    /// The directory the record lives in.
    directory: PathBuf,
    /// The account the directory and the record must belong to.
    owner: u32,
}

impl PropagationFile {
    /// The place the release handler publishes in.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const live = () => new PropagationFile('/run/git-worktree-clone-mount-propagation', 0);
    /// ```
    pub fn live() -> Self {
        return PropagationFile::at(PathBuf::from(PROPAGATION_DIRECTORY), 0);
    }

    /// A place at a fixture directory,
    /// owned by the account the control expects.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const at = (directory: string, owner: number) => new PropagationFile(directory, owner);
    /// ```
    pub fn at(
        directory: PathBuf,
        owner: u32,
    ) -> Self {
        return PropagationFile {
            directory,
            owner,
        };
    }

    /// The record's own path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const path = () => join(this.directory, 'result');
    /// ```
    pub fn record(&self) -> PathBuf {
        return self.directory.join(PROPAGATION_NAME);
    }

    /// What:
    ///  Publish one whole record,
    ///  replacing the earlier one atomically.
    /// Why:
    ///  The directory is made when a fresh boot has none,
    ///  the text is proved readable by the route's own parser before a byte is
    ///  written,
    ///  and every way out after the staging sibling was made removes it,
    ///  so this process's next publish is never blocked by its own failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// file.publish(record); // create, write, fsync, rename, fsync
    /// ```
    ///
    /// # Errors
    /// The directory is not the designed root-owned folder,
    /// the text is not one this route's parser accepts,
    /// or the staging write,
    /// its sync or the rename answered an error.
    pub fn publish(
        &self,
        record: &PropagationRecord,
    ) -> Result<(), PropagationFileRefusal> {
        let text: String = match parse_record(record.text().as_str()) {
            Ok(parsed) if parsed == *record => record.text(),
            Ok(_) => return Err(PropagationFileRefusal::NotWhatWasWritten),
            Err(refusal) => {
                return Err(PropagationFileRefusal::NotARecord {
                    refusal,
                });
            }
        };
        let directory: std::fs::File = make_and_prove(
            self.directory.as_path(),
            self.owner,
            PROPAGATION_DIRECTORY_MODE,
        )
        .map_err(PropagationFileRefusal::Directory)?;
        write_the_record(&directory, staging_name().as_str(), text.as_str())?;
        // The writer proves the record it published is one its own reader reads back, so a field the grammar refuses
        // never stands as the answer a person looks up.
        return match self.read()? {
            Some(read_back) if read_back == *record => Ok(()),
            _ => Err(PropagationFileRefusal::NotWhatWasWritten),
        };
    }

    /// Read the record,
    /// or nothing when none was published.
    /// A directory that is not there is no record yet,
    /// which is what a boot that has not run the handler answers,
    /// and this read never makes one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const record = file.read(); // undefined when none was published
    /// ```
    ///
    /// # Errors
    /// The directory or the record is not the designed root-owned file,
    /// or the text is not a record this route wrote.
    pub fn read(&self) -> Result<Option<PropagationRecord>, PropagationFileRefusal> {
        let Some(opened) = self.open_to_read()? else {
            return Ok(None);
        };
        let Some(file) = open_receipt_at(&opened, PROPAGATION_NAME).map_err(|error| {
            return PropagationFileRefusal::File {
                reason: error.to_string(),
            };
        })?
        else {
            return Ok(None);
        };
        return read_the_bytes(&file, self.owner);
    }

    /// Open the directory to read it,
    /// proving it is the designed root-owned folder,
    /// and answering nothing when it is not there at all.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const opened = openToRead('/run/git-worktree-clone-mount-propagation', 0); // undefined when absent
    /// ```
    ///
    /// # Errors
    /// The directory could not be opened,
    /// or is not the designed root-owned folder.
    fn open_to_read(&self) -> Result<Option<std::fs::File>, PropagationFileRefusal> {
        let text: CString = c_string_of(self.directory.as_path()).map_err(|reason| {
            return PropagationFileRefusal::Directory(RootDirectoryRefusal::Open {
                reason,
            });
        })?;
        // SAFETY: `text` is a NUL-terminated string that outlives the call.
        let descriptor: libc::c_int = unsafe {
            libc::open(
                text.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(PropagationFileRefusal::Directory(
                RootDirectoryRefusal::Open {
                    reason: error.to_string(),
                },
            ));
        }
        // SAFETY: `open` returned a new descriptor that nothing else owns.
        let opened: std::fs::File =
            std::fs::File::from(unsafe { OwnedFd::from_raw_fd(descriptor) });
        let facts: std::fs::Metadata = opened.metadata().map_err(|error| {
            return PropagationFileRefusal::Directory(RootDirectoryRefusal::Open {
                reason: error.to_string(),
            });
        })?;
        if facts.uid() != self.owner {
            return Err(PropagationFileRefusal::Directory(
                RootDirectoryRefusal::Owner {
                    found: facts.uid(),
                    wanted: self.owner,
                },
            ));
        }
        let permission_bits: u32 = facts.mode() & 0o7777;
        if permission_bits != PROPAGATION_DIRECTORY_MODE {
            return Err(PropagationFileRefusal::Directory(
                RootDirectoryRefusal::Mode {
                    found: permission_bits,
                    wanted: PROPAGATION_DIRECTORY_MODE,
                },
            ));
        }
        return Ok(Some(opened));
    }
}

/// What:
///  The bytes of one record file,
///  proved regular,
///  owned,
///  at its mode and bounded,
///  and parsed by the route's own parser.
/// Why:
///  Every proof the directory's reader makes is made again for the file,
///  because the account could replace the file inside a directory root owns,
///  and the bound is proved before the bytes are parsed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = readTheBytes(file, 0); // throws on owner, mode, length or shape
/// ```
///
/// # Errors
/// The file is not the designed root-owned regular file,
/// or its text is not a record this route wrote.
fn read_the_bytes(
    file: &std::fs::File,
    owner: u32,
) -> Result<Option<PropagationRecord>, PropagationFileRefusal> {
    let facts: std::fs::Metadata = file.metadata().map_err(|error| {
        return PropagationFileRefusal::File {
            reason: error.to_string(),
        };
    })?;
    if !facts.is_file() {
        return Err(PropagationFileRefusal::NotRegular);
    }
    if facts.uid() != owner {
        return Err(PropagationFileRefusal::RecordOwner {
            found: facts.uid(),
            wanted: owner,
        });
    }
    let permission_bits: u32 = facts.mode() & 0o7777;
    if permission_bits != PROPAGATION_MODE {
        return Err(PropagationFileRefusal::RecordMode {
            found: permission_bits,
        });
    }
    if facts.len() > MAXIMUM_PROPAGATION_BYTES as u64 {
        return Err(PropagationFileRefusal::TooLarge {
            bytes: facts.len(),
        });
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(facts.len() as usize);
    let mut bounded: std::fs::File = file.try_clone().map_err(|error| {
        return PropagationFileRefusal::File {
            reason: error.to_string(),
        };
    })?;
    Read::by_ref(&mut bounded)
        .take(MAXIMUM_PROPAGATION_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            return PropagationFileRefusal::File {
                reason: error.to_string(),
            };
        })?;
    if bytes.len() > MAXIMUM_PROPAGATION_BYTES {
        return Err(PropagationFileRefusal::TooLarge {
            bytes: bytes.len() as u64,
        });
    }
    let Ok(text) = String::from_utf8(bytes) else {
        return Err(PropagationFileRefusal::NotText);
    };
    return match parse_record(text.as_str()) {
        Ok(record) => Ok(Some(record)),
        Err(refusal) => Err(PropagationFileRefusal::NotARecord {
            refusal,
        }),
    };
}

/// What:
///  The staging write and the rename one publish makes,
///  through the descriptor of the directory already proved.
/// Why:
///  Every writer of a root-owned record shares one shape,
///  and the shape is here rather than in the caller so the directory's own
///  sync and the removal of a failed sibling cannot be forgotten once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeTheRecord(directory, staging, text); // create, write, fsync, rename, fsync, remove on failure
/// ```
///
/// # Errors
/// The staging file could not be made or written,
/// its sync or the rename answered an error,
/// or the directory's own sync answered one.
pub fn write_the_record(
    directory: &std::fs::File,
    staging: &str,
    text: &str,
) -> Result<(), PropagationFileRefusal> {
    let staged: Result<(), PropagationFileRefusal> = (|| {
        let mut file: std::fs::File =
            create_at(directory, staging, PROPAGATION_MODE).map_err(|error| {
                return PropagationFileRefusal::Staging {
                    reason: error.to_string(),
                };
            })?;
        file.set_permissions(std::fs::Permissions::from_mode(PROPAGATION_MODE))
            .map_err(|error| {
                return PropagationFileRefusal::Staging {
                    reason: error.to_string(),
                };
            })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            return PropagationFileRefusal::Staging {
                reason: error.to_string(),
            };
        })?;
        file.sync_all().map_err(|error| {
            return PropagationFileRefusal::Staging {
                reason: error.to_string(),
            };
        })?;
        rename_at(directory, staging, PROPAGATION_NAME).map_err(|error| {
            return PropagationFileRefusal::Staging {
                reason: error.to_string(),
            };
        })?;
        return sync_directory(directory).map_err(|error| {
            return PropagationFileRefusal::Staging {
                reason: error.to_string(),
            };
        });
    })();
    if staged.is_err() {
        // A rename may have completed before a failed directory sync, and then there is no sibling to remove.
        let _removed: std::io::Result<()> = unlink_at(directory, staging);
    }
    return staged;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "propagation_record_file_tests.rs"]
mod tests;
