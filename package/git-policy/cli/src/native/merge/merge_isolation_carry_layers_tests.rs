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
    let runner: ContextRunner<'static> = ContextRunner { context, cwd, without_prefix: false, object_directory: None, environment: Vec::new() };
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
        ("the person deleted it", Layer { base: base.clone(), ours: None, theirs: theirs.clone() }, Outcome::Gone(DELETE_MODIFY), None),
    ];
    for (label, layer, expected, mode) in layers {
        let Some(decided): Option<Decision> = decide_without_the_pass(&layer) else { panic!("{label} is decided without Git") };
        assert_eq!(decided.outcome, expected, "{label}");
        assert_eq!(decided.mode.as_deref(), mode, "{label}");
    }
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
    let clean: Decision = decide(&runner, b"a.txt", &Layer { base: side(&base), ours: side(&ours), theirs: side(&theirs) }).expect("the word pass");
    assert_eq!(clean.outcome, Outcome::Merged { bytes: b"ONE\ntwo\nTHREE\n".to_vec(), conflict: false, parts: Vec::new() });
    let same_line: String = blob(repo.as_path(), "d.txt", b"one\nTWO-by-the-person\nthree\n");
    let other_line: String = blob(repo.as_path(), "e.txt", b"one\nTWO-by-the-merge\nthree\n");
    let conflicted: Decision = decide(&runner, b"a.txt", &Layer { base: side(&base), ours: side(&same_line), theirs: side(&other_line) }).expect("the word pass");
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
    let decided: Decision = decide(&runner, b"b.bin", &Layer { base: side(&base), ours: side(&ours), theirs: side(&theirs) }).expect("the pass leaves it");
    assert_eq!(decided.outcome, Outcome::Kept(NOT_TEXT));
    assert_eq!(decided.mode.as_deref(), Some("100644"));
    remove(root.as_path());
}
