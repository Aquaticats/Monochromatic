//! Controls for the words a wake's answer is reported with.

use super::*;

/// What:
///  Every answer but the service's own reading of the line is work that waits,
///  and each says what happened in words the person's report can carry.
/// Why:
///   The defect these words answer was a command that read the write's own return as "the service holds the record"
///  (the review of 2026-11-13,
///  "Out-of-scope observation"),
///  so the one answer a command may hand work over on is `Taken`,
///  and the four others are told apart by what stopped the line.
#[test]
fn only_the_service_reading_the_line_is_work_it_holds() {
    assert!(Woken::Taken.holds_the_work(), "the service closed the connection after the line, so it read it");
    for (woken, what) in [
        (Woken::NothingTaken, "no byte of the wake"),
        (Woken::PartTaken, "part of the wake"),
        (Woken::Buffered, "stands in the connection unread"),
        (Woken::Refused { code: FailureCode::Rejected }, "refused it"),
    ] {
        assert!(!woken.holds_the_work(), "{woken:?} is no answer that the service holds the record");
        let words: String = woken.why_nothing_took_it();
        assert!(words.contains(what), "the words for {woken:?} name what happened ({words})");
        assert!(!words.ends_with('.'), "the words stand as the reason of a report's own sentence ({words})");
    }
}

/// What:
///  A refusal names whether another instance serves the repository or the service could not tell whose it is,
///  and any other code is the service's own word.
/// Why:
///   The two refusals send a person to different places:
///  another instance means the wake reached a socket of another worktree,
///  and "unavailable" means the service could not decide,
///  which a later command may resolve by itself.
#[test]
fn a_refusal_says_which_of_the_two_it_was() {
    let another: String = Woken::Refused { code: FailureCode::Rejected }.why_nothing_took_it();
    assert!(another.contains("serves another instance"), "{another}");
    let undecided: String = Woken::Refused { code: FailureCode::Unavailable }.why_nothing_took_it();
    assert!(undecided.contains("could not tell whether this repository"), "{undecided}");
    let other: String = Woken::Refused { code: FailureCode::RepositoryFull }.why_nothing_took_it();
    assert!(other.contains(FailureCode::RepositoryFull.word()), "a code of its own is named as the service words it ({other})");
}

/// The answer the service's own reading of the line is,
/// so the words never claim a failure where there is none.
#[test]
fn the_answer_the_service_read_the_line_is_not_worded_as_a_failure() {
    let words: String = Woken::Taken.why_nothing_took_it();
    assert!(words.contains("read the wake"), "{words}");
    assert!(!words.contains("did not") && !words.contains("no byte"), "a wake the service read is not worded as one it did not ({words})");
}
