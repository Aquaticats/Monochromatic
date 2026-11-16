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
        match std::fs::OpenOptions::new().write(true).create_new(true).open(path.as_path()) {
            Ok(mut file) => {
                // A control ends the writer here, which is the instant a kill would leave an empty record.
                #[cfg(test)]
                crate::service_fault::hooks::run_on_any_thread(IN_THE_PLACE_AFTER_ITS_FILE);
                let _written = std::io::Write::write_all(&mut file, format!("{pid}\n{identity}\n").as_bytes());
                // A control ends the writer here, which is the instant a kill would leave one line of two.
                #[cfg(test)]
                crate::service_fault::hooks::run_on_any_thread(IN_THE_PLACE_AFTER_HALF_OF_IT);
                let _mode = std::fs::set_permissions(path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(PRIVATE_DIRECTORY_MODE));
                return Some(HeldPlace { path });
            }
            // A name another service of this process took between the counter and the open: the next number is tried.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                crate::service_warn!("the place of this service in its journal could not be taken, so it cannot tell whether it is alone: {error}");
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
pub fn holders(journal: &Path, _mine: &HeldPlace) -> Holders {
    let directory: PathBuf = journal.join(PLACES_DIRECTORY);
    let entries = match std::fs::read_dir(directory.as_path()) {
        Ok(entries) => entries,
        // A journal no service has taken a place in yet holds none.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Holders::Alone,
        Err(_unreadable) => return Holders::Unknown,
    };
    let mut live: usize = 0;
    for entry in entries {
        let Ok(found) = entry else { return Holders::Unknown };
        let path: PathBuf = found.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(PLACE_EXTENSION) {
            continue;
        }
        let text: String = match std::fs::read_to_string(path.as_path()) {
            Ok(text) => text,
            // The record went between the listing and the read: the service that ends now left nothing.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_unreadable) => return Holders::Unknown,
        };
        match holder_is_live(text.as_str()) {
            Holders::Alone => live += 1,
            Holders::Shared => {
                // The process of that place is gone, so nothing of it can land a request, and its record goes as it is
                // found so the next reader does not carry it.
                let _removed = std::fs::remove_file(path.as_path());
            }
            Holders::Unknown => return Holders::Unknown,
        }
    }
    return if live <= 1 { Holders::Alone } else { Holders::Shared };
}

/// What:
///  The count of live places alone,
///  which is what this branch answered before the third review's first finding:
///  it says `Alone` when at most one live place is there, whoever's it is.
/// Why:
///  The control of that finding is red while this answer stands,
/// and the answer itself is the defect: a service that took no place still counted the journal's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holders = theCountAlone(journal); // 'alone' when one live place is there, whoever's it is
/// ```
pub(crate) fn the_count_alone(journal: &Path) -> Holders {
    let directory: PathBuf = journal.join(PLACES_DIRECTORY);
    let entries = match std::fs::read_dir(directory.as_path()) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Holders::Alone,
        Err(_unreadable) => return Holders::Unknown,
    };
    let mut live: usize = 0;
    for entry in entries {
        let Ok(found) = entry else { return Holders::Unknown };
        let path: PathBuf = found.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(PLACE_EXTENSION) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path.as_path()) else { continue };
        if holder_is_live(text.as_str()) == Holders::Alone {
            live += 1;
        }
    }
    return if live <= 1 { Holders::Alone } else { Holders::Shared };
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
