//! What:
//!  The sentence a typed network command prints after Git's own words when OpenSSH refused the connection over a host key,
//!  or over a key nobody could unlock,
//!  and the record that says one such sentence was printed for this command.
//! Why:
//!  A person at a new host key, or with a key no agent holds, reads OpenSSH's line and Git's `fatal:` with no next step;
//!  the wrapper owns the child, so it is the process that can name one
//!  (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`).
//!  The three texts are the proposal's kept files byte for byte, filled with the host the person's remote names
//!  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "The words").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // sentenceFor(tail, target): string | undefined; printedForThisCommand(value?: boolean): boolean
//! ```

// What: `AtomicBool` is one flag many threads read and write without a lock; its sibling `Cell` is one thread's own.
// Why:  The upload's own child is started from a thread of this process, and the record is read there.
//
// In TS you'd write (pseudocode):
// ```ts
// const printed = new Int32Array(1); // shared, atomic swaps
// ```
use std::sync::atomic::{AtomicBool, Ordering};

/// What:
///  The words the reading needs, which the three refusals share: OpenSSH's last line for any host-key refusal, and its line for
///  a key the server did not accept or could not ask a passphrase for.
/// Why:
///  The reading is the shipped detector's, so one refusal cannot be read two ways by two faces of the wrapper
///  (`lfs/lfs_pre_push.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = (tail: string) => strictRefusal(tail) ? 'strict' : unknownHostKey(tail) ? 'host-key' : keyFailure(tail) ? 'passphrase' : undefined;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// The host key is unknown and the person's settings refuse to ask about a new one.
    StrictChecking,
    /// The host key is unknown, or one this machine holds changed.
    HostKey,
    /// The key the connection offered was refused, or its passphrase could not be asked for.
    KeyNotUsable,
}

/// What:
///  Whether a sentence was printed for this command, so a second face of it stays silent instead of saying it twice.
/// Why:
///  A typed push that uploads Git LFS objects runs the upload's own child before the push,
///  and that upload explains a refused key from the same transcript;
///  one refusal must be told once
///  (`doc/decision/cli-git-typed-network-command-relays-at-a-new-host-key.md`, the limit naming the double sentence).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PRINTED = { value: false };
/// ```
static PRINTED: AtomicBool = AtomicBool::new(false);

/// What:
///  Whether a sentence about a refused key was printed for this command already.
/// Why:
///  The upload's explanation asks this before adding its own, so the person reads one paragraph and not two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// printedForThisCommand(); // true when this command already said it
/// ```
pub fn printed_for_this_command() -> bool {
    return PRINTED.load(Ordering::SeqCst);
}

/// What:
///  Record that a sentence about a refused key was printed for this command.
/// Why:
///  A face that prints the sentence itself (the probe of a typed push) must leave the record set, so the upload's own explanation
///  stays silent rather than saying it twice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rememberPrinted();
/// ```
pub fn remember_printed() {
    PRINTED.store(true, Ordering::SeqCst);
}

/// What:
///  The refusal a refusal's transcript reads as, or nothing when it reads as no refusal at all.
/// Why:
///  Three sentences are printed for three refusals, and each is chosen from what OpenSSH printed;
///  a transcript of a missing repository, a refused connection or a changed key at the default log level reads as none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusalIn(tail: string): Refusal | undefined
/// ```
pub fn refusal_in(tail: &str) -> Option<Refusal> {
    // The three readings below drop what Git relayed from the far end themselves, so a server's hook writing the refusal's words
    //  on a push that then lands is not read as a refusal `lfs/lfs_pre_push.rs`, `own_lines_of`).
    // A strict refusal is also an unknown key's own line, so it is asked first: its advice overrides the person's setting.
    if crate::lfs_pre_push::reads_as_a_strict_host_key_refusal(tail) {
        return Some(Refusal::StrictChecking);
    }
    if crate::lfs_pre_push::reads_as_unknown_host_key(tail) {
        return Some(Refusal::HostKey);
    }
    if crate::lfs_pre_push::reads_as_a_key_failure(tail) {
        return Some(Refusal::KeyNotUsable);
    }
    return None;
}

/// What:
///  The sentence for this refusal, filled with the host the person's remote names.
/// Why:
///  The host is the one the person typed, because a `known_hosts` line is recorded under the name the connection asked about;
///  advice naming the address instead would ask them to accept a second key for the same machine
///  (`doc/planning/cli-git-typed-network-command-next-step-at-a-new-host-key-design.md`, "Where the host comes from").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sentenceFor(refusal: Refusal, target: Target): string
/// ```
pub fn sentence_for(refusal: Refusal, target: &super::a_target_the_sentence_may_name::Target) -> String {
    // `match` picks by variant and binds nothing here: each arm is one text.
    return match refusal {
        Refusal::HostKey => format!(
            "monochromatic-git-policy-cli: this machine does not know the SSH host key of `{host}`, or holds an outdated one for it, and no command the wrapper runs asks whether to trust a new one, so no connection got past the key. Connect with plain `ssh {connect}`, which the wrapper does not touch, and answer the question it asks; if it names a key that changed instead, compare the fingerprint it shows with the one you know to be right before you accept it, then remove the line it names and connect once more, which then asks about the new key, and run this command again.\n",
            host = target.host,
            connect = target.connect,
        ),
        Refusal::StrictChecking => format!(
            "monochromatic-git-policy-cli: this machine has no SSH host key recorded for `{host}`, and your settings refuse to ask about a new one (`StrictHostKeyChecking=yes`), so the connection stopped before any login. Connect once with plain `ssh -o StrictHostKeyChecking=ask {connect}`, which the wrapper does not touch and which asks once, then run this command again; or add that host's key to `known_hosts` yourself.\n",
            host = target.host,
            connect = target.connect,
        ),
        Refusal::KeyNotUsable => String::from(
            "monochromatic-git-policy-cli: the key this connection offered was refused or could not be unlocked, and no command the \
             wrapper runs asks a person for a passphrase, so a key no agent holds cannot be used and a key the server does not \
             accept answers the same way. Start an agent with `eval \"$(ssh-agent)\"` (`eval (ssh-agent -c)` in fish) and load the \
             key with `ssh-add` (`ssh-add -l` lists what the agent holds), or use a key without a passphrase, then run this command \
             again; a key the server still refuses needs the server's account to accept it.\n",
        ),
    };
}

/// What:
///  Print the sentence this transcript calls for, when the child ended non-zero, and record that one was printed.
/// Why:
///  Nothing is printed on a command that succeeded: a server's hook can write the refusal's words into a push that then lands,
///  and a message about a key that refused nothing misleads.
///  The record lets the LFS upload stay silent afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (code !== 0 && (r = refusalIn(tail))) { write(2, sentenceFor(r, target)); printed = true; }
/// ```
pub fn print_after_a_refusal(kept: &[u8], ended_with: i32, target: &super::a_target_the_sentence_may_name::Target) {
    if ended_with == 0 {
        return;
    }
    // What: `from_utf8_lossy` reads the bytes as text and replaces what is not UTF-8, which a refusal's lines never contain.
    // Why:  The reading compares words, and one stray byte must not hide them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const tail = decoder.decode(kept, { stream: false });
    // ```
    let tail: String = String::from_utf8_lossy(kept).into_owned();
    let Some(refusal) = refusal_in(tail.as_str()) else {
        return;
    };
    let sentence: String = sentence_for(refusal, target);
    // What: `write_all` writes every byte of the sentence; a failure is the closed standard error and nothing else.
    // Why:  The person reads the sentence on the stream the command's own words came on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { process.stderr.write(sentence); } catch {}
    // ```
    let mut standard_error = std::io::stderr();
    let _written = std::io::Write::write_all(&mut standard_error, sentence.as_bytes());
    PRINTED.store(true, Ordering::SeqCst);
}

/// The words and their reading are controlled in the test module.
#[cfg(test)]
#[path = "the_sentence_after_the_refusal_tests.rs"]
mod tests;
