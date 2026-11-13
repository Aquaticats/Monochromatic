//! What:
//!  Whether a push address can never be a remote at all,
//! and the words for one that cannot.
//! Why:
//!  The owner's decision of 2026-11-13 ("End the plan at once,
//! say so once",
//! issue 6293):
//! a push whose address cannot be a remote ends its plan on the first refusal,
//! with one line that says the push was not made and why;
//! an address that might come back (a drive that is not mounted,
//! a host that does not answer,
//! a login that is refused) keeps its retries as before.
//! The line between the two cannot come from Git's words:
//! measured with Git 2.56.0,
//! `DISABLED`,
//! `no-such-remote`,
//! an unmounted `/mnt/not-mounted/repo.git`,
//! a directory that exists but is no repository,
//! an address with a space and one with a newline all answer the same
//! `fatal: '<address>' does not appear to be a git repository`;
//! so the address itself is read,
//! and only a shape no machine state can make a remote is final:
//!  - an empty address:
//!    `fatal: bad repository ''`,
//!    and no string names nothing;
//!  - this project's own word `DISABLED`,
//!    which rule `APQ` tells every agent to point a throwaway
//!    clone's push address at (`../DISABLED` and a repository of that name do receive a push,
//!    so this is a
//!    reservation of the project and not a syntax of Git);
//!  - an address that starts with a dash,
//!    which Git reads as an option
//!    (`error: did you mean '--force'?`,
//!    exit 129) and never as a place to push to;
//!  - an address that starts with `://`,
//!    whose protocol name is empty:
//!    `fatal: protocol '' is not supported`,
//!    and no protocol has an empty name.
//! Every other address is retried,
//!    whatever Git answered:
//! a URL with an unknown scheme waits for its helper to be installed
//! (`git: 'remote-foo' is not a git command`),
//! a host that does not answer for its network
//! (`fatal: unable to access '…': Failed to connect`),
//! and a path that is not there for what may be mounted or cloned later.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! cannotBeARemote('DISABLED') // 'the-reserved-word'; cannotBeARemote('https://host/x') // undefined
//! ```

/// The word this project reserves for a push address that must never be pushed to
/// (`AGENTS.md`,
/// rule `APQ`:
/// `git remote set-url --push origin DISABLED`).
pub const DISABLED_ADDRESS: &str = "DISABLED";

/// The prefix of a URL whose protocol name is empty.
const NO_PROTOCOL: &str = "://";

/// Why an address can never name a remote.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CannotBeARemote {
    /// The address is empty,
    /// so it names nothing.
    Empty,
    /// The address is the word this project reserves for "nothing is pushed".
    TheReservedWord,
    /// The address starts with a dash,
    /// so Git reads it as an option.
    AnOption,
    /// The address begins with `://`,
    /// so its protocol has no name.
    NoProtocolName,
}

/// What:
///  Why `address` can never be a remote,
///  or nothing when it might be one now or later.
/// Why:
///   The four answers come from what Git itself refuses whatever the machine's state is;
/// every other shape is left to its retries,
/// and this module's own doc quotes Git's answer for each of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cannotBeARemote(address) // CannotBeARemote | undefined
/// ```
pub fn cannot_be_a_remote(address: &str) -> Option<CannotBeARemote> {
    if address.is_empty() {
        return Some(CannotBeARemote::Empty);
    }
    if address == DISABLED_ADDRESS {
        return Some(CannotBeARemote::TheReservedWord);
    }
    if address.starts_with('-') {
        return Some(CannotBeARemote::AnOption);
    }
    if address.starts_with(NO_PROTOCOL) {
        return Some(CannotBeARemote::NoProtocolName);
    }
    return None;
}

/// What:
///  The first of a push's addresses that can never be a remote,
/// with why,
/// or nothing when every one of them might work.
/// Why:
///   A remote may name more than one push address,
/// and Git stops the push at the first that fails
/// (measured with Git 2.56.0:
/// `remote.origin.pushurl` twice,
/// `DISABLED` first,
/// and the second,
/// working address received nothing),
/// so one address that can never be a remote is enough for no try to land.
/// An empty list names no address,
/// which is a push whose destination this release could not read:
/// nothing is known,
/// so it keeps its retries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// firstThatCannotBeRemote(['DISABLED', '/srv/real.git']) // ['DISABLED', 'the-reserved-word']
/// ```
pub fn first_that_cannot_be_remote(addresses: &[String]) -> Option<(&str, CannotBeARemote)> {
    for address in addresses {
        if let Some(reason) = cannot_be_a_remote(address.as_str()) {
            return Some((address.as_str(), reason));
        }
    }
    return None;
}

/// What:
///  An address as this program prints it:
///  a URL's user information replaced by `***`,
///  because it may hold a password.
/// Why:
///   Git masks the password of a URL in its own messages,
/// and a line this program prints must not be the one that leaks one.
/// An address with no `scheme://` and no `@` in its authority is printed as it is,
/// and so is a path that merely holds an `@`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// asAPersonReadsIt('https://user:secret@host/x') // 'https://***@host/x'
/// ```
fn as_a_person_reads_it(address: &str) -> String {
    let Some(scheme_end) = address.find(NO_PROTOCOL) else { return address.to_owned() };
    let authority_start: usize = scheme_end + NO_PROTOCOL.len();
    let after_the_authority_start: &str = &address[authority_start..];
    let authority_end: usize = match after_the_authority_start.find('/') {
        Some(slash) => authority_start + slash,
        None => address.len(),
    };
    let authority: &str = &address[authority_start..authority_end];
    let Some(at) = authority.rfind('@') else { return address.to_owned() };
    return format!("{}***@{}", &address[..authority_start], &address[authority_start + at + 1..]);
}

/// What:
///  The one line the person reads for a push whose address can never be a remote.
/// Why:
///   It says what was not done,
/// why it can never be done,
/// and that the service has stopped at once,
/// and it says where the commit is and the two ways to send it.
/// The words are one line because the notice of a failed push already names the branch and the commit above them,
/// and a person scanning a terminal reads the first line only.
/// The address is printed only for the two shapes that are not self-describing,
/// and every address printed is masked of the user information a URL may hold
/// (`as_a_person_reads_it`);
/// an address whose shape this file cannot classify never reaches this function at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// words('DISABLED', CannotBeARemote.TheReservedWord) // 'monochromatic-git-policy-cli: ...'
/// ```
pub fn words(address: &str, reason: CannotBeARemote) -> String {
    let read: String = as_a_person_reads_it(address);
    let why: String = match reason {
        CannotBeARemote::Empty => String::from("the commit service could not push: the remote's push address is empty, and an empty address can never be a repository"),
        CannotBeARemote::TheReservedWord => format!(
            "the commit service did not push: the remote's push address is `{DISABLED_ADDRESS}`, the word this project uses to mean that nothing is pushed, and that address can never be a repository"
        ),
        CannotBeARemote::AnOption => format!(
            "the commit service did not push: the remote's push address `{read}` starts with a dash, so Git reads it as an option and never as a place to push to"
        ),
        CannotBeARemote::NoProtocolName => format!(
            "the commit service did not push: the remote's push address `{read}` begins with `{NO_PROTOCOL}`, so it names no protocol, and no protocol has an empty name"
        ),
    };
    return format!(
        "{why}. No try can land it, so the commit service stopped trying at once, and this is the one notice of it. The commit is saved in \
         this repository: set the remote's push address to one a push can reach (`git remote set-url --push <remote> <address>`) and commit \
         again, or push by hand."
    );
}

/// The readings above are controlled beside them.
#[cfg(test)]
#[path = "push_address_that_cannot_be_a_remote_tests.rs"]
mod tests;
