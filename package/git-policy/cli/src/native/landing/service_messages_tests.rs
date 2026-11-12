//! Controls for what a user is told when the commit service cannot land a commit:
//!  no implementation term,
//! an action to take,
//! and text that derives from the named constants.

use super::*;

/// One sample of every failure,
/// and every platform,
/// so a new variant cannot go unrendered.
fn samples() -> Vec<ClientFailure> {
    let mut all: Vec<ClientFailure> = vec![
        ClientFailure::DidNotStart,
        ClientFailure::NothingAnswered { address: String::from("/run/user/1000/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock"), waited: Duration::from_secs(10) },
        ClientFailure::OtherAccount,
        ClientFailure::MayNotHaveLanded,
        ClientFailure::NoCommonVersion,
        ClientFailure::OtherRelease,
        ClientFailure::RepositoryFull(String::from("/work/repo")),
        ClientFailure::RepositoryReadOnly(String::from("/work/repo")),
        ClientFailure::PermissionDenied(String::from("/work/repo")),
        ClientFailure::RepositoryFrozen(String::from("/work/repo")),
        ClientFailure::BranchKeepsChanging { branch: String::from("main"), times: 5 },
        ClientFailure::BranchMoved { branch: String::from("main") },
        ClientFailure::ConclusionStateChanged { branch: String::from("main") },
        ClientFailure::BranchMovedMessageSaved { branch: String::from("main"), file: String::from("/work/repo/.git/monochromatic-git-policy-cli-saved-message-ab12"), amend: false },
        ClientFailure::BranchMovedMessageSaved { branch: String::from("main"), file: String::from("/work/repo/.git/monochromatic-git-policy-cli-saved-message-ab12"), amend: true },
        ClientFailure::BranchSwitched { branch: String::from("main"), now: String::from("side"), file: Some(String::from("/work/repo/.git/monochromatic-git-policy-cli-saved-message-ab12")) },
        ClientFailure::BranchSwitched { branch: String::from("main"), now: String::from("side"), file: None },
        ClientFailure::ServiceSilent,
        ClientFailure::ServiceSilentAfterPreparing { shows: crate::service_messages::WhatTheBranchShows::Made },
        ClientFailure::ServiceSilentAfterPreparing { shows: crate::service_messages::WhatTheBranchShows::NotMade },
        ClientFailure::ServiceSilentAfterPreparing { shows: crate::service_messages::WhatTheBranchShows::Unread },
        ClientFailure::StepTooSlow,
        ClientFailure::StillWithService { minutes: 20, blocker: Some(String::from("a landing behind another program's index lock")) },
        ClientFailure::StillWithService { minutes: 20, blocker: None },
        ClientFailure::WithdrawalUnconfirmed { blocker: None },
        ClientFailure::WithdrawalUnconfirmed { blocker: Some(String::from("monochromatic-git-policy-cli: the commit service has been inside a write for 21 minutes and has not heard back: `git update-ref --stdin` in /work/repo (it moves refs/heads/main to abc123; the commit is also at refs/worktree/cli-git-service/7 until the update ends).")) },
        ClientFailure::Refused,
        ClientFailure::IndexInUse(String::from("the editor (process 4242)")),
        ClientFailure::ReferenceLocked {
            branch: String::from("main"),
            words: String::from("Nothing proves that a Git process still holds the lock `/work/repo/.git/refs/heads/main.lock`. If no Git process is running in this repository, remove it with `rm -- /work/repo/.git/refs/heads/main.lock`."),
        },
        ClientFailure::Unavailable,
        ClientFailure::ProgramMissing(String::from("/opt/monochromatic-git-policy-cli/cli-git-service")),
        ClientFailure::CouldNotStart(String::from("the runtime directory has no space left")),
        ClientFailure::SocketPathTooLong {
            path: String::from("/home/a/temp/agent/a-long-folder/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock"),
            length: 113,
            longest: 107,
            longest_runtime: 40,
        },
        ClientFailure::ServiceElsewhere {
            made: String::from("/run/user/1000/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock"),
            looked_for: String::from("/home/a/temp/agent/rt/monochromatic-git-policy-cli/0123456789abcdef0123456789abcdef.sock"),
            manager_runtime: String::from("/run/user/1000"),
            client_runtime: String::from("/home/a/temp/agent/rt"),
        },
        ClientFailure::UnitInTheWay(String::from("/home/a/.config/systemd/user/cli-git.socket")),
        ClientFailure::WorkNotHandedOver(String::from("the work directory cannot be written")),
        ClientFailure::ContentPruned,
        ClientFailure::Interrupted { signal: libc::SIGINT, blocker: Some(String::from("a landing behind another program's index lock")) },
        ClientFailure::Interrupted { signal: libc::SIGTERM, blocker: None },
        ClientFailure::InterruptedNotKnown { signal: libc::SIGINT, blocker: None },
        ClientFailure::InterruptedNotKnown { signal: libc::SIGTERM, blocker: Some(String::from("the commit service has been inside a write for 21 minutes and has not heard back")) },
    ];
    for platform in [Platform::Windows, Platform::Bsd, Platform::LinuxWithoutUserServices, Platform::Other] {
        all.push(ClientFailure::NoActivation(platform));
    }
    return all;
}

/// Compile-time proof that `samples` covers every variant:
/// adding one breaks this match until a sample exists.
fn variant_index(failure: &ClientFailure) -> usize {
    return match failure {
        ClientFailure::DidNotStart => 0,
        ClientFailure::NoActivation(_) => 1,
        ClientFailure::OtherAccount => 2,
        ClientFailure::MayNotHaveLanded => 3,
        ClientFailure::NoCommonVersion => 4,
        ClientFailure::RepositoryFull(_) => 5,
        ClientFailure::RepositoryReadOnly(_) => 6,
        ClientFailure::BranchKeepsChanging { .. } => 7,
        ClientFailure::ServiceSilent => 8,
        ClientFailure::ServiceSilentAfterPreparing { .. } => 24,
        ClientFailure::Refused => 9,
        ClientFailure::IndexInUse(_) => 10,
        ClientFailure::ReferenceLocked { .. } => 25,
        ClientFailure::Unavailable => 11,
        ClientFailure::ProgramMissing(_) => 12,
        ClientFailure::CouldNotStart(_) => 13,
        ClientFailure::UnitInTheWay(_) => 15,
        ClientFailure::WorkNotHandedOver(_) => 14,
        ClientFailure::BranchMoved { .. } => 16,
        ClientFailure::BranchMovedMessageSaved { .. } => 17,
        ClientFailure::OtherRelease => 18,
        ClientFailure::BranchSwitched { .. } => 19,
        ClientFailure::ContentPruned => 23,
        ClientFailure::PermissionDenied(_) => 27,
        ClientFailure::RepositoryFrozen(_) => 25,
        ClientFailure::ConclusionStateChanged { .. } => 26,
        ClientFailure::StepTooSlow => 22,
        ClientFailure::StillWithService { .. } => 20,
        ClientFailure::WithdrawalUnconfirmed { .. } => 21,
        ClientFailure::SocketPathTooLong { .. } => 28,
        ClientFailure::ServiceElsewhere { .. } => 29,
        ClientFailure::NothingAnswered { .. } => 32,
        ClientFailure::Interrupted { .. } => 30,
        ClientFailure::InterruptedNotKnown { .. } => 31,
    };
}

/// The text without the parts between backticks,
/// which quote commands the user types.
fn without_commands(text: &str) -> String {
    let mut plain: String = String::new();
    let mut quoted: bool = false;
    for character in text.chars() {
        if character == '`' {
            quoted = !quoted;
        } else if !quoted {
            plain.push(character);
        }
    }
    return plain;
}

/// Whether `word` (one or more words) occurs in `text` as whole words,
/// ignoring case.
fn contains_whole(text: &str, word: &str) -> bool {
    let haystack: String = text.to_lowercase();
    let needle: String = word.to_lowercase();
    let mut from: usize = 0;
    while let Some(at) = haystack[from..].find(needle.as_str()) {
        let start: usize = from + at;
        let end: usize = start + needle.len();
        let before_ok: bool = haystack[..start].chars().next_back().is_none_or(|c| { return !c.is_alphanumeric(); });
        let after_ok: bool = haystack[end..].chars().next().is_none_or(|c| { return !c.is_alphanumeric(); });
        if before_ok && after_ok {
            return true;
        }
        from = start + 1;
    }
    return false;
}

/// Every variant has a sample,
/// and every sample is a variant of its own:
/// a number two variants share would let one of them go unrendered.
/// The highest number is the last one,
/// so a sample added without a number of its own fails here.
#[test]
fn every_failure_variant_is_rendered() {
    let covered: std::collections::BTreeSet<usize> = samples().iter().map(variant_index).collect();
    let expected: usize = variant_index(&ClientFailure::NothingAnswered { address: String::new(), waited: Duration::ZERO }) + 1;
    assert_eq!(covered.len(), expected);
}

/// No failure text contains a term from the denylist,
/// outside quoted commands.
#[test]
fn no_message_leaks_an_implementation_term() {
    for failure in samples() {
        let plain: String = without_commands(render(&failure).as_str());
        for term in DENYLIST {
            assert!(!contains_whole(plain.as_str(), term), "{failure:?} leaks {term:?}: {plain}");
        }
    }
    for notice in [STARTING_NOTICE, UPDATING_NOTICE] {
        for term in DENYLIST {
            assert!(!contains_whole(notice, term), "{notice:?} leaks {term:?}");
        }
    }
    for (number, of) in [(1, 1), (2, 5), (9, 9)] {
        let notice: String = position_notice(number, of);
        for term in DENYLIST {
            assert!(!contains_whole(notice.as_str(), term), "{notice:?} leaks {term:?}");
        }
    }
    for waited in [Duration::ZERO, Duration::from_millis(400), Duration::from_secs(1), Duration::from_secs(12), Duration::from_secs(3_600)] {
        let notice: String = unplaced_notice(waited);
        for term in DENYLIST {
            assert!(!contains_whole(notice.as_str(), term), "{notice:?} leaks {term:?}");
        }
    }
}


/// What:
///  A commit the service named no place for is told what the client knows,
/// and the seconds it has waited,
/// and nothing about whether the landing has begun.
/// Why:
///   The landing may be waiting behind another program's `index.lock` while the client is told this,
/// so every claim about
///  the landing would be a claim the client cannot make.
#[test]
fn the_unplaced_notice_says_what_the_client_knows_and_no_more() {
    let waited: String = unplaced_notice(Duration::from_secs(12));
    assert_eq!(waited, "The commit service has not said where this commit stands yet; it has waited 12 seconds.");
    assert_eq!(unplaced_notice(Duration::from_secs(1)), "The commit service has not said where this commit stands yet; it has waited 1 second.");
    assert_eq!(unplaced_notice(Duration::from_millis(400)), "The commit service has not said where this commit stands yet; it has waited 1 second.", "a part second is a second waited");
    for claim in ["has not started", "still reading", "has begun", "is waiting for the index"] {
        assert!(!waited.contains(claim), "the notice may not say {claim:?}: {waited}");
    }
}

/// What:
///  A commit whose service has been quiet for one silence bound is told how long,
///  what the command does about it,
///  and that nothing is asked of the reader.
/// Why:
///  The person cannot tell a busy service from a stopped one,
///  so the words may not claim a stop the client has not seen:
///  the service that printed this line on 2026-11-11 had landed the commit
///  (`doc/troubleshooting/git-policy-cli-service-answers-a-commit-late-while-a-round-is-busy.md`).
#[test]
fn the_silent_notice_names_the_seconds_and_asks_nothing_of_the_reader() {
    let waited: String = silent_notice(Duration::from_secs(5));
    assert_eq!(
        waited,
        "The commit service has said nothing for 5 seconds, so the same commit is being sent to it again; \
         a commit cannot land twice, and this command goes on by itself, so there is nothing for you to do."
    );
    assert_eq!(
        silent_notice(Duration::from_millis(400)),
        "The commit service has said nothing for 1 second, so the same commit is being sent to it again; \
         a commit cannot land twice, and this command goes on by itself, so there is nothing for you to do.",
        "a part second is a second waited"
    );
    assert!(!waited.contains("stopped"), "the client cannot say the service stopped: {waited}");
}

/// The denylist check can fail:
/// a text that leaks is caught,
/// and a word inside another word is not.
#[test]
fn the_denylist_catches_a_leak_and_ignores_inner_words() {
    assert!(contains_whole("The daemon refused the request id.", "request id"));
    assert!(contains_whole("Socket closed (EPIPE).", "socket"));
    assert!(!contains_whole("Please avoid the voided path.", "oid"));
    assert!(!contains_whole("The rapid pipeline.", "pid"));
    assert!(contains_whole("process pid 42", "pid"));
}

/// Every failure tells the user something to do:
/// a quoted command or one of the plain verbs that start a remedy.
#[test]
fn every_message_gives_an_action() {
    let verbs = ["run", "use", "install", "free", "make", "wait", "update", "raise", "ask", "turn", "check", "commit"];
    for failure in samples() {
        let text: String = render(&failure);
        let has_command: bool = text.contains('`');
        let has_verb: bool = verbs.iter().any(|verb| { return contains_whole(text.as_str(), verb); });
        assert!(has_command || has_verb, "{failure:?} gives no action: {text}");
        assert!(!text.trim().is_empty());
    }
}

/// The text derives from the named constants and the facts it names,
/// so it cannot drift from them.
#[test]
fn messages_quote_their_constants_and_facts() {
    assert!(render(&ClientFailure::UnitInTheWay(String::from("/home/a/.config/systemd/user/cli-git.socket"))).contains("/home/a/.config/systemd/user/cli-git.socket"));
    assert!(render(&ClientFailure::DidNotStart).contains(LOG_COMMAND));
    assert!(render(&ClientFailure::Unavailable).contains(LOG_COMMAND));
    assert!(render(&ClientFailure::OtherRelease).contains(RESTART_COMMAND));
    assert!(render(&ClientFailure::NoCommonVersion).contains(STOP_COMMAND));
    let unsure: String = render(&ClientFailure::MayNotHaveLanded);
    for command in CHECK_COMMANDS {
        assert!(unsure.contains(command), "{unsure}");
    }
    assert!(render(&ClientFailure::NoActivation(Platform::LinuxWithoutUserServices)).contains(LINGER_COMMAND));
    let branch: String = render(&ClientFailure::BranchKeepsChanging { branch: String::from("topic"), times: 7 });
    assert!(branch.contains("topic") && branch.contains('7'), "{branch}");
    assert!(render(&ClientFailure::BranchMoved { branch: String::from("topic") }).contains("topic"));
    let changed: String = render(&ClientFailure::ConclusionStateChanged { branch: String::from("topic") });
    assert!(changed.contains("topic") && changed.contains("cherry-pick") && changed.contains("revert"), "{changed}");
    let saved: String = render(&ClientFailure::BranchMovedMessageSaved { branch: String::from("topic"), file: String::from("/r/.git/saved"), amend: true });
    assert!(saved.contains("`git commit --amend -F /r/.git/saved`") && saved.contains("topic"), "{saved}");
    let plain: String = render(&ClientFailure::BranchMovedMessageSaved { branch: String::from("topic"), file: String::from("/r/.git/saved"), amend: false });
    assert!(plain.contains("`git commit -F /r/.git/saved`"), "{plain}");
    let switched: String = render(&ClientFailure::BranchSwitched { branch: String::from("topic"), now: String::from("side"), file: Some(String::from("/r/.git/saved")) });
    assert!(switched.contains("topic") && switched.contains("side") && switched.contains("`git commit -F /r/.git/saved`"), "{switched}");
    let unsaved: String = render(&ClientFailure::BranchSwitched { branch: String::from("topic"), now: String::from("side"), file: None });
    assert!(!unsaved.contains("saved in"), "{unsaved}");
    assert!(render(&ClientFailure::RepositoryFull(String::from("/work/repo"))).contains("/work/repo"));
    let silent: String = render(&ClientFailure::ServiceSilent);
    for command in CHECK_COMMANDS.into_iter().chain([LOG_COMMAND, RESTART_COMMAND]) {
        assert!(silent.contains(command), "{silent}");
    }
    assert!(!silent.contains("raise") && !silent.contains("setting"), "the text must not name a setting that does not exist: {silent}");
    assert!(render(&ClientFailure::IndexInUse(String::from("the editor"))).contains("the editor"));
}

/// Failures that changed nothing say so;
/// the one that may have changed something does not claim otherwise.
#[test]
fn only_the_unsure_failure_does_not_claim_nothing_changed() {
    for failure in samples() {
        let claims: bool = render(&failure).contains(NOTHING_CHANGED);
        let may_have_landed: bool = matches!(
            failure,
            ClientFailure::MayNotHaveLanded
                | ClientFailure::ServiceSilent
                | ClientFailure::ServiceSilentAfterPreparing { .. }
                | ClientFailure::WithdrawalUnconfirmed { .. }
                | ClientFailure::InterruptedNotKnown { .. }
                | ClientFailure::WorkNotHandedOver(_)
        );
        assert_eq!(!claims, may_have_landed, "{failure:?}");
    }
}

/// Queue notices count the commits ahead.
#[test]
fn position_notices_count_the_commits_ahead() {
    assert_eq!(position_notice(1, 4), "This commit is next.");
    assert_eq!(position_notice(2, 4), "Waiting for 1 earlier commit.");
    assert_eq!(position_notice(4, 4), "Waiting for 3 earlier commits.");
}

/// A read-only filesystem keeps today's sentence,
/// a permission error says whose permission it is and what to check,
/// and a repository a move froze says so and when it thaws:
/// three different sentences,
/// none of them borrowed from another.
#[test]
fn a_read_only_filesystem_a_permission_error_and_a_frozen_move_each_have_their_own_sentence() {
    let read_only: String = render(&ClientFailure::RepositoryReadOnly(String::from("/work/repo")));
    assert!(read_only.contains("cannot be written to") && read_only.contains("mounted read-only"), "{read_only}");
    let denied: String = render(&ClientFailure::PermissionDenied(String::from("/work/repo")));
    assert!(denied.contains("/work/repo") && denied.contains("permission"), "{denied}");
    assert!(!denied.contains("mounted read-only") && !denied.contains("move"), "a permission problem is not a read-only disk or a move: {denied}");
    let frozen: String = render(&ClientFailure::RepositoryFrozen(String::from("/work/repo")));
    assert!(frozen.contains("a repository move has frozen the repository"), "{frozen}");
    assert!(frozen.contains("thaws when the move's thaw step runs"), "{frozen}");
    assert!(frozen.contains(NOTHING_CHANGED), "{frozen}");
    assert!(!frozen.contains("permission"), "{frozen}");
}

/// The line for a rebuild names the branch,
/// the builds so far,
/// the time so far and when the rebuilding stops,
/// and promises nothing it does not do.
#[test]
fn the_rebuild_notice_names_the_branch_the_builds_the_time_and_the_ceiling() {
    let said: String = rebuild_notice("main", 3, std::time::Duration::from_secs(41), std::time::Duration::from_secs(1200));
    for part in ["the branch main moved", "3 builds so far, 41 seconds", "built again on the new tip", "nothing conflicted", "stops after 1200 seconds"] {
        assert!(said.contains(part), "{part}: {said}");
    }
    assert!(said.ends_with('\n') && !said.contains("so it was built again"), "{said}");
}

/// What:
///  A git older than the running service is told both ways to get a service it can work with,
/// and the one a rollback needs names the command that stops the newer service.
/// Why:
///  Finding 2 of the audit of service restarts and version skew (2026-11-09):
/// with a newer service running,
/// an older client exits in milliseconds with advice to update the wrapper,
/// which is the opposite of a rollback,
/// and the newer service then runs until its idle exit (`IDLE_EXIT`) because nothing names the command that stops it.
/// `AGENTS.md` rule `DGT`:
/// name every valid remediation path.
#[test]
fn an_older_git_is_told_how_to_get_a_service_it_can_work_with() {
    let text: String = render(&ClientFailure::NoCommonVersion);
    assert!(text.contains(STOP_COMMAND), "the stop a rollback needs is named: {text}");
    assert!(text.contains("Update this git"), "the upgrade is still named: {text}");
    assert!(text.contains("install the release"), "the rollback is named: {text}");
    assert!(text.contains(NOTHING_CHANGED), "nothing landed: {text}");
    assert!(!text.contains(RESTART_COMMAND), "a restart starts the newer service again, so it is not the remedy here: {text}");
}
