//! What:
//!  What the wait under a bound answers with:
//!  the two bounds one command's own line gives it,
//!  who ended the command,
//!  the bytes its two pipes gave,
//!  and the sentence a refused answer prints.
//! Why:
//!  A control reads an answer as the command's own only when it is,
//!  and the shape that says so is what the wait hands back;
//!  the shape and the words it prints are one role,
//!  so they live in a file of their own beside the wait,
//!  which keeps the wait inside the repository's per-file line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Answer = { line: LineBound; status: ExitStatus; stdout: bytes; stderr: bytes; ending: Ending; heldOpen: string[] };
//! ```

use std::path::Path;

/// How long the pipes of a command may take to reach their end once the command is gone
/// before this wait leaves them to whoever holds them.
/// Why 500 ms:
///  everything a gone command wrote sits in the pipe's buffer already,
///  so this window covers the two reader threads being scheduled and no more;
///  a command that ends in a millisecond costs a millisecond,
///  and a pipe a descendant keeps open costs this window once,
///  however long it holds it.
pub const PIPE_GRACE: std::time::Duration = std::time::Duration::from_millis(500);

/// How long a command this wait has ended may take to be reaped.
/// Why five seconds:
///  a `SIGKILL`ed child is reaped at once,
///  so this window only covers the kernel and the scheduler;
///  a command still not reaped in it is a defect of this wait,
///  not a slow host.

/// The bound this wait keeps for a command whose own line names none:
///  the shared backstop,
///  which is also where the supervisor ends a command that never shows it is working.
/// Why:
///  Nothing waits without a bound,
///  a test included,
///  and a line that carries no `/usr/bin/timeout` is still a command a control waits for:
///  a trace wrapper,
///  a namespace probe,
///  a program a control spawns for itself.
///  The backstop is the bound every other command of this harness already runs under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bound = SHARED.backstop; // the bound this wait keeps for a line that states none
/// ```
pub const UNSTATED_BOUND: std::time::Duration = super::supervisor::SHARED.backstop;

/// What:
///  The two bounds one command's own line gives this wait:
///  the bound this wait keeps for the command,
///  and the bound that line states.
/// Why:
///  One reading of the line answers both (`line_bound_of`),
///  so no caller hands the wait a bound and a stated bound that no line gives:
///  a command of this harness carries its bound in its own line,
///  and a line that states none runs under the shared backstop.
///  The stated bound is what tells the death the line's own `timeout` deals at that bound
///  from a command's own `SIGKILL`,
///  because the status alone cannot say which of the two sent the signal (`ending_of_an_exit_read_first`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LineBound = { kept: Duration; stated: Duration | undefined }; // undefined: the line states no bound
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineBound {
    /// The bound this wait keeps for the command.
    pub kept: std::time::Duration,
    /// The bound the command's own line states,
    /// when that line is this harness's bounded form.
    pub stated: Option<std::time::Duration>,
}

impl LineBound {
    /// The pair for a child whose own line states no bound of this harness's bounded form:
    ///  the shared backstop,
    ///  and nothing stated.
    /// Why a constant:
    ///  the supervisor's controls spawn their child themselves,
    ///  so no `timeout` line of this harness is in front of it.
    pub const UNSTATED: LineBound = LineBound {
        kept: UNSTATED_BOUND,
        stated: None,
    };
}

/// Who ended one command this wait ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ending {
    /// The command ended by itself,
    /// and its status and output are its own,
    /// whatever that status was:
    /// a code of 124 or 137,
    /// or a signal's death.
    Itself,
    /// The bound ended it,
    /// at the bound its own line states:
    ///  by this wait's kill of the process group the command led,
    ///  or by the command's own `timeout`,
    ///  which keeps the same value and starts no earlier than the command.
    AtTheBound,
}

/// What:
///  What one command showed its caller,
///  and who ended it.
/// Why:
///  The ending is this wait's own act when it ends the command at the bound,
///  and a reading of the status when the exit reached it first:
///  a command that kills itself with `SIGKILL` before its bound is its own,
///  and only a death by `SIGKILL` at or after the bound the command's own line states is the bound's.
pub struct Answer {
    /// The two bounds this wait was given,
    ///  as one reading of the command's line gave them.
    pub line: LineBound,
    /// Its own ending and what its pipes gave before it went.
    pub status: std::process::ExitStatus,
    /// Exact standard output bytes.
    pub stdout: Vec<u8>,
    /// Exact standard error bytes.
    pub stderr: Vec<u8>,
    /// Who ended the command.
    pub ending: Ending,
    /// The pipes a process that outlived the command still holds,
    /// named for a failure.
    pub held_open: Vec<&'static str>,
    /// The supervisor's message when it ended the command,
    /// claimed while the command was still unreaped so the number it names was still the command's.
    pub supervisor_message: Option<String>,
}

impl Answer {
    /// What:
    ///  Panics unless the command ended by itself and nothing of it still holds a pipe.
    /// Why:
    ///  A control may read an answer as the command's own only when it is:
    ///  this wait ending the command at the bound,
    ///  and a pipe another process may still write to,
    ///  are the two ways it is not,
    ///  and each is said in the control's own failure instead of being passed off as a status.
    pub fn refuse_a_second_ending(
        &self,
        spelled: &str,
        directory: Option<&Path>,
    ) {
        let bound: std::time::Duration = self.line.kept;
        let location: String = directory
            .map(|at| {
                return format!(" in {}", at.display());
            })
            .unwrap_or_default();
        let held_sentence: Option<String> = match self.held_open.as_slice() {
            [] => None,
            pipes => Some(format!(
                "a process that outlived it still holds its {} {:?} after the command went, so what it wrote there may be short",
                pipes.join(" and "),
                PIPE_GRACE
            )),
        };
        match (self.ending, held_sentence) {
            (Ending::Itself, None) => {}
            (Ending::Itself, Some(named)) => {
                panic!("{spelled}{location} ended by itself, and {named}")
            }
            (Ending::AtTheBound, named) => {
                let also: String = named
                    .map(|sentence| {
                        return format!(", and {sentence}");
                    })
                    .unwrap_or_default();
                panic!(
                    "the command was ended at its {bound:?} wall-clock bound: {spelled}{location}{also}"
                );
            }
        }
    }
}
