//! Controls for what one layer of one path ends as:
//! the sides that agree or stand missing decide without Git,
//! and a path both sides changed is merged by the word pass or keeps both sides between markers.
//! Each control that needs objects runs real Git in a disposable repository.

use super::*;
use crate::merge_second_pass::ContextRunner;
use crate::test_support::{fixture, git_context, git_text, remove, repository};
use std::path::{Path, PathBuf};

/// A repository to keep objects in,
/// and the runner over it.
fn place(name: &str) -> (PathBuf, PathBuf, ContextRunner<'static>) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: &'static crate::transaction_git::GitContext = Box::leak(Box::new(git_context()));
    let cwd: &'static Path = Box::leak(repo.clone().into_boxed_path());
    let runner: ContextRunner<'static> = ContextRunner { context, cwd, without_prefix: false, repository: Vec::new(), object_directory: None, environment: Vec::new() };
    return (root, repo, runner);
}

/// The object of these bytes,
/// as the path records them.
fn blob(repo: &Path, path: &str, bytes: &[u8]) -> String {
    std::fs::write(repo.join(path), bytes).expect("the content");
    return git_text(repo, &["hash-object", "-w", "--", path]).trim().to_owned();
}

/// A side holding a plain file.
fn side(oid: &str) -> Option<Side> {
    return Some(Side { mode: String::from("100644"), oid: String::from(oid) });
}

/// What:
///  The sides that agree or stand missing are decided without reading an object.
/// Why:
///  A layer the person did not change is the merge's,
/// and one the merge did not change is theirs;
/// reading
///  nothing is what keeps a merge of a thousand paths cheap.
#[test]
fn agreeing_sides_need_no_object_read() {
    let base: Option<Side> = side(&"a".repeat(40));
    let theirs: Option<Side> = side(&"b".repeat(40));
    let layers: [(&str, Layer, Outcome, Option<&str>); 5] = [
        ("ours is the base", Layer { base: base.clone(), ours: base.clone(), theirs: theirs.clone() }, Outcome::Merges, Some("100644")),
        ("theirs is the base", Layer { base: base.clone(), ours: theirs.clone(), theirs: base.clone() }, Outcome::Ours, Some("100644")),
        ("both agree", Layer { base: base.clone(), ours: theirs.clone(), theirs: theirs.clone() }, Outcome::Ours, Some("100644")),
        ("the merge deletes it", Layer { base: base.clone(), ours: side(&"c".repeat(40)), theirs: None }, Outcome::Kept(MODIFY_DELETE), Some("100644")),
        ("the path held nothing of the person's", Layer { base: base.clone(), ours: None, theirs: theirs.clone() }, Outcome::Gone(DELETE_MODIFY), None),
    ];
    for (label, layer, expected, mode) in layers {
        let Some(decided): Option<Decision> = decide_without_the_pass(&layer) else { panic!("{label} is decided without Git") };
        assert_eq!(decided.outcome, expected, "{label}");
        assert_eq!(decided.mode.as_deref(), mode, "{label}");
    }
}

/// What:
///  A mode the person set at a path the merge deletes keeps their side whole,
/// where an entry equal to the base's
///  leaves the merger's deletion to stand.
/// Why:
///  The equality rules read a mode changed alone as no change at all,
/// so a deletion took the only part of the
///  person's entry there and the merge reported itself made
/// (owner's rule "never discard user data").
#[test]
fn a_mode_changed_alone_at_a_path_the_merge_deletes_is_kept() {
    let base: Option<Side> = Some(Side { mode: String::from("100644"), oid: "a".repeat(40) });
    let person: Option<Side> = Some(Side { mode: String::from("100755"), oid: "a".repeat(40) });
    let decided: Decision = decide_without_the_pass(&Layer { base: base.clone(), ours: person, theirs: None }).expect("a mode alone is decided without reading an object");
    assert_eq!(decided.outcome, Outcome::Kept(MODIFY_DELETE), "the person's side is kept whole");
    assert_eq!(decided.mode.as_deref(), Some("100755"), "the index and the file end with the mode they set");
    let untouched: Decision = decide_without_the_pass(&Layer { base: base.clone(), ours: base.clone(), theirs: None }).expect("an unchanged side is decided without reading an object");
    assert_eq!(untouched.outcome, Outcome::Merges, "nothing of theirs stands at a path whose entry is still the base's");
}

/// What:
///  A mode one side changed and the other left alone is the changed side's.
/// Why:
///  A mode is one bit,
/// so a person's `chmod` and the merge's are never both new;
/// taking the changed side is what
///  keeps an executable bit either side set.
#[test]
fn a_mode_one_side_changed_is_that_sides() {
    let base: Option<Side> = Some(Side { mode: String::from("100644"), oid: "a".repeat(40) });
    let person: Option<Side> = Some(Side { mode: String::from("100755"), oid: "a".repeat(40) });
    let decided: Decision = decide_without_the_pass(&Layer { base: base.clone(), ours: person, theirs: base.clone() }).expect("the person's chmod alone");
    assert_eq!(decided.mode.as_deref(), Some("100755"));
}

/// What:
///  A path both sides changed on different lines is merged,
/// and one they changed on the same line keeps both sides between markers,
/// as Git's own markers do.
/// Why:
///   The owner's answer for a merge over the person's edits is "Carry edits forward",
/// and a conflict the word
///  pass cannot settle keeps both sides in the file with the command exiting 0.
#[test]
fn a_text_both_sides_changed_is_merged_or_kept_between_markers() {
    let (root, repo, runner) = place("carry-layers-text");
    let base: String = blob(repo.as_path(), "a.txt", b"one\ntwo\nthree\n");
    let ours: String = blob(repo.as_path(), "b.txt", b"ONE\ntwo\nthree\n");
    let theirs: String = blob(repo.as_path(), "c.txt", b"one\ntwo\nTHREE\n");
    let clean: Decision = crate::lazy_runtime::run_on_runtime(decide(&runner, b"a.txt", &Layer { base: side(&base), ours: side(&ours), theirs: side(&theirs) })).expect("the runtime").expect("the word pass");
    assert_eq!(clean.outcome, Outcome::Merged { bytes: b"ONE\ntwo\nTHREE\n".to_vec(), conflict: false, parts: Vec::new() });
    let same_line: String = blob(repo.as_path(), "d.txt", b"one\nTWO-by-the-person\nthree\n");
    let other_line: String = blob(repo.as_path(), "e.txt", b"one\nTWO-by-the-merge\nthree\n");
    let conflicted: Decision = crate::lazy_runtime::run_on_runtime(decide(&runner, b"a.txt", &Layer { base: side(&base), ours: side(&same_line), theirs: side(&other_line) })).expect("the runtime").expect("the word pass");
    let Outcome::Merged { bytes, conflict, .. } = conflicted.outcome else { panic!("a conflict keeps both sides in one text") };
    assert!(conflict);
    let text: String = String::from_utf8_lossy(bytes.as_slice()).into_owned();
    assert!(text.contains("<<<<<<< your changes\n"), "{text}");
    assert!(text.contains("TWO-by-the-person"), "{text}");
    assert!(text.contains("=======\n"), "{text}");
    assert!(text.contains("TWO-by-the-merge"), "{text}");
    assert!(text.contains(">>>>>>> the merge\n"), "{text}");
    remove(root.as_path());
}

/// What:
///  A path both sides changed that is not text keeps the person's bytes and names the reason.
/// Why:
///   A binary file has no markers to hold two sides in,
/// and the merge's version is in the commit,
/// so the person's
///  own copy beside it loses nothing.
#[test]
fn a_binary_both_sides_changed_is_kept_and_named() {
    let (root, repo, runner) = place("carry-layers-binary");
    let base: String = blob(repo.as_path(), "b.bin", b"\x00\x01\x02base");
    let ours: String = blob(repo.as_path(), "c.bin", b"\x00\x01\x02the-person");
    let theirs: String = blob(repo.as_path(), "d.bin", b"\x00\x01\x02the-merge");
    let decided: Decision = crate::lazy_runtime::run_on_runtime(decide(&runner, b"b.bin", &Layer { base: side(&base), ours: side(&ours), theirs: side(&theirs) })).expect("the runtime").expect("the pass leaves it");
    assert_eq!(decided.outcome, Outcome::Kept(NOT_TEXT));
    assert_eq!(decided.mode.as_deref(), Some("100644"));
    remove(root.as_path());
}

/// A side holding a symbolic link,
/// whose object is its target's text.
fn link(oid: &str) -> Option<Side> {
    return Some(Side { mode: String::from("120000"), oid: String::from(oid) });
}

/// What:
///  A link on the merge's side and a regular file on the person's keep the person's file whole,
///  with the person's own mode.
/// Why:
///   The word pass reads bytes as text and writes what it makes with one mode;
/// with the merge's mode a person's
///  edited file was written as a link whose target was their bytes
/// (a cross-vendor review,
/// round 133,
/// its second finding),
/// so
///  the file they had was gone.
#[test]
fn a_merge_side_link_keeps_the_persons_regular_file_and_its_mode() {
    let (root, repo, runner) = place("carry-layers-merge-link");
    let base: String = blob(repo.as_path(), "a.txt", b"base\n");
    let ours: String = blob(repo.as_path(), "b.txt", b"the person edited a regular file\n");
    let theirs: String = blob(repo.as_path(), "c.txt", b"feature-target");
    let layer: Layer = Layer { base: side(&base), ours: side(&ours), theirs: link(&theirs) };
    let decided: Decision = crate::lazy_runtime::run_on_runtime(decide(&runner, b"a.txt", &layer)).expect("the runtime").expect("the layer is decided");
    assert_eq!(decided.outcome, Outcome::Kept(NOT_TEXT));
    assert_eq!(decided.mode.as_deref(), Some("100644"), "the person's regular file keeps its own mode");
    remove(root.as_path());
}

/// What:
///  A link whose target text equals the old file's bytes is a change of kind,
///  not an unchanged side.
/// Why:
///   The object of a link is its target's text,
/// so a comparison of object ids alone read the person's link as the
///  branch's own file and wrote the merge's blob with the link's mode:
/// the link then pointed at the merge's bytes
///  (a cross-vendor review,
/// round 133,
/// its third finding).
#[test]
fn a_link_whose_target_equals_the_old_bytes_keeps_the_persons_link() {
    let base: Option<Side> = side(&"a".repeat(40));
    let person: Option<Side> = link(&"a".repeat(40));
    let merge: Option<Side> = side(&"b".repeat(40));
    let decided: Decision =
        decide_without_the_pass(&Layer { base, ours: person, theirs: merge }).expect("a change of kind is decided without reading an object");
    assert_eq!(decided.outcome, Outcome::Kept(NOT_TEXT));
    assert_eq!(decided.mode.as_deref(), Some("120000"), "the link keeps pointing where the person pointed it");
}

/// What:
///  Two sides that each added a link keep the person's link whole.
/// Why:
///  A link's bytes are its target's text,
/// and a word-level merge of two targets wrote its markers into the link,
///  so the target became marker text and no longer identified anything
/// (a cross-vendor review,
/// round 133,
/// its fourth
///  finding).
#[test]
fn two_added_links_keep_the_persons_link() {
    let (root, repo, runner) = place("carry-layers-two-links");
    let ours: String = blob(repo.as_path(), "a.txt", b"ours-target");
    let theirs: String = blob(repo.as_path(), "b.txt", b"theirs-target");
    let layer: Layer = Layer { base: None, ours: link(&ours), theirs: link(&theirs) };
    let decided: Decision = crate::lazy_runtime::run_on_runtime(decide(&runner, b"a.txt", &layer)).expect("the runtime").expect("the layer is decided");
    assert_eq!(decided.outcome, Outcome::Kept(NOT_TEXT));
    assert_eq!(decided.mode.as_deref(), Some("120000"));
    let Outcome::Kept(_) = decided.outcome else { panic!("the person's link is kept, never merged as text") };
    remove(root.as_path());
}
