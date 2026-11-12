//! Controls for the words the wrapper hands its own Git child,
//! and for the output it shows a person from what that child printed:
//! a fast-forward's own line is the merge's line,
//! the message the person typed travels with it,
//! and no output-hiding option is handed to the child.

use super::*;
use crate::merge_options::{FastForwardChoice, ParsedMerge, parse_merge_shape};

/// The command line of a merge with the words under test.
fn shape_of(words: &[&str]) -> MergeShape {
    let region: Vec<OsString> = words.iter().map(|word| { return OsString::from(*word); }).collect();
    let parsed = parse_merge_shape(region.as_slice());
    return match parsed {
        ParsedMerge::Supported(found) => found,
        ParsedMerge::Unsupported(reason) => panic!("{words:?} is read: {reason}"),
    };
}

/// The arguments as text.
fn shown(words: &[OsString]) -> Vec<String> {
    return words.iter().map(|word| { return word.to_string_lossy().into_owned(); }).collect();
}

/// What one fast-forward prints,
/// with its stat and the two lines above it.
fn printed() -> Vec<u8> {
    return b"Updating 1234567..89abcde\nFast-forward (no commit created; -m option ignored)\n a.txt | 1 +\n 1 file changed, 1 insertion(+)\n".to_vec();
}

/// What:
///  The message the person typed reaches the child,
///  and no option that only hides what Git prints does.
/// Why:
///   The message decides Git's line and reflog entry about a fast-forward,
///  while `-q`,
///  `--stat` and
///  `--no-stat` are not given to a run the wrapper starts (`README.md`,
/// "No `--quiet` on the wrapper's own Git runs").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theArguments({ shape, newOid: 'abc' })).toEqual(['merge', '--ff-only', '--no-edit', 'abc']);
/// ```
#[test]
fn the_message_reaches_the_child_and_no_hiding_option_does() {
    assert_eq!(shown(&the_arguments(&shape_of(&["--no-edit", "feature"]), "abc")), ["merge", "--ff-only", "--no-edit", "abc"]);
    assert_eq!(
        shown(&the_arguments(&shape_of(&["--no-edit", "-q", "feature"]), "abc")),
        ["merge", "--ff-only", "--no-edit", "abc"],
        "the quiet is the wrapper's own to keep quiet with"
    );
    assert_eq!(
        shown(&the_arguments(&shape_of(&["--no-edit", "--no-stat", "feature"]), "abc")),
        ["merge", "--ff-only", "--no-edit", "abc"]
    );
    assert_eq!(
        shown(&the_arguments(&shape_of(&["--no-edit", "--message=given by -m", "feature"]), "abc")),
        ["merge", "--ff-only", "--no-edit", "--message", "given by -m\n", "abc"]
    );
}

/// What:
///  What a person reads about a carried fast-forward:
///  nothing under `-q`,
///  the two lines that name the step when the stat is off,
///  and everything otherwise.
/// Why:
///  Those are exactly the shapes measured from Git 2.56.0's own fast-forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(theOutput({ shape, config, printed })).toBe('Updating 1234567..89abcde\nFast-forward (no commit created; -m option ignored)\n');
/// ```
#[test]
fn the_output_follows_the_persons_quiet_and_stat_words() {
    let stat: MergeConfig = MergeConfig { stat: true, ..MergeConfig::default() };
    let no_stat: MergeConfig = MergeConfig { stat: false, ..MergeConfig::default() };
    let whole: String = String::from_utf8(printed()).expect("text");
    let header: String = String::from("Updating 1234567..89abcde\nFast-forward (no commit created; -m option ignored)\n");
    assert_eq!(the_output(&shape_of(&["--no-edit", "feature"]), &stat, printed().as_slice()), whole);
    assert_eq!(the_output(&shape_of(&["--no-edit", "--stat", "feature"]), &no_stat, printed().as_slice()), whole);
    assert_eq!(the_output(&shape_of(&["--no-edit", "feature"]), &no_stat, printed().as_slice()), header);
    assert_eq!(the_output(&shape_of(&["--no-edit", "--no-stat", "feature"]), &stat, printed().as_slice()), header);
    assert_eq!(the_output(&shape_of(&["--no-edit", "-n", "feature"]), &stat, printed().as_slice()), header);
    assert_eq!(the_output(&shape_of(&["--no-edit", "-q", "feature"]), &stat, printed().as_slice()), "");
    // A Git that printed only the two lines says the same thing either way.
    assert_eq!(the_output(&shape_of(&["--no-edit", "feature"]), &stat, header.as_bytes()), header);
}

/// The merge-commit child is the step that applies a merge commit's tree,
/// and the commit is its last word.
#[test]
fn the_commit_is_the_last_word_of_the_merge_commit_child() {
    assert_eq!(shown(&the_merge_commit_arguments("fedcba9876543210")), ["merge", "--ff-only", "--no-edit", "fedcba9876543210"]);
}

/// `--ff-only` is read as the choice it is,
/// and the three fast-forward words tie-break in the order they are typed.
#[test]
fn the_fast_forward_choices_are_read_in_order() {
    assert_eq!(shape_of(&["--ff-only", "feature"]).fast_forward, FastForwardChoice::Only);
    assert_eq!(shape_of(&["--ff", "--no-ff", "feature"]).fast_forward, FastForwardChoice::Never);
    assert_eq!(shape_of(&["--no-ff", "--ff-only", "feature"]).fast_forward, FastForwardChoice::Only);
    assert_eq!(shape_of(&["--ff-only", "--ff", "feature"]).fast_forward, FastForwardChoice::Allowed);
}
