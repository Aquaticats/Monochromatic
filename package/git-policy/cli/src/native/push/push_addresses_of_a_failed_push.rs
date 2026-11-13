//! What:
//!  The addresses a failed push of a remote was sent to,
//!  read from the remote's own configuration,
//!  in the two faces a push has (the synchronous one and the awaited one).
//! Why:
//!   A remote name is not an address:
//! `git remote set-url --push origin DISABLED` keeps the push address where Git keeps it,
//! and the words of a failed push carry it only inside Git's own sentence
//! (`fatal: 'DISABLED' does not appear to be a git repository` is the same line for a path that is not there),
//! so the address a retry plan's fate depends on is asked for
//! (`push_address_that_cannot_be_a_remote.rs`).
//! A remote with several push addresses has each of them;
//! a destination that is no remote name at all is its own address.
//! Nothing here fails the push:
//! a query that cannot run leaves the destination as the one address to judge,
//! and a judgement that sees no address keeps its retries
//! ("where you cannot tell the two apart,
//! it is the second kind").
//! It is asked only after a push failed,
//! so a commit whose push lands pays no query for it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! pushAddressesOf(git, 'origin') // ['DISABLED']
//! ```

/// The failed push's destination,
/// and the query that reads a remote's addresses without starting a process when it can.
use crate::auto_push_single_flight::{PushGit, run_query, run_query_awaiting};
/// The answer one query gives,
/// whose standard output holds one address per line.
use crate::bounded_query::QueryOutput;
/// Owned OS strings for Git arguments.
use std::ffi::OsString;

/// What:
///  The addresses a push of `remote` goes to,
///  read by a caller on its own thread.
/// Why:
///   The synchronous face of a push asks the same question the awaited one does,
/// with the same words,
/// so a push cannot differ between the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushAddressesOf(git, remote) // QueryOutput → addresses
/// ```
pub(crate) fn push_addresses_of(git: &PushGit, remote: &str) -> Vec<String> {
    let arguments: Vec<OsString> = query_arguments(remote);
    return push_addresses_from(run_query(git, &arguments).ok().as_ref(), remote);
}

/// The same reading as a caller that is itself a task of the runtime.
pub(crate) async fn push_addresses_of_awaiting(git: &PushGit, remote: &str) -> Vec<String> {
    let arguments: Vec<OsString> = query_arguments(remote);
    return push_addresses_from(run_query_awaiting(git, &arguments).await.ok().as_ref(), remote);
}

/// What:
///  The question both faces ask:
///  every push address the configuration gives `remote`.
/// Why:
///   `--push` reads the push addresses rather than the fetch one,
/// and `--all` gives every one of them,
/// because Git stops a push at the first that fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['remote', 'get-url', '--push', '--all', remote]
/// ```
fn query_arguments(remote: &str) -> Vec<OsString> {
    return ["remote", "get-url", "--push", "--all", remote].iter().map(OsString::from).collect();
}

/// What:
///  The addresses one answer of `git remote get-url --push --all` names,
///  byte for byte as the configuration spells them,
///  or the destination itself when the answer names none.
/// Why:
///   A destination that is no remote name is its own address,
/// and Git refuses the query for it;
/// an answer that failed or named nothing says nothing about where the push went,
/// so the destination is judged as it stands,
/// which keeps its retries unless it is itself an address that can never be a remote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// addressesFrom(answer, remote) // answer?.code === 0 ? lines(answer.stdout) : [remote]
/// ```
fn push_addresses_from(answer: Option<&QueryOutput>, remote: &str) -> Vec<String> {
    let Some(output) = answer else { return vec![remote.to_owned()] };
    if output.code != Some(0) {
        return vec![remote.to_owned()];
    }
    let mut addresses: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        // What: The address is taken as the configuration spells it, trailing and leading spaces included.
        // Why:  `DISABLED ` is a legal path whose one difference from the reserved word is a space, and a bare
        //       repository of that name receives a push; trimming it read that repository as the reserved word and
        //       ended a plan a retry would have landed (the review of issue 6293, finding 4).
        if !line.is_empty() {
            addresses.push(line.to_owned());
        }
    }
    return if addresses.is_empty() { vec![remote.to_owned()] } else { addresses };
}

/// The readings above are controlled beside them.
#[cfg(test)]
#[path = "push_addresses_of_a_failed_push_tests.rs"]
mod tests;
