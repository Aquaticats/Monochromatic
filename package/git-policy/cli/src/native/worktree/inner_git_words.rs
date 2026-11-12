//! What:
//!  Where the words of a Git run the wrapper or the service started on its own go,
//! now that none of those runs carries
//!  `--quiet` or `-q`:
//! one journal entry,
//! from the service for a run it made and from the wrapper for a run it made while
//!  carrying out a person's command.
//! Why:
//!  The owner's rule is "No the wrapper should never append --quiet.
//! Letting native git emit whatever it wants to is expected",
//!  and for the wrapper's and the service's own runs the answer was "Drop it everywhere (Recommended)".
//! Every such run is
//!  captured (`run_git` pipes both streams into `GitOutput`),
//! so Git's words would be discarded with the capture;
//! this
//!  keeps them where a failure of the same run is already recorded (`journalctl`).
//! A hook's standard output and the service's
//!  protocol replies are never the destination,
//! and no other option or redirect hides the words in the meantime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = [stdout, stderr].filter((text) => text.trim() !== '').join('\n'); if (words) journal.info(`git ${command} said: ${words}`);
//! ```

/// Masks the userinfo of any URL in Git's words,
/// as every journal entry of the service is masked.
use crate::lfs_output_mask::mask_url_secrets_text;
/// The entry from a wrapper a person ran,
/// which only Linux writes.
#[cfg(target_os = "linux")]
use crate::client_journal::record_entry;
/// The priority an entry carries,
/// and the service's way to write one.
use crate::journald_native::Priority;
use crate::service_timing::{in_service, log_entry};
/// The captured output of one Git start.
use crate::transaction_git::GitOutput;

/// The field a reader filters the journal on to find the entries of this module.
pub const COMMAND_FIELD: &str = "CLI_GIT_INNER_COMMAND";

/// What:
///  The text of a Git run's words:
/// its standard output,
/// then its standard error,
/// trimmed,
/// or nothing when it said nothing.
/// Why:
///   A run that said nothing makes no entry,
/// so an entry always carries something to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [stdout, stderr].map((text) => text.trim()).filter(Boolean).join('\n') || undefined
/// ```
pub fn words_of(stdout: &[u8], stderr: &[u8]) -> Option<String> {
    // The two streams in the order they are shown: standard output, then standard error.
    let mut parts: Vec<String> = Vec::new();
    for bytes in [stdout, stderr] {
        // Bytes that are not UTF-8 become the replacement character, so a word is never lost to an encoding.
        let text: String = String::from_utf8_lossy(bytes).trim().to_owned();
        if !text.is_empty() {
            parts.push(text);
        }
    }
    if parts.is_empty() {
        // `None` is "this run said nothing", which makes no entry.
        return None;
    }
    // `Some(...)` wraps the joined words: "there is something to record".
    return Some(parts.join("\n"));
}

/// What:
///  Record the words of a finished run,
/// whichever program made it:
/// the service's log when this is the service,
/// the client
///  entry otherwise.
/// Why:
///   The auto-push's fetch and fast-forward run in the service and in the wrapper's own stacked push,
/// and the shadow's `init`
///  in both,
/// so the choice is made here once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordWords(['fetch', '--no-tags', remote], `${stdout}${stderr}`)
/// ```
pub fn record_text<S: AsRef<str>>(arguments: &[S], text: &str) {
    let trimmed: &str = text.trim();
    if trimmed.is_empty() {
        return;
    }
    // The command line as it reads in a terminal, so the entry names what ran.
    let mut words: Vec<&str> = Vec::new();
    for argument in arguments {
        words.push(argument.as_ref());
    }
    let command: String = words.join(" ");
    let message: String = mask_url_secrets_text(format!("git {command} said: {trimmed}").as_str());
    let fields: [(&str, String); 1] = [(COMMAND_FIELD, mask_url_secrets_text(command.as_str()))];
    if in_service() {
        let _delivered = log_entry(Priority::Info, message.as_str(), &fields);
        return;
    }
    deliver_from_wrapper(message.as_str(), &fields);
}

/// Send the entry of a wrapper a person ran to the journal;
/// whether the socket took it is of no use to the command.
#[cfg(target_os = "linux")]
fn deliver_from_wrapper(message: &str, fields: &[(&str, String)]) {
    let _sent: bool = record_entry(Priority::Info, message, fields);
}

/// Only Linux writes the wrapper's journal entries;
/// elsewhere the words are not kept.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn deliver_from_wrapper(_message: &str, _fields: &[(&str, String)]) {}

/// What:
///  Record the words of a finished run from its captured output.
/// Why:
///   Most runs are captured as a `GitOutput`;
/// this is `record_text` for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordWords(['reset', newCommit, '--', path], output)
/// ```
pub fn record_words<S: AsRef<str>>(arguments: &[S], output: &GitOutput) {
    record_streams(arguments, output.stdout.as_slice(), output.stderr.as_slice());
}

/// What:
///  Record the words of a finished run from its two streams.
/// Why:
///   The auto-push's queries come back as two byte streams,
/// not as a `GitOutput`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordWords(['merge', '--ff-only', oid], { stdout, stderr })
/// ```
pub fn record_streams<S: AsRef<str>>(arguments: &[S], stdout: &[u8], stderr: &[u8]) {
    if let Some(text) = words_of(stdout, stderr) {
        record_text(arguments, text.as_str());
    }
}

/// Controls for what is recorded.
#[cfg(test)]
#[path = "inner_git_words_tests.rs"]
mod tests;
