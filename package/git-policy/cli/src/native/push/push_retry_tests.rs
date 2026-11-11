//! Controls for the push retry's schedule:
//! the pauses grow and stop growing,
//! a variable sets the first pause,
//! a plan is
//! per branch,
//! and only the conflict report is final.

use super::*;
use crate::work_item::WorkKind;

fn item(branch: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: PathBuf::from("/srv/repo"),
        git_dir: PathBuf::from("/srv/repo/.git"),
        branch_ref: branch.to_owned(),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::from("2026-11-03T05:40:00.000Z"),
        tries: 0,
    };
}

/// Each pause is twice the one before,
/// from the first,
/// up to the longest.
#[test]
fn the_pauses_double_and_stop_at_the_longest() {
    let first: Duration = Duration::from_secs(10);
    let pauses: Vec<Duration> = (1..=4).map(|attempt| { return pause_before(attempt, first); }).collect();
    assert_eq!(pauses, vec![first, first * 2, first * 4, first * 8]);
    assert_eq!(pause_before(1_000, first), LONGEST_PAUSE);
    assert_eq!(pause_before(u32::MAX, first), LONGEST_PAUSE);
    assert!(pause_before(0, first) == first, "a try number of zero is the first");
}

/// The variable sets the first pause in whole seconds;
/// anything else is the default,
/// and never longer than the longest.
#[test]
fn the_variable_sets_the_first_pause() {
    assert_eq!(first_pause_from(None), FIRST_PAUSE);
    assert_eq!(first_pause_from(Some(std::ffi::OsStr::new("3"))), Duration::from_secs(3));
    for unusable in ["0", "-1", "1.5", "soon", ""] {
        assert_eq!(first_pause_from(Some(std::ffi::OsStr::new(unusable))), FIRST_PAUSE, "{unusable:?}");
    }
    assert_eq!(first_pause_from(Some(std::ffi::OsStr::new("99999"))), LONGEST_PAUSE);
}

/// A plan is due after its pause and not before,
/// replaces an earlier plan of its branch,
/// and belongs to its Git directory.
#[test]
fn a_plan_is_per_branch_and_due_after_its_pause() {
    let mut retries: PushRetries = PushRetries::with_first_pause(Duration::from_secs(10));
    let now: Instant = Instant::now();
    let here: &Path = Path::new("/srv/repo/.git");
    let elsewhere: &Path = Path::new("/srv/other/.git");
    assert_eq!(retries.schedule(here, &item("refs/heads/main"), 1, now), Duration::from_secs(10));
    assert_eq!(retries.schedule(here, &item("refs/heads/main"), 2, now), Duration::from_secs(20), "the same branch has one plan");
    retries.schedule(here, &item("refs/heads/topic"), 1, now);
    assert_eq!(retries.next_wait(here, now), Some(Duration::from_secs(10)));
    assert_eq!(retries.next_wait(elsewhere, now), None);
    assert!(retries.take_due(here, now + Duration::from_secs(9)).is_empty());
    assert!(retries.take_due(elsewhere, now + Duration::from_secs(60)).is_empty());
    let due: Vec<Due> = retries.take_due(here, now + Duration::from_secs(10));
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].item.branch_ref, "refs/heads/topic");
    retries.forget(here, "refs/heads/main");
    assert_eq!(retries.next_wait(here, now), None);
}

/// What:
///  A forget ends the plan a branch had,
/// while the plans of other branches stay.
/// Why:
///   A landed push makes an earlier failure of its branch untrue,
/// and the other branches' retries are still wanted.
#[test]
fn a_plan_written_before_the_forget_is_removed_by_it() {
    let root: PathBuf = crate::test_support::fixture("retry-forget-older");
    let here: &Path = Path::new("/srv/repo/.git");
    save(root.as_path(), &StoredPlan { git_dir: here.to_path_buf(), item: item("refs/heads/main"), attempt: 1 }).expect("saved");
    let other: StoredPlan = StoredPlan { git_dir: here.to_path_buf(), item: item("refs/heads/topic"), attempt: 1 };
    save(root.as_path(), &other).expect("saved");
    let mut retries: PushRetries = PushRetries::restored(Duration::from_secs(10), root.as_path());
    retries.forget(here, "refs/heads/main");
    assert_eq!(load(root.as_path()), vec![other], "only the forgotten branch's plan is gone");
    crate::test_support::remove(root.as_path());
}

/// Only a reported conflict needs the person;
/// a stall and a refused connection do not.
#[test]
fn only_a_conflict_needs_the_person() {
    assert!(needs_the_person("monochromatic-git-policy-cli: auto-push: not published: merging origin/main into main conflicts in: a.txt, b.txt. Nothing was pushed"));
    for output in [
        "monochromatic-git-policy-cli: git push made no progress for 30 seconds and was stopped (limit=stall, bytes_moved=0, elapsed_seconds=30)",
        "fatal: unable to access 'https://example.invalid/': Could not resolve host: example.invalid",
        "",
    ] {
        assert!(!needs_the_person(output), "{output}");
    }
}

// The texts below are Git 2.56.0's own words, captured by running `git push` from that release against fixtures on
// 2026-11-03 (a bare remote with a declining hook; a local HTTP server that answers 401 or 403; a loopback `sshd` that accepts no
// key; a port nothing listens on), and checked against its source at tag `v2.56.0`.

/// `builtin/receive-pack.c` `"pre-receive hook declined"`,
/// shown by `transport.c` `print_ref_status` as `[remote rejected]`;
/// the
/// `remote:` line is the hook's own output.
const PRE_RECEIVE_DECLINED: &str = "remote: protected: pushes to main need a review        \nTo /srv/remote.git\n ! [remote rejected] main -> main (pre-receive hook declined)\nerror: failed to push some refs to '/srv/remote.git'\n";

/// `builtin/receive-pack.c` `ret = "hook declined"` for the update hook,
/// with `rp_error("hook declined to update %s")`.
const UPDATE_DECLINED: &str = "remote: update says no        \nremote: error: hook declined to update refs/heads/main        \nTo /srv/remote.git\n ! [remote rejected] main -> main (hook declined)\nerror: failed to push some refs to '/srv/remote.git'\n";

/// A hosting service's own reason in the same position,
/// which names a hook and carries no code of its own.
/// GitHub writes this reason with its own `GH006` line in front of it,
/// so a refusal without the code is one nothing says is the protected-branch rule of a hosting service.
const PROTECTED_BRANCH_DECLINED: &str = " ! [remote rejected] main -> main (protected branch hook declined)\n";

/// What:
///  GitHub's answer to the commit service's auto-push of `main`,
///  word for word,
/// at `2026-11-11T06:42-05:00`.
/// Why:
///   The wrapper printed this refusal as a server hook's decline and stopped pushing the branch until the next commit,
/// and a plain `git push origin main` of the same commit two minutes later landed it
///  (`doc/troubleshooting/github-protected-branch-refusal-that-cleared.md`).
/// The `GH006` line is the only part of it that says the refusal is the hosting service's protected-branch rule:
/// the reason beside it is the
///  position where any server writes its own words,
/// and the reason after the code is empty.
const GITHUB_PROTECTED_BRANCH: &str = "remote: error: GH006: Protected branch update failed for refs/heads/main.\nremote: \nTo https://github.com/Aquaticats/Monochromatic\n ! [remote rejected]       main -> main (protected branch hook declined)\nerror: failed to push some refs to 'https://github.com/Aquaticats/Monochromatic'\n";

/// `remote-curl.c` `die(_("Authentication failed for '%s'"))`,
/// for a 401 with a user name and no usable password.
const AUTHENTICATION_FAILED: &str = "remote: login needed\nfatal: Authentication failed for 'http://127.0.0.1:39197/r.git/'\n";

/// `prompt.c` `die("could not read %s%s", prompt, "terminal prompts disabled")`,
/// for a 401 with no user name and prompts off.
const PROMPTS_DISABLED: &str = "fatal: could not read Username for 'http://127.0.0.1:39197': terminal prompts disabled\n";

/// `http.c` `"The requested URL returned error: %ld"`,
/// for a 403.
const FORBIDDEN: &str = "remote: forbidden\nfatal: unable to access 'http://127.0.0.1:39871/r.git/': The requested URL returned error: 403\n";

/// OpenSSH's refusal of a key,
/// which Git relays;
/// real `ssh` against a loopback `sshd` that accepts no key.
const SSH_REFUSED: &str = "user@127.0.0.1: Permission denied (publickey).\nfatal: Could not read from remote repository.\n\nPlease make sure you have the correct access rights\nand the repository exists.\n";

/// `http.c` through curl,
/// for a port nothing listens on.
const CONNECTION_REFUSED: &str = "fatal: unable to access 'http://127.0.0.1:1/r.git/': Failed to connect to 127.0.0.1:1 after 0 ms: Could not connect to server\n";

/// curl,
/// for a name that does not resolve.
const UNRESOLVED: &str = "fatal: unable to access 'http://nonexistent.invalid/r.git/': Could not resolve host: nonexistent.invalid\n";

/// A login failure is final:
/// the service cannot log in,
/// and retrying only asks the remote again.
#[test]
fn an_authentication_failure_is_not_tried_again() {
    for output in [AUTHENTICATION_FAILED, PROMPTS_DISABLED, FORBIDDEN, SSH_REFUSED] {
        assert_eq!(final_failure(output), Some(FinalFailure::Login), "{output}");
        assert!(needs_the_person(output), "{output}");
    }
}

/// What:
///  A refusal GitHub answers with its protected-branch code is tried again,
///  and named by nothing as an unknown reason.
/// Why:
///   A protected-branch refusal is the server's answer at that moment,
///  never a property of the commit:
/// GitHub refused `29bae9bb87` with `GH006` at `2026-11-09T23:30:11-05:00` and took it unchanged moments later,
/// and refused the auto-push of `main` with it again at `2026-11-11T06:42-05:00`,
/// which a push by hand landed two minutes later
///  (`doc/troubleshooting/github-protected-branch-refusal-that-cleared.md`).
///   The owner's decision of 2026-11-11 is to retry it,
/// which the service's plan does within a fixed number of tries.
#[test]
fn a_github_protected_branch_refusal_is_tried_again() {
    assert_eq!(final_failure(GITHUB_PROTECTED_BRANCH), None, "{GITHUB_PROTECTED_BRANCH}");
    assert!(!needs_the_person(GITHUB_PROTECTED_BRANCH), "the person is not sent to push by hand for it: {GITHUB_PROTECTED_BRANCH}");
    assert_eq!(
        unrecognised_rejection(GITHUB_PROTECTED_BRANCH),
        None,
        "a refusal the release knows is not reported as a reason it does not: {GITHUB_PROTECTED_BRANCH}"
    );
}

/// A server hook's decline is final,
/// whichever hook declined and whoever wrote its reason,
/// as long as no protected-branch code says what the refusal is.
#[test]
fn a_server_hook_decline_is_not_tried_again() {
    for output in [PRE_RECEIVE_DECLINED, UPDATE_DECLINED, PROTECTED_BRANCH_DECLINED] {
        assert_eq!(final_failure(output), Some(FinalFailure::Declined), "{output}");
        assert_eq!(unrecognised_rejection(output), None, "a decline that is known is not reported as unknown: {output}");
    }
}

/// Being offline is not a login failure and not a decline:
/// it is tried again,
/// as is a stall.
#[test]
fn an_offline_failure_is_still_tried_again() {
    for output in [
        CONNECTION_REFUSED,
        UNRESOLVED,
        "monochromatic-git-policy-cli: git push made no progress for 30 seconds and was stopped (limit=stall, bytes_moved=0, elapsed_seconds=30)",
    ] {
        assert_eq!(final_failure(output), None, "{output}");
    }
}

/// An HTTP 5xx answer of the LFS server is outside our control,
/// like a network that is down:
/// the push that met it ends with the
///  program's words and the wrapper's explanation,
/// and no final failure is read from them,
/// so the plan tries the push again by
///  itself (pick 12 of the shipped-limits audit,
/// the coordinator's under the owner's delegation,
/// open to the owner's veto).
#[test]
fn an_lfs_server_error_is_tried_again_by_the_plan() {
    for status in ["500", "502", "503", "504"] {
        let program_words: String = format!("batch response: Fatal error: Server error https://<credentials>@lfs.example.invalid/objects/batch from HTTP {status}");
        let output: String = format!("{program_words}\n{}", crate::lfs_pre_push::failed_message("exited with status 2"));
        assert_eq!(final_failure(output.as_str()), None, "{output}");
        assert_eq!(unrecognised_rejection(output.as_str()), None, "a failure that is not a rejection names nothing: {output}");
    }
}

/// What:
///  The plain words of a failure that stops the retry say what is known,
///  and nothing the wrapper cannot know.
/// Why:
///   GitHub refused a push of `main` at `2026-11-09T23:30:11-05:00` with
///  GitHub's own words `GH006: Protected branch update failed for refs/heads/main.`,
///  the wrapper said that trying again would be refused the same way,
///  and a plain `git push origin main` moments later succeeded
///  (`doc/troubleshooting/github-protected-branch-refusal-that-cleared.md`).
/// The owner's decision keeps the retry's behaviour,
/// so the words say the decline,
///  that this branch waits for the next commit,
///  and that a push by hand may still land.
#[test]
fn the_words_of_a_final_failure_say_what_is_known_and_promise_no_outcome() {
    assert_eq!(plain_words(FinalFailure::Conflict), None, "a conflict's report already says what to do");
    let login: &str = plain_words(FinalFailure::Login).expect("a login failure is worded");
    assert!(login.contains("wants a login") && login.contains("until your next commit"), "{login}");
    let declined: &str = plain_words(FinalFailure::Declined).expect("a decline is worded");
    for said in ["a hook on the server", "declined it", "until your next commit", "may succeed"] {
        assert!(declined.contains(said), "{said} in {declined}");
    }
    for absent in ["refused the same way", "would be refused"] {
        assert!(!declined.contains(absent), "{absent} in {declined}");
    }
}

/// What:
///  A protected-branch refusal is given a fixed number of tries and no more,
///  while a plain hook decline never reaches a count of its own.
/// Why:
///   The owner asked on 2026-11-11 that the wrapper retry GitHub's `GH006`,
/// and "nothing should ever wait forever or indefinitely" means the retry ends;
/// the count is at least two because the first failure of a push always plans its next try,
/// so a bound of one would leave the service planning a try it must not make.
#[test]
fn a_protected_branch_refusal_is_given_a_fixed_number_of_tries() {
    assert!(PROTECTED_BRANCH_TRIES >= 2, "the first failure plans the second try, so no bound can be below two");
    for try_number in 1..PROTECTED_BRANCH_TRIES {
        assert!(!out_of_tries(GITHUB_PROTECTED_BRANCH, try_number), "try {try_number} is not the last one");
    }
    for try_number in [PROTECTED_BRANCH_TRIES, PROTECTED_BRANCH_TRIES + 1, 1_000] {
        assert!(out_of_tries(GITHUB_PROTECTED_BRANCH, try_number), "try {try_number} is past the bound");
    }
    for output in [PRE_RECEIVE_DECLINED, UPDATE_DECLINED, PROTECTED_BRANCH_DECLINED] {
        assert!(!out_of_tries(output, 1_000), "a decline with no protected-branch code is final and never counted: {output}");
    }
    for output in [CONNECTION_REFUSED, UNRESOLVED] {
        assert!(!out_of_tries(output, 1_000), "a cause that can pass is retried until it passes and has no count: {output}");
    }
}

/// What:
///  The words of a protected-branch refusal that outlasted its tries name what the server did,
///  the number of tries,
///  how long their pauses covered,
///  and both ways out;
///  and they promise no try that is not coming.
/// Why:
///   The person has to know that the service gave up and when,
/// otherwise the quiet branch looks like a push that is still being tried.
#[test]
fn the_words_of_a_refusal_that_outlasted_its_tries_name_the_tries_and_the_time() {
    let first: Duration = Duration::from_secs(10);
    let span: Duration = (1..PROTECTED_BRANCH_TRIES).map(|try_number| { return pause_before(try_number, first); }).fold(Duration::ZERO, |sum, pause| { return sum + pause; });
    let words: String = out_of_tries_words(first);
    let tries_phrase: String = format!("tried it {PROTECTED_BRANCH_TRIES} times");
    let span_phrase: String = format!("{} minutes and {} seconds", span.as_secs() / 60, span.as_secs() % 60);
    for said in [
        "the server refused this push as an update to a protected branch",
        "stopped trying this branch",
        "signed commits",
        "commit again",
        "`git push` by hand",
        tries_phrase.as_str(),
        span_phrase.as_str(),
    ] {
        assert!(words.contains(said), "{said} in {words}");
    }
    for absent in ["until the push lands", "keeps trying"] {
        assert!(!words.contains(absent), "{absent} in {words}");
    }
    // A plan whose pauses add up to less than a minute says seconds, and one that adds up to none says so.
    let short: Duration = Duration::from_secs(1);
    let short_span: u64 = (1..PROTECTED_BRANCH_TRIES).map(|try_number| { return pause_before(try_number, short).as_secs(); }).sum();
    let short_words: String = out_of_tries_words(short);
    assert!(short_words.contains(&format!("{short_span} seconds")), "{short_words}");
    assert!(!short_words.contains("minutes"), "a span under a minute names no minutes: {short_words}");
    let tiny_words: String = out_of_tries_words(Duration::from_millis(1));
    assert!(tiny_words.contains("less than a second"), "{tiny_words}");
}

/// A rejection by the server whose reason this release does not know stays retried,
/// and is found so the journal can name it.
#[test]
fn a_rejection_nobody_knows_is_retried_and_named() {
    let output: &str = "To /srv/remote.git\n ! [remote rejected] main -> main (prohibited by the review system)\nerror: failed to push some refs\n";
    assert_eq!(final_failure(output), None);
    assert_eq!(unrecognised_rejection(output).as_deref(), Some("! [remote rejected] main -> main (prohibited by the review system)"));
    assert_eq!(unrecognised_rejection(CONNECTION_REFUSED), None, "a failure that is not a rejection names nothing");
}
