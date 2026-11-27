//! What:
//!  One process's record as the kernel prints it,
//!  the five fields a forced removal reads from it,
//!  and what a read that failed proves.
//! Why:
//!  The caller's chain and the protected set are read from `/proc/<pid>/stat`:
//!  the parent,
//!  the process group,
//!  the session and the start time
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  F11,
//!  which cites the field order at Linux `v7.3`,
//!  `fs/proc/array.c`).
//!  The crate's own readers answer the start time and the parent and not the
//!  session (`process::process_tree::member_read`),
//!  so this folder reads the four fields it needs itself and the report names
//!  the one read a later step should expose
//!  (`process_session_record` is this folder's own, per a build-ahead round's
//!  rules).
//!  A record that could not be read is kept apart from one that has gone,
//!  because a blind read refuses a forced removal and a task that has gone does
//!  not: "a task that has gone (`ENOENT`,
//!  `ESRCH`) is gone,
//!  not blind;
//!  a blind spot refuses".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = readProcessRecord(pid);   // { kind: 'found', record } | { kind: 'gone' } | { kind: 'blind', errno }
//! ```

use crate::js_text::trim_javascript;
use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;

/// How many fields after the command name `/proc/<pid>/stat` carries.
/// Why:
///  The fields the arm reads are numbered from 3 in the kernel's own list,
///  and the state is the first one after the command name,
///  so a record with fewer fields than these is malformed rather than short.
pub const STAT_FIELDS: usize = 20;

/// The index of the state field among the fields after the command name.
pub const STATE_FIELD: usize = 0;

/// The index of the parent field.
pub const PARENT_FIELD: usize = 1;

/// The index of the process group field.
pub const GROUP_FIELD: usize = 2;

/// The index of the session field.
pub const SESSION_FIELD: usize = 3;

/// The index of the start-time field,
/// in clock ticks since boot.
pub const STARTED_FIELD: usize = 19;

/// The longest command name the kernel prints,
/// because `comm` is cut to fourteen bytes and a terminator.
pub const COMMAND_MAXIMUM_BYTES: usize = 15;

/// What:
///  The command name one record carries,
///  cut to the kernel's own bound and kept as the kernel holds it.
/// Why:
///  The design's refusals print `(<command>)` inside a sentence,
///  and a command name is the one field of a record another account can choose
///  freely within fourteen bytes,
///  so the record bounds it and a sentence that shows it escapes it
///  (`AGENTS.md` rule `SYB`:
///  text crossing a syntax boundary is encoded at its final interpolation).
///  The escape is deliberately not here:
///  a name escaped in the record and shown by a sentence that escapes again
///  would read `\\x0a` where the kernel held a line feed,
///  and the record is also what a later reader compares with the kernel's own
///  bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CommandWord { constructor(readonly text: string) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandWord {
    /// The kernel's name for the process,
    /// cut to the kernel's own bound.
    text: String,
}

impl CommandWord {
    /// What:
    ///  The kernel's name for a process,
    ///  cut to the kernel's own bound.
    /// Why:
    ///  A `comm` is at most fifteen bytes and a NUL,
    ///  and a name longer than that would be a name no process can have:
    ///  so the record keeps the bytes the kernel keeps and no more.
    ///  The cut falls on a character boundary,
    ///  so a name of more than one byte per character loses no half character.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const word = CommandWord.of('python3');
    /// ```
    pub fn of(comm: &str) -> CommandWord {
        // What: the bytes the record keeps, built one character at a time.
        // Why:  the bound counts bytes, and a character that would cross it is dropped whole.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let kept = '';
        // ```
        let mut kept: String = String::with_capacity(COMMAND_MAXIMUM_BYTES);
        // What: one character of the kernel's name.
        // Why:  each is added only if it fits, so the bound is never split inside a character.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const character of comm) { if (byteLength(kept) + byteLength(character) > COMMAND_MAXIMUM_BYTES) break; kept += character; }
        // ```
        for character in comm.chars() {
            if kept.len() + character.len_utf8() > COMMAND_MAXIMUM_BYTES {
                break;
            }
            kept.push(character);
        }
        return CommandWord {
            text: kept,
        };
    }

    /// What:
    ///  One of this wrapper's own words,
    ///  kept whole.
    /// Why:
    ///  The bound is for a name another account wrote,
    ///  and a sentence this build wrote fits a report as it is.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// CommandWord.saying('a command this wrapper could not read')
    /// ```
    pub fn saying(word: &'static str) -> CommandWord {
        return CommandWord {
            text: String::from(word),
        };
    }

    /// What:
    ///  The name as the kernel holds it,
    ///  bounded.
    /// Why:
    ///  A sentence that shows this name must take it through
    ///  `shown_so_it_cannot_rewrite_the_line`,
    ///  because these bytes are the program's own and can hold an escape,
    ///  a line feed or the character that ends the sentence's wrapping.
    pub fn text(&self) -> &str {
        return self.text.as_str();
    }
}

/// What:
///  One process record with the fields a forced removal reads.
/// Why:
///  The chain walk compares a parent's number and start time with the child's
///  recorded pair,
///  and the protected set compares session numbers,
///  so one value carries what both need rather than two reads that could
///  disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProcessRecord = { pid; command; parent; group; session; started; state };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessRecord {
    /// The process's number.
    pub pid: i32,
    /// The kernel's name for it.
    pub command: CommandWord,
    /// Its state letter (`R`, `S`, `D`, `Z`, ...).
    pub state: char,
    /// The parent the kernel records for it.
    pub parent: i32,
    /// Its process group.
    pub group: i32,
    /// Its session,
    /// which is what the protected set's first half compares.
    pub session: i32,
    /// The start time the kernel recorded,
    /// in clock ticks since boot.
    pub started: u64,
}

impl ProcessRecord {
    /// Whether the kernel holds this process as one that has exited.
    /// Why:
    ///  A zombie holds no working directory and no descriptor,
    ///  and a process waiting to be reaped would otherwise be pinned and
    ///  signalled as a live blocker.
    pub fn has_exited(&self) -> bool {
        return self.state == 'Z' || self.state == 'X';
    }
}

/// What:
///  What reading one process record came to.
/// Why:
///  Three answers must stay apart:
///  the record,
///  a task that has gone,
///  and a read that was refused,
///  because the last one refuses a forced removal and the middle one does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadOutcome = { kind: 'found'; record } | { kind: 'gone'; why } | { kind: 'blind'; errno };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadOutcome {
    /// The record.
    Found(ProcessRecord),
    /// No task has that number,
    /// or the one that has it has exited.
    Gone,
    /// The kernel refused the read,
    /// with the errno and the word a refusal carries.
    Blind {
        /// The errno the kernel answered,
        /// as a number a report prints.
        errno: i32,
        /// The refusal word this read's cause answers.
        word: RefusalWord,
    },
}

/// What:
///  Which errno means the task has gone and which means the reader was refused.
/// Why:
///  `ENOENT` and `ESRCH` are the two answers a read of a task that ended gives,
///  and every other refusal is a read this process may not make,
///  which is a blind spot rather than an absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = code === 'ENOENT' || code === 'ESRCH';
/// ```
pub fn cause_of(errno: i32) -> Option<RefusalWord> {
    if errno == 2 || errno == 3 {
        // `ENOENT` and `ESRCH`: the task is not there any more.
        return None;
    }
    return Some(RefusalWord::HolderBlind);
}

/// What:
///  What one `/proc/<pid>/stat` text says.
/// Why:
///  The command name sits in parentheses and may hold spaces and parentheses,
///  so every field is read after the last closing parenthesis,
///  and the fields are the kernel's own order.
///  A text with fewer fields than the arm reads is malformed,
///  which is a blind read and not a gone task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = parseProcessRecord(pid, text);
/// ```
pub fn parse_process_record(
    pid: i32,
    text: &str,
) -> ReadOutcome {
    let Some(opened) = text.find('(') else {
        return ReadOutcome::Blind {
            errno: 0,
            word: RefusalWord::HolderBlind,
        };
    };
    let Some(closed) = text.rfind(')') else {
        return ReadOutcome::Blind {
            errno: 0,
            word: RefusalWord::HolderBlind,
        };
    };
    if closed < opened {
        return ReadOutcome::Blind {
            errno: 0,
            word: RefusalWord::HolderBlind,
        };
    }
    let Some(command) = text.get(opened + 1..closed) else {
        return ReadOutcome::Blind {
            errno: 0,
            word: RefusalWord::HolderBlind,
        };
    };
    let Some(after) = text.get(closed + 2..) else {
        return ReadOutcome::Blind {
            errno: 0,
            word: RefusalWord::HolderBlind,
        };
    };
    let fields: Vec<&str> = trim_javascript(after).split(' ').collect();
    if fields.len() <= STARTED_FIELD {
        return ReadOutcome::Blind {
            errno: 0,
            word: RefusalWord::HolderBlind,
        };
    }
    let Some(state) = fields[STATE_FIELD].chars().next() else {
        return ReadOutcome::Blind {
            errno: 0,
            word: RefusalWord::HolderBlind,
        };
    };
    let numbers: Option<(i32, i32, i32, u64)> = (|| {
        return Some((
            fields[PARENT_FIELD].parse().ok()?,
            fields[GROUP_FIELD].parse().ok()?,
            fields[SESSION_FIELD].parse().ok()?,
            fields[STARTED_FIELD].parse().ok()?,
        ));
    })();
    let Some((parent, group, session, started)) = numbers else {
        return ReadOutcome::Blind {
            errno: 0,
            word: RefusalWord::HolderBlind,
        };
    };
    return ReadOutcome::Found(ProcessRecord {
        pid,
        command: CommandWord::of(command),
        state,
        parent,
        group,
        session,
        started,
    });
}

/// What:
///  Read one process's record from a `/proc` root,
///  or answer that it has gone or that this process may not read it.
/// Why:
///  A control points the reader at a fixture tree,
///  and the handler reads the live one,
///  so the root is a parameter rather than a constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = readProcessRecord('/proc', pid);
/// ```
pub fn read_process_record(
    proc_root: &std::path::Path,
    pid: i32,
) -> ReadOutcome {
    let path: std::path::PathBuf = proc_root.join(pid.to_string()).join("stat");
    let text: String = match std::fs::read_to_string(&path) {
        Ok(read) => read,
        Err(error) => {
            let errno: i32 = error.raw_os_error().unwrap_or(0);
            return match cause_of(errno) {
                Some(word) => ReadOutcome::Blind {
                    errno,
                    word,
                },
                None => ReadOutcome::Gone,
            };
        }
    };
    let outcome: ReadOutcome = parse_process_record(pid, text.as_str());
    return match outcome {
        ReadOutcome::Found(record) if record.has_exited() => ReadOutcome::Gone,
        other => other,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "process_session_record_tests.rs"]
mod tests;
