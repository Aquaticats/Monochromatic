//! Controls of the verdict of one refused push:
//! what the push reached decides that its item is done,
//! and what Git said at the address that failed decides whether a plan keeps trying,
//! ends at once because no retry can mend it,
//! or ends because that address can never be a remote.
//! Why:
//!  Every output below is a shape Git 2.56.0 printed for that push on 2026-11-13,
//! measured with the probes beside the report of this run,
//! so a Git release that changes one breaks a control here rather than a person's notice.

use super::*;
/// The reading of one push's output, which the handlers make and this file makes the same way.
use crate::push_attempts_of_a_failed_push::failed_push_by_address;
use crate::auto_push::AutoPushOutcome;

/// The addresses of the two-remote cases:
/// one that receives,
/// then one that answers.
const TWO_ADDRESSES: [&str; 2] = ["/srv/receives.git", "/srv/answers.git"];

/// The client of a push that exits `exit_code` with `shown` from a remote with `addresses`.
fn failed(exit_code: i64, shown: &str, addresses: &[&str]) -> AutoPushResult {
    let configured: Vec<String> = addresses.iter().map(|address| { return (*address).to_owned(); }).collect();
    return AutoPushResult { outcome: AutoPushOutcome::Failed, exit_code, shown: shown.to_owned(), push_addresses: configured };
}

/// What:
///  What one push's output says about the addresses it reached,
///  read the way the handlers read it.
/// Why:
///   The verdict is handed the reading rather than making it,
///  so the handlers read a push's addresses once for the branch-gone question and once for the verdict's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const read = failedPushByAddress({ output, destinationRef: 'refs/heads/main', configured: addresses })
/// ```
fn read(output: &str, addresses: &[&str]) -> FailedPushByAddress {
    let configured: Vec<String> = addresses.iter().map(|address| { return (*address).to_owned(); }).collect();
    return failed_push_by_address(output, "refs/heads/main", configured.as_slice());
}

/// What:
///  A delivery to the first address followed by a refusal at the second.
/// Why:
///  This is the shape the second review's finding 2 was measured on:
/// `To /srv/receives.git` with `* [new branch]`,
/// then the hook's own line,
/// then the refusal at the second address.
const DELIVERED_THEN_A_HOOK_DECLINED: &str = "To /srv/receives.git\n * [new branch]      main -> main\nremote: declined        \n\
To /srv/answers.git\n ! [remote rejected] main -> main (pre-receive hook declined)\nerror: failed to push some refs to '/srv/answers.git'\n";

/// A delivery to the first address followed by a login the second one wants.
const DELIVERED_THEN_A_LOGIN: &str = "To /srv/receives.git\n * [new branch]      main -> main\nremote: login needed\n\
fatal: Authentication failed for 'http://127.0.0.1:39197/r.git/'\n";

/// A delivery to the first address followed by a host that does not answer.
const DELIVERED_THEN_A_HOST_THAT_DOES_NOT_ANSWER: &str = "To /srv/receives.git\n * [new branch]      main -> main\n\
fatal: unable to access 'https://unreachable-host.invalid/repo.git/': Could not resolve host: unreachable-host.invalid\n";

/// A hook that declined the only address,
/// with nothing delivered.
const NOTHING_DELIVERED_AND_A_HOOK_DECLINED: &str =
    "To /srv/answers.git\n ! [remote rejected] main -> main (pre-receive hook declined)\nerror: failed to push some refs to '/srv/answers.git'\n";

/// What:
///  A hook that declines the second address ends the plan after a delivery,
/// with one report.
/// Why:
///  The second review of issue 6293 (finding 2) measured the opposite:
/// every round delivered nothing new to the first address,
/// was declined at the second,
/// and planned another try,
/// with pauses growing to the longest pause and no last one.
/// A decline is a failure no retry can mend (`push_retry_failure.rs`),
/// so the plan ends,
/// and the person reads where
/// the commit is and which address refused it.
#[test]
fn a_decline_at_the_second_address_after_a_delivery_ends_the_plan_with_one_report() {
    let settled: SettledFailedPush =
        verdict_of_a_failed_push(failed(1, DELIVERED_THEN_A_HOOK_DECLINED, &TWO_ADDRESSES), read(DELIVERED_THEN_A_HOOK_DECLINED, &TWO_ADDRESSES), "refs/heads/main", Path::new("/srv/repo"), false);
    let SettledFailedPush::Reached(said) = settled else { panic!("a branch that reached an address is done, not {settled:?}") };
    assert!(said.contains("/srv/receives.git"), "the words name the address that holds the commit: {said}");
    assert!(said.contains("/srv/answers.git"), "and the address that refused it: {said}");
    assert!(said.contains("is not trying that branch through it again"), "and promise no retry of it: {said}");
    assert!(said.contains("a hook on the server"), "the cause is the words this release uses for a decline: {said}");
    assert!(said.contains("pre-receive hook declined"), "and Git's own words follow: {said}");
}

/// What:
///  A login the second address wants ends the plan after a delivery,
/// with one report.
/// Why:
///  The same reading decides it:
/// the commit is on the first address,
/// and retrying asks the second for a login again.
#[test]
fn a_login_at_the_second_address_after_a_delivery_ends_the_plan_with_one_report() {
    let settled: SettledFailedPush =
        verdict_of_a_failed_push(
        failed(128, DELIVERED_THEN_A_LOGIN, &["/srv/receives.git", "http://127.0.0.1:39197/r.git"]),
        read(DELIVERED_THEN_A_LOGIN, &["/srv/receives.git", "http://127.0.0.1:39197/r.git"]),
        "refs/heads/main",
        Path::new("/srv/repo"),
        false,
    );
    let SettledFailedPush::Reached(said) = settled else { panic!("a branch that reached an address is done, not {settled:?}") };
    assert!(said.contains("the login it has lacks permission") || said.contains("wants a login"), "the cause is a login: {said}");
    assert!(said.contains("http://127.0.0.1:39197/r.git"), "the address Git named, with the slash Git appends removed: {said}");
}

/// What:
///  A failure at the second address that may pass keeps the plan.
/// Why:
///  The other side of the rule,
/// from the first brief:
/// a push that might come back is never given up.
/// A host that does not answer is one of those,
/// so the plan is kept and the record promises the retry.
#[test]
fn a_host_that_does_not_answer_at_the_second_address_keeps_the_plan() {
    let settled: SettledFailedPush = verdict_of_a_failed_push(
        failed(128, DELIVERED_THEN_A_HOST_THAT_DOES_NOT_ANSWER, &["/srv/receives.git", "https://unreachable-host.invalid/repo.git"]),
        read(DELIVERED_THEN_A_HOST_THAT_DOES_NOT_ANSWER, &["/srv/receives.git", "https://unreachable-host.invalid/repo.git"]),
        "refs/heads/main",
        Path::new("/srv/repo"),
        false,
    );
    let SettledFailedPush::Unreached(Verdict::Retry(said)) = settled else { panic!("a failure that may pass is retried, not {settled:?}") };
    assert!(said.contains("/srv/receives.git"), "the retry's words say where the commit is: {said}");
    assert!(said.contains("https://unreachable-host.invalid/repo.git"), "and which address failed: {said}");
}

/// What:
///  A decline at the second address ends the plan with the branch-gone sentence when the branch was deleted while the
///  push ran.
/// Why:
///  The third review of issue 6293 (finding 3): the record is the last thing said about this push, and a person who is
/// not told the branch is gone waits for a retry that will not come.
#[test]
fn a_decline_at_the_second_address_with_the_branch_gone_says_so_and_tries_nothing_again() {
    let settled: SettledFailedPush = verdict_of_a_failed_push(
        failed(1, DELIVERED_THEN_A_HOOK_DECLINED, &TWO_ADDRESSES),
        read(DELIVERED_THEN_A_HOOK_DECLINED, &TWO_ADDRESSES),
        "refs/heads/main",
        Path::new("/srv/repo"),
        true,
    );
    let SettledFailedPush::Reached(said) = settled else { panic!("a branch that reached an address is done, not {settled:?}") };
    assert!(said.contains("the branch main no longer exists in this repository"), "the words say the branch is gone: {said}");
    assert!(said.contains("nothing is tried again for it"), "and that nothing is tried again: {said}");
    assert!(said.contains("/srv/receives.git"), "the words still name where the commit is: {said}");
    assert!(!said.contains("was not made"), "and never say the push was not made: {said}");
    assert_eq!(said.matches("pre-receive hook declined").count(), 1, "Git's own output is printed once: {said}");
}

/// What:
///  A failure at the second address that may pass keeps the plan only while the branch is there to push.
/// Why:
///  The other side of the rule: a deleted branch ends every further attempt, but the delivery is still reported, so the
///  answer is a record and not a retry.
#[test]
fn a_failure_that_may_pass_at_the_second_address_ends_when_the_branch_is_gone() {
    let addresses: [&str; 2] = ["/srv/receives.git", "https://unreachable-host.invalid/repo.git"];
    let settled: SettledFailedPush = verdict_of_a_failed_push(
        failed(128, DELIVERED_THEN_A_HOST_THAT_DOES_NOT_ANSWER, &addresses),
        read(DELIVERED_THEN_A_HOST_THAT_DOES_NOT_ANSWER, &addresses),
        "refs/heads/main",
        Path::new("/srv/repo"),
        true,
    );
    let SettledFailedPush::Reached(said) = settled else { panic!("with the branch gone nothing is retried, not {settled:?}") };
    assert!(said.contains("the branch main no longer exists in this repository"), "the words say the branch is gone: {said}");
    assert!(said.contains("https://unreachable-host.invalid/repo.git"), "and name the address that failed: {said}");
}

/// What:
///  A decline at the only address,
/// with nothing delivered,
/// still settles the push as failed once.
/// Why:
///  This is the classification the release has always used,
/// and the change must not have moved it:
/// nothing reached a remote,
/// and a decline needs the person.
#[test]
fn a_decline_with_nothing_delivered_is_failed_once() {
    let settled: SettledFailedPush =
        verdict_of_a_failed_push(
        failed(1, NOTHING_DELIVERED_AND_A_HOOK_DECLINED, &["/srv/answers.git"]),
        read(NOTHING_DELIVERED_AND_A_HOOK_DECLINED, &["/srv/answers.git"]),
        "refs/heads/main",
        Path::new("/srv/repo"),
        false,
    );
    let SettledFailedPush::Unreached(Verdict::Failed(said)) = settled else { panic!("a decline that delivered nothing fails once, not {settled:?}") };
    assert!(said.contains("a hook on the server"), "the cause is named: {said}");
}

/// What:
///  A push that reached an address and failed at one that can never be a remote is done,
/// with the delivery line.
/// Why:
///  The first review's finding 2,
/// kept:
/// the commit is on a remote,
/// and that address is given up for itself.
#[test]
fn a_delivery_before_the_reserved_word_is_done_with_the_delivery_line() {
    let delivered: &str = "To /srv/receives.git\n * [new branch]      main -> main\nfatal: 'DISABLED' does not appear to be a git repository\n";
    let settled: SettledFailedPush = verdict_of_a_failed_push(
        failed(128, delivered, &["/srv/receives.git", "DISABLED"]),
        read(delivered, &["/srv/receives.git", "DISABLED"]),
        "refs/heads/main",
        Path::new("/srv/repo"),
        false,
    );
    let SettledFailedPush::Reached(said) = settled else { panic!("a branch that reached an address is done, not {settled:?}") };
    assert!(said.contains("`DISABLED`") && said.contains("/srv/receives.git"), "both addresses are named: {said}");
}
