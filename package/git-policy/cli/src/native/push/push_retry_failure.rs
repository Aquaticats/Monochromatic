//! What:
//!  What a failed push's own words mean for the service's retry:
//! whether the failure can pass at all,
//! how many tries the one cause that
//!  is counted gets,
//! and the words the person's failure record carries.
//! Why:
//!  The retry plan and its pauses are `push_retry.rs`;
//! this file is the reading of one push's output,
//! which
//!  `service_work.rs` asks for the verdict of a push and the drain asks for the ending of a counted one.
//! Every rule here
//!  cites the words it reads,
//! so a release that meets a new cause can add it in one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cause = finalFailure(output); if (outOfTries(output, tryNumber)) fail(outOfTriesWords(first));
//! ```

/// The pause schedule of the plan,
/// which the words of a counted failure measure their span with.
use crate::push_retry::pause_before;
use std::time::Duration;

/// The words of the reconcile's report of a conflict,
/// the one failure that retrying cannot mend.
pub const CONFLICT_PHRASE: &str = " conflicts in: ";

/// Why a failed push cannot pass by itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalFailure {
    /// The reconcile reported a conflict.
    Conflict,
    /// The remote asked for a login the service does not have.
    Login,
    /// A hook on the server declined the push.
    Declined,
}

/// The mark of a line in which the server rejected one reference;
/// Git prints it for every refused reference
/// (`transport.c`,
/// `print_ref_status('!', "[remote rejected]", ...)`,
/// Git 2.56.0).
pub const REMOTE_REJECTED: &str = "[remote rejected]";

/// The words every hook decline of Git's own server side carries in the rejection's reason:
/// `"pre-receive hook declined"` and
/// `"push-to-checkout hook declined"` and `"hook declined"` (the update hook) in `builtin/receive-pack.c`,
/// Git 2.56.0;
/// a hosting
/// service's own reason that names a hook,
/// such as `protected branch hook declined`,
/// ends the same way.
/// The reason alone says nothing about which rule of the server refused the push,
/// and GitHub writes that same
///  reason for a protected-branch refusal it answers with `PROTECTED_BRANCH_CODE`,
/// which is retried;
/// a decline without that code stays final.
pub const HOOK_DECLINED: &str = "hook declined";

/// What:
///  The code GitHub's own server puts in the message of a push it refused because the update met the branch's
///  protection rules,
/// with the reference it refused.
/// Why:
///   The code is the only part of the refusal that says which rule answered:
/// the reason beside it is
///  `protected branch hook declined`,
/// which the server writes for any of its own rules,
/// and the words after the code were empty in every refusal this repository recorded
///  (`doc/troubleshooting/github-protected-branch-refusal-that-cleared.md`).
/// It is looked for anywhere in the push's output,
/// because a server's own message reaches the client as a `remote:` line
///  of its own,
/// apart from the rejection line Git prints.
pub const PROTECTED_BRANCH_CODE: &str = "GH006";

/// What:
///  How many tries in all the service gives a push the server refuses with `PROTECTED_BRANCH_CODE`,
/// counting the first one,
/// before it stops and asks the person.
/// Why:
///   Such a refusal can clear by itself and can also refuse every try,
/// so it is neither final nor retried for ever.
/// Citation:
///  the three refusals recorded in `doc/troubleshooting/github-protected-branch-refusal-that-cleared.md`:
/// the
///  push of `29bae9bb87` at `2026-11-09T23:30:11-05:00`,
/// which a plain push landed moments later;
/// the push of `1535eab9bf` at
///  `2026-11-10T02:24:59-05:00`,
/// every commit of it signed,
/// landed by hand minutes later;
/// and the auto-push of `main` at `2026-11-11T06:42-05:00`,
///  which a plain push landed two minutes later.
/// With the default first pause of `FIRST_PAUSE` the pauses before these five tries add up to 150 seconds,
/// so the
///  last try comes after the two minutes a recorded refusal took to clear,
/// with one try to spare;
/// and a branch whose rules refuse every push ends after about two and a half
///  minutes instead of never.
/// This figure counts tries and not pauses,
/// so a configured `RETRY_VARIABLE` moves the span and never the count.
pub const PROTECTED_BRANCH_TRIES: u32 = 5;

/// The words with which Git and the programs it runs say that the remote wanted a login the push did not have,
/// each from a
/// fixture run of `git push` from Git 2.56.0 on 2026-11-03:
/// - `remote-curl.c` `die(_("Authentication failed for '%s'"))`:
///   a 401 and no usable password;
/// - `prompt.c` `die("could not read %s%s", prompt, "terminal prompts disabled")`:
///   a 401 with no user name and no prompt allowed;
/// - `http.c` `"The requested URL returned error: %ld"` with 401 or 403:
///   the server refused the login or the permission;
/// - OpenSSH's `Permission denied (publickey).`,
///   which Git relays when the key is refused.
pub const LOGIN_PHRASES: [&str; 5] = ["Authentication failed for", "terminal prompts disabled", "returned error: 401", "returned error: 403", "Permission denied ("];

/// The lines of an output in which the server rejected a reference.
fn rejection_lines(output: &str) -> Vec<&str> {
    let mut rejected: Vec<&str> = Vec::new();
    for line in output.lines() {
        let trimmed: &str = line.trim();
        if trimmed.contains(REMOTE_REJECTED) {
            rejected.push(trimmed);
        }
    }
    return rejected;
}

/// The line in which the server refused the push because the update met its protected-branch rules,
/// or nothing.
pub fn protected_branch_refusal(output: &str) -> Option<&str> {
    for line in output.lines() {
        if line.contains(PROTECTED_BRANCH_CODE) {
            return Some(line.trim());
        }
    }
    return None;
}

/// What:
///  The failure of a push that no retry can mend,
/// from Git's own words;
/// nothing for a failure that can pass (a stall,
/// the
///  network,
/// a held lock,
/// the server's protected-branch refusal) and for a rejection whose reason this release does not know.
/// Why:
///   A hook decline with no protected-branch code is the server's rule as this release has always read it,
/// and the
///  owner's decision of 2026-11-03 keeps it out of the retry plan;
/// GitHub's protected-branch refusal says which rule answered,
/// and it is
///  retried within `PROTECTED_BRANCH_TRIES`,
/// so it is not read as final here.
pub fn final_failure(output: &str) -> Option<FinalFailure> {
    if output.contains(CONFLICT_PHRASE) {
        return Some(FinalFailure::Conflict);
    }
    let protected: bool = protected_branch_refusal(output).is_some();
    for line in rejection_lines(output) {
        if !protected && line.contains(HOOK_DECLINED) {
            return Some(FinalFailure::Declined);
        }
    }
    for phrase in LOGIN_PHRASES {
        if output.contains(phrase) {
            return Some(FinalFailure::Login);
        }
    }
    return None;
}

/// What:
///  Whether the push that failed with these words has used up every try the service gives it,
/// `try_number` being the number of the try that just ended
///  (the push of a new commit is one).
/// Why:
///   The owner asked on 2026-11-11 that the wrapper retry GitHub's `GH006`,
/// and the standing rules "nothing should ever wait forever or indefinitely" and "Commits should never fail"
/// pull against
///  each other,
/// so the retry is bounded:
/// a refusal that keeps coming back is the person's to fix,
/// and this is where the service stops
///  trying it.
/// Every other retried failure is tried until it passes and has no count of its own,
/// so the answer depends on the words
///  and not on the count alone.
pub fn out_of_tries(output: &str, try_number: u32) -> bool {
    return protected_branch_refusal(output).is_some() && try_number >= PROTECTED_BRANCH_TRIES;
}

/// The first line in which the server rejected the push for a reason this release does not know,
/// or nothing:
/// a push with such a
/// line stays retried,
/// and the journal names the line so that the reason can be learned.
pub fn unrecognised_rejection(output: &str) -> Option<String> {
    // A refusal the release knows is named as what it is, whether or not it is final.
    if final_failure(output).is_some() || protected_branch_refusal(output).is_some() {
        return None;
    }
    let rejected: Vec<&str> = rejection_lines(output);
    if let Some(line) = rejected.first() {
        return Some((*line).to_string());
    }
    return None;
}

/// What:
///  The plain words the person's failure record starts with for a failure that stops the retry,
/// or nothing for a conflict,
/// whose
///  report already says what to do.
/// Why:
///   The owner's decision is that a login failure and a server hook's decline stop until the next commit
///  ("Offline is retried,
/// but auth and server hook declines stop until your next commit.");
/// the person is told which one it was,
///  in words,
/// above Git's own.
///  A decline is the server's answer at that moment and nothing about a later try,
/// so its words promise no outcome and say that a push by hand may succeed
///  (GitHub refused a push of `main` with `GH006` at `2026-11-09T23:30:11-05:00` and a plain `git push` of the same commit then succeeded:
/// `doc/troubleshooting/github-protected-branch-refusal-that-cleared.md`).
/// A protected-branch refusal is not worded here:
/// the service retries it (`PROTECTED_BRANCH_TRIES`),
/// and its words come from `out_of_tries_words` when the tries run out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plainWords(cause) // 'monochromatic-git-policy-cli: the remote wants a login ...'
/// ```
pub fn plain_words(cause: FinalFailure) -> Option<&'static str> {
    return match cause {
        FinalFailure::Conflict => None,
        FinalFailure::Login => Some(
            "monochromatic-git-policy-cli: the remote refused this push because it wants a login that the commit service does not have, or because the login it has lacks \
             permission, so the commit service stops pushing this branch until your next commit. Log in in a way the service can use (a credential \
             helper, or an SSH key without a passphrase that its environment reaches) and make sure that login may push, then push by hand or commit again.",
        ),
        FinalFailure::Declined => Some(
            "monochromatic-git-policy-cli: the server refused this push: a hook on the server (a check the server runs before it accepts a push) declined it, with the message shown below. The commit service stops pushing this branch \
             until your next commit. A push by hand may succeed if the cause has cleared: fix what the message names, or wait, then push by hand or commit again.",
        ),
    };
}

/// Whether a failed push's output says something stopped it that only the person can resolve.
pub fn needs_the_person(output: &str) -> bool {
    return final_failure(output).is_some();
}

/// One count and the unit that follows it.
fn count_words(count: u64, unit: &str) -> String {
    return if count == 1 { format!("1 {unit}") } else { format!("{count} {unit}s") };
}

/// What:
///  A span of time as a person reads it:
/// whole minutes and seconds,
/// seconds alone below a minute,
/// and "less than a second" for a span of whole seconds that rounds to none.
/// Why:
///   The figure the words carry is a value a person compares with the wait they made,
/// so it is said in the units they count in
///  and never as a number of milliseconds.
fn span_words(span: Duration) -> String {
    let seconds: u64 = span.as_secs();
    if seconds == 0 {
        return String::from("less than a second");
    }
    let minutes: u64 = seconds / 60;
    let rest: u64 = seconds % 60;
    if minutes == 0 {
        return count_words(rest, "second");
    }
    if rest == 0 {
        return count_words(minutes, "minute");
    }
    return format!("{} and {}", count_words(minutes, "minute"), count_words(rest, "second"));
}


/// The pauses the plan asks between the tries of a failure it gives `PROTECTED_BRANCH_TRIES` of:
/// the first pause and each doubling after it,
/// with none after the last try.
fn pauses_before_the_last_try(first: Duration) -> Duration {
    let mut span: Duration = Duration::ZERO;
    for try_number in 1..PROTECTED_BRANCH_TRIES {
        span = span.saturating_add(pause_before(try_number, first));
    }
    return span;
}


/// What:
///  The plain words the person's failure record starts with when a protected-branch refusal outlasted every try
///  `first` is the pause before the first new try.
/// Why:
///   The person is told what happened,
/// that the service tried it again,
/// how long the tries covered,
/// and what to do,
/// and
///  GitHub's own words follow them in the record;
/// the figure for the time is the pauses the plan asked between those tries,
/// because that is what
///  the service knows,
/// and the push runs themselves are measured by nobody.
/// The words name the two shapes the refusal has:
/// a branch rule that
///  refuses every try,
/// which needs the commit or the rule changed,
/// and a delay of the server's own,
/// which a plain push may pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outOfTriesWords(firstPauseMs) // 'monochromatic-git-policy-cli: ... tried it 5 times, over 2 minutes and 30 seconds, ...'
/// ```
pub fn out_of_tries_words(first: Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: the server refused this push as an update to a protected branch. The commit service tried it {PROTECTED_BRANCH_TRIES} times, over {span}, and has stopped trying this branch. \
         What the server said on the last try is below. \
         A rule of the branch that refuses this commit every time (a branch that requires signed commits refuses a commit that is not signed) needs the commit or the rule changed: fix what the message names, then commit again. \
         When nothing in the message names something you can change, the refusal was the server's answer at that moment, and running `git push` by hand may land this commit unchanged.",
        span = span_words(pauses_before_the_last_try(first))
    );
}

/// The reading of a failed push's words is controlled beside it.
#[cfg(test)]
#[path = "push_retry_failure_tests.rs"]
mod tests;
