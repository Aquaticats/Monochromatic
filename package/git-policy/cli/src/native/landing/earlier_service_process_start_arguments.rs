//! What:
//!  Which earlier commit service one command line of a process names:
//! the words that start the service mode,
//! the journal option and the folder its word spells,
//! the instance option,
//! and the rest of the line that decides what a word naming no earlier journal means.
//! Why:
//!  A process keeps the arguments it was started with,
//! and every release's units started the service another way,
//! so the answer reads all of those shapes
//! and errs toward "may be running";
//! the answers it gives are declared beside it (`earlier_service_process_look_answers.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = isServiceOf(words, look, names); // a Hold, an unsure word, or nothing
//! ```

/// The words a relative journal is resolved in,
/// and the spelling a folder that is not there is compared by.
#[cfg(unix)]
use crate::earlier_service_look_paths::{
    path_word,
    resolved_as_far_as_the_file_system_knows,
};
use crate::earlier_service_process_look_answers::{
    Hold,
    INSTANCE_OPTION,
    JOURNAL_OPTION,
    JournalWord,
    SERVICE_MODE_WORDS,
    ServiceFound,
    ServiceLook,
};
/// The word this release's own service starts behind,
/// which makes it no earlier helper.
#[cfg(unix)]
use crate::wrapper_names_old_and_new::COMMAND_WORD;
#[cfg(unix)]
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Whether the service mode words start at position `at` of the command line and are not behind the current command word.
/// Why:
///   The service of this release starts as `<program> monochromatic-git-policy-cli service run ...`,
/// with the same mode words
///  and the same instance option as the earlier units that name the earlier command word,
/// and it is no earlier helper:
/// it
///  writes its own journal,
/// and it is the service a person usually has running when they ask for the earlier batches to be
///  settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isEarlierModeAt = (words: string[], at: number) => words[at] === 'service' && words[at + 1] === 'run' && words[at - 1] !== 'monochromatic-git-policy-cli'
/// ```
fn is_earlier_mode_at(
    words: &[&[u8]],
    at: usize,
) -> bool {
    let Some(pair) = words.get(at..at + SERVICE_MODE_WORDS.len()) else {
        return false;
    };
    let behind_current_word: bool = at > 0 && words[at - 1] == COMMAND_WORD.current.as_bytes();
    return pair[0] == SERVICE_MODE_WORDS[0].as_bytes()
        && pair[1] == SERVICE_MODE_WORDS[1].as_bytes()
        && !behind_current_word;
}

/// Whether a byte ends a command line word.
pub(crate) fn is_word_end(byte: &u8) -> bool {
    return *byte == 0;
}

/// The folders of the earlier journals as the look knows them,
/// and where the process that is looked at keeps its files.
#[cfg(unix)]
pub(crate) struct JournalNames<'a> {
    /// The earlier journals,
    /// as the service spells them.
    pub(crate) spellings: &'a [PathBuf],
    /// `<proc root>/<pid>` of the process,
    /// whose `cwd` is the folder a relative word is resolved in.
    pub(crate) process: &'a Path,
}

/// The hold a helper that runs on the journal spelled at `index` makes:
/// the shared journal is the second spelling.
#[cfg(unix)]
fn hold_of_journal_at(index: usize) -> Hold {
    return if index == 1 {
        Hold::SharedHelperRuns
    } else {
        Hold::InstanceHelperRuns
    };
}

/// What:
///  Which earlier journal a journal word of a command line names:
/// the instance's own (the first of the look's journals),
/// the
///  shared one (the second),
/// no earlier journal's,
/// or one that cannot be resolved.
/// Why:
///   The word is compared as it is spelled,
/// then by the folder it resolves to (device and inode):
/// the service derives
///  `/var/home/user/...`,
/// and an earlier helper started with `/home/user/...` names the same folder through the link `/home`;
/// a
///  relative word means the folder in the working directory of the process that was started with it,
/// which `/proc/<pid>/cwd`
///  gives (the kernel follows it).
/// A stat that fails for a reason other than the folder being absent says nothing about which folder the word names,
///  and neither does a relative word whose working directory cannot be read:
/// an open question must
///  not start a second service on the journal.
/// An absolute word whose folder is not there is the one case that is settled without the folder:
/// it is
///  compared by the path it spells,
/// and a folder that is not there holds no batch of this worktree,
/// so the
///  word alone holds nothing back.
/// Which of these it is decides what the person is told to stop,
/// and
///  [`is_service_of`] decides what the rest of a command line makes of a word that names no earlier journal:
/// a
///  folder that is there is another program's,
/// while a folder that is not there may be a spelling whose link another
///  command removed after an earlier helper of this worktree was started with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const journalWord = (word: string, names: JournalNames): JournalWord => names.find(j => spelled(j) === word)?.hold ?? sameFolderAs(resolve(word, names.process + '/cwd')) ?? (!resolvable(word) ? unsure(word, whyNot(word)) : !absent(word) ? { elsewhere: true } : samePathAs(resolvedSoFar(word))?.hold ?? { absentAndElsewhere: true })
/// ```
#[cfg(unix)]
fn journal_word_names(
    names: &JournalNames,
    word: &[u8],
) -> JournalWord {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::MetadataExt;
    for (index, journal) in names.spellings.iter().enumerate() {
        if path_word(journal.as_path()).as_slice() == word {
            return JournalWord::Names(hold_of_journal_at(index));
        }
    }
    let spelled: &Path = Path::new(std::ffi::OsStr::from_bytes(word));
    let absolute: bool = spelled.is_absolute();
    let resolved: PathBuf = if absolute {
        spelled.to_path_buf()
    } else {
        names.process.join("cwd").join(spelled)
    };
    let found = match std::fs::metadata(resolved.as_path()) {
        Ok(found) => found,
        // An absolute word whose folder is not there names a folder that holds no batch now,
        // and a starting earlier helper of the worktree is in that state until it makes its folder:
        // the path it spells says whether it is one of this worktree's journals.
        Err(error) if absolute && error.kind() == std::io::ErrorKind::NotFound => {
            let named: PathBuf = match resolved_as_far_as_the_file_system_knows(spelled) {
                Ok(named) => named,
                Err(unknown) => {
                    return JournalWord::Unsure {
                        word: word.to_vec(),
                        relative: false,
                        reason: unknown.to_string(),
                    };
                }
            };
            for (index, journal) in names.spellings.iter().enumerate() {
                if journal.is_absolute()
                    && matches!(resolved_as_far_as_the_file_system_knows(journal), Ok(known) if known == named)
                {
                    return JournalWord::Names(hold_of_journal_at(index));
                }
            }
            return JournalWord::AbsentAndElsewhere;
        }
        Err(error) => {
            return JournalWord::Unsure {
                word: word.to_vec(),
                relative: !absolute,
                reason: error.to_string(),
            };
        }
    };
    for (index, journal) in names.spellings.iter().enumerate() {
        if matches!(std::fs::metadata(journal), Ok(known) if (known.dev(), known.ino()) == (found.dev(), found.ino()))
        {
            return JournalWord::Names(hold_of_journal_at(index));
        }
    }
    return JournalWord::Elsewhere;
}

/// Whether the earlier release's mode words start at any place of `words`.
#[cfg(unix)]
fn has_earlier_mode(words: &[&[u8]]) -> bool {
    for at in 0..words.len() {
        if is_earlier_mode_at(words, at) {
            return true;
        }
    }
    return false;
}

/// Whether this release's mode words start at any place of `words` behind its own command word:
/// the service of this release,
/// which serves the worktree's new journal.
#[cfg(unix)]
fn has_current_mode(words: &[&[u8]]) -> bool {
    for at in 1..words.len() {
        let Some(pair) = words.get(at..at + SERVICE_MODE_WORDS.len()) else {
            continue;
        };
        if words[at - 1] == COMMAND_WORD.current.as_bytes()
            && pair[0] == SERVICE_MODE_WORDS[0].as_bytes()
            && pair[1] == SERVICE_MODE_WORDS[1].as_bytes()
        {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether one command line (words split at NUL bytes) is a service of the worktree.
/// Why:
///   Two shapes of start arguments exist.
/// The units written before the service became a mode of the program started it as
///  `<program> --journal <state>/cli-git/service/<instance> --git <git> --instance <instance>`,
/// and the first releases' single
///  service as `<program> --journal <state>/cli-git/service --git <git>`;
/// neither has the mode words.
/// The later units start
///  `<program> cli-git service run --journal <folder> --instance <instance>`.
/// A process keeps the arguments it was started
///  with,
/// so helpers of every shape can be alive.
/// A process with the journal option followed by a word that may name one of the
///  earlier folders is a service of the worktree;
/// so is one with the mode words and the instance option followed by the instance.
/// A word that names no earlier journal is read with the rest of the command line:
/// a folder that is there is another program's,
/// so the program writes its batches there;
/// a folder that is
///  not there is read as this worktree's helper when the same command line names this worktree's
///  instance,
/// because that is how the earlier release's unit started a helper of one worktree
///  (`service_units/cli-git@.service`,
/// tag `monochromatic-git-policy-cli-v0.5.34`),
/// and a link in the journal spelling it
///  was given may have been removed after that start.
/// This release's own service is no earlier helper:
/// its
///  mode words follow this release's command word,
/// and its journal is the new folder,
/// so it holds nothing back
///  whatever word it carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isServiceOf = (words: string[], names: JournalNames, look: ServiceLook) => {
///   const word = firstJournalWord(words, names)
///   if (word?.names) return word.names
///   if (word?.unsure) return word.unsure
///   if (word?.absentAndElsewhere && namesInstance(words, look) && !hasCurrentMode(words)) return { hold: 'InstanceHelperRuns' }
///   return hasPair(words, 'service', 'run') && namesInstance(words, look) ? { hold: 'InstanceHelperRuns' } : undefined
/// }
/// ```
#[cfg(unix)]
pub(crate) fn is_service_of(
    words: &[&[u8]],
    look: &ServiceLook,
    names: &JournalNames,
) -> Option<ServiceFound> {
    let mode_in_a_row: bool = has_earlier_mode(words);
    let mut instance_named: bool = false;
    let mut journal_named: Option<JournalWord> = None;
    let mut journal_elsewhere: Option<JournalWord> = None;
    for pair in words.windows(2) {
        if pair[0] == INSTANCE_OPTION.as_bytes() && pair[1] == look.instance.as_bytes() {
            instance_named = true;
        }
        if pair[0] != JOURNAL_OPTION.as_bytes() {
            continue;
        }
        let named: JournalWord = journal_word_names(names, pair[1]);
        // A word that names one of the earlier journals,
        // or that cannot be resolved,
        // is stronger evidence than one that names no earlier journal,
        // wherever it stands on the command line.
        if matches!(named, JournalWord::Names(_) | JournalWord::Unsure { .. }) {
            if journal_named.is_none() {
                journal_named = Some(named);
            }
        } else if journal_elsewhere.is_none() {
            journal_elsewhere = Some(named);
        }
    }
    // The helper shape of this release's earlier units:
    // the mode words and this worktree's instance.
    let helper_shape: bool = mode_in_a_row && instance_named;
    let helper_of_this_worktree: bool = instance_named && !has_current_mode(words);
    let Some(named) = journal_named.or(journal_elsewhere) else {
        return if helper_shape {
            Some(ServiceFound::Hold(Hold::InstanceHelperRuns))
        } else {
            None
        };
    };
    return match named {
        JournalWord::Names(hold) => Some(ServiceFound::Hold(hold)),
        JournalWord::Unsure {
            word,
            relative,
            reason,
        } => Some(ServiceFound::Unsure {
            word,
            relative,
            reason,
        }),
        // A folder that is not there holds no batch by itself,
        // but the word may be a spelling the command line's instance says was this worktree's.
        JournalWord::AbsentAndElsewhere if helper_of_this_worktree => {
            Some(ServiceFound::Hold(Hold::InstanceHelperRuns))
        }
        // A folder that is there and is another's says where the program writes,
        // so only the helper shape of the mode words counts for it.
        JournalWord::AbsentAndElsewhere | JournalWord::Elsewhere if helper_shape => {
            Some(ServiceFound::Hold(Hold::InstanceHelperRuns))
        }
        JournalWord::AbsentAndElsewhere | JournalWord::Elsewhere => None,
    };
}
