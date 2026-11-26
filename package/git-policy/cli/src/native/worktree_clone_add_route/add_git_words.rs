//! What:
//!  The words the route's one Git invocation is given: the person's arguments with
//!  `--no-checkout`, `--lock` and the managed sentence inserted where Git reads
//!  options, and the refusal of a person who asked for their own lock.
//! Why:
//!  The design's invocation is
//!  `git worktree add --no-checkout --lock --reason <sentence> <slot> <commit-ish>`,
//!  from one list and never through a shell
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`, "The invocation").
//!  A person's own `--lock` or `--reason` would be the words the wrapper's proof
//!  reads, so the route refuses that form instead of locking the registration
//!  with words it did not write.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = theLockedAddWords(personArgs, insertAt, nonce); // throws for --lock/--reason
//! ```

use crate::worktree_clone_binding_record::lock_words::lock_words;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt as _;

/// The three options the route adds to the person's add,
/// in the order Git reads them.
pub const ROUTE_OPTIONS: [&str; 2] = ["--no-checkout", "--lock"];

/// The words a person may give that the route's own invocation takes for itself,
/// as Git spells them before any `=<value>`.
pub const PERSON_LOCK_WORDS: [&[u8]; 4] = [b"--lock", b"--reason", b"--checkout", b"--no-checkout"];

/// What:
///  The person's own words with the route's options inserted after `add`.
/// Why:
///  The insertion point is the plan's, read from the same option table the ladder
///  uses, so a long word, a bundled short word and a value cannot move it.
///
/// # Errors
/// The person names `--lock`,
/// `--reason`,
/// `--checkout` or `--no-checkout`,
/// or the insertion point is outside the argument list.
pub fn the_locked_add_words(
    arguments: &[OsString],
    insert_at: usize,
    nonce: &RequestNonce,
) -> Result<Vec<OsString>, String> {
    if insert_at == 0 || insert_at > arguments.len() {
        return Err(String::from(
            "the add's insertion point is outside the argument list, so the managed lock cannot be taken",
        ));
    }
    if let Some(word) = person_lock_word(arguments) {
        return Err(format!(
            "the add names `{}`, and the ZFS clone route takes the managed lock itself. Ways forward: run the add without `--lock`, `--reason` or `--checkout`, and release the lock you made if you want the wrapper's",
            word.to_string_lossy()
        ));
    }
    let mut words: Vec<OsString> = Vec::with_capacity(arguments.len() + ROUTE_OPTIONS.len() + 2);
    words.extend(arguments[..insert_at].iter().cloned());
    words.push(OsString::from(ROUTE_OPTIONS[0]));
    words.push(OsString::from(ROUTE_OPTIONS[1]));
    words.push(OsString::from("--reason"));
    words.push(OsString::from(lock_words(nonce)));
    words.extend(arguments[insert_at..].iter().cloned());
    return Ok(words);
}

/// What:
///  The first word that asks for a lock or a checkout the route takes itself,
///  when the person gave one.
/// Why:
///  A person's `--reason` would replace the sentence the wrapper's proof reads,
///  and their `--checkout` would undo `--no-checkout`.
pub fn person_lock_word(arguments: &[OsString]) -> Option<&OsString> {
    return arguments.iter().find(|word| {
        let bytes: &[u8] = word.as_bytes();
        return PERSON_LOCK_WORDS.iter().any(|wanted| {
            return bytes == *wanted
                || bytes
                    .strip_prefix(*wanted)
                    .is_some_and(|rest| return rest.starts_with(b"="));
        });
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "add_git_words_tests.rs"]
mod tests;
