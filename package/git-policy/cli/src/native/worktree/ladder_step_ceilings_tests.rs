//! Controls for the ceilings of the ladder's own Git steps:
//! which steps have one,
//! which number each gets,
//! and that the person's command gets none.

use super::*;

/// Words for the steps the ladder starts itself,
/// and the ones it never starts.
fn words(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// A tree difference and a dirty restore have a ceiling,
/// the fallback checkout has its own,
/// and nothing else gets one from here.
#[test]
fn the_class_of_the_step_decides_its_ceiling() {
    assert_eq!(ceiling_of(words(&["-C", "/w", "read-tree", "-m", "-u", "one", "two"]).as_slice()), Some(LADDER_READ_TREE_CEILING));
    assert_eq!(ceiling_of(words(&["-C", "/w", "read-tree", "--reset", "-u", "one"]).as_slice()), Some(LADDER_READ_TREE_CEILING));
    assert_eq!(ceiling_of(words(&["-C", "/w", "reset", "--hard", "--no-recurse-submodules"]).as_slice()), Some(LADDER_CHECKOUT_CEILING));
    // A `reset` that is not the fallback checkout, a `clean`, and a word list with no subcommand are not the ladder's steps.
    assert_eq!(ceiling_of(words(&["-C", "/w", "reset", "--mixed"]).as_slice()), None);
    assert_eq!(ceiling_of(words(&["-C", "/w", "clean", "--force", "-d"]).as_slice()), None);
    assert_eq!(ceiling_of(words(&[]).as_slice()), None);
}

/// The two ceilings are ordered as the work is:
/// a tree difference writes the wrapper's own stage and can be run again,
/// while a whole checkout writes every file of a worktree.
#[test]
fn a_tree_difference_is_bounded_below_a_whole_checkout() {
    assert!(LADDER_READ_TREE_CEILING < LADDER_CHECKOUT_CEILING, "{LADDER_READ_TREE_CEILING:?} is not below {LADDER_CHECKOUT_CEILING:?}");
    // A ceiling of ten minutes or more is a bound a person can wait out; anything under a minute would cut real work.
    assert!(LADDER_READ_TREE_CEILING >= std::time::Duration::from_secs(60), "{LADDER_READ_TREE_CEILING:?}");
}

/// A control shortens the ceiling of this thread only,
/// so a step ends in a moment instead of at ten or twenty minutes.
#[test]
fn a_control_can_shorten_the_ceiling_of_its_own_thread() {
    let shortened: Duration = Duration::from_millis(250);
    CEILING_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<Duration>>| slot.set(Some(shortened)));
    assert_eq!(ceiling_of(words(&["-C", "/w", "read-tree", "-m", "-u", "one", "two"]).as_slice()), Some(shortened));
    assert_eq!(ceiling_of(words(&["-C", "/w", "reset", "--hard"]).as_slice()), Some(shortened));
    // A step of another class still has no ceiling.
    assert_eq!(ceiling_of(words(&["-C", "/w", "clean", "--force", "-d"]).as_slice()), None);
    CEILING_FOR_THIS_THREAD.with(|slot: &std::cell::Cell<Option<Duration>>| slot.set(None));
    assert_eq!(ceiling_of(words(&["-C", "/w", "read-tree", "-m", "-u", "one", "two"]).as_slice()), Some(LADDER_READ_TREE_CEILING));
}
