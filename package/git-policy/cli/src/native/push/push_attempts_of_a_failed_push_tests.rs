//! Controls of the reading of a failed push's output:
//! which addresses Git attempted,
//! whether the recorded ref arrived at any of them,
//! and which address Git's own failure sentence names.
//! Why:
//!  Every output below is what Git 2.56.0 printed for that shape on 2026-11-13,
//! measured with `probe-git-push-output.ts` and `probe-git-push-up-to-date.ts` beside the report of this run,
//! so a shape that changes with a Git release breaks a control here instead of a person's notice.

use super::*;

/// A repository that received the commit, then an address that is not a repository.
const REPOSITORY_THEN_DEAD: &str = "To /srv/remote-0.git\n * [new branch]      main -> main\nfatal: 'DISABLED' does not appear to be a git repository\nfatal: Could not read from remote repository.\n\nPlease make sure you have the correct access rights\nand the repository exists.\n";

/// The reserved word first: Git fails before it reaches the repository.
const DEAD_THEN_REPOSITORY: &str = "fatal: 'DISABLED' does not appear to be a git repository\nfatal: Could not read from remote repository.\n\nPlease make sure you have the correct access rights\nand the repository exists.\n";

/// A host that does not answer, then the reserved word Git never attempts.
const UNREACHABLE_THEN_DEAD: &str = "fatal: unable to access 'https://unreachable-host.invalid/repo.git/': Could not resolve host: unreachable-host.invalid\n";

/// Two addresses that both took the commit.
const TWO_REPOSITORIES: &str = "To /srv/remote-0.git\n * [new branch]      main -> main\nTo /srv/remote-1.git\n * [new branch]      main -> main\n";

/// A hook on the first address declined, and Git went on to the second, which took the commit.
const DECLINED_THEN_REPOSITORY: &str = "remote: a hook says no        \nTo /srv/remote-0.git\n ! [remote rejected] main -> main (pre-receive hook declined)\nerror: failed to push some refs to '/srv/remote-0.git'\nTo /srv/remote-1.git\n * [new branch]      main -> main\n";

/// The first address already held the commit, and the second is not a repository.
const UP_TO_DATE_THEN_DEAD: &str = "Everything up-to-date\nfatal: 'DISABLED' does not appear to be a git repository\nfatal: Could not read from remote repository.\n\nPlease make sure you have the correct access rights\nand the repository exists.\n";

/// A remote whose ref lock another process held:
/// the address answers, and refuses the ref.
const LOCK_HELD: &str = "remote: error: cannot lock ref 'refs/heads/main': Unable to create '/srv/DISABLED /refs/heads/main.lock': File exists.\nremote: \nTo DISABLED \n ! [remote rejected] main -> main (reference already exists)\nerror: failed to push some refs to 'DISABLED '\n";

/// The addresses the configuration above names.
fn configured(addresses: &[&str]) -> Vec<String> {
    return addresses.iter().map(|address| { return (*address).to_owned(); }).collect();
}

/// What:
///  A push Git delivered to one address and refused at the next reports the arrival.
/// Why:
///  The review's finding 2: the commit is on the first remote while the notice said it was only on this computer,
/// and the plan that keeps the branch backed up was ended.
#[test]
fn a_push_that_delivered_before_it_failed_reports_the_arrival() {
    let addresses: Vec<String> = configured(&["/srv/remote-0.git", "DISABLED"]);
    let read: FailedPushByAddress = failed_push_by_address(REPOSITORY_THEN_DEAD, "refs/heads/main", addresses.as_slice());
    assert!(read.arrived, "the first address took the ref");
    assert_eq!(read.arrived_at(), vec!["/srv/remote-0.git"]);
    assert_eq!(read.attempted.len(), 1, "Git printed one lane, for the address it reached");
    assert_eq!(read.failing.as_deref(), Some("DISABLED"), "and its fatal names the address it failed at");
    let two: Vec<String> = configured(&["/srv/remote-0.git", "/srv/remote-1.git"]);
    let both: FailedPushByAddress = failed_push_by_address(TWO_REPOSITORIES, "refs/heads/main", two.as_slice());
    assert_eq!(both.arrived_at(), vec!["/srv/remote-0.git", "/srv/remote-1.git"]);
    assert_eq!(both.failing, None, "a push that succeeded names no address it failed at");
}

/// What:
///  An address that already held the commit counts as an arrival, although Git prints no lane for it.
/// Why:
///  A retry of a delivered branch, or a person's push by hand, meets exactly this shape,
/// and reading it as "nothing arrived" would print the false notice of finding 2 again.
#[test]
fn an_address_that_already_held_the_commit_is_an_arrival() {
    let addresses: Vec<String> = configured(&["/srv/remote-0.git", "DISABLED"]);
    let read: FailedPushByAddress = failed_push_by_address(UP_TO_DATE_THEN_DEAD, "refs/heads/main", addresses.as_slice());
    assert!(read.arrived, "the first address already held the ref");
    assert_eq!(read.failing.as_deref(), Some("DISABLED"));
}

/// What:
///  A push that failed at an address Git names before it reached any other blames that address.
/// Why:
///  The review's finding 3: with `[a host that does not answer, DISABLED]` Git stops at the host and never attempts
/// `DISABLED`, yet the notice blamed `DISABLED` and ended a plan a retry could complete.
#[test]
fn the_address_git_failed_at_is_the_one_its_sentence_names() {
    let addresses: Vec<String> = configured(&["https://unreachable-host.invalid/repo.git", "DISABLED"]);
    let read: FailedPushByAddress = failed_push_by_address(UNREACHABLE_THEN_DEAD, "refs/heads/main", addresses.as_slice());
    assert!(!read.arrived, "nothing was delivered");
    assert!(read.attempted.is_empty(), "Git printed no lane, because it never reached an address");
    assert_eq!(
        read.failing.as_deref(),
        Some("https://unreachable-host.invalid/repo.git"),
        "the address in Git's own sentence, not the first address of a rule and not the reserved word beside it"
    );
    let dead_first: Vec<String> = configured(&["DISABLED", "/srv/remote-0.git"]);
    let first: FailedPushByAddress = failed_push_by_address(DEAD_THEN_REPOSITORY, "refs/heads/main", dead_first.as_slice());
    assert!(!first.arrived);
    assert_eq!(first.failing.as_deref(), Some("DISABLED"), "the address Git failed at first");
}

/// What:
///  An address that is the reserved word plus a space is read as the address, not as the reserved word.
/// Why:
///  The review's finding 4: `DISABLED ` is a legal path whose bare repository receives a push,
/// and trimming it turned a transient failure into a permanent give-up with a notice that called it impossible.
#[test]
fn an_address_is_read_byte_for_byte_as_git_printed_it() {
    let addresses: Vec<String> = configured(&["DISABLED "]);
    let read: FailedPushByAddress = failed_push_by_address(LOCK_HELD, "refs/heads/main", addresses.as_slice());
    assert!(!read.arrived, "the remote refused the ref");
    assert_eq!(read.attempted.len(), 1);
    assert_eq!(read.attempted[0].address, "DISABLED ", "the trailing space the configuration holds and Git printed");
    assert_eq!(
        read.failing.as_deref(),
        Some("DISABLED "),
        "the address Git's own sentence names, with the space inside its quotes"
    );
}

/// What:
///  A refusal on one address and an arrival at the next is an arrival, and the refusal's address is the failing one.
/// Why:
///  Git goes on to the next push URL after a rejection (measured), so the branch can be backed up while one address
/// refused it; reading the rejection as the whole push failed the branch that was published.
#[test]
fn a_refusal_before_an_arrival_is_still_an_arrival() {
    let addresses: Vec<String> = configured(&["/srv/remote-0.git", "/srv/remote-1.git"]);
    let read: FailedPushByAddress = failed_push_by_address(DECLINED_THEN_REPOSITORY, "refs/heads/main", addresses.as_slice());
    assert!(read.arrived, "the second address took the ref");
    assert_eq!(read.arrived_at(), vec!["/srv/remote-1.git"]);
    assert_eq!(read.attempted.len(), 2, "Git printed a lane for each address it reached");
    assert_eq!(read.failing.as_deref(), Some("/srv/remote-0.git"), "the address whose ref was refused");
}

/// What:
///  A sentence that names no configured address blames nothing.
/// Why:
///  A login Git refuses quotes only the host, and one that cannot resolve a host names no address at all;
/// guessing an address there ends a plan that may come back, which is what the owner's decision forbids.
#[test]
fn a_sentence_that_names_no_configured_address_blames_nothing() {
    let addresses: Vec<String> = configured(&["https://host.invalid/repo.git", "DISABLED"]);
    let login: &str = "fatal: could not read Username for 'https://host.invalid': terminal prompts disabled\n";
    assert_eq!(failed_push_by_address(login, "refs/heads/main", addresses.as_slice()).failing, None);
    let ssh: &str = "git@host.invalid: Permission denied (publickey).\nfatal: Could not read from remote repository.\n";
    assert_eq!(failed_push_by_address(ssh, "refs/heads/main", addresses.as_slice()).failing, None);
    let nothing_known: FailedPushByAddress = failed_push_by_address(REPOSITORY_THEN_DEAD, "refs/heads/main", &[]);
    assert_eq!(nothing_known.failing, None, "a configuration this release could not read blames nothing");
    assert!(nothing_known.arrived, "and the arrival is still read");
}

/// What:
///  Only the recorded ref's own status line says anything about the recorded ref.
/// Why:
///  A status line for another branch, and a line that is no status at all,
///  must not report an arrival for this branch.
#[test]
fn only_the_recorded_refs_own_status_line_reports_an_arrival() {
    let other: &str = "To /srv/remote-0.git\n * [new branch]      topic -> topic\nfatal: 'DISABLED' does not appear to be a git repository\n";
    let addresses: Vec<String> = configured(&["/srv/remote-0.git", "DISABLED"]);
    assert!(!failed_push_by_address(other, "refs/heads/main", addresses.as_slice()).arrived, "another branch's update is not this one");
    let mentioned: &str = "hint: 'main -> main' would be the ref\nfatal: 'DISABLED' does not appear to be a git repository\n";
    assert!(!failed_push_by_address(mentioned, "refs/heads/main", addresses.as_slice()).arrived, "a line that is no status line");
    let a_branch_outside_heads: &str = "To /srv/remote-0.git\n * [new ref]         refs/tags/v1 -> refs/tags/v1\n";
    assert!(
        failed_push_by_address(a_branch_outside_heads, "refs/tags/v1", addresses.as_slice()).arrived,
        "a ref outside refs/heads is read by its whole name"
    );
}

/// What:
///  The arrival question the reconcile asks is answered by the same reading.
/// Why:
///  The reconcile moves the local branch to the merge it published,
/// and that answer must not differ from the one the verdict uses.
#[test]
fn the_reconcile_asks_the_same_question() {
    assert!(arrived_somewhere(REPOSITORY_THEN_DEAD, "refs/heads/main"));
    assert!(!arrived_somewhere(UNREACHABLE_THEN_DEAD, "refs/heads/main"));
    assert!(arrived_somewhere(UP_TO_DATE_THEN_DEAD, "refs/heads/main"));
    assert!(!arrived_somewhere(LOCK_HELD, "refs/heads/main"));
    assert!(!arrived_somewhere("", "refs/heads/main"), "nothing printed is no arrival");
}
