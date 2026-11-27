//! What:
//!  What a fold is given and what it answers:
//!  the kinds of name it folds,
//!  the pair of spellings of each name,
//!  the process surroundings it reads,
//!  how the fold of one name ended,
//!  and what happened in one Git directory.
//! Why:
//!   Two files speak of these:
//!  the fold itself (`earlier_names_folded_into_current.rs`)
//!  and the lines it says (`what_a_fold_says_to_a_person.rs`),
//!  so they live here,
//!  in a file that uses no other file of this folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type KeptKind = 'log' | 'folder';
//! type KeptName = { name: RenamedName; kind: KeptKind };
//! type FoldSurroundings = { stateDir?: string; procRoot: string };
//! type Outcome = 'nothing' | 'folded' | 'leftForALaterCommand' | 'busy' | { left: string[] } | { failed: string } | { stopped: string };
//! type FoldReport = { held?: Hold; outcomes: [RenamedName, Outcome][] };
//! ```

/// The reason a look at an earlier program can hold a fold back,
/// which a report carries when it did.
use crate::earlier_service_process_look::Hold;
/// The pair of spellings of one kept name:
/// the current one and the one an earlier release used.
use crate::wrapper_names_old_and_new::RenamedName;
/// The OS string an environment variable holds.
use std::ffi::OsString;
/// The state folder and the process table a fold reads.
use std::path::PathBuf;

/// What kind of thing a kept name is,
/// which decides how it folds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeptKind {
    /// A log of lines,
    /// which keeps numbered older copies beside the one under the current name.
    Log,
    /// A folder of saved copies,
    /// records or notices.
    Folder,
}

/// One kept name to fold.
#[derive(Clone, Copy, Debug)]
pub struct KeptName {
    /// The pair of spellings.
    pub name: RenamedName,
    /// How it folds.
    pub kind: KeptKind,
}

/// What the look at the earlier program needs from the process around it.
#[derive(Clone, Debug)]
pub struct FoldSurroundings {
    /// The user's state folder,
    /// where the earlier commit service kept its journals.
    pub state_dir: Option<PathBuf>,
    /// The process table to look at.
    pub proc_root: PathBuf,
}

/// The path an environment variable holds,
/// or nothing when it is unset or empty.
fn non_empty_environment_path(name: &str) -> Option<PathBuf> {
    let value: OsString = std::env::var_os(name)?;
    if value.is_empty() {
        return None;
    }
    return Some(PathBuf::from(value));
}

/// The state folder under the home folder,
/// or nothing when no home is set.
fn home_state_dir() -> Option<PathBuf> {
    let home: PathBuf = non_empty_environment_path("HOME")?;
    return Some(home.join(".local").join("state"));
}

impl FoldSurroundings {
    /// The surroundings of this process:
    /// its environment and the live process table.
    pub fn of_this_process() -> Self {
        let named: Option<PathBuf> = non_empty_environment_path("XDG_STATE_HOME");
        let state_dir: Option<PathBuf> = named.or_else(home_state_dir);
        return FoldSurroundings {
            state_dir,
            proc_root: PathBuf::from("/proc"),
        };
    }
}

/// How the fold of one kept name ended.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// Nothing existed twice.
    Nothing,
    /// The earlier name is gone and its content is under the current name.
    Folded,
    /// A program still has the moved log open for writing:
    /// every byte it held so far is in the current log,
    /// nothing is said,
    ///  and a later command adds what the program writes next and removes the moved log once the program has closed it.
    LeftForALaterCommand,
    /// Another process is folding it.
    Busy,
    /// Entries of the earlier folder stayed where they were;
    /// each is listed.
    Left(Vec<PathBuf>),
    /// The file system refused;
    /// the message names the cause.
    /// Everything is still in place.
    Failed(String),
    /// A control stopped the fold.
    Stopped(&'static str),
}

/// What happened in one Git directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FoldReport {
    /// Why nothing was folded,
    /// when an earlier program may be writing.
    pub held: Option<Hold>,
    /// The outcome of each name that existed twice.
    pub outcomes: Vec<(RenamedName, Outcome)>,
}
