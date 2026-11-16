//! What:
//!  The services that write one journal,
//!  each known by the place it takes there:
//!  a record made when a service starts and removed when it ends,
//!  judged alive by the birth identity of the process that wrote it.
//! Why:
//!  A decision about a request is only as wide as where it is written.
//! Kept in one service's memory, it holds where one service is the only one:
//!  the promise "this commit has not begun to land and never will" is one service's to keep only when nothing else
//!  can land the request.
//! Production runs one service per journal by socket activation,
//! and a development run and the binary controls give several services one journal
//! (`doc/planning/cli-git-resident-service.md`),
//!  which the journal's own batch claim exists for: two services of one journal once forgot a request that had landed.
//!  So a service that shares its journal with another cannot promise that a request it does not hold will never land,
//!  and it says what it knows instead of `Withdrawn` (`service_busy.rs`, `withdraw_early`).
//!  The place is an exclusive create in the journal's own directory,
//! no lock and no waiting:
//! a name is made whole or not at all,
//!  and a name whose writer died is told apart by the birth identity of the process it names
//!  (`process_identity.rs`), so a killed service leaves nothing that keeps the next one from promising anything.
//!  Nothing here is written on the path a commit waits on: a place is taken once, when a service starts, and removed
//!  when it ends, and a withdrawal answer reads the directory,
//!  which is a cached listing of a handful of names on a connection's own job.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const place = takeOnesPlace(journal); // { path } | undefined
//! const holders = servicesOfThisJournal(journal); // 'alone' | 'shared' | 'unknown'
//! ```

use crate::private_storage::{PRIVATE_DIRECTORY_MODE, create_private_directory_all};
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The directory of the places inside a journal.
const PLACES_DIRECTORY: &str = "services";

/// The instant a place's file exists and holds no byte of it,
/// for a control that ends the writer there (`service_journal_holders_tests.rs`).
#[cfg(test)]
pub(crate) const IN_THE_PLACE_AFTER_ITS_FILE: &str = "in-the-place-after-its-file";

/// The instant a place's file holds half of what it names,
/// for a control that ends the writer there.
#[cfg(test)]
pub(crate) const IN_THE_PLACE_AFTER_HALF_OF_IT: &str = "in-the-place-after-half-of-it";

/// The extension of one place.
const PLACE_EXTENSION: &str = "service";

/// The extension of a place that is being written,
/// which no reader counts as a place.
const STAGING_EXTENSION: &str = "service-staging";

/// How long a staging name, or a place of an earlier build that names no process, may stand before a reader takes it
/// away:
/// a place is written in one small write and one flush, so anything older than this was left by a process that is gone.
///  It is the lease a claim of this service takes (`CLAIM_LEASE`), the longest this service lets its own record stand
/// unread.
const LEFT_OVER_FOR: Duration = Duration::from_secs(120);

/// The staging name of the place that will be `path`:
/// the same name with the staging extension.
fn beside_the_final_name(path: &Path) -> PathBuf {
    return path.with_extension(STAGING_EXTENSION);
}

/// Numbers the places of one process,
/// so two boards of one process over one journal are two services and see each other.
static PLACES_TAKEN: AtomicU64 = AtomicU64::new(0);

/// What:
///  How many services write the journal a caller asked about,
///  as far as the places in it say.
/// Why:
///  A caller that would promise a request will never land needs the first of these,
/// and the other two are reasons not to promise:
/// `Shared` says another service may land the request,
/// and `Unknown` says the directory could not be read, which is never evidence that nobody else is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Holders = 'alone' | 'shared' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Holders {
    /// This service is the only live one writing the journal.
    Alone,
    /// Another live service writes the same journal.
    Shared,
    /// The places could not be read, so nothing about them is known.
    Unknown,
}

/// The place one service took in a journal,
/// removed when this value ends.
pub struct HeldPlace {
    /// The record of the place.
    path: PathBuf,
}

impl HeldPlace {
    /// The record of this place,
    /// for a control that reads what another service sees.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }
}

impl Drop for HeldPlace {
    fn drop(&mut self) {
        let _removed = std::fs::remove_file(self.path.as_path());
    }
}

/// What:
///  Takes this service's place in `journal`,
///  and nothing when no place could be made.
/// Why:
///   A service whose place cannot be written is not one that can tell whether it is alone,
/// so it promises nothing it cannot keep (`Holders::Unknown`),
///  and saying so once is better than a service that cannot answer a withdrawal at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const place = takeOnesPlace(journal); // { path } | undefined
/// ```
pub fn take_ones_place(journal: &Path) -> Option<HeldPlace> {
    let directory: PathBuf = journal.join(PLACES_DIRECTORY);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        crate::service_warn!("the places of the services of this journal could not be made, so this one cannot tell whether it is alone: {error}");
        return None;
    }
    let pid: u32 = std::process::id();
    let identity: String = match crate::process_identity::current_birth_identity() {
        Ok(identity) => identity,
        Err(error) => {
            crate::service_warn!("the birth identity of this process could not be read, so this service cannot tell whether it is alone: {error}");
            return None;
        }
    };
    for _attempt in 0..8 {
        let number: u64 = PLACES_TAKEN.fetch_add(1, Ordering::SeqCst);
        let path: PathBuf = directory.join(format!("{pid}-{number}.{PLACE_EXTENSION}"));
        // What: The record is written whole under a name of its own and then given its final name in one rename.
        // Why:  A place made under its final name and filled afterwards is, for a moment, a record nobody can read,
        //       and a kill makes that moment last for every later service of this journal
        //       (`doc/planning/cli-git-no-unbounded-waits.md`, and the third review of this branch, finding 2).
        //       A name that appears in one step is whole when it appears, and a name left over is under the writer's
        //       own staging name, which no reader counts as a place.
        let staged: PathBuf = beside_the_final_name(path.as_path());
        let mut file: std::fs::File = match std::fs::OpenOptions::new().write(true).create_new(true).open(staged.as_path()) {
            Ok(file) => file,
            // A staging name another writer of this process took: the next number is tried.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                crate::service_warn!("the place of this service in its journal could not be taken, so it cannot tell whether it is alone: {error}");
                return None;
            }
        };
        // A control ends the writer here, which is the instant a kill would leave an empty record.
        #[cfg(test)]
        crate::service_fault::hooks::run_on_any_thread(IN_THE_PLACE_AFTER_ITS_FILE);
        let written: Result<(), std::io::Error> = std::io::Write::write_all(&mut file, format!("{pid}\n{identity}\n").as_bytes());
        // A control ends the writer here, which is the instant a kill would leave one line of two.
        #[cfg(test)]
        crate::service_fault::hooks::run_on_any_thread(IN_THE_PLACE_AFTER_HALF_OF_IT);
        // The bytes reach the device before the name is published, so the record a reader meets is the record a power
        // cut would have left or none at all.
        let forced: Result<(), std::io::Error> = written.and_then(|()| { return file.sync_all(); });
        let _mode = std::fs::set_permissions(staged.as_path(), std::os::unix::fs::PermissionsExt::from_mode(PRIVATE_DIRECTORY_MODE));
        drop(file);
        if forced.is_err() {
            let _removed = std::fs::remove_file(staged.as_path());
            continue;
        }
        match std::fs::rename(staged.as_path(), path.as_path()) {
            Ok(()) => return Some(HeldPlace { path }),
            Err(error) => {
                let _removed = std::fs::remove_file(staged.as_path());
                crate::service_warn!("the place of this service in its journal could not be given its name, so it cannot tell whether it is alone: {error}");
                return None;
            }
        }
    }
    crate::service_warn!("no place in this journal could be taken for this service, so it cannot tell whether it is alone");
    return None;
}

/// What:
///  How many services write `journal`, by the places in it,
///  with the places of processes that are gone removed as they are found.
/// Why:
///   A service that was killed leaves its record,
/// and a record whose process is gone describes nothing that can land a request;
///  a write that removes it is this service's own and small,
/// and it happens on a withdrawal's job and never on the path a commit waits on.
///  A record that cannot be read,
/// or whose writer cannot be asked about,
/// counts as live: nothing about it is evidence that it is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holders = servicesOfThisJournal(journal, mine); // 'alone' | 'shared' | 'unknown'
/// ```
pub fn holders(journal: &Path, mine: &HeldPlace) -> Holders {
    let directory: PathBuf = journal.join(PLACES_DIRECTORY);
    let entries = match std::fs::read_dir(directory.as_path()) {
        Ok(entries) => entries,
        // The caller holds a place, so a journal that holds none is a journal whose directory went: nothing is known.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Holders::Unknown,
        Err(_unreadable) => return Holders::Unknown,
    };
    let mut live: usize = 0;
    let mut mine_seen: bool = false;
    for entry in entries {
        let Ok(found) = entry else { return Holders::Unknown };
        let path: PathBuf = found.path();
        let extension: Option<&str> = path.extension().and_then(std::ffi::OsStr::to_str);
        if extension == Some(STAGING_EXTENSION) {
            // A place that was being written and never given its name: it names no service, and it goes once it is older
            // than any write of a place takes.
            take_away_when_left_over(path.as_path());
            continue;
        }
        if extension != Some(PLACE_EXTENSION) {
            continue;
        }
        let text: String = match std::fs::read_to_string(path.as_path()) {
            Ok(text) => text,
            // The record went between the listing and the read: the service that ends now left nothing.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_unreadable) => return Holders::Unknown,
        };
        match holder_is_live(text.as_str()) {
            Holders::Alone => {
                live += 1;
                if path == mine.path() {
                    mine_seen = true;
                }
            }
            Holders::Shared => {
                // The process of that place is gone, so nothing of it can land a request, and its record goes as it is
                // found so the next reader does not carry it.
                let _removed = std::fs::remove_file(path.as_path());
            }
            Holders::Unknown => {
                // What: A place that names no process, or names one nothing can be asked about, stands only while it
                //       could be being written.
                // Why:  One written by an earlier build of this branch can be torn by a kill, and a record that names
                //       no process is not evidence of a live service: left for ever it would take early withdrawal away
                //       from the journal for good. Until the write window has passed it is left, because removing it
                //       would take a live service's place.
                if take_away_when_left_over(path.as_path()) {
                    continue;
                }
                return Holders::Unknown;
            }
        }
    }
    // What: The caller's own place must be among the live ones.
    // Why:  `Alone` is the answer every promise rests on, and a journal whose one live place belongs to another service
    //       is not one the caller may promise anything about.
    return match (mine_seen, live) {
        (true, 1) => Holders::Alone,
        (true, _) => Holders::Shared,
        (false, _) => Holders::Unknown,
    };
}

/// What:
///  Takes away a record of a place that names no service once it is older than any write of a place takes,
///  and says whether it did.
/// Why:
///   A record left by a writer that was killed is nobody's, and a record that is still within the write window may be a
/// live service's own, so only age can tell the two apart
/// (`LEFT_OVER_FOR`).
///  The removal is this service's own small write, on a withdrawal's own job.
fn take_away_when_left_over(path: &Path) -> bool {
    let Ok(metadata) = std::fs::symlink_metadata(path) else { return false };
    let Ok(modified) = metadata.modified() else { return false };
    let Ok(age) = modified.elapsed() else { return false };
    if age < LEFT_OVER_FOR {
        return false;
    }
    return std::fs::remove_file(path).is_ok();
}

/// `holders` for a caller that has only the path of its own place,
/// which a control needs because a place may not be moved into a thread.
pub fn holders_beside(journal: &Path, mine: &Path) -> Holders {
    let directory: PathBuf = journal.join(PLACES_DIRECTORY);
    let entries = match std::fs::read_dir(directory.as_path()) {
        Ok(entries) => entries,
        Err(_unreadable) => return Holders::Unknown,
    };
    let mut live: usize = 0;
    let mut mine_seen: bool = false;
    for entry in entries {
        let Ok(found) = entry else { return Holders::Unknown };
        let path: PathBuf = found.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(PLACE_EXTENSION) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path.as_path()) else { continue };
        if holder_is_live(text.as_str()) == Holders::Alone {
            live += 1;
            if path == mine {
                mine_seen = true;
            }
        }
    }
    return match (mine_seen, live) {
        (true, 1) => Holders::Alone,
        (true, _) => Holders::Shared,
        (false, _) => Holders::Unknown,
    };
}

/// Whether the service whose place holds `text` is still there,
/// as `Alone` when it is,
/// `Shared` when it is provably gone,
/// and `Unknown` when nothing can say.
fn holder_is_live(text: &str) -> Holders {
    let mut lines = text.lines();
    let (Some(named), Some(identity)) = (lines.next(), lines.next()) else { return Holders::Unknown };
    let Ok(pid) = named.trim().parse::<i64>() else { return Holders::Unknown };
    return match crate::process_identity::process_birth_identity(pid) {
        Ok(Some(found)) if found == identity.trim() => Holders::Alone,
        // The process of that number is gone, so no service of it can land anything.
        Ok(None) => Holders::Shared,
        Ok(Some(_another)) => Holders::Shared,
        Err(_cannot_say) => Holders::Unknown,
    };
}

/// Controls of the places stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_holders_tests.rs"]
mod tests;
