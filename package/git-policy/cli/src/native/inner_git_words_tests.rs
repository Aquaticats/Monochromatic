//! Controls for what is kept of the words of a Git run that carries no `--quiet`: what counts as words, what the service's
//!  entry holds, and that a run which said nothing makes no entry.
//! Why:
//!  The owner dropped `--quiet` from every run the wrapper and the service start (`doc/planning/cli-git-slow-server-and-inner-quiet.md`),
//!  and the words must reach the journal instead of the discarded capture.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(entriesOf(() => recordText(['fetch'], 'From origin'))).toHaveLength(1);
//! ```

use super::*;
use crate::service_timing::capture::entries_of;

/// Standard output comes first, standard error second, each trimmed, and a run that said nothing has no words.
#[test]
fn the_words_of_a_run_are_its_trimmed_streams_in_order() {
    assert_eq!(words_of(b"Updating a..b\n", b"warning: x\n"), Some(String::from("Updating a..b\nwarning: x")));
    assert_eq!(words_of(b"", b"From origin\n"), Some(String::from("From origin")));
    assert_eq!(words_of(b"  \n", b"\n\n"), None);
    assert_eq!(words_of(b"", b""), None);
}

/// A run that said something makes one entry at the ordinary level, naming the command and carrying the words and the field a
///  reader filters on; a run that said nothing makes none.
#[test]
fn the_service_keeps_the_words_in_one_ordinary_entry() {
    let entries = entries_of(|| {
        record_text(&["fetch", "--no-tags", "origin"], "From origin\n * branch main -> main\n");
        record_text(&["fetch", "--no-tags", "origin"], "  \n");
    });
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(entries[0].priority, Priority::Info);
    assert!(entries[0].message.contains("git fetch --no-tags origin said: From origin\n * branch main -> main"), "{:?}", entries[0]);
    assert!(entries[0].fields.contains(&(String::from(COMMAND_FIELD), String::from("fetch --no-tags origin"))), "{:?}", entries[0]);
}

/// The userinfo of a URL in Git's words is masked, as every entry of the service is.
#[test]
fn credentials_in_the_words_are_masked() {
    let entries = entries_of(|| {
        record_text(&["fetch", "origin"], "From https://user:secret@example.invalid/repo\n");
    });
    assert_eq!(entries.len(), 1);
    assert!(!entries[0].message.contains("secret"), "{:?}", entries[0]);
    assert!(entries[0].message.contains("<credentials>"), "{:?}", entries[0]);
}
