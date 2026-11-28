//! What:
//!  How root's holder scan reaches the account's wrapper:
//!  the second root-owned file beside the receipt,
//!  its name,
//!  its mode and owner,
//!  the request its content must answer,
//!  and the refusal a file that never arrived is.
//! Why:
//!  The wrapper needs root's finding to name the holders in its report and in
//!  `doctor`,
//!  to show the caller's protected set among the processes the removal spares,
//!  and to know whether a scan was clear before the destroy that follows it,
//!  and the receipt cannot carry it:
//!  its grammar has fixed keys and no holder field
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "How root's finding reaches the wrapper",
//!  V1 and V2).
//!  The file holds the scan's own text,
//!  which already has a writer,
//!  a parser and a bound,
//!  so this reader adds the two facts the design puts beside it:
//!  root published it before the receipt,
//!  and a missing file is a missing answer and never a clear one.
//!  A receipt whose nonce matches and whose scan file is missing is therefore a
//!  refusal of the removal at that step,
//!  which is what V2 adopts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scan = readTheScanFile(ScanFile.live(), dataset, nonce);  // throws when the file is missing or foreign
//! ```

use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::clone_holder_scan::holder_scan_acceptance::{
    ScanExpectation,
    StaleResult,
};
use crate::clone_holder_scan::holder_scan_parse::parse_holder_scan;
use crate::clone_holder_scan::holder_scan_refusals::HolderScanRefusal;
use crate::clone_holder_scan::holder_scan_text::MAXIMUM_RESULT_BYTES;
use crate::worktree_clone_forced_removal::forced_removal_words::SCAN_FILE_SUFFIX;
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::{
    DIRECTORY_MODE,
    RECEIPT_MODE,
    RESULTS_DIRECTORY,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::io::Read;
use std::os::unix::fs::{
    MetadataExt,
    OpenOptionsExt,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Why the scan file did not answer the request at hand.
/// Why:
///  A missing file,
///  a file another account owns,
///  a file of the wrong mode,
///  a file that is not a scan result,
///  and a result for another request are five different things for a person to
///  look at,
///  and only the last is the route's own stale-answer refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScanFileRefusal = { kind: 'absent' | 'folder' | 'owner' | 'mode' | 'too-long' | 'not-a-result' | 'stale'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanFileRefusal {
    /// The results folder could not be opened or is not the designed one.
    Folder {
        /// The folder's path.
        path: PathBuf,
        /// What the call or the facts answered.
        reason: String,
    },
    /// The dataset's scan file is not there.
    /// Why:
    ///  A missing file is a missing answer,
    /// so the removal refuses at the step that needed it
    ///  rather than reading the receipt as a clear scan.
    Absent {
        /// The file's path.
        path: PathBuf,
    },
    /// The file is not a regular file.
    NotRegular {
        /// The file's path.
        path: PathBuf,
    },
    /// The file is owned by another account than the one that must write it.
    Owner {
        /// The file's path.
        path: PathBuf,
        /// The owner found.
        found: u32,
        /// The owner the route requires.
        wanted: u32,
    },
    /// The file does not carry its designed mode.
    Mode {
        /// The file's path.
        path: PathBuf,
        /// The mode found.
        found: u32,
        /// The mode the route requires.
        wanted: u32,
    },
    /// The file is longer than a result may be.
    TooLong {
        /// The file's path.
        path: PathBuf,
    },
    /// The file could not be read.
    Unreadable {
        /// The file's path.
        path: PathBuf,
        /// What the call answered.
        reason: String,
    },
    /// The text is not one the scan's own reader accepts.
    NotAResult {
        /// The file's path.
        path: PathBuf,
        /// The reader's own refusal.
        refusal: HolderScanRefusal,
    },
    /// The result answers another dataset or another request.
    Stale {
        /// The file's path.
        path: PathBuf,
        /// The scan's own stale-answer refusal.
        stale: StaleResult,
    },
}

/// What:
///  The root-owned folder the scan file lives in,
///  and the account that must own it.
/// Why:
///  A control hands in a fixture directory it owns,
///  and the handler's own is `/run`'s,
///  so the folder is a value rather than a constant in the reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ScanFile { constructor(readonly folder: string, readonly owner: number) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanFile {
    /// The results folder,
    /// which is the receipt's own.
    folder: PathBuf,
    /// The account the folder and the file must belong to.
    owner: u32,
}

impl ScanFile {
    /// The folder root publishes the scan's answer in,
    /// beside the receipt.
    pub fn live() -> ScanFile {
        return ScanFile {
            folder: PathBuf::from(RESULTS_DIRECTORY),
            owner: 0,
        };
    }

    /// A folder at a fixture directory,
    /// owned by the account a control expects.
    pub fn at(
        folder: PathBuf,
        owner: u32,
    ) -> ScanFile {
        return ScanFile {
            folder,
            owner,
        };
    }

    /// The folder.
    pub fn folder(&self) -> &Path {
        return self.folder.as_path();
    }

    /// The file one dataset's scan is filed under:
    /// the canonical numeric GUID and the scan's own suffix.
    /// Why:
    ///  The GUID is written in the file and never in its name alone,
    ///  and the suffix keeps the scan apart from the receipt of the same GUID,
    ///  so one result never overwrites the other.
    pub fn entry_name(
        &self,
        dataset: DatasetGuid,
    ) -> String {
        return format!("{}{SCAN_FILE_SUFFIX}", dataset.file_name());
    }

    /// What:
    ///  Read the scan file one request's answer must be in.
    /// Why:
    ///  The folder and the file are proved before their bytes are read,
    ///  the read is bounded by the result's own bound,
    ///  and the parsed result must answer the GUID and the nonce the caller
    ///  holds,
    ///  so a file for another clone,
    ///  or for an earlier scan of this clone,
    ///  is refused by name before anything it holds is read.
    ///
    /// # Errors
    /// The folder is not the designed root-owned `0755` one,
    /// the file is absent or not a root-owned `0644` regular file,
    /// the text is not a result,
    /// or the result answers another dataset or request.
    pub fn read_the_scan_file(
        &self,
        dataset: DatasetGuid,
        nonce: &RequestNonce,
    ) -> Result<HolderScan, ScanFileRefusal> {
        let directory: std::fs::File = self.open_the_folder()?;
        let name: String = self.entry_name(dataset);
        let path: PathBuf = self.folder.join(name.as_str());
        // The entry is opened relative to the folder descriptor that was proved, so a path swapped after the proof
        // cannot steer the read, which is the receipt's own boundary.
        let opened: Result<std::fs::File, std::io::Error> = open_at(&directory, name.as_str());
        let mut file: std::fs::File = match opened {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(ScanFileRefusal::Absent {
                    path,
                });
            }
            Err(error) => {
                return Err(ScanFileRefusal::Unreadable {
                    path,
                    reason: error.to_string(),
                });
            }
        };
        drop(directory);
        let facts: std::fs::Metadata = file.metadata().map_err(|error| {
            return ScanFileRefusal::Unreadable {
                path: path.clone(),
                reason: error.to_string(),
            };
        })?;
        if !facts.is_file() {
            return Err(ScanFileRefusal::NotRegular {
                path,
            });
        }
        if facts.uid() != self.owner {
            return Err(ScanFileRefusal::Owner {
                path,
                found: facts.uid(),
                wanted: self.owner,
            });
        }
        if facts.mode() & 0o7777 != RECEIPT_MODE {
            return Err(ScanFileRefusal::Mode {
                path,
                found: facts.mode() & 0o7777,
                wanted: RECEIPT_MODE,
            });
        }
        let mut bytes: Vec<u8> = Vec::with_capacity(MAXIMUM_RESULT_BYTES.min(facts.len() as usize));
        let bound: u64 = u64::try_from(MAXIMUM_RESULT_BYTES)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let read: std::io::Result<usize> = (&mut file).take(bound).read_to_end(&mut bytes);
        read.map_err(|error| {
            return ScanFileRefusal::Unreadable {
                path: path.clone(),
                reason: error.to_string(),
            };
        })?;
        if bytes.len() > MAXIMUM_RESULT_BYTES {
            return Err(ScanFileRefusal::TooLong {
                path,
            });
        }
        let text: String = match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_error) => {
                return Err(ScanFileRefusal::Unreadable {
                    path,
                    reason: String::from("the file is not text this reader can read"),
                });
            }
        };
        let scan: HolderScan = parse_holder_scan(text.as_str()).map_err(|refusal| {
            return ScanFileRefusal::NotAResult {
                path: path.clone(),
                refusal,
            };
        })?;
        scan.accept(&ScanExpectation {
            dataset,
            nonce,
        })
        .map_err(|stale| {
            return ScanFileRefusal::Stale {
                path: path.clone(),
                stale,
            };
        })?;
        return Ok(scan);
    }

    /// Open the folder with no link followed,
    /// and prove it is the designed one.
    fn open_the_folder(&self) -> Result<std::fs::File, ScanFileRefusal> {
        let path: &Path = self.folder.as_path();
        let opened: Result<std::fs::File, std::io::Error> = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(path);
        let folder: std::fs::File = opened.map_err(|error| {
            return ScanFileRefusal::Folder {
                path: path.to_path_buf(),
                reason: error.to_string(),
            };
        })?;
        let facts: std::fs::Metadata = folder.metadata().map_err(|error| {
            return ScanFileRefusal::Folder {
                path: path.to_path_buf(),
                reason: error.to_string(),
            };
        })?;
        if !facts.is_dir() {
            return Err(ScanFileRefusal::Folder {
                path: path.to_path_buf(),
                reason: String::from("it is not a directory"),
            });
        }
        if facts.uid() != self.owner {
            return Err(ScanFileRefusal::Folder {
                path: path.to_path_buf(),
                reason: format!(
                    "it is owned by account {} and the route requires {}",
                    facts.uid(),
                    self.owner
                ),
            });
        }
        if facts.mode() & 0o7777 != DIRECTORY_MODE {
            return Err(ScanFileRefusal::Folder {
                path: path.to_path_buf(),
                reason: format!(
                    "its mode is {:o} and the route requires {DIRECTORY_MODE:o}",
                    facts.mode() & 0o7777
                ),
            });
        }
        return Ok(folder);
    }
}

/// What:
///  Open one entry relative to a proved folder descriptor,
///  following no link.
/// Why:
///  Opening by a path resolved again after the proof reads whatever is at that path now,
///  and this boundary is the receipt's own:
///  the descriptor is the authority.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openAt(folderFd, name, O_RDONLY | O_NOFOLLOW);
/// ```
fn open_at(
    directory: &std::fs::File,
    name: &str,
) -> Result<std::fs::File, std::io::Error> {
    use std::os::fd::{
        AsRawFd,
        FromRawFd,
    };
    let named: std::ffi::CString = std::ffi::CString::new(name)?;
    // SAFETY: `named` is one NUL-terminated name, `directory` holds a live descriptor, and the returned descriptor is
    // owned by the `File` this makes from it.
    let descriptor: std::ffi::c_int = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            named.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: the descriptor above is open and this is the only owner of it.
    return Ok(unsafe { std::fs::File::from_raw_fd(descriptor) });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "holder_scan_file_tests.rs"]
mod tests;
