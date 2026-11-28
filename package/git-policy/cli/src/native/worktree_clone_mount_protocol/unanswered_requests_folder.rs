//! What:
//!  The folder of unanswered requests in a repository's Git common
//!  directory:
//!  where it lives,
//!  the bounded read of one entry,
//!  the publish of a wait's entry,
//!  the record a helper replaces an attempt with,
//!  and the removal of a clone's attempts.
//! Why:
//!  The folder is the account's record of the clones still waiting,
//!  so a command can learn cheaply that its repository holds none,
//!  and a command killed between two commands leaves its trigger behind
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The record:
//! one file per clone").
//!  One file per attempt is published whole under a staging name made with
//!  `O_EXCL`,
//!  synced and linked,
//!  so a reader never sees half an entry and two writers cannot share one,
//!  and the entry is read without following a link,
//!  one page at a time,
//!  by the account that owns it.
//!  Nothing here decides whether a nudge is due:
//!  that needs the gaps step 22 sizes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = UnansweredRequests.inTheCommonDir(commonDir);
//! folder.recordTheRequest(entry); folder.removeEveryAttempt(guid);
//! ```

use crate::worktree_clone_mount_protocol::unanswered_request_entry::RequestEntry;
use crate::worktree_clone_mount_protocol::unanswered_request_names::{
    EntryName,
    fresh_staging_name,
    one_past_the_highest,
};
use crate::worktree_clone_mount_protocol::unanswered_requests_refusals::RequestsRefusal;
use std::os::unix::fs::{
    DirBuilderExt,
    OpenOptionsExt,
};
use std::path::{
    Path,
    PathBuf,
};

/// The folder's name in the Git common directory.
/// The wrapper makes it on the first entry and removes it with its last,
/// so a repository whose request was answered pays one `stat` again.
pub(crate) const REQUESTS_FOLDER_NAME: &str = "zfs-unanswered-requests";

/// The mode the folder is made with:
/// the account alone may read and write it.
const FOLDER_MODE: u32 = 0o700;

/// What:
///  The folder of unanswered requests of one repository,
///  and the account that owns it.
/// Why:
///   A control hands in a fixture directory it owns,
///  so the same reads and writes are exercised without a repository's `.git`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UnansweredRequests { constructor(readonly directory: string, readonly account: number) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnansweredRequests {
    /// The folder's path.
    directory: PathBuf,
    /// The account every file in it must belong to.
    account: u32,
}

impl UnansweredRequests {
    /// The folder of a repository whose Git common directory is at `common_dir`.
    pub(crate) fn in_the_common_dir(common_dir: &Path) -> Self {
        return UnansweredRequests {
            directory: common_dir.join(REQUESTS_FOLDER_NAME),
            account: this_account(),
        };
    }

    /// A folder at a fixture directory,
    /// owned by the account a control expects.
    #[cfg(test)]
    pub(crate) fn at(
        directory: PathBuf,
        account: u32,
    ) -> Self {
        return UnansweredRequests {
            directory,
            account,
        };
    }

    /// The folder's path.
    #[cfg(any(test, feature = "zfs-lost-event-nudge"))]
    pub(crate) fn path(&self) -> &Path {
        return self.directory.as_path();
    }

    /// The account every file in the folder must belong to.
    #[cfg(test)]
    pub(crate) fn account(&self) -> u32 {
        return self.account;
    }

    /// What:
    ///  Whether the folder stands at all,
    ///  which is the one `stat` a command pays when nothing waits.
    /// Why:
    ///   The cheap negative the record needs is a folder that is not there:
    ///  no `read_dir`,
    ///  no receipt read and no device.
    pub(crate) fn stands(&self) -> bool {
        return std::fs::symlink_metadata(self.directory.as_path())
            .is_ok_and(|facts| return facts.is_dir());
    }

    /// What:
    ///  Write a wait's own entry,
    ///  before the durable intent,
    ///  and answer the attempt's name.
    /// Why:
    ///   A wait killed before its deadline leaves this entry behind,
    ///  which is the trigger a later command reads,
    ///  and the write is synced once so a crash after it cannot lose it.
    ///
    /// # Errors
    /// The folder cannot be made or written,
    /// or the random source for the staging name failed.
    pub(crate) fn record_the_request(
        &self,
        entry: &RequestEntry,
    ) -> Result<EntryName, RequestsRefusal> {
        self.make()?;
        let attempts: Vec<u32> = self.attempts_of(entry.guid);
        let name: EntryName = EntryName {
            guid: entry.guid,
            attempt: one_past_the_highest(attempts.as_slice()),
        };
        if !self.publish_a_new_attempt(name, entry)? {
            return Err(RequestsRefusal::Folder {
                path: self.directory.join(name.file_name()),
                reason: String::from("another writer took this attempt's name first"),
            });
        }
        return Ok(name);
    }

    /// What:
    ///  Take one attempt's name for a writer,
    ///  and answer whether this caller is the one that took it.
    /// Why:
    ///   The link decides which writer of a burst owns the next attempt,
    ///  so the loser writes nothing more and spawns nothing,
    ///  and the answer is the one fact the race has.
    ///
    /// # Errors
    /// The folder or the staging file cannot be written.
    pub(crate) fn publish_a_new_attempt(
        &self,
        name: EntryName,
        entry: &RequestEntry,
    ) -> Result<bool, RequestsRefusal> {
        return self.write_whole(name, entry);
    }

    /// What:
    ///  Remove every attempt of one clone,
    ///  then the folder when its last entry went.
    /// Why:
    ///   A receipt that answers any attempt answers the clone's request,
    ///  and an older attempt can never be answered under this route's one
    ///  nonce per clone,
    ///  so the answered clone's every attempt goes at once.
    pub(crate) fn remove_every_attempt(
        &self,
        guid: crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid,
    ) {
        let Ok(names) = self.names() else {
            return;
        };
        for name in names {
            match EntryName::parse(name.as_str()) {
                Ok(found) if found.guid == guid => {
                    let _gone = std::fs::remove_file(self.directory.join(name.as_str()));
                }
                Ok(_) | Err(_) => {}
            }
        }
        // What: the folder goes with its last entry, and a folder a racing creator left costs one `read_dir`.
        // Why:  a repository whose request was answered returns to the one-`stat` case, and a failure here is
        //       not worth a line: the next entry makes the folder again.
        let _removed = std::fs::remove_dir(self.directory.as_path());
    }

    /// What:
    ///  The one entry a reader takes for each clone:
    ///  its highest readable attempt,
    ///  where an empty or unreadable attempt above it is skipped.
    /// Why:
    ///   A crash can leave the newest attempt empty,
    ///  and the attempt under it is the clone's trigger,
    ///  so the reader never answers that the clone waits for nothing.
    pub(crate) fn the_entries(&self) -> Vec<(EntryName, RequestEntry)> {
        let Ok(names) = self.names() else {
            return Vec::new();
        };
        let mut numbered: Vec<EntryName> = Vec::<EntryName>::new();
        for name in names {
            if let Ok(found) = EntryName::parse(name.as_str()) {
                numbered.push(found);
            }
        }
        // What: highest attempt first, so the first readable one of a clone ends its turn.
        // Why:  the design's reader takes the highest attempt and uses the highest readable lower one.
        numbered.sort_by(|left, right| {
            return right
                .guid
                .value()
                .cmp(&left.guid.value())
                .then(right.attempt.cmp(&left.attempt));
        });
        let mut taken: Vec<(EntryName, RequestEntry)> = Vec::<(EntryName, RequestEntry)>::new();
        for name in numbered {
            if taken
                .iter()
                .any(|(found, _)| return found.guid == name.guid)
            {
                continue;
            }
            if let Ok(entry) = super::unanswered_requests_one_entry_read::read_one(
                self.directory.as_path(),
                self.account,
                name,
            ) {
                taken.push((name, entry));
            }
        }
        return taken;
    }

    /// The attempt numbers of one clone present in the folder,
    /// empty or readable,
    /// because the number a claim takes is decided by the names alone.
    /// A folder that cannot be listed holds none,
    /// and the write that follows refuses on its own terms.
    pub(crate) fn attempts_of(
        &self,
        guid: crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid,
    ) -> Vec<u32> {
        let mut attempts: Vec<u32> = Vec::<u32>::new();
        let Ok(names) = self.names() else {
            return attempts;
        };
        for name in names {
            if let Ok(found) = EntryName::parse(name.as_str())
                && found.guid == guid
            {
                attempts.push(found.attempt);
            }
        }
        attempts.sort_unstable();
        return attempts;
    }

    /// Every name in the folder,
    /// where a folder that is not there holds none.
    fn names(&self) -> Result<Vec<String>, RequestsRefusal> {
        let entries = match std::fs::read_dir(self.directory.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => {
                return Err(RequestsRefusal::Folder {
                    path: self.directory.clone(),
                    reason: error.to_string(),
                });
            }
        };
        let mut names: Vec<String> = Vec::<String>::new();
        for found in entries {
            let Ok(listed) = found else {
                continue;
            };
            if let Some(name) = listed.file_name().to_str() {
                names.push(String::from(name));
            }
        }
        return Ok(names);
    }

    /// Make the folder when it is absent,
    /// readable by the account alone.
    fn make(&self) -> Result<(), RequestsRefusal> {
        let refused = |error: std::io::Error| {
            return RequestsRefusal::Folder {
                path: self.directory.clone(),
                reason: error.to_string(),
            };
        };
        return match std::fs::DirBuilder::new()
            .mode(FOLDER_MODE)
            .create(self.directory.as_path())
        {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => Err(refused(error)),
        };
    }

    /// Write one entry whole under a staging name,
    /// sync it,
    /// link it to the attempt's name,
    /// then remove the staging name.
    fn write_whole(
        &self,
        name: EntryName,
        entry: &RequestEntry,
    ) -> Result<bool, RequestsRefusal> {
        let refused = |path: PathBuf, error: std::io::Error| {
            return RequestsRefusal::Folder {
                path,
                reason: error.to_string(),
            };
        };
        self.make()?;
        let staging: String = fresh_staging_name().map_err(|refusal| {
            return RequestsRefusal::Folder {
                path: self.directory.clone(),
                reason: refusal.message(),
            };
        })?;
        let staging_path: PathBuf = self.directory.join(staging.as_str());
        let text: String = entry.text();
        let write = || -> std::io::Result<()> {
            let mut file: std::fs::File = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(staging_path.as_path())?;
            std::io::Write::write_all(&mut file, text.as_bytes())?;
            file.sync_all()?;
            return std::fs::hard_link(
                staging_path.as_path(),
                self.directory.join(name.file_name()).as_path(),
            );
        };
        let landed: std::io::Result<()> = write();
        // What: the staging name is this writer's, so its removal after the link is the writer's own cleanup.
        // Why:  a leftover blocks no later writer because every writer's name is fresh, but leaving it would
        //       make `doctor` print a killed writer's name for a writer that finished.
        let _cleaned: std::io::Result<()> = std::fs::remove_file(staging_path.as_path());
        return match landed {
            Ok(()) => Ok(true),
            // What: a name another writer took is the one answer that is not a failure.
            // Why:  the claim's whole decision is which writer linked first, and the loser returns without a line.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(error) => Err(refused(self.directory.join(name.file_name()), error)),
        };
    }
}

/// What:
///  The account this process runs as,
///  which every file of the folder must belong to.
/// Why:
///   The entry is the account's own record and never root's,
///  so a file another account wrote is refused rather than read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.geteuid()
/// ```
pub(crate) fn this_account() -> u32 {
    // SAFETY: `geteuid` reads the process credentials and cannot fail.
    return unsafe { libc::geteuid() };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unanswered_requests_folder_tests.rs"]
mod tests;
