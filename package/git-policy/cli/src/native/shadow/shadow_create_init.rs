//! What:
//!  The `git init` that makes a reftable shadow's tables,
//!  and what a start the ceiling ends or the system refuses leaves behind:
//!  the partial directory is discarded and the start is made again in a fresh one,
//!  at most `SHADOW_CREATION_ATTEMPTS` times.
//! Why:
//!   A cut `init` does not run twice to the same state in place,
//!  so the product ends it at its ceiling only because the directory it leaves is discarded
//!  (`transaction_git_step_rerun_and_end_rules.rs`,
//!  `may_be_ended`);
//!  a start that fails another way is not retried.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // while (attempt < SHADOW_CREATION_ATTEMPTS) await runGit(init); rm(shadow, { recursive: true });
//! ```

/// Where Git's words go now that the `init` carries no `--quiet`.
use crate::inner_git_words::record_words;
/// The facts a start of `init` makes the shadow from.
use crate::shadow_create_template::ShadowTemplate;
/// The variables the real repository's redirects name,
/// removed so the shadow's `init` cannot be sent elsewhere.
use crate::shadow_git::REPOSITORY_REDIRECT_VARIABLES;
/// The attempt one start of `init` is,
/// counted for the controls and read by a control's Git stand-in.
use crate::shadow_init_attempts::note_init_attempt;
/// Process context,
/// the required Git start,
/// and the failure text of a step that did run.
use crate::transaction_git::{
    GitContext,
    GitFailure,
    GitOutput,
    GitRequest,
    checked_from,
    refused_by_the_system,
    run_git,
};
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// How many times the shadow's `git init` is started,
/// each in a fresh directory,
/// when a start is cut at its ceiling or refused by the
///  system.
/// Two:
/// the start and one more,
/// the same "redone once" the quiet-window rule reads (`doc/planning/cli-git-shipped-limits-audit.md`,
///  pick 4,
/// the coordinator's reading,
/// open to veto);
/// a shadow whose `init` is cut twice fails its transaction.
pub(crate) const SHADOW_CREATION_ATTEMPTS: u32 = 2;

/// What:
///  Creates an empty bare repository with `git init`,
/// for the ref backend whose tables this crate does not write.
/// Why:
///   Reftable tables carry their own format,
/// so only Git creates them.
///  The pathname handed to Git is the shadow's own bytes:
/// a spelling of it writes every byte that is not UTF-8 as U+FFFD,
/// and Git then initializes a different directory,
///  so the creation protects a path that does not exist and the commit cannot be prepared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGit({ args: ['init', '--bare', '--template=', `--object-format=${format}`, '--ref-format=reftable', shadowBytes] });
/// ```
pub(crate) fn init_with_git(
    context: &GitContext,
    template: &ShadowTemplate,
    shadow: &Path,
) -> Result<(), GitFailure> {
    let arguments: Vec<OsString> = vec![
        OsString::from("init"),
        OsString::from("--bare"),
        OsString::from("--template="),
        OsString::from(format!("--object-format={}", template.object_format)),
        OsString::from("--ref-format=reftable"),
        shadow.as_os_str().to_os_string(),
    ];
    let mut init: GitRequest = GitRequest::new(template.common_dir.as_path(), &arguments);
    init.without_prefix = true;
    init.unset = REPOSITORY_REDIRECT_VARIABLES
        .iter()
        .copied()
        .map(OsString::from)
        .collect();
    let mut attempt: u32 = 1;
    loop {
        // The attempt is counted here, before the start: a start the ceiling ends before its program writes anything is an attempt the wrapper made.
        note_init_attempt(&mut init, attempt);
        let outcome = run_git(context, &init);
        // A start ended at its ceiling, or refused a process or a thread, leaves a directory that a second `init` in place does not bring to
        //  the state of an uncut run (the cut experiment: 50 of 104 second runs differed in the reftable tables). It is discarded and the
        //  start is made again in a fresh directory, which the experiment found equal to an uncut run (104 of 104).
        let cut: bool = start_was_cut_or_refused(&outcome);
        if cut && attempt < SHADOW_CREATION_ATTEMPTS {
            let note: String = format!(
                "the shadow's init was cut or refused (attempt {attempt} of {SHADOW_CREATION_ATTEMPTS}); its directory {} is discarded and the step starts again in a fresh one",
                shadow.display()
            );
            crate::client_journal::record_entry(
                crate::journald_native::Priority::Warning,
                note.as_str(),
                &[],
            );
            discard_partial_shadow(shadow)?;
            attempt += 1;
            continue;
        }
        if cut {
            discard_partial_shadow(shadow)?;
        }
        let made = checked_from(outcome, init.arguments.as_slice())?;
        // The `init` carries no `--quiet`; what Git said of it is kept in the journal.
        record_words(
            &["init", "--bare", "--template=", "--ref-format=reftable"],
            &made,
        );
        return Ok(());
    }
}

/// What:
///  Whether a start of `init` was ended at its ceiling or refused a process or a thread.
/// Why:
///   Only these leave a partial directory that a fresh start replaces.
/// An I/O error of another kind (no such program,
/// no permission)
///  is not retried:
/// a fresh start would fail the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcome.error?.kind === 'timed-out' || outcome.output?.stderr.includes('Resource temporarily unavailable')
/// ```
pub(crate) fn start_was_cut_or_refused(outcome: &std::io::Result<GitOutput>) -> bool {
    return match outcome {
        // `WouldBlock` is `EAGAIN`: the system refused the process or the thread before the command ran.
        Err(error) => matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ),
        Ok(finished) => refused_by_the_system(finished),
    };
}

/// What:
///  Removes what a cut `git init` left at the shadow's path.
/// Why:
///   A shadow is private to its transaction and holds nothing but what `init` made,
/// so a partial one is discarded whole and never
///  completed in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(shadow, { recursive: true, force: true });
/// ```
fn discard_partial_shadow(shadow: &Path) -> Result<(), GitFailure> {
    return match std::fs::remove_dir_all(shadow) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(GitFailure(format!(
            "the partial shadow {} cannot be discarded: {error}",
            shadow.display()
        ))),
    };
}
