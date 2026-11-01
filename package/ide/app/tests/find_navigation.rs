//! Match choice relative to the reading selection:
//!  active index,
//!  next,
//!  previous,
//!  wrapping,
//!  and validity.

/// Production matches feed the navigation helpers,
///  so both are exercised together.
use ide_app::find::{FindMatches, MAX_FIND_MATCHES, find_matches};
/// The helpers under test are pure;
///  the native bar only applies their answers.
use ide_app::find_navigation::{
    FindResults, active, at_or_after, at_or_before, navigable_matches, paint_ranges,
    positioned_matches, status, visible,
};
/// Accepted results are tagged exactly like worker replies.
use ide_app::find_worker::FindIdentity;

/// Three matches at 0..2,
///  6..8,
///  and 12..14 with gaps between them.
fn matches() -> FindMatches {
    // What: `expect` extracts the successful matches or fails the test with this message.
    // Why: The fixture query is valid, so an error would be a defect in the matcher.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return findMatches('ab -- ab -- ab', 'ab', MAX_FIND_MATCHES);
    // ```
    return find_matches("ab -- ab -- ab", "ab", MAX_FIND_MATCHES).expect("fixture matches");
}

/// Only a selection covering exactly one match makes it active.
#[test]
fn active_match_is_the_exactly_selected_match() {
    let found = matches();
    // What: `&found.ranges` lends the shared list as a plain slice; `Some(1)` is a present index.
    // Why: Helpers never take ownership of accepted results.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(active(found.ranges, 6, 8)).toBe(1);
    // ```
    assert_eq!(active(&found.ranges, 6, 8), Some(1));
    assert_eq!(active(&found.ranges, 0, 2), Some(0));
    assert_eq!(active(&found.ranges, 12, 14), Some(2));
    assert_eq!(active(&found.ranges, 6, 7), None);
    assert_eq!(active(&found.ranges, 6, 9), None);
    assert_eq!(active(&found.ranges, 5, 8), None);
    assert_eq!(active(&found.ranges, 6, 6), None);
    assert_eq!(active(&found.ranges, 20, 22), None);
    assert_eq!(active(&[], 0, 0), None);
}

/// Enter moves forward from the selection end and wraps after the last match.
#[test]
fn next_match_advances_and_wraps_to_the_first() {
    let found = matches();
    assert_eq!(at_or_after(&found.ranges, 0), Some(0));
    assert_eq!(at_or_after(&found.ranges, 2), Some(1));
    assert_eq!(at_or_after(&found.ranges, 6), Some(1));
    assert_eq!(at_or_after(&found.ranges, 7), Some(2));
    assert_eq!(at_or_after(&found.ranges, 8), Some(2));
    assert_eq!(
        at_or_after(&found.ranges, 14),
        Some(0),
        "next after the last match must wrap to the first"
    );
    assert_eq!(at_or_after(&found.ranges, 99), Some(0));
    assert_eq!(at_or_after(&[], 0), None);
}

/// Shift+Enter moves backward from the selection start and wraps before the first match.
#[test]
fn previous_match_retreats_and_wraps_to_the_last() {
    let found = matches();
    assert_eq!(at_or_before(&found.ranges, 12), Some(1));
    assert_eq!(at_or_before(&found.ranges, 14), Some(2));
    assert_eq!(at_or_before(&found.ranges, 8), Some(1));
    assert_eq!(at_or_before(&found.ranges, 7), Some(0));
    assert_eq!(at_or_before(&found.ranges, 6), Some(0));
    assert_eq!(
        at_or_before(&found.ranges, 0),
        Some(2),
        "previous before the first match must wrap to the last"
    );
    assert_eq!(at_or_before(&found.ranges, 1), Some(2));
    assert_eq!(at_or_before(&[], 5), None);
}

/// A single match is its own next and previous match.
#[test]
fn single_match_wraps_to_itself() {
    let found = find_matches("only one needle", "needle", MAX_FIND_MATCHES).expect("one match");
    assert_eq!(at_or_after(&found.ranges, 15), Some(0));
    assert_eq!(at_or_before(&found.ranges, 9), Some(0));
}

/// Painting receives only matches intersecting the materialized character window.
#[test]
fn visible_window_selects_intersecting_matches_only() {
    let found = matches();
    assert_eq!(visible(&found.ranges, 0, 14).len(), 3);
    assert_eq!(visible(&found.ranges, 2, 6).len(), 0);
    assert_eq!(visible(&found.ranges, 1, 7).len(), 2);
    assert_eq!(visible(&found.ranges, 7, 12).len(), 1);
    assert_eq!(visible(&found.ranges, 7, 12)[0].start, 6);
    assert_eq!(visible(&found.ranges, 14, 99).len(), 0);
    assert_eq!(visible(&found.ranges, 9, 3).len(), 0);
    assert_eq!(visible(&[], 0, 10).len(), 0);
}

/// The count text distinguishes an active match,
///  no active match,
///  truncation,
///  and no match.
#[test]
fn status_text_names_active_total_truncation_and_no_match() {
    let found = matches();
    let second = status(&found, Some(1));
    assert_eq!(second.label, "2/3");
    assert_eq!(second.detail, "Match 2 of 3");
    assert!(!second.no_match);
    let unselected = status(&found, None);
    assert_eq!(unselected.label, "0/3");
    assert_eq!(unselected.detail, "3 matches, none selected");
    let bounded = find_matches("ab ab ab", "ab", 2).expect("bounded matches");
    assert_eq!(status(&bounded, Some(0)).label, "1/2+");
    assert_eq!(status(&bounded, Some(0)).detail, "Match 1 of 2 or more");
    assert_eq!(
        status(&bounded, None).detail,
        "2 or more matches, none selected"
    );
    let nothing = find_matches("ab", "zz", MAX_FIND_MATCHES).expect("no matches");
    let absent = status(&nothing, None);
    assert_eq!(absent.label, "No matches");
    assert_eq!(absent.detail, "No matches");
    assert!(absent.no_match);
}

/// The tag of the accepted fixture results:
///  file generation 3,
///  revision 5,
///  query 7.
fn accepted_identity() -> FindIdentity {
    return FindIdentity {
        file: 3,
        revision: 5,
        query: 7,
    };
}

/// Results painted for another file generation or revision would mark wrong positions.
#[test]
fn results_are_painted_only_for_their_file_generation_and_revision() {
    // What: `Some(...)` wraps present results; `None` would mean the bar has no accepted results.
    // Why: The native state stores accepted results as an optional value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const accepted: FindResults | undefined = new FindResults(identity, matches());
    // ```
    let accepted = Some(FindResults::new(accepted_identity(), matches()));
    assert_eq!(
        paint_ranges(&accepted, 3, 5).len(),
        3,
        "positive control: current results are painted"
    );
    assert_eq!(
        paint_ranges(&accepted, 4, 5).len(),
        0,
        "matches from another file generation were painted"
    );
    assert_eq!(
        paint_ranges(&accepted, 3, 6).len(),
        0,
        "matches from another content revision were painted"
    );
    assert!(positioned_matches(&accepted, 3, 5).is_some());
    assert!(positioned_matches(&accepted, 4, 5).is_none());
    assert!(positioned_matches(&accepted, 3, 6).is_none());
    assert_eq!(paint_ranges(&None, 3, 5).len(), 0);
    assert!(positioned_matches(&None, 3, 5).is_none());
}

/// Enter must not step through matches of another file,
///  revision,
///  or superseded find text.
#[test]
fn results_are_navigable_only_for_their_exact_identity() {
    let accepted = Some(FindResults::new(accepted_identity(), matches()));
    assert!(
        navigable_matches(&accepted, accepted_identity()).is_some(),
        "positive control: current results are navigable"
    );
    let mut other_file = accepted_identity();
    other_file.file = 4;
    assert!(
        navigable_matches(&accepted, other_file).is_none(),
        "matches from another file generation were navigable"
    );
    let mut other_revision = accepted_identity();
    other_revision.revision = 6;
    assert!(
        navigable_matches(&accepted, other_revision).is_none(),
        "matches from another content revision were navigable"
    );
    let mut other_query = accepted_identity();
    other_query.query = 8;
    assert!(
        navigable_matches(&accepted, other_query).is_none(),
        "matches from a superseded query were navigable"
    );
    assert!(navigable_matches(&None, accepted_identity()).is_none());
}
