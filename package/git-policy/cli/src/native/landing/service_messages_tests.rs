//! Controls for what a user is told when the commit service cannot land a commit:
//!  no implementation term, an action to take, and text that derives from the named constants.

use super::*;

/// One sample of every failure, and every platform, so a new variant cannot go unrendered.
fn samples() -> Vec<ClientFailure> {
    let mut all: Vec<ClientFailure> = vec![
        ClientFailure::DidNotStart,
        ClientFailure::OtherAccount,
        ClientFailure::MayNotHaveLanded,
        ClientFailure::NoCommonVersion,
        ClientFailure::OtherRelease,
        ClientFailure::RepositoryFull(String::from("/work/repo")),
        ClientFailure::RepositoryReadOnly(String::from("/work/repo")),
        ClientFailure::BranchKeepsChanging { branch: String::from("main"), times: 5 },
        ClientFailure::BranchMoved { branch: String::from("main") },
        ClientFailure::BranchMovedMessageSaved { branch: String::from("main"), file: String::from("/work/repo/.git/cli-git-saved-message-ab12"), amend: false },
        ClientFailure::BranchMovedMessageSaved { branch: String::from("main"), file: String::from("/work/repo/.git/cli-git-saved-message-ab12"), amend: true },
        ClientFailure::BranchSwitched { branch: String::from("main"), now: String::from("side"), file: Some(String::from("/work/repo/.git/cli-git-saved-message-ab12")) },
        ClientFailure::BranchSwitched { branch: String::from("main"), now: String::from("side"), file: None },
        ClientFailure::ServiceSilent,
        ClientFailure::StepTooSlow,
        ClientFailure::StillWithService { minutes: 20, blocker: Some(String::from("a landing behind another program's index lock")) },
        ClientFailure::StillWithService { minutes: 20, blocker: None },
        ClientFailure::WithdrawalUnconfirmed { blocker: None },
        ClientFailure::WithdrawalUnconfirmed { blocker: Some(String::from("cli-git: the commit service has been inside a write for 21 minutes and has not heard back: `git update-ref --stdin` in /work/repo (it moves refs/heads/main to abc123; the commit is also at refs/worktree/cli-git-service/7 until the update ends).")) },
        ClientFailure::Refused,
        ClientFailure::IndexInUse(String::from("the editor (process 4242)")),
        ClientFailure::Unavailable,
        ClientFailure::ProgramMissing(String::from("/opt/cli-git/cli-git-service")),
        ClientFailure::CouldNotStart(String::from("the runtime directory has no space left")),
        ClientFailure::UnitInTheWay(String::from("/home/a/.config/systemd/user/cli-git.socket")),
        ClientFailure::WorkNotHandedOver(String::from("the work directory cannot be written")),
        ClientFailure::ContentPruned,
    ];
    for platform in [Platform::Windows, Platform::Bsd, Platform::LinuxWithoutUserServices, Platform::Other] {
        all.push(ClientFailure::NoActivation(platform));
    }
    return all;
}

/// Compile-time proof that `samples` covers every variant: adding one breaks this match until a sample exists.
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
        ClientFailure::Refused => 9,
        ClientFailure::IndexInUse(_) => 10,
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
        ClientFailure::StepTooSlow => 22,
        ClientFailure::StillWithService { .. } => 20,
        ClientFailure::WithdrawalUnconfirmed { .. } => 21,
    };
}

/// The text without the parts between backticks, which quote commands the user types.
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

/// Whether `word` (one or more words) occurs in `text` as whole words, ignoring case.
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

/// Every variant has a sample.
#[test]
fn every_failure_variant_is_rendered() {
    let covered: std::collections::BTreeSet<usize> = samples().iter().map(variant_index).collect();
    let expected: usize = variant_index(&ClientFailure::ContentPruned) + 1;
    assert_eq!(covered.len(), expected);
}

/// No failure text contains a term from the denylist, outside quoted commands.
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
}

/// The denylist check can fail: a text that leaks is caught, and a word inside another word is not.
#[test]
fn the_denylist_catches_a_leak_and_ignores_inner_words() {
    assert!(contains_whole("The daemon refused the request id.", "request id"));
    assert!(contains_whole("Socket closed (EPIPE).", "socket"));
    assert!(!contains_whole("Please avoid the voided path.", "oid"));
    assert!(!contains_whole("The rapid pipeline.", "pid"));
    assert!(contains_whole("process pid 42", "pid"));
}

/// Every failure tells the user something to do: a quoted command or one of the plain verbs that start a remedy.
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

/// The text derives from the named constants and the facts it names, so it cannot drift from them.
#[test]
fn messages_quote_their_constants_and_facts() {
    assert!(render(&ClientFailure::UnitInTheWay(String::from("/home/a/.config/systemd/user/cli-git.socket"))).contains("/home/a/.config/systemd/user/cli-git.socket"));
    assert!(render(&ClientFailure::DidNotStart).contains(LOG_COMMAND));
    assert!(render(&ClientFailure::Unavailable).contains(LOG_COMMAND));
    assert!(render(&ClientFailure::OtherRelease).contains(RESTART_COMMAND));
    let unsure: String = render(&ClientFailure::MayNotHaveLanded);
    for command in CHECK_COMMANDS {
        assert!(unsure.contains(command), "{unsure}");
    }
    assert!(render(&ClientFailure::NoActivation(Platform::LinuxWithoutUserServices)).contains(LINGER_COMMAND));
    let branch: String = render(&ClientFailure::BranchKeepsChanging { branch: String::from("topic"), times: 7 });
    assert!(branch.contains("topic") && branch.contains('7'), "{branch}");
    assert!(render(&ClientFailure::BranchMoved { branch: String::from("topic") }).contains("topic"));
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

/// Failures that changed nothing say so; the one that may have changed something does not claim otherwise.
#[test]
fn only_the_unsure_failure_does_not_claim_nothing_changed() {
    for failure in samples() {
        let claims: bool = render(&failure).contains(NOTHING_CHANGED);
        let may_have_landed: bool = matches!(failure, ClientFailure::MayNotHaveLanded | ClientFailure::ServiceSilent | ClientFailure::WithdrawalUnconfirmed { .. } | ClientFailure::WorkNotHandedOver(_));
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
