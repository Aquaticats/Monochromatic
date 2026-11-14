//! Controls for what a user is told when the commit service cannot land a commit:
//!  no implementation term,
//! an action to take,
//! and text that derives from the named constants.

use super::*;
use std::collections::BTreeSet;

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
        ClientFailure::ServiceNotReading { took_nothing: true },
        ClientFailure::ServiceNotReading { took_nothing: false },
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

//region One line for each variant of `ClientFailure`: the name this file knows it by, and the pattern that matches it.

/// What:
///  Every variant of `ClientFailure` on a line of its own:
///  the name this file knows it by,
/// and the pattern that matches it.
/// Why:
///   The compiler checks that the lines cover every variant of the enum,
/// so a variant added to `ClientFailure` breaks this file until it has a line here,
/// and the name sits on the line of the pattern it names,
/// so there is no number,
/// and no second list,
/// to keep in step.
///  A number two variants shared is what let `every_failure_variant_is_rendered` pass
///  with the sample of `RepositoryFrozen` taken away (measured 2026-11-14).
///  Writing the name out keeps a repeated name expressible,
/// which is the mistake that test has to catch.
macro_rules! each_failure_variant {
    ( $( $name:literal => $pattern:pat ),* $(,)? ) => {
        /// The name of `failure`'s variant.
        fn variant_name(failure: &ClientFailure) -> &'static str {
            return match failure {
                $( $pattern => $name, )*
            };
        }

        /// Every name `variant_name` can give,
        /// one for each variant of `ClientFailure`.
        const VARIANT_NAMES: &[&str] = &[ $( $name ),* ];
    };
}

each_failure_variant! {
    "DidNotStart" => ClientFailure::DidNotStart,
    "NothingAnswered" => ClientFailure::NothingAnswered { .. },
    "NoActivation" => ClientFailure::NoActivation(_),
    "OtherAccount" => ClientFailure::OtherAccount,
    "MayNotHaveLanded" => ClientFailure::MayNotHaveLanded,
    "NoCommonVersion" => ClientFailure::NoCommonVersion,
    "OtherRelease" => ClientFailure::OtherRelease,
    "RepositoryFull" => ClientFailure::RepositoryFull(_),
    "RepositoryReadOnly" => ClientFailure::RepositoryReadOnly(_),
    "PermissionDenied" => ClientFailure::PermissionDenied(_),
    "RepositoryFrozen" => ClientFailure::RepositoryFrozen(_),
    "BranchKeepsChanging" => ClientFailure::BranchKeepsChanging { .. },
    "BranchMoved" => ClientFailure::BranchMoved { .. },
    "ConclusionStateChanged" => ClientFailure::ConclusionStateChanged { .. },
    "BranchMovedMessageSaved" => ClientFailure::BranchMovedMessageSaved { .. },
    "BranchSwitched" => ClientFailure::BranchSwitched { .. },
    "ServiceSilent" => ClientFailure::ServiceSilent,
    "ServiceNotReading" => ClientFailure::ServiceNotReading { .. },
    "ServiceSilentAfterPreparing" => ClientFailure::ServiceSilentAfterPreparing { .. },
    "StillWithService" => ClientFailure::StillWithService { .. },
    "WithdrawalUnconfirmed" => ClientFailure::WithdrawalUnconfirmed { .. },
    "Interrupted" => ClientFailure::Interrupted { .. },
    "InterruptedNotKnown" => ClientFailure::InterruptedNotKnown { .. },
    "StepTooSlow" => ClientFailure::StepTooSlow,
    "Refused" => ClientFailure::Refused,
    "IndexInUse" => ClientFailure::IndexInUse(_),
    "ReferenceLocked" => ClientFailure::ReferenceLocked { .. },
    "Unavailable" => ClientFailure::Unavailable,
    "ProgramMissing" => ClientFailure::ProgramMissing(_),
    "CouldNotStart" => ClientFailure::CouldNotStart(_),
    "SocketPathTooLong" => ClientFailure::SocketPathTooLong { .. },
    "ServiceElsewhere" => ClientFailure::ServiceElsewhere { .. },
    "UnitInTheWay" => ClientFailure::UnitInTheWay(_),
    "WorkNotHandedOver" => ClientFailure::WorkNotHandedOver(_),
    "ContentPruned" => ClientFailure::ContentPruned,
}

//endregion

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

/// What:
///  Every variant of `ClientFailure` has a sample of its own,
/// and no two variants are counted as one.
/// Why:
///   `samples` promises one of every failure and the other tests here read it as if it did,
/// so a variant no sample names would go unrendered.
/// The names,
/// and how many there are,
/// come from the one line each variant has above:
/// nothing here is a number or a list kept in step by hand.
///  Measured 2026-11-14:
///  with the sample of `RepositoryFrozen` taken away,
///  the numbered form of this test passed.
#[test]
fn every_failure_variant_is_rendered() {
    let covered: BTreeSet<&'static str> = samples().iter().map(variant_name).collect();
    let expected: BTreeSet<&'static str> = VARIANT_NAMES.iter().copied().collect();
    let unsampled: Vec<&str> = expected.difference(&covered).copied().collect();
    assert!(unsampled.is_empty(), "these variants of `ClientFailure` have no sample: {unsampled:?}");
    assert_eq!(
        covered.len(),
        VARIANT_NAMES.len(),
        "two variants of `ClientFailure` share one name, so one of them could go unrendered; the samples gave these names: {covered:?}",
    );
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
    // What: The failure that a program other than this release answers names the link whose pointing decides it.
    // Why:  A link that cannot be pointed is what keeps that program being started, and a person can only look at it by name.
    assert!(
        render(&ClientFailure::OtherRelease).contains(crate::service_instance::PROGRAM_LINK),
        "the ending names the link: {}",
        render(&ClientFailure::OtherRelease)
    );
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
    // What a service that stopped leaves a person to do: read the state of this worktree's service, restart the one unit it
    //  names, and read whether the commit was made. A restart of every service of the account is what these words must never
    //  ask for, and what the wildcard `RESTART_COMMAND` of the other failures still does.
    for failure in [
        ClientFailure::ServiceSilent,
        ClientFailure::ServiceNotReading { took_nothing: true },
        ClientFailure::ServiceNotReading { took_nothing: false },
        ClientFailure::NothingAnswered { address: String::from("/run/user/1000/x.sock"), waited: Duration::from_secs(10) },
    ] {
        let words: String = render(&failure);
        for command in [STATUS_COMMAND, RESTART_ONE_UNIT_COMMAND] {
            assert!(words.contains(command), "{failure:?} names {command}: {words}");
        }
        assert!(!words.contains(RESTART_COMMAND), "{failure:?} must not restart every service of the account: {words}");
        assert!(!words.contains(LOG_COMMAND), "{failure:?} must not read the journal of every service: {words}");
    }
    // A commit that was not made says so and says the staged work is untouched; one whose result nobody knows yet names the
    //  read that shows it.
    for not_made in [
        ClientFailure::ServiceNotReading { took_nothing: true },
        ClientFailure::ServiceNotReading { took_nothing: false },
        ClientFailure::NothingAnswered { address: String::from("/run/user/1000/x.sock"), waited: Duration::from_secs(10) },
    ] {
        let words: String = render(&not_made);
        assert!(words.contains(NOTHING_CHANGED), "{not_made:?} says nothing in the repository changed: {words}");
        assert!(words.contains("was not made"), "{not_made:?} says the commit was not made: {words}");
    }
    for not_known in [ClientFailure::ServiceSilent, ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::Unread }] {
        let words: String = render(&not_known);
        assert!(words.contains(CHECK_COMMANDS[0]), "{not_known:?} names what shows whether the commit was made: {words}");
        assert!(!words.contains(NOTHING_CHANGED), "{not_known:?} must not claim nothing changed: {words}");
    }
    let silent: String = render(&ClientFailure::ServiceSilent);
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
                // The branch was read afterwards. `Made` says the commit was made; `Unread` does not know; and `NotMade`
                //  says the branch holds no commit of this command's yet, which a service that wakes may still land, so
                //  none of the three may claim that nothing changed.
                | ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::Made | WhatTheBranchShows::Unread | WhatTheBranchShows::NotMade }
                | ClientFailure::WithdrawalUnconfirmed { .. }
                | ClientFailure::InterruptedNotKnown { .. }
                | ClientFailure::WorkNotHandedOver(_)
        );
        assert_eq!(!claims, may_have_landed, "{failure:?}");
    }
}

/// What:
///  A commit whose whole request did reach the service is not described as one that can never land.
/// Why:
///  The review of this change showed the landing's client-gone check losing to the connection writer's heartbeat,
///  so a service that wakes can land a request whose command is gone:
/// the branch read shows "not made" only at the moment it was read, and the words may say no more than that.
///  A commit whose request never arrived whole is the other case, and its words say the reason that is true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(render({ kind: 'silent-after-preparing', shows: 'not-made' })).toMatch(/may still land it/)
/// ```
#[test]
fn a_request_that_reached_the_service_is_not_described_as_one_that_never_can_land() {
    let not_made: String =
        render(&ClientFailure::ServiceSilentAfterPreparing { shows: WhatTheBranchShows::NotMade });
    assert!(not_made.contains("may still land it"), "the words allow a service that wakes to land it: {not_made}");
    assert!(not_made.contains(CHECK_COMMANDS[0]), "they name the read that shows whether it has: {not_made}");
    assert!(!not_made.contains("dropped before it lands"), "they do not claim a drop the landing does not promise: {not_made}");
    assert!(not_made.contains("once `git log -1` has shown that it was not made"), "and the step that runs it again waits for that read: {not_made}");
    // The other way round: a request that never reached the service whole says the reason that is true.
    for never_arrived in [
        ClientFailure::ServiceNotReading { took_nothing: true },
        ClientFailure::ServiceNotReading { took_nothing: false },
        ClientFailure::NothingAnswered { address: String::from("/run/user/1000/x.sock"), waited: Duration::from_secs(10) },
    ] {
        let words: String = render(&never_arrived);
        assert!(words.contains("framing newline"), "{never_arrived:?} says a request is read only when its newline arrived: {words}");
        assert!(!words.contains("dropped before it lands"), "{never_arrived:?} claims no drop: {words}");
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
