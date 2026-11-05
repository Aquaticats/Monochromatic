//! What:
//!  Finish the batches that the commit service of an earlier release left open, when the service of this release starts for
//!  the same worktree.
//! Why:
//!  The unit this release writes keeps the service journal in a state folder named after the renamed program, so a batch that
//!  the earlier helper had accepted and not finished (its branch moved and its index update lost, for example after a logout
//!  or a kill) sits in the earlier folder, where nothing of this release looks. Left there, the person's index would show the
//!  landed commit as reverted. The owner's rule is "never discard user data", so the new service finishes those batches
//!  with the recovery it already has, in place and before it accepts a connection: nothing is moved or copied, and a batch
//!  that landed stays remembered where it is. It does so only when no process is running the service of that worktree, because
//!  a running service finishes its own batches and two services must never settle one batch. The proof is a look at the
//!  running processes (`/proc`); a look that cannot be made counts as "may be running", the adoption is tried again at the
//!  next start, and nothing waits.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!(await serviceMayRun(instance))) await recoverInstance(earlierJournal, instance);
//! ```

/// The context a recovery runs in,
///  and the recovery itself.
use crate::service_land::{LandContext, RecoveryReport, recover_instance_without_drops};
/// The names of the folders.
use crate::wrapper_names_old_and_new::DATA_FOLDER;
/// Paths.
use crate::service_journal::landed_commits;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The name of the folder below the state folder that holds the service journals.
const SERVICE_FOLDER: &str = "service";

/// The words that start the service mode after the program name, and the option that names the worktree's instance.
const SERVICE_MODE_WORDS: [&str; 2] = ["service", "run"];

/// The option that names the instance a service is for.
const INSTANCE_OPTION: &str = "--instance";

/// Most bytes of one process's command line that are read;
///  the options of the service are in the first few hundred bytes.
const COMMAND_LINE_BYTES: u64 = 65_536;

/// The two folders an earlier release kept journals in for one worktree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EarlierJournals {
    /// The journal of the earlier release's service for this instance.
    pub per_instance: PathBuf,
    /// The journal above every instance's own, which the single service of the first releases used.
    pub shared: PathBuf,
}

/// What:
///  Where the earlier release kept the journals that belong to the journal of this release's service,
///  or nothing when the journal is not in the place the unit gives it (a development run names any folder).
///  `Option<EarlierJournals>` is "the two folders or nothing".
/// Why:
///   The unit passes `<state>/monochromatic-git-policy-cli/service/<instance>`; the earlier unit passed
///  `<state>/cli-git/service/<instance>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function earlierJournalsOf(journal: string, instance: string): EarlierJournals | undefined;
/// ```
pub fn earlier_journals_of(journal: &Path, instance: &str) -> Option<EarlierJournals> {
    if journal.file_name()? != instance {
        return None;
    }
    let service_folder: &Path = journal.parent()?;
    if service_folder.file_name()? != SERVICE_FOLDER {
        return None;
    }
    let data_folder: &Path = service_folder.parent()?;
    if data_folder.file_name()? != DATA_FOLDER.current {
        return None;
    }
    let state: &Path = data_folder.parent()?;
    let shared: PathBuf = state.join(DATA_FOLDER.earlier).join(SERVICE_FOLDER);
    return Some(EarlierJournals { per_instance: shared.join(instance), shared });
}

/// Whether two neighbouring command line words are the service mode words.
fn is_mode_pair(pair: &[&[u8]]) -> bool {
    return pair[0] == SERVICE_MODE_WORDS[0].as_bytes() && pair[1] == SERVICE_MODE_WORDS[1].as_bytes();
}

/// Whether a byte ends a command line word.
fn is_word_end(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether one command line (words split at NUL bytes) is a service of the instance:
///  it has the service mode words in a row and the instance option followed by the instance.
fn is_service_of(words: &[&[u8]], instance: &str) -> bool {
    let mode_in_a_row: bool = words.windows(SERVICE_MODE_WORDS.len()).any(is_mode_pair);
    let mut instance_named: bool = false;
    for pair in words.windows(2) {
        if pair[0] == INSTANCE_OPTION.as_bytes() && pair[1] == instance.as_bytes() {
            instance_named = true;
        }
    }
    return mode_in_a_row && instance_named;
}

/// What:
///  Whether a service of the instance may be running:
///  a process other than this one has the command line of one,
///  or the processes cannot be listed.
/// Why:
///   Adoption needs the earlier service gone.
///  A process that ends while it is looked at is gone;
///  a process whose command line cannot be read is not one of ours (the same user's are readable) and is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serviceMayRun(instance: string, ownPid: number, procRoot: string): Promise<boolean>;
/// ```
pub fn service_of_instance_may_run(instance: &str, own_pid: u32, proc_root: &Path) -> bool {
    use std::io::Read;
    let Ok(listing) = std::fs::read_dir(proc_root) else {
        return true;
    };
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        if pid == own_pid {
            continue;
        }
        let Ok(file) = std::fs::File::open(entry.path().join("cmdline")) else {
            continue;
        };
        let mut bytes: Vec<u8> = Vec::new();
        if file.take(COMMAND_LINE_BYTES).read_to_end(&mut bytes).is_err() {
            continue;
        }
        let words: Vec<&[u8]> = bytes.split(is_word_end).collect();
        if is_service_of(words.as_slice(), instance) {
            return true;
        }
    }
    return false;
}

/// What:
///  The request identities, each with the commit it became, that the earlier release's service finished for this worktree,
///  read from its journals (empty when the journal is not in the place the unit gives it).
///  `HashMap<String, String>` maps an identity to a commit name.
/// Why:
///   A client that the earlier helper never answered sends its request again to the service of this release; that service
///  answers a request it knows to be landed and lands nothing twice, so it must know what the earlier helper landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function earlierCommitsLanded(journal: string, instance: string): Map<string, string>;
/// ```
pub fn earlier_commits_landed(journal: &Path, instance: &str) -> HashMap<String, String> {
    let mut landed: HashMap<String, String> = HashMap::new();
    let Some(earlier) = earlier_journals_of(journal, instance) else {
        return landed;
    };
    for folder in [earlier.shared, earlier.per_instance] {
        landed.extend(landed_commits(folder.as_path()));
    }
    return landed;
}

/// What:
///  Finish the open batches of the worktree's instance in the journals the earlier release kept,
///  when no service of the instance is running.
///  `&LandContext` is the new service's context: its Git and the new journal's place.
/// Why:
///   The recovery decides each batch by the branch and the index, finishes the index update that may be missing, and marks
///  the finished batch done in the folder it found it in. It never drops the intent of a batch that looks unlanded: the look at
///  the processes can miss a live helper, and a live landing writes its intent before it moves the branch, so that intent stays
///  on disk (and is reported as left) for the earlier helper's own start-up recovery.
///  Calling it again finds nothing new to finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function adoptOpenBatches(context: LandContext, instance: string, procRoot: string): RecoveryReport[];
/// ```
pub fn adopt_open_batches(context: &LandContext, instance: &str, proc_root: &Path) -> Vec<RecoveryReport> {
    let mut reports: Vec<RecoveryReport> = Vec::new();
    let Some(earlier) = earlier_journals_of(context.journal.as_path(), instance) else {
        return reports;
    };
    if !earlier.per_instance.is_dir() && !earlier.shared.is_dir() {
        return reports;
    }
    if service_of_instance_may_run(instance, std::process::id(), proc_root) {
        crate::service_log!("the earlier service's journal {} is left for now: a service of this worktree may be running", earlier.shared.display());
        return reports;
    }
    for journal in [earlier.per_instance, earlier.shared] {
        if !journal.is_dir() {
            continue;
        }
        let earlier_context: LandContext = LandContext { git: context.git.clone(), journal, on_objects_written: None, lock_ceiling: context.lock_ceiling };
        reports.extend(recover_instance_without_drops(&earlier_context, instance));
    }
    return reports;
}
