//! What:
//!  Controls on the pairs of names the program answers to and keeps.
//! Why:
//!  The owner decided the spelling of each current name and that each earlier spelling stays readable;
//!  a pair that changed by accident would strand files an older release wrote.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(PROGRAM).toEqual({ current: 'monochromatic-git-policy-cli', earlier: 'cli-git-native' });
//! ```

/// Import the pairs under test.
use super::{
    COMMAND_WORD, COMMIT_TIMING_LOG, DATA_FOLDER, DISCARD_AUDIT_LOG, MERGE_INTENT_FOLDER, MESSAGE_PREFIX, PROGRAM,
    PUSH_TRIGGER_LOG, RenamedName, SERVICE_UNIT, SETTINGS_FILE, WORKTREE_ADD_TIMING_LOG, WORKTREE_COPY_RECORD_FOLDER,
};

/// Every pair this release keeps,
///  in the order the module declares them.
const ALL_PAIRS: [RenamedName; 12] = [
    PROGRAM,
    COMMAND_WORD,
    SETTINGS_FILE,
    MESSAGE_PREFIX,
    DATA_FOLDER,
    DISCARD_AUDIT_LOG,
    COMMIT_TIMING_LOG,
    WORKTREE_ADD_TIMING_LOG,
    PUSH_TRIGGER_LOG,
    MERGE_INTENT_FOLDER,
    WORKTREE_COPY_RECORD_FOLDER,
    SERVICE_UNIT,
];

/// The program is named after the crate and the earlier spelling is the installed file the owner renamed.
#[test]
fn program_and_command_word_follow_the_crate_name() {
    assert_eq!(PROGRAM.current, "monochromatic-git-policy-cli");
    assert_eq!(PROGRAM.earlier, "cli-git-native");
    assert_eq!(COMMAND_WORD.current, PROGRAM.current);
    assert_eq!(COMMAND_WORD.earlier, "cli-git");
}

/// The settings file keeps its extension and gains the program's name.
#[test]
fn settings_file_is_the_program_name_with_the_same_ending() {
    assert_eq!(SETTINGS_FILE.current, format!("{}.config.jsonc", PROGRAM.current));
    assert_eq!(SETTINGS_FILE.earlier, format!("{}.config.jsonc", COMMAND_WORD.earlier));
}

/// A current spelling never equals its earlier spelling,
///  or the move of an earlier name would be a move onto itself.
#[test]
fn no_pair_has_the_same_spelling_twice() {
    for pair in ALL_PAIRS {
        assert_ne!(pair.current, pair.earlier, "{pair:?}");
    }
}

/// No current spelling starts with an earlier one followed by nothing or a separator that the earlier reader would take
///  as its own,
///  so a reader of the earlier name never mistakes a current name for it.
#[test]
fn an_earlier_spelling_never_matches_a_current_name_by_prefix_and_separator() {
    for pair in ALL_PAIRS {
        for other in ALL_PAIRS {
            let prefix_of_current: bool = other.current.starts_with(pair.earlier);
            assert!(!prefix_of_current, "{} starts with {}", other.current, pair.earlier);
        }
    }
}

/// The names of the files and folders the program keeps all begin with the program's name,
///  so one search finds everything it left in a Git directory.
#[test]
fn kept_names_begin_with_the_program_name() {
    for pair in [DATA_FOLDER, DISCARD_AUDIT_LOG, COMMIT_TIMING_LOG, WORKTREE_ADD_TIMING_LOG, PUSH_TRIGGER_LOG, MERGE_INTENT_FOLDER, WORKTREE_COPY_RECORD_FOLDER] {
        assert!(pair.current.starts_with(PROGRAM.current), "{pair:?}");
        assert!(pair.earlier.starts_with("cli-git"), "{pair:?}");
    }
}
