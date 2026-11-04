//! Controls for the destination rules that need no repository: the refspec judgement against local ref names.

use super::*;

/// Names as the owned strings the judgement takes.
fn owned(names: &[&str]) -> Vec<String> {
    return names.iter().map(|name| { return (*name).to_owned(); }).collect();
}

/// The refs the branch `feature` is sent to under `refspecs`, with the other local branches `main` and `other`.
fn judged(refspecs: &[&str]) -> Judged {
    let locals = owned(&["refs/heads/feature", "refs/heads/main", "refs/heads/other"]);
    return judge("refs/heads/feature", "origin", &owned(refspecs), &locals);
}

/// A refspec of a form that depends on more than local ref names is declined, never guessed.
#[test]
fn a_refspec_form_that_needs_more_than_ref_names_is_not_interpreted() {
    for text in ["HEAD", "feature:review", ":", "+:", "refs/heads/feature:", "refs/heads/*:refs/heads/fixed", "refs/heads/a*b*:refs/heads/*", "refs/heads/x:review"] {
        assert_eq!(judged(&[text]), Judged::Unjudged(text.to_owned()), "{text}");
    }
}

/// A star on each side maps the captured text; a pattern without a remote side keeps the name.
#[test]
fn a_pattern_maps_the_captured_text_to_the_remote_side() {
    assert_eq!(judged(&["refs/heads/*:refs/heads/mirror/*"]), Judged::Targets(owned(&["refs/heads/mirror/feature"])));
    assert_eq!(judged(&["+refs/heads/f*:refs/x/*"]), Judged::Targets(owned(&["refs/x/eature"])));
    assert_eq!(judged(&["refs/heads/*"]), Judged::Targets(owned(&["refs/heads/feature"])));
}

/// An explicit refspec takes its destination before any pattern, whatever their order in the configuration.
#[test]
fn an_explicit_refspec_beats_a_pattern_for_a_destination() {
    for refspecs in [["refs/heads/*:refs/heads/*", "refs/heads/other:refs/heads/feature"], ["refs/heads/other:refs/heads/feature", "refs/heads/*:refs/heads/*"]] {
        assert_eq!(judged(&refspecs), Judged::Targets(Vec::new()), "{refspecs:?}");
    }
}

/// Two explicit sources for one destination and a source that is no ref refuse the whole push.
#[test]
fn explicit_refspecs_that_collide_or_name_a_missing_ref_refuse_the_push() {
    assert!(matches!(judged(&["refs/heads/feature:refs/heads/r", "refs/heads/other:refs/heads/r"]), Judged::Refused(reason) if reason.contains("refs/heads/r")));
    assert!(matches!(judged(&["refs/heads/feature", "refs/heads/gone"]), Judged::Refused(reason) if reason.contains("refs/heads/gone")));
}

/// A negative refspec removes the destinations it matches, exactly or by pattern.
#[test]
fn a_negative_refspec_removes_matching_destinations() {
    assert_eq!(judged(&["refs/heads/*:refs/heads/*", "^refs/heads/feature"]), Judged::Targets(Vec::new()));
    assert_eq!(judged(&["refs/heads/*:refs/heads/*", "^refs/heads/f*"]), Judged::Targets(Vec::new()));
    assert_eq!(judged(&["refs/heads/*:refs/heads/*", "^refs/heads/other"]), Judged::Targets(owned(&["refs/heads/feature"])));
}

/// One source can go to several destinations, as Git sends it.
#[test]
fn one_source_goes_to_each_of_its_destinations() {
    assert_eq!(judged(&["refs/heads/feature:refs/a", "refs/heads/feature:refs/b"]), Judged::Targets(owned(&["refs/a", "refs/b"])));
}
