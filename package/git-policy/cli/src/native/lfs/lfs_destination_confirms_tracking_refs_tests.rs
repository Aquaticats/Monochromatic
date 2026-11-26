//! Controls for the evidence a destination itself gives about this clone's tracking refs,
//! every answer it can give,
//!  and both faces of the plan that read it.

use super::*;
use crate::lfs_upload_settings::GitQueryAwaiting;

/// What a stubbed Git answers:
/// the answer by the first word of the question,
/// the questions that fail,
/// and how many were asked.
struct Stub<'a> {
    /// Answers by the question's first word.
    answers: &'a [(&'a str, &'a str)],
    /// Questions that fail.
    failing: &'a [&'a str],
    /// How many questions were asked.
    asked: usize,
}

impl Stub<'_> {
    /// The answer to one question,
    /// counted.
    fn answer(
        &mut self,
        arguments: &[&str],
    ) -> Result<String, String> {
        self.asked += 1;
        let key: &str = arguments[0];
        if self.failing.contains(&key) {
            return Err(format!("{key} failed"));
        }
        return Ok(self
            .answers
            .iter()
            .find(|(name, _)| {
                return *name == key;
            })
            .map_or(String::new(), |(_, answer)| {
                return (*answer).to_owned();
            }));
    }
}

impl GitQueryAwaiting for Stub<'_> {
    fn ask<'a>(
        &'a mut self,
        arguments: &'a [&'a str],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send + 'a>>
    {
        return Box::pin(async move {
            return self.answer(arguments);
        });
    }
}

/// The commit this clone believes `origin/main` holds,
/// in the listing a `for-each-ref` gives.
const HELD_MAIN: &str = "1111111111111111111111111111111111111111\t\trefs/remotes/origin/main";
/// A symbolic tracking ref,
/// which holds no belief about the destination.
const HELD_HEAD: &str =
    "1111111111111111111111111111111111111111\trefs/remotes/origin/main\trefs/remotes/origin/HEAD";
/// The same branch and commit,
/// as the destination reports them.
const SAME_MAIN: &str = "1111111111111111111111111111111111111111\trefs/heads/main";
/// The branch at another commit,
/// as a destination that moved it on reports it.
const MOVED_MAIN: &str = "2222222222222222222222222222222222222222\trefs/heads/main";
/// Another branch,
/// as a destination that no longer has this one reports its branches.
const GONE_MAIN: &str = "2222222222222222222222222222222222222222\trefs/heads/other";
/// This clone's tracking ref for another branch,
/// at the commit it held before a teammate moved that branch on.
const STALE_OTHER: &str = "5555555555555555555555555555555555555555\t\trefs/remotes/origin/other";
/// That branch as the destination reports it,
/// five commits further on.
const MOVED_OTHER: &str = "6666666666666666666666666666666666666666\trefs/heads/other";

/// What:
///  The evidence one listing and one answer support.
/// Why:
///   The expectation is written as the refs that may be stopped at and whether every ref was confirmed,
/// so a case
///  says what it expects instead of repeating the struct.
fn evidence(
    confirmed: &[&str],
    every: bool,
) -> ConfirmedTrackingRefs {
    return ConfirmedTrackingRefs {
        confirmed: confirmed
            .iter()
            .map(|name| {
                return (*name).to_owned();
            })
            .collect(),
        every,
    };
}

/// What:
///  The answers of both faces of the decision,
/// and how many questions each asked.
/// Why:
///   The synchronous push and the service's landing round must trust the same refs,
/// so every case is asked of
///  both here.
fn both_faces(
    answers: &[(&str, &str)],
    failing: &[&str],
) -> (
    (ConfirmedTrackingRefs, usize),
    (ConfirmedTrackingRefs, usize),
) {
    let mut stub: Stub<'_> = Stub {
        answers,
        failing,
        asked: 0,
    };
    let planned: ConfirmedTrackingRefs = {
        let mut synchronous = |arguments: &[&str]| -> Result<String, String> {
            return stub.answer(arguments);
        };
        tracking_refs_confirmed_by("origin", None, &mut synchronous)
    };
    let synchronous_asked: usize = stub.asked;
    let mut awaited_stub: Stub<'_> = Stub {
        answers,
        failing,
        asked: 0,
    };
    let (awaited, _ticks): (ConfirmedTrackingRefs, u32) =
        crate::ticker_beside_a_wait::with_a_ticker_beside(tracking_refs_confirmed_by_awaiting(
            "origin",
            None,
            &mut awaited_stub,
        ));
    return ((planned, synchronous_asked), (awaited, awaited_stub.asked));
}

/// What:
///  One answer,
/// what both faces decide from it,
/// and how many questions they ask for it.
/// Why:
///   One helper keeps the cases of the decision beside each other,
/// so a new answer cannot be left
///  unchecked on one face.
fn check(
    answers: &[(&str, &str)],
    failing: &[&str],
    expected: ConfirmedTrackingRefs,
    asked: usize,
) {
    let (synchronous, awaited) = both_faces(answers, failing);
    assert_eq!(synchronous, (expected, asked), "{answers:?} {failing:?}");
    assert_eq!(awaited, synchronous, "{answers:?} {failing:?}");
}

/// A listing keeps the refs that hold a belief of their own,
/// and stops at the symbolic ref and at another
///  remote's ref.
#[test]
fn a_listing_keeps_the_tracking_refs_that_hold_a_belief_of_their_own() {
    let listing: String = [
        HELD_MAIN,
        HELD_HEAD,
        "4444444444444444444444444444444444444444\t\trefs/remotes/elsewhere/main",
    ]
    .join("\n");
    let tracked: Vec<TrackingRef> = tracking_refs(listing.as_str(), "origin").expect("a listing");
    assert_eq!(tracked.len(), 1);
    assert_eq!(
        (tracked[0].branch.as_str(), tracked[0].oid.as_str()),
        ("main", "1111111111111111111111111111111111111111")
    );
    assert!(
        tracking_refs("", "origin")
            .expect("an empty listing")
            .is_empty()
    );
}

/// What:
///  A listing in a shape the format did not ask for is refused,
/// never read as a clone with no tracking refs.
/// Why:
///   Reading it as empty would trust every ref the clone holds,
/// which is the one decision this file exists to
///  withhold.
#[test]
fn a_listing_in_another_shape_is_no_listing() {
    for listing in [
        "refs/remotes/origin/main",
        "1111111111111111111111111111111111111111 refs/remotes/origin/main",
        // What a caller that trims the answer's ends would leave of a line whose last field was empty.
        "1111111111111111111111111111111111111111\trefs/remotes/origin/main",
        "only-one-field",
    ] {
        assert!(tracking_refs(listing, "origin").is_none(), "{listing:?}");
    }
}

/// The destination's answer reads as pairs,
/// and a listing that is not one is no answer at all.
#[test]
fn the_destination_answer_reads_as_pairs_and_an_unreadable_one_is_no_answer() {
    let heads: HashMap<String, String> = destination_heads(
        [
            SAME_MAIN,
            "2222222222222222222222222222222222222222\trefs/heads/side",
        ]
        .join("\n")
        .as_str(),
    )
    .expect("two heads");
    assert_eq!(heads.len(), 2);
    assert_eq!(
        heads.get("refs/heads/main").map(String::as_str),
        Some("1111111111111111111111111111111111111111")
    );
    assert_eq!(
        heads.get("refs/heads/side").map(String::as_str),
        Some("2222222222222222222222222222222222222222")
    );
    assert!(
        destination_heads("").expect("an empty answer").is_empty(),
        "an empty destination has no branches"
    );
    assert!(destination_heads("fatal: not a repository").is_none());
}

/// A tracking ref is confirmed only at the commit it holds,
/// for the branch it came from.
#[test]
fn a_tracking_ref_is_confirmed_only_at_the_commit_it_holds() {
    let tracked: Vec<TrackingRef> = tracking_refs(HELD_MAIN, "origin").expect("a listing");
    let heads = |reported: &str| -> HashMap<String, String> {
        return destination_heads(reported).expect("a listing");
    };
    assert_eq!(
        confirmed_tracking_refs(tracked.as_slice(), &heads(SAME_MAIN), "origin"),
        vec![String::from("refs/remotes/origin/main")]
    );
    assert!(confirmed_tracking_refs(tracked.as_slice(), &heads(MOVED_MAIN), "origin").is_empty());
    assert!(confirmed_tracking_refs(tracked.as_slice(), &heads(GONE_MAIN), "origin").is_empty());
}

/// What:
///  One tracking ref the destination moved on drops itself and nothing else.
/// Why:
///   A clone routinely holds a tracking ref a teammate moved on or a merged branch left behind;
/// an
///  all-or-nothing rule made every push read the whole history because of one such ref
/// (`keep/stale-tracking-ref-cost.txt` of the integration pass measured 21 batch requests and 2052
///  objects named against main's one and one).
#[test]
fn a_stale_tracking_ref_drops_only_itself() {
    let tracked: Vec<TrackingRef> =
        tracking_refs([HELD_MAIN, STALE_OTHER].join("\n").as_str(), "origin").expect("a listing");
    let heads: HashMap<String, String> =
        destination_heads([SAME_MAIN, MOVED_OTHER].join("\n").as_str()).expect("a listing");
    let confirmed: Vec<String> = confirmed_tracking_refs(tracked.as_slice(), &heads, "origin");
    assert_eq!(
        confirmed,
        vec![String::from("refs/remotes/origin/main")],
        "only the ref the destination reports at its commit is kept"
    );
    let held: Option<HashMap<String, String>> = Some(heads);
    assert_eq!(
        evidence_holds(tracked.as_slice(), held, "origin"),
        evidence(&["refs/remotes/origin/main"], false)
    );
}

/// What:
///  Every answer the destination and the local listing can give,
/// and what both faces decide from it.
/// Why:
///   The decision is the whole guard of the upload:
/// a confirmed ref is a skip,
/// and every other answer sends
///  the objects.
#[test]
fn both_faces_read_every_answer_alike() {
    let main_confirmed: ConfirmedTrackingRefs = evidence(&["refs/remotes/origin/main"], true);
    let nothing: ConfirmedTrackingRefs = ConfirmedTrackingRefs::unknown();
    let nothing_to_skip: ConfirmedTrackingRefs = evidence(&[], true);
    check(
        &[("for-each-ref", HELD_MAIN), ("ls-remote", SAME_MAIN)],
        &[],
        main_confirmed.clone(),
        2,
    );
    check(
        &[("for-each-ref", HELD_MAIN), ("ls-remote", MOVED_MAIN)],
        &[],
        nothing.clone(),
        2,
    );
    check(
        &[("for-each-ref", HELD_MAIN), ("ls-remote", GONE_MAIN)],
        &[],
        nothing.clone(),
        2,
    );
    check(
        &[("for-each-ref", HELD_MAIN), ("ls-remote", "not a listing")],
        &[],
        nothing.clone(),
        2,
    );
    check(
        &[("for-each-ref", HELD_MAIN)],
        &["ls-remote"],
        nothing.clone(),
        2,
    );
    check(&[], &["for-each-ref"], nothing.clone(), 1);
    check(
        &[("for-each-ref", "refs/remotes/origin/main")],
        &[],
        nothing.clone(),
        1,
    );
    check(
        &[(
            "for-each-ref",
            "1111111111111111111111111111111111111111 refs/remotes/origin/main",
        )],
        &[],
        nothing.clone(),
        1,
    );
    check(
        &[("for-each-ref", HELD_HEAD)],
        &[],
        nothing_to_skip.clone(),
        1,
    );
    check(&[("for-each-ref", "")], &[], nothing_to_skip, 1);
}
