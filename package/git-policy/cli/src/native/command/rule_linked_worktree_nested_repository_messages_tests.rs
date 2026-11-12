//! What:
//!  The two sentences a saved nested repository leaves:
//!  the notice that says where it went,
//!  and the rejection that says it could not be moved.
//! Why:
//!  A person finds the repository only through these words,
//!  so each must name the repository,
//!  the place or the reason,
//!  and every way forward;
//!  one repository is said in the singular and several in the plural,
//!  because a sentence that reads as if one repository were lost when two were is a lie about the loss.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(nestedRepositoryNotSavedMessage({ repositories: [Buffer.from('nested')], ... })).toContain('The repository is still where it was.');
//! ```

use super::{NotSaved, nested_repository_not_saved_message, saved_nested_repository_notice};
use crate::conflict_sides::{SavedCopy, SaveFailure};
use crate::rule_worktree_dirty::SHARED_PATHS_NAMED;
use std::path::PathBuf;

/// One saved copy of the repository `nested` at `place`.
fn copy(place: &str) -> SavedCopy {
    return SavedCopy {
        path: String::from("nested"),
        saved_to: Some(PathBuf::from(place)),
    };
}

/// A failure that names the repository and the system's own words.
fn failure() -> SaveFailure {
    return SaveFailure {
        subject: String::from("nested"),
        reason: String::from("Permission denied (os error 13)"),
        kind: crate::conflict_sides::FailureKind::Io,
    };
}

/// The notice names the repository,
/// the place it holds now,
/// and the command that shows what it holds.
#[test]
fn the_notice_names_the_repository_and_where_it_went() {
    let notice: String = saved_nested_repository_notice(&[copy("/common/monochromatic-git-policy-cli/discarded/2026-11-11T03-00-00/nested")]);
    assert!(notice.contains("the Git repository nested"), "{notice}");
    assert!(notice.contains("/common/monochromatic-git-policy-cli/discarded/2026-11-11T03-00-00/nested"), "{notice}");
    assert!(
        notice.contains("`git -C /common/monochromatic-git-policy-cli/discarded/2026-11-11T03-00-00/nested status`"),
        "{notice}"
    );
    assert!(notice.contains("the rest of the clean runs as typed"), "{notice}");
}

/// A copy with no place is not one this notice can explain,
///  so it writes nothing rather than a place that does not exist.
#[test]
fn a_copy_with_no_place_is_not_announced() {
    let absent: SavedCopy = SavedCopy { path: String::from("nested"), saved_to: None };
    assert_eq!(saved_nested_repository_notice(&[absent]), "");
}

/// One repository is named in the singular,
///  with the reason,
/// the place the copies go and the deliberate way to delete it.
#[test]
fn the_rejection_names_one_repository_in_the_singular() {
    let subject: Vec<Vec<u8>> = vec![b"nested".to_vec()];
    let saved_root: PathBuf = PathBuf::from("/common/monochromatic-git-policy-cli/discarded");
    let message: String = nested_repository_not_saved_message(NotSaved {
        repositories: subject.as_slice(),
        failure: &failure(),
        saved_root: saved_root.as_path(),
        still_moved: &[],
    });
    for expected in [
        "this clean would delete the Git repository nested and everything in it",
        "nothing of it could be saved first",
        "Permission denied (os error 13)",
        "The repository is still where it was.",
        "/common/monochromatic-git-policy-cli/discarded",
        "--no-enforce-worktree",
    ] {
        assert!(message.contains(expected), "{expected} missing from {message}");
    }
}

/// Several repositories are named in the plural,
///  and only the first few are listed.
#[test]
fn the_rejection_names_many_repositories_and_counts_the_rest() {
    let many: Vec<Vec<u8>> = (0..SHARED_PATHS_NAMED + 2).map(|number| { return format!("repo{number}").into_bytes(); }).collect();
    let saved_root: PathBuf = PathBuf::from("/common/monochromatic-git-policy-cli/discarded");
    let message: String = nested_repository_not_saved_message(NotSaved {
        repositories: many.as_slice(),
        failure: &failure(),
        saved_root: saved_root.as_path(),
        still_moved: &[],
    });
    assert!(message.contains("the Git repositories repo0, repo1"), "{message}");
    assert!(message.contains("and 2 more"), "{message}");
    assert!(message.contains("everything in them"), "{message}");
    assert!(message.contains("nothing of them could be saved first"), "{message}");
    assert!(message.contains("The repositories are still where they were."), "{message}");
    assert!(!message.contains("repo7"), "{message}");
}

/// What:
///  A repository the stopped save could not put back is named with the place it is,
///  and the sentence no longer claims the worktree is untouched.
/// Why:
///   Every other sentence in this rejection would be false then,
///  and that place is the only thing that tells the person where their repository went.
#[test]
fn a_repository_that_could_not_be_put_back_is_named_where_it_is() {
    let subject: Vec<Vec<u8>> = vec![b"nested".to_vec(), b"vendor/lib".to_vec()];
    let saved_root: PathBuf = PathBuf::from("/common/.git/monochromatic-git-policy-cli/discarded");
    let stayed: Vec<SavedCopy> = vec![copy("/common/.git/monochromatic-git-policy-cli/discarded/2026-11-11T03-00-00/nested")];
    let message: String = nested_repository_not_saved_message(NotSaved {
        repositories: subject.as_slice(),
        failure: &failure(),
        saved_root: saved_root.as_path(),
        still_moved: stayed.as_slice(),
    });
    for expected in [
        "nested => /common/.git/monochromatic-git-policy-cli/discarded/2026-11-11T03-00-00/nested",
        "could not be put back where they were and are kept here:",
        "Every other repository is still where it was.",
    ] {
        assert!(message.contains(expected), "{expected} missing from {message}");
    }
    assert!(!message.contains("nothing was changed"), "{message}");
}
