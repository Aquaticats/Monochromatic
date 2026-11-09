//! Controls for the destination rules that need no repository:
//! the refspec judgement against local ref names.

use super::*;
use crate::test_support::{fixture, git, remove};
use std::path::Path;

/// Names as the owned strings the judgement takes.
fn owned(names: &[&str]) -> Vec<String> {
    return names.iter().map(|name| { return (*name).to_owned(); }).collect();
}

/// The refs the branch `feature` is sent to under `refspecs`,
/// with the other local branches `main` and `other`.
fn judged(refspecs: &[&str]) -> Judged {
    let locals = owned(&["refs/heads/feature", "refs/heads/main", "refs/heads/other"]);
    return judge("refs/heads/feature", "origin", &owned(refspecs), &locals);
}

/// A refspec of a form that depends on more than local ref names is declined,
/// never guessed.
#[test]
fn a_refspec_form_that_needs_more_than_ref_names_is_not_interpreted() {
    for text in ["HEAD", "feature:review", ":", "+:", "refs/heads/feature:", "refs/heads/*:refs/heads/fixed", "refs/heads/a*b*:refs/heads/*", "refs/heads/x:review"] {
        assert_eq!(judged(&[text]), Judged::Unjudged(text.to_owned()), "{text}");
    }
}

/// A star on each side maps the captured text;
/// a pattern without a remote side keeps the name.
#[test]
fn a_pattern_maps_the_captured_text_to_the_remote_side() {
    assert_eq!(judged(&["refs/heads/*:refs/heads/mirror/*"]), Judged::Targets(owned(&["refs/heads/mirror/feature"])));
    assert_eq!(judged(&["+refs/heads/f*:refs/x/*"]), Judged::Targets(owned(&["refs/x/eature"])));
    assert_eq!(judged(&["refs/heads/*"]), Judged::Targets(owned(&["refs/heads/feature"])));
}

/// An explicit refspec takes its destination before any pattern,
/// whatever their order in the configuration.
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

/// A negative refspec removes the destinations it matches,
/// exactly or by pattern.
#[test]
fn a_negative_refspec_removes_matching_destinations() {
    assert_eq!(judged(&["refs/heads/*:refs/heads/*", "^refs/heads/feature"]), Judged::Targets(Vec::new()));
    assert_eq!(judged(&["refs/heads/*:refs/heads/*", "^refs/heads/f*"]), Judged::Targets(Vec::new()));
    assert_eq!(judged(&["refs/heads/*:refs/heads/*", "^refs/heads/other"]), Judged::Targets(owned(&["refs/heads/feature"])));
}

/// One source can go to several destinations,
/// as Git sends it.
#[test]
fn one_source_goes_to_each_of_its_destinations() {
    assert_eq!(judged(&["refs/heads/feature:refs/a", "refs/heads/feature:refs/b"]), Judged::Targets(owned(&["refs/a", "refs/b"])));
}

/// The coordinator's Git for a fixture repository,
/// with the environment the other push controls give it.
fn push_git(root: &Path) -> PushGit {
    let mut environment: Vec<(OsString, OsString)> = [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ]
    .iter()
    .map(|(name, value)| {
        return (OsString::from(name), OsString::from(value));
    })
    .collect();
    environment.extend(crate::test_support::scratch_environment());
    return PushGit { real_git: std::path::PathBuf::from("/usr/bin/git"), cwd: root.to_path_buf(), environment, lfs: false };
}

/// What:
///  The awaited faces of the destination read answer what the synchronous faces answer,
/// for each shape of the configuration.
/// Why:
///   The landing round's push names both sides of itself as a task,
/// and a difference between the faces would be a push of one branch to a place
///  a plain `git push` never goes.
#[test]
fn the_awaited_reads_answer_what_the_synchronous_reads_answer() {
    let root = fixture("destination-awaiting-same-answer");
    git(&root, &["init", "--quiet", "--initial-branch=main"]);
    git(&root, &["commit", "--quiet", "--allow-empty", "--message=base"]);
    git(&root, &["remote", "add", "origin", "https://example.invalid/repo.git"]);
    let context = push_git(&root);
    // Each shape is written over the configuration of the repository before it, so the file of a repository with
    //  nothing set is kept and put back.
    let nothing_set: Vec<u8> = std::fs::read(root.join(".git").join("config")).expect("the configuration of a new repository");
    let settings: [&[(&str, &str)]; 6] = [
        &[],
        &[("branch.main.remote", "origin"), ("branch.main.merge", "refs/heads/main")],
        &[("remote.origin.push", "refs/heads/main:refs/heads/other")],
        &[("remote.origin.mirror", "true")],
        &[("push.default", "matching")],
        &[("branch.main.remote", "origin"), ("branch.main.merge", "refs/heads/main"), ("branch.main.merge", "refs/heads/main2")],
    ];
    for configuration in settings {
        std::fs::write(root.join(".git").join("config"), &nothing_set).expect("a configuration with nothing set");
        for (key, value) in configuration {
            git(&root, &["config", key, value]);
        }
        let named = named_push(&context, "refs/heads/main");
        let (awaited, _ticks): (Result<NamedPush, SingleFlightError>, u32) =
            crate::ticker_beside_a_wait::with_a_ticker_beside(named_push_awaiting(&context, "refs/heads/main"));
        assert_eq!(format!("{awaited:?}"), format!("{named:?}"), "{configuration:?}");
        let destination = read_destination(&context, "refs/heads/main");
        let (awaited_destination, _destination_ticks): (Result<Option<Destination>, SingleFlightError>, u32) =
            crate::ticker_beside_a_wait::with_a_ticker_beside(read_destination_awaiting(&context, "refs/heads/main"));
        assert_eq!(format!("{awaited_destination:?}"), format!("{destination:?}"), "{configuration:?}");
    }
    remove(&root);
}
