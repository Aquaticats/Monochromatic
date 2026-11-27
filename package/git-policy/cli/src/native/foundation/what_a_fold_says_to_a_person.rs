//! What:
//!  The two lines a fold says to a person:
//!  the one that says a fold waits for an earlier program to stop,
//!  and the one that says a fold could not finish,
//!  each said once in a process.
//! Why:
//!   A fold runs on a command's own path,
//!  so these lines are the only thing a person sees of it when names stay apart;
//!  the fold itself is `earlier_names_folded_into_current.rs`,
//!  and which outcome says what is `what_a_fold_is_given_and_what_it_answers.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! let waitSaid = false; // whether this process already said the waiting line
//! let failureSaid = false; // whether it already said the failure line
//! function waitingLine(directory: string, hold: Hold): string;
//! function failureLine(directory: string, name: RenamedName, outcome: Outcome): string | undefined;
//! ```

/// The reason a fold waits,
/// and what a person can do about it.
use crate::earlier_service_process_look::Hold;
/// The instance a Git directory belongs to,
/// which the commands the waiting line names carry.
use crate::service_instance::instance_of;
/// How the fold of one name ended,
/// which decides whether a line is said.
use crate::what_a_fold_is_given_and_what_it_answers::Outcome;
/// The prefix of every line this program says,
/// and the pair of spellings of the name a failure names.
use crate::wrapper_names_old_and_new::{
    MESSAGE_PREFIX,
    RenamedName,
};
/// The Git directory the lines are about.
use std::path::Path;
/// The switches that say each line once in a process.
use std::sync::atomic::AtomicBool;

/// Whether this process has already said that a fold waits.
pub(crate) static WAIT_SAID: AtomicBool = AtomicBool::new(false);

/// Whether this process has already said that a fold could not finish.
pub(crate) static FAILURE_SAID: AtomicBool = AtomicBool::new(false);

/// The line that says a fold waits,
/// with what to do in plain words.
pub(crate) fn waiting_line(
    directory: &Path,
    hold: Hold,
) -> String {
    let instance: String = instance_of(directory);
    let stop: String = match hold {
        Hold::InstanceHelperRuns => format!(
            "stop it with `systemctl --user stop cli-git@{instance}.socket cli-git@{instance}.service`"
        ),
        _ => String::from(
            "stop the commit services of the earlier release (`systemctl --user list-units 'cli-git@*'` lists them)",
        ),
    };
    return format!(
        "{}: names the earlier release gave its saved data still sit beside the new names in {}. Nothing is lost and nothing is \
         needed from you: a later command folds them into the new names once the earlier release's program has stopped, because \
         {}. To let it happen sooner, {stop}.\n",
        MESSAGE_PREFIX.current,
        directory.display(),
        hold.words()
    );
}

/// The line that says a fold could not finish.
pub(crate) fn failure_line(
    directory: &Path,
    name: RenamedName,
    outcome: &Outcome,
) -> Option<String> {
    let because: String = match outcome {
        Outcome::Failed(message) => message.clone(),
        Outcome::Left(kept) => format!(
            "{} entries could not be moved because both names they would take are already in use; they stay where they are",
            kept.len()
        ),
        _ => return None,
    };
    return Some(format!(
        "{}: {} in {} could not be folded into {} ({because}). Everything is still in place, and the next command tries again.\n",
        MESSAGE_PREFIX.current,
        name.earlier,
        directory.display(),
        name.current
    ));
}
