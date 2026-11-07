//! Controls for the classes of Git step:
//!  which subcommand lands in which class,
//! the bound each class gets from its default and from the variable,
//!  and the relations between the bounds and the graces that keep a healthy step from being announced or stopped.

use super::*;

/// Every class,
/// so a relation is checked for each.
const CLASSES: [ChildClass; 2] = [ChildClass::Plumbing, ChildClass::Object];

/// Subcommands that move bytes,
/// or compare the index with the worktree,
/// or talk to a remote:
///  inputs for the claim that anything not named a query is a bulk step.
const BULK_STEPS: [&str; 10] =
    ["pack-objects", "index-pack", "add", "hash-object", "maintenance", "reset", "update-index", "ls-files", "diff", "show"];

/// Request arguments from words.
fn arguments(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// Every named query subcommand is classed as plumbing,
/// whatever options precede it.
#[test]
fn query_subcommands_are_classed_as_plumbing_after_any_global_options() {
    for name in PLUMBING_SUBCOMMANDS {
        assert_eq!(class_of(&arguments(&[name, "--quiet"])), ChildClass::Plumbing, "{name}");
        assert_eq!(
            class_of(&arguments(&["--git-dir=/shadow", "-c", "core.hooksPath=/none", "-C", "/repo", name])),
            ChildClass::Plumbing,
            "{name} behind global options"
        );
    }
}

/// A step that moves bytes,
/// refreshes the index or reaches a remote gets the bound of bulk work,
/// whatever its options,
///  and so does maintenance,
/// which runs a repack when it is not detached.
#[test]
fn steps_that_move_bytes_or_compare_the_index_are_bulk_work() {
    for name in BULK_STEPS {
        assert_eq!(class_of(&arguments(&["-c", "core.fsync=none", name, "--quiet", "--detach"])), ChildClass::Object, "{name}");
    }
}

/// No step that refreshes the index with the worktree is a query:
///  the refresh hashes every stat-dirty file,
/// so such a step must not share the short bound.
#[test]
fn no_bulk_step_is_in_the_query_list() {
    for name in BULK_STEPS {
        assert!(!PLUMBING_SUBCOMMANDS.contains(&name), "{name}");
    }
}

/// A subcommand nobody classed,
/// and a request with no subcommand,
/// get the longer bound rather than an early stop.
#[test]
fn an_unknown_step_gets_the_longer_bound() {
    assert_eq!(class_of(&arguments(&["frobnicate", "--now"])), ChildClass::Object);
    assert_eq!(class_of(&arguments(&["--version"])), ChildClass::Object);
    assert_eq!(class_of(&[]), ChildClass::Object);
}

/// The variable sets the bound of every class to the same whole seconds.
#[test]
fn the_variable_sets_every_class_to_the_same_bound() {
    let seconds: u64 = OBJECT_QUIET_WINDOW.as_secs() + PLUMBING_QUIET_WINDOW.as_secs() + 1;
    let text: String = seconds.to_string();
    for class in CLASSES {
        assert_eq!(class.limit_from(Some(OsStr::new(text.as_str()))), Duration::from_secs(seconds), "{class:?}");
    }
}

/// A missing,
/// zero or malformed value falls back to the class's own default,
/// never to "no limit".
#[test]
fn an_unusable_value_falls_back_to_the_class_default() {
    for class in CLASSES {
        assert_eq!(class.limit_from(None), class.default_limit(), "{class:?}");
        for value in ["0", "", "-1", "1.5", "soon"] {
            assert_eq!(class.limit_from(Some(OsStr::new(value))), class.default_limit(), "{class:?} {value:?}");
        }
    }
}

/// A bulk step may run longer than a query step and is announced no sooner,
/// and no first notice comes after the step
///  would already have been stopped.
#[test]
fn the_object_class_is_the_more_patient_one_and_every_notice_precedes_its_bound() {
    assert!(OBJECT_QUIET_WINDOW > PLUMBING_QUIET_WINDOW);
    assert!(OBJECT_NOTICE_GRACE >= PLUMBING_NOTICE_GRACE);
    for class in CLASSES {
        assert!(class.notice_grace() < class.default_limit(), "{class:?}");
    }
}

/// The class names differ,
/// so a notice says which bound applies.
#[test]
fn the_classes_have_different_labels() {
    assert_ne!(ChildClass::Plumbing.label(), ChildClass::Object.label());
}
