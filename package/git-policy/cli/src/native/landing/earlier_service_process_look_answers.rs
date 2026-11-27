//! What:
//!  What the look at an earlier commit service answers with,
//! and the words it reads:
//! why the commit service of one worktree may be running or may start,
//! the program the look could not rule out,
//! the process whose command line could not be proved,
//! what one journal word of a command line says,
//! what a command line says about the process that carries it,
//! and the folders a look needs to recognise that service.
//! Why:
//!  The reading of one command line (`earlier_service_process_start_arguments.rs`)
//! and the look over the machine (`earlier_service_process_look.rs`)
//! both answer with these,
//! so they sit in a file that reads no process and opens no table.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Hold = 'instanceSocketListens' | 'sharedSocketListens' | ... ; const words = hold.words();
//! ```

use crate::earlier_service_command_line_read::UnreadableProcess;
use crate::earlier_service_unsure_program::UnsureProgram;
use std::path::PathBuf;
use std::time::Duration;

/// The words that start the service mode after the program name,
/// and the option that names the worktree's instance.
pub(crate) const SERVICE_MODE_WORDS: &[&str] = &["service", "run"];

/// The option that names the instance a service is for.
pub(crate) const INSTANCE_OPTION: &str = "--instance";

/// The option that names the folder a service keeps its journal in.
/// Every unit of an earlier release passed it as a word of its
///  own,
/// followed by the folder.
pub(crate) const JOURNAL_OPTION: &str = "--journal";

/// Why the look cannot promise that no service of the worktree runs or will start,
/// and which one it is.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hold {
    /// The earlier release's socket of this worktree (`cli-git/<instance>.sock`) listens:
    /// a connection to it starts the helper.
    InstanceSocketListens,
    /// The socket of the single service of the first releases (`cli-git/service.sock`) listens;
    /// that service serves every worktree.
    SharedSocketListens,
    /// A process runs as the earlier helper of this worktree:
    /// its journal is this worktree's,
    /// or its mode words and instance are.
    InstanceHelperRuns,
    /// A process runs as the single service of the first releases:
    /// its journal is the shared one;
    /// it serves every worktree.
    SharedHelperRuns,
    /// A process has a journal option whose folder could not be resolved,
    /// so it may be an earlier helper of either kind.
    HelperUnsure,
    /// The sockets or the processes could not be looked at (a table or a command line that cannot be read,
    /// a look that ran out of time).
    CannotLook,
}

impl Hold {
    /// The reason in plain words,
    /// as a sentence part that follows "because".
    pub fn words(self) -> &'static str {
        return match self {
            Hold::InstanceSocketListens => {
                "the socket of the earlier release's commit service for this worktree listens, and a connection to it starts that service at any moment"
            }
            Hold::SharedSocketListens => {
                "the socket of the single commit service of the first releases listens (it serves every worktree), and a connection to it starts that service at any moment"
            }
            Hold::InstanceHelperRuns => {
                "a program of the earlier release is running as the commit service of this worktree, and it finishes its own batches"
            }
            Hold::SharedHelperRuns => {
                "a program of the first releases is running as the single commit service of every worktree, and it finishes its own batches"
            }
            Hold::HelperUnsure => {
                "a running program has the earlier commit service's journal option with a folder this check could not resolve, so it may be that service"
            }
            Hold::CannotLook => {
                "the sockets and processes of the earlier helper could not be looked at, so it is not known whether one runs"
            }
        };
    }
}

/// What a look found:
/// why the worktree's earlier service may be running or may start,
/// and,
/// when a program that cannot be ruled
///  out caused it,
/// which program that is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Holding {
    /// The reason,
    /// which tells the person what to stop.
    pub because: Hold,
    /// The program behind [`Hold::HelperUnsure`];
    /// nothing for every other reason.
    pub program: Option<UnsureProgram>,
    /// The process behind [`Hold::CannotLook`] when a command line could not be proved;
    /// nothing for every other reason.
    pub unreadable: Option<UnreadableProcess>,
}

impl From<Hold> for Holding {
    fn from(because: Hold) -> Self {
        return Holding {
            because,
            program: None,
            unreadable: None,
        };
    }
}

/// What the start arguments of one process say about it.
#[cfg(unix)]
pub(crate) enum ServiceFound {
    /// It is a service of the worktree,
    /// of the kind the reason names.
    Hold(Hold),
    /// A journal word could not be resolved,
    /// so it may be one:
    /// the word,
    /// whether it is relative,
    /// and the system's answer.
    Unsure {
        word: Vec<u8>,
        relative: bool,
        reason: String,
    },
}

/// What one journal word of a command line says about the process that carries it.
/// Why:
///   A word that names no earlier journal is not one answer but two:
/// a folder that is there and is
///  another program's proves that the program writes there,
/// while a folder that is not there proves nothing about
///  the folder an earlier helper writes,
/// because the word may be a spelling whose link another command removed after
///  that helper was started.
/// The rest of the command line decides which of the two a program is
///  ([`is_service_of`]).
#[cfg(unix)]
pub(crate) enum JournalWord {
    /// The word names one of the earlier journals,
    /// of the kind the reason names.
    Names(Hold),
    /// The word is an absolute path whose folder is not there,
    /// and the path it spells,
    /// resolved as far as the file system knows it,
    /// is no earlier journal's.
    AbsentAndElsewhere,
    /// The word names a folder that is there and is no earlier journal's.
    Elsewhere,
    /// The word could not be resolved,
    /// so it may name one:
    /// the word,
    /// whether it is relative,
    /// and the system's answer.
    Unsure {
        word: Vec<u8>,
        relative: bool,
        reason: String,
    },
}

/// What a look needs to know to recognise a service of one worktree.
#[derive(Clone, Debug)]
pub struct ServiceLook {
    /// The worktree's instance name.
    pub instance: String,
    /// The folders an earlier release's service of the worktree kept its journal in:
    /// the instance's own first,
    /// the shared one
    ///  second.
    /// A process that was started with one of them as its journal is a service of the worktree,
    /// and which one it is
    ///  tells the person what to stop.
    pub journals: Vec<PathBuf>,
    /// This process,
    /// which is never another service.
    pub own_pid: u32,
    /// The user whose processes are looked at.
    pub own_user: u32,
    /// How long the whole look may take before it counts as "may be running".
    pub ceiling: Duration,
}
