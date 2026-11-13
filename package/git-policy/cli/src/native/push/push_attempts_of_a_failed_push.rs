//! What:
//!  What one failed push did,
//! address by address:
//!  the addresses Git printed a `To` line for and what became of the recorded ref at each,
//!  whether the recorded ref arrived anywhere,
//!  and the address Git's own failure sentence names.
//! Why:
//!   A remote may name several push addresses,
//! and Git pushes to each in order and stops at the first that fails
//! (measured with Git 2.56.0 on 2026-11-13,
//! and quoted in the controls of this module):
//! with `[<a repository>, DISABLED]` the repository receives the commit and Git still exits 128 on `DISABLED`;
//! with `[<a host that does not answer>, DISABLED]` Git stops at the host and never attempts `DISABLED`;
//! and with a repository that already holds the commit Git prints `Everything up-to-date` and no `To` line.
//! So neither "the push failed" nor "an address is one that can never be a remote" says what happened:
//!  - a verdict that ends the plan because *an* address can never be a remote ends it for a push that was
//!    delivered,
//!    and tells the person the commit is only on this computer while a remote holds it;
//!  - a verdict that blames the address in the configuration rather than the one Git failed at blames an
//!    address Git never attempted,
//!    and gives up a backup a retry could complete.
//! What Git printed decides,
//!    and nothing is trimmed or normalised on the way:
//! the address of a `To` line is the rest of that line byte for byte,
//! so `DISABLED ` (a legal path whose one difference from the reserved word is a space) is not the reserved
//! word (the review's finding 4).
//! Nothing here is a decision:
//! `service_work_failed_push_verdict.rs` ends a plan or keeps it from these facts,
//! and the reconcile moves the branch to the merge it published from them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! failedPushByAddress({ output, destinationRef, configured }) // { attempted, delivered, failing }
//! ```

/// What:
///  One address Git printed a `To` line for.
/// Why:
///   The lane is the only place Git says which address it is about to push to,
/// and the ref's status under it is the only place it says whether the ref arrived there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttemptedAddress = { address: string; arrived: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptedAddress {
    /// The address as Git printed it,
    /// byte for byte,
    /// with neither leading nor trailing space removed.
    pub address: String,
    /// Whether the recorded ref arrived at this address,
    /// which a sent or up-to-date status line says and a refusal does not.
    pub arrived: bool,
}

/// What:
///  What a failed push did,
///  address by address.
/// Why:
///   The three facts answer the three questions a verdict asks:
/// which addresses were reached (the report for the person),
/// whether the ref arrived anywhere (is the branch backed up),
/// and
/// which address Git failed at (is that address one that can never be a remote).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FailedPushByAddress = { attempted; arrived; failing };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailedPushByAddress {
    /// Every address Git printed a `To` line for,
    /// in the order it attempted them.
    pub attempted: Vec<AttemptedAddress>,
    /// Whether the recorded ref arrived at any address,
    /// by a status line or by `Everything up-to-date`.
    pub arrived: bool,
    /// The address Git's own failure sentence names,
    /// as the configuration spells it,
    /// or nothing when no sentence names one of the configured addresses.
    pub failing: Option<String>,
}

impl FailedPushByAddress {
    /// The addresses that hold the recorded ref,
    /// in the order Git attempted them.
    pub fn arrived_at(&self) -> Vec<&str> {
        // What: One pass over the lanes Git printed, keeping the address of each that took the ref.
        // Why:  A chain of adapters here is a closure the package linter can name a function for, and a loop says the
        //       same thing without one.
        let mut holders: Vec<&str> = Vec::new();
        for lane in &self.attempted {
            if lane.arrived {
                holders.push(lane.address.as_str());
            }
        }
        return holders;
    }
}

/// What:
///  Whether the recorded ref arrived at some address of this failed push.
/// Why:
///   The reconcile asks only this:
/// a merge it published has to move the local branch,
/// and Git's exit status is the worst fate of the addresses,
/// so it says nothing about the merge's own delivery.
/// A push that printed nothing this reading understands reports no arrival,
/// which is the safe side:
/// no branch is moved on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// arrivedSomewhere(output, destinationRef) // failedPushByAddress({ output, destinationRef, configured: [] }).arrived
/// ```
pub fn arrived_somewhere(output: &str, destination_ref: &str) -> bool {
    return read(output, destination_ref).1;
}

/// What:
///  Everything the reading of one failed push gives.
/// Why:
///   The three facts are one pass over Git's output,
/// so the verdict cannot read a push two ways.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// failedPushByAddress({ output, destinationRef, configured })
/// ```
pub fn failed_push_by_address(output: &str, destination_ref: &str, configured: &[String]) -> FailedPushByAddress {
    let (attempted, arrived): (Vec<AttemptedAddress>, bool) = read(output, destination_ref);
    let failing: Option<String> = address_git_failed_at(output, configured);
    return FailedPushByAddress { attempted, arrived, failing };
}

/// What:
///  The attempts Git printed and whether the recorded ref arrived.
/// Why:
///   One pass over the lines of Git's output,
/// because a `To` line opens the lane the status lines below it belong to.
/// The rules are the shapes Git 2.56.0 printed for each case this release can meet,
/// and every shape this reading does not recognise counts as no arrival.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = (output, ref) => { let current; for (const line of output.split('\n')) { ... } }
/// ```
fn read(output: &str, destination_ref: &str) -> (Vec<AttemptedAddress>, bool) {
    let destination: &str = short_name(destination_ref);
    let mut attempted: Vec<AttemptedAddress> = Vec::new();
    let mut arrived: bool = false;
    for line in output.lines() {
        if let Some(address) = line.strip_prefix(DESTINATION_LINE_PREFIX) {
            attempted.push(AttemptedAddress { address: address.to_owned(), arrived: false });
            continue;
        }
        if line == NOTHING_TO_SEND_LINE {
            // Git prints this for every push URL that already holds the ref, and prints no `To` line for it.
            arrived = true;
            continue;
        }
        let Some(refused_or_sent) = status_of(line) else { continue };
        if destination_of(line).as_deref() != Some(destination) {
            continue;
        }
        if refused_or_sent == Status::Arrived {
            arrived = true;
            if let Some(lane) = attempted.last_mut() {
                lane.arrived = true;
            }
        }
    }
    return (attempted, arrived);
}

/// What happened to the recorded ref at one address,
/// from the marker of its status line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    /// The ref was written there,
    /// or was already there.
    Arrived,
    /// The remote,
    /// or Git itself,
    /// refused to write the ref there.
    Refused,
}

/// What:
///  The fate of one ref at one address,
/// from the line that reports it.
/// Why:
///   Git prints one such line per ref and address,
/// with the fate in the second column and a space before it.
/// The measured bytes of Git 2.56.0 (`probe-git-push-status-lines.ts` beside this run's report):
///
/// ```text
/// " * [new branch]      main -> feature"
/// "   b185c99..4eb74e0  main -> main"
/// " + 4eb74e0...19fd10c main -> main (forced update)"
/// " ! [rejected]        main -> main (fetch first)"
/// " ! [remote rejected] main -> main (pre-receive hook declined)"
/// ```
///
/// So the fate is the character after the first,
/// and a line whose second character is not one of them
/// (a `fatal:`,
/// a `hint:`,
/// a `remote:` from a hook,
/// the `To` line itself) answers nothing.
/// Reading the first character instead made `! [rejected]` an arrival,
/// which reported a refused ref as delivered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const statusOf = (line) => { const at = line[0] === ' ' ? 1 : 0; const marker = line[at]; return ' +-*!=X'.includes(marker) && line.includes(' -> ') ? (marker === '!' || marker === 'X' ? 'refused' : 'arrived') : undefined; };
/// ```
fn status_of(line: &str) -> Option<Status> {
    let mut characters = line.chars();
    let first: char = characters.next()?;
    // The fate stands one column after the leading space Git prints before it.
    let marker: char = if first == ' ' { characters.next()? } else { first };
    if !STATUS_MARKERS.contains(&marker) {
        return None;
    }
    if !line.contains(REFERENCE_ARROW) {
        return None;
    }
    return Some(if REFUSAL_MARKERS.contains(&marker) { Status::Refused } else { Status::Arrived });
}

/// What:
///  The ref a status line reports,
///  as the short name Git prints it.
/// Why:
///   The destination is the last field of the line,
/// and a refusal appends its reason in parentheses after it.
/// A branch name holds no space,
/// so the short name is what stands between the arrow and the first space after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const destinationOf = (line) => line.slice(line.lastIndexOf(' -> ') + 4).split(' ')[0];
/// ```
fn destination_of(line: &str) -> Option<String> {
    let after: &str = line.rsplit_once(REFERENCE_ARROW)?.1;
    let name: &str = after.split(' ').next()?;
    return (!name.is_empty()).then_some(name.to_owned());
}

/// The branch name Git prints for a ref:
/// a branch's short name,
/// and any other ref as its whole name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shortName = (ref) => ref.startsWith('refs/heads/') ? ref.slice('refs/heads/'.length) : ref;
/// ```
fn short_name(destination_ref: &str) -> &str {
    return destination_ref.strip_prefix("refs/heads/").unwrap_or(destination_ref);
}

/// What:
///  The configured push address Git's own failure sentence names,
///  or nothing.
/// Why:
///  Every shape Git fails with names the address it failed at,
/// between single quotes:
/// `fatal: '<address>' does not appear to be a git repository`,
/// `fatal: unable to access '<address>/': <curl's reason>`,
/// and `error: failed to push some refs to '<address>'`.
/// The address is compared exactly as the configuration spells it,
/// so a trailing space inside the quotes is the address's own.
/// The URL case adds one trailing slash to the URL,
/// so one trailing slash is removed before comparing,
/// and nothing else is.
/// A sentence that names no configured address leaves nothing known,
/// which never ends a plan and never blames an address.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failing = (output, configured) => output.split('\n').reverse().filter((l) => /^(fatal|error): /.test(l)).map(quoted).flatMap((q) => configured.includes(q) ? [q] : configured.includes(q.slice(0, -1)) && q.endsWith('/') ? [q.slice(0, -1)] : [])[0];
/// ```
fn address_git_failed_at(output: &str, configured: &[String]) -> Option<String> {
    for line in output.lines().rev() {
        if !(line.starts_with("fatal: ") || line.starts_with("error: ")) {
            continue;
        }
        let Some(named) = quoted(line) else { continue };
        for address in configured {
            if named == *address {
                return Some(address.clone());
            }
            // `unable to access '<url>/'` appends the path Git asks for; the URL itself is the address.
            if named.ends_with('/') && named.strip_suffix('/') == Some(address.as_str()) {
                return Some(address.clone());
            }
        }
    }
    return None;
}

/// What:
///  The run between the first and the last single quote of a line,
///  which is the address Git's sentence names.
/// Why:
///   Git quotes the address in every failure that names one,
/// and a line with no run is no sentence about an address.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quoted = (line) => { const first = line.indexOf("'"); const last = line.lastIndexOf("'"); return first >= 0 && last > first ? line.slice(first + 1, last) : undefined; }
/// ```
fn quoted(line: &str) -> Option<&str> {
    let first: usize = line.find('\'')?;
    let last: usize = line.rfind('\'')?;
    if last <= first {
        return None;
    }
    return Some(&line[first + 1..last]);
}

/// The prefix of the line that opens one address's lane in Git's report.
const DESTINATION_LINE_PREFIX: &str = "To ";

/// What Git prints instead of a lane for an address that already holds the ref.
const NOTHING_TO_SEND_LINE: &str = "Everything up-to-date";

/// What separates the ref a status line reports from the one it writes.
const REFERENCE_ARROW: &str = " -> ";

/// The fates a ref's status line reports,
/// in the column after its leading space:
/// a space for an update,
/// `+` for a forced one,
/// `-` for a deletion,
/// `*` for a new ref,
/// `=` for one that was up to date,
/// and `!` or `X` for a refusal.
const STATUS_MARKERS: [char; 7] = [' ', '+', '-', '*', '=', '!', 'X'];

/// The markers of the lines that report a refusal.
const REFUSAL_MARKERS: [char; 2] = ['!', 'X'];

/// The reading above is controlled beside it.
#[cfg(test)]
#[path = "push_attempts_of_a_failed_push_tests.rs"]
mod tests;
