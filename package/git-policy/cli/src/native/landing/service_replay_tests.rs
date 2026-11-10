//! Controls for the retry of a commit the service sent back:
//! the merge of the captured change onto the branch with real
//!  Git (clean,
//! conflicting,
//! and the captured file that already holds the other commit's edit),
//! and the words a conflict
//!  ends the command with.

use super::*;
use crate::policy_events::PolicyEvent;
use crate::service_messages::DENYLIST;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

const FILE: &str = "f.txt";

/// The file with the lines named replaced.
fn lines_with(edits: &[(usize, &str)]) -> String {
    return (1..=9)
        .map(|number| {
            let line: String = edits.iter().find(|(at, _)| { return *at == number; }).map_or_else(|| { return format!("line {number}"); }, |(_, edit)| { return (*edit).to_string(); });
            return line + "\n";
        })
        .collect();
}

struct World {
    root: PathBuf,
    repo: PathBuf,
    context: GitContext,
    base: String,
}

/// A repository whose first commit holds the file,
/// with the branch left on it.
fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(FILE), lines_with(&[])).expect("file");
    git(repo.as_path(), &["add", FILE]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::create_dir(root.join("scratch")).expect("scratch");
    return World { root, repo, context: git_context(), base };
}

/// Commits `edits` of the file on the current branch with `message`.
fn land(world: &World, edits: &[(usize, &str)], message: &str) {
    std::fs::write(world.repo.join(FILE), lines_with(edits)).expect("edit");
    git(world.repo.as_path(), &["commit", "--quiet", "--all", format!("--message={message}").as_str()]);
}

/// The tree of a commit that applies `edits` to the base,
/// made on a side branch the way a captured change is.
fn captured(world: &World, edits: &[(usize, &str)]) -> Captured {
    git(world.repo.as_path(), &["checkout", "--quiet", "-b", "mine", world.base.as_str()]);
    land(world, edits, "mine");
    let tree: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "main"]);
    return Captured { base: world.base.clone(), unborn: false, tree, captured: 0, landed: Vec::new(), conclusion: None };
}

fn merge(world: &World, captured: &Captured) -> Merged {
    let tip: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let common: PathBuf = world.repo.join(".git");
    let scratch: PathBuf = world.root.join("scratch");
    let site: MergeSite<'_> =
        MergeSite { context: &world.context, cwd: world.repo.as_path(), common_dir: common.as_path(), object_directory: &common.join("objects"), scratch: scratch.as_path() };
    return merge_captured(&site, captured, tip.as_str(), 1).expect("the merge runs");
}

/// The commit at the tip,
/// as the service would remember it with the stamp of its capture.
fn tip_captured(world: &World, captured: u64) -> LandedCapture {
    return LandedCapture { commit: git_text(world.repo.as_path(), &["rev-parse", "HEAD"]), captured };
}

fn blob_of(world: &World, tree: &str) -> String {
    return git_text(world.repo.as_path(), &["show", format!("{tree}:{FILE}").as_str()]) + "\n";
}

/// Edits of different lines merge:
/// the tree holds the landed edit and the captured one.
#[test]
fn edits_of_different_lines_merge_into_one_tree() {
    let world: World = world("replay-clean");
    let mine: Captured = captured(&world, &[(7, "mine")]);
    land(&world, &[(1, "theirs")], "theirs");
    let Merged::Tree(tree) = merge(&world, &mine) else { panic!("expected a clean merge") };
    assert_eq!(blob_of(&world, tree.as_str()), lines_with(&[(1, "theirs"), (7, "mine")]));
    remove(world.root.as_path());
}

/// The same line changed differently cannot merge:
/// the path is named,
/// and so is the commit that landed first.
#[test]
fn edits_of_the_same_line_conflict_and_the_first_commit_is_named() {
    let world: World = world("replay-conflict");
    let mine: Captured = captured(&world, &[(1, "mine")]);
    land(&world, &[(1, "theirs")], "theirs");
    land(&world, &[(1, "theirs"), (9, "later")], "later");
    let Merged::Conflict(paths) = merge(&world, &mine) else { panic!("expected a conflict") };
    assert_eq!(paths, vec![FILE.as_bytes().to_vec()]);
    let tip: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let first = first_landed(&world.context, world.repo.as_path(), Some(world.base.as_str()), tip.as_str(), paths.as_slice()).expect("the log runs").expect("a commit");
    assert_eq!(first.subject, "theirs", "the commit that landed first, not the last");
    assert_eq!(first.short, git_text(world.repo.as_path(), &["rev-parse", "--short", "HEAD~1"]));
    remove(world.root.as_path());
}

//region names that alias under one encoding
// What: a name whose bytes are read as Latin-1 and written as UTF-8 spells a different real name.
// Why: the synthetic merge base is an index:
//  its records are bytes, so each name must reach Git as the bytes it came from.

/// A name with a two-byte UTF-8 character.
const NON_ASCII_FILE: &str = "é.txt";

/// The real,
/// distinct name that `NON_ASCII_FILE`'s bytes spell when they are read as Latin-1 and written as UTF-8.
const ALIASING_FILE: &str = "Ã©.txt";

/// The exact bytes of one file of a tree.
fn file_bytes_of(world: &World, tree: &str, name: &str) -> Vec<u8> {
    return git(world.repo.as_path(), &["show", format!("{tree}:{name}").as_str()]).stdout;
}

/// The mode and type a tree records for one name,
/// nothing when the tree holds no such name.
fn recorded_mode(world: &World, tree: &str, name: &str) -> Option<Vec<u8>> {
    let listing: Vec<u8> = git(world.repo.as_path(), &["ls-tree", "-z", tree]).stdout;
    for record in listing.split(|byte| { return *byte == 0; }) {
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else { continue };
        if record[tab + 1..] == *name.as_bytes() {
            return Some(record[..tab].to_vec());
        }
    }
    return None;
}

/// A conflict on one name keeps the landed edit of a different name whose bytes that name spells:
/// the synthetic base holds each name as its own bytes,
/// so an edit meant for one file never lands on another.
#[test]
fn a_conflict_on_one_name_keeps_the_other_names_landed_edit() {
    let root: PathBuf = fixture("replay-non-ascii-names");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let base_text: &str = "a\nb\nc\nd\n";
    let landed_text: &str = "a\nB\nc\nd\n";
    let prepared_text: &str = "a\nB\nC\nd\n";
    std::fs::write(repo.join(NON_ASCII_FILE), base_text).expect("base file");
    std::fs::write(repo.join(ALIASING_FILE), "unrelated base\n").expect("base file");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join(NON_ASCII_FILE), landed_text).expect("landed file");
    std::fs::write(repo.join(ALIASING_FILE), landed_text).expect("landed file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=landed"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "mine", base.as_str()]);
    std::fs::write(repo.join(NON_ASCII_FILE), prepared_text).expect("prepared file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=mine"]);
    let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::create_dir(root.join("scratch")).expect("scratch");
    let world: World = World { root, repo, context: git_context(), base };
    let mine: Captured = Captured { base: world.base.clone(), unborn: false, tree, captured: 0, landed: Vec::new(), conclusion: None };
    let merged_tree: String = match merge(&world, &mine) {
        Merged::Tree(merged) | Merged::Combined { tree: merged, .. } => merged,
        Merged::Conflict(paths) => panic!("the captured bytes contain the landed edit of the conflicting name: {paths:?}"),
    };
    assert_eq!(file_bytes_of(&world, merged_tree.as_str(), NON_ASCII_FILE), prepared_text.as_bytes(), "the captured bytes land");
    assert_eq!(file_bytes_of(&world, merged_tree.as_str(), ALIASING_FILE), landed_text.as_bytes(), "the other name keeps its landed edit");
    for name in [NON_ASCII_FILE, ALIASING_FILE] {
        let recorded: Vec<u8> = recorded_mode(&world, merged_tree.as_str(), name).unwrap_or_else(|| { panic!("{name} is missing from the merged tree") });
        let mode: String = String::from_utf8_lossy(recorded.as_slice()).into_owned();
        assert!(mode.starts_with("100644 blob "), "{name}: {mode}");
    }
    remove(world.root.as_path());
}

//endregion

/// A captured file that already holds the landed edit and one more on the next line lands as captured:
/// Git alone calls the
///  adjacent hunks a conflict,
/// and the worktree both commands share is why the captured bytes contain the landed edit.
#[test]
fn a_captured_file_that_holds_the_landed_edit_and_an_adjacent_one_lands_as_captured() {
    let world: World = world("replay-adjacent");
    let mine: Captured = captured(&world, &[(4, "theirs"), (5, "mine")]);
    land(&world, &[(4, "theirs")], "theirs");
    let Merged::Tree(tree) = merge(&world, &mine) else { panic!("expected the captured bytes to land") };
    assert_eq!(blob_of(&world, tree.as_str()), lines_with(&[(4, "theirs"), (5, "mine")]));
    remove(world.root.as_path());
}

/// The later capture of one worktree lands its bytes over an overlapping landed edit it already holds:
/// the commit that landed
///  was captured earlier,
/// so the person's file held its edit when this one was captured.
#[test]
fn a_later_capture_lands_its_bytes_over_an_overlapping_earlier_one() {
    let world: World = world("replay-later");
    let mine: Captured = captured(&world, &[(1, "mine")]);
    land(&world, &[(1, "theirs")], "theirs");
    let with_order: Captured = Captured { captured: 200, landed: vec![tip_captured(&world, 100)], ..mine };
    let Merged::Tree(tree) = merge(&world, &with_order) else { panic!("expected the later capture to land") };
    assert_eq!(blob_of(&world, tree.as_str()), lines_with(&[(1, "mine")]));
    remove(world.root.as_path());
}

/// The earlier capture of one worktree keeps the landed bytes of a later capture,
/// which hold its edit.
#[test]
fn an_earlier_capture_keeps_the_bytes_of_an_overlapping_later_one() {
    let world: World = world("replay-earlier");
    let mine: Captured = captured(&world, &[(1, "mine")]);
    land(&world, &[(1, "theirs")], "theirs");
    let with_order: Captured = Captured { captured: 100, landed: vec![tip_captured(&world, 200)], ..mine };
    let Merged::Tree(tree) = merge(&world, &with_order) else { panic!("expected the landed bytes to stay") };
    assert_eq!(blob_of(&world, tree.as_str()), lines_with(&[(1, "theirs")]));
    remove(world.root.as_path());
}

/// The replay reads the order of a shared path with a top-anchored literal spec the wrapper wrote,
/// so two glob modes that disagree in the caller's environment must not end the replay as a failure.
#[test]
fn a_replay_reads_its_path_order_under_the_callers_conflicting_glob_modes() {
    let mut world: World = world("replay-glob-modes");
    world.context.overlay.push((std::ffi::OsString::from("GIT_GLOB_PATHSPECS"), std::ffi::OsString::from("1")));
    world.context.overlay.push((std::ffi::OsString::from("GIT_NOGLOB_PATHSPECS"), std::ffi::OsString::from("1")));
    let mine: Captured = captured(&world, &[(1, "mine")]);
    land(&world, &[(1, "theirs")], "theirs");
    let with_order: Captured = Captured { captured: 200, landed: vec![tip_captured(&world, 100)], ..mine };
    let Merged::Tree(tree) = merge(&world, &with_order) else { panic!("expected the later capture to land") };
    assert_eq!(blob_of(&world, tree.as_str()), lines_with(&[(1, "mine")]));
    remove(world.root.as_path());
}

/// Order decides nothing without a stamp for every commit that changed the path,
/// or without the commit's own stamp.
#[test]
fn an_unknown_commit_or_an_unknown_stamp_leaves_an_overlap_a_conflict() {
    let world: World = world("replay-unknown");
    let mine: Captured = captured(&world, &[(1, "mine")]);
    land(&world, &[(1, "theirs")], "theirs");
    land(&world, &[(1, "theirs"), (9, "later")], "later");
    let only_one_known: Captured = Captured { captured: 200, landed: vec![tip_captured(&world, 100)], ..mine.clone() };
    assert!(matches!(merge(&world, &only_one_known), Merged::Conflict(_)), "the commit before the tip is unknown");
    let no_own_stamp: Captured = Captured { captured: 0, landed: vec![tip_captured(&world, 100)], ..mine };
    assert!(matches!(merge(&world, &no_own_stamp), Merged::Conflict(_)));
    remove(world.root.as_path());
}

/// The stamp is never zero where the platform has a clock,
/// and a later reading is never smaller.
#[test]
fn capture_stamps_do_not_go_back() {
    let first: u64 = capture_stamp();
    let second: u64 = capture_stamp();
    assert!(first > 0 && second >= first);
}

/// A change that overlaps what landed,
/// with different bytes,
/// stays a conflict even when the file holds the landed edit
///  partly.
#[test]
fn a_captured_file_that_rewrote_the_landed_lines_still_conflicts() {
    let world: World = world("replay-rewrote");
    let mine: Captured = captured(&world, &[(3, "theirs"), (4, "mine")]);
    land(&world, &[(3, "theirs"), (4, "theirs again")], "theirs");
    assert!(matches!(merge(&world, &mine), Merged::Conflict(_)));
    remove(world.root.as_path());
}

/// The conflict finding names every path,
/// the commit,
/// the saved message and the command,
/// in words with no internal term.
#[test]
fn the_conflict_text_names_what_the_person_needs_and_leaks_no_internal_term() {
    let conflict: ReplayConflict = ReplayConflict {
        paths: vec![b"a/one.txt".to_vec(), b"two.txt".to_vec()],
        landed: Some(LandedFirst { short: String::from("3f2a1c9"), subject: String::from("Add the thing") }),
        git_dir: PathBuf::from("/work/repo/.git"),
    };
    let saved: PathBuf = PathBuf::from("/work/repo/.git/monochromatic-git-policy-cli-saved-message-ab12");
    let text: String = conflict_text(&conflict, Some(saved.as_path()));
    for fact in ["a/one.txt", "two.txt", "3f2a1c9", "Add the thing", "/work/repo/.git/monochromatic-git-policy-cli-saved-message-ab12", NOTHING_CHANGED] {
        assert!(text.contains(fact), "{fact} missing: {text}");
    }
    assert!(text.contains("`git commit -F /work/repo/.git/monochromatic-git-policy-cli-saved-message-ab12`") && text.contains("`git show 3f2a1c9`"), "{text}");
    let unknown: String = conflict_text(&ReplayConflict { landed: None, ..conflict }, None);
    for shown in [text, unknown] {
        let plain: String = shown.split('`').enumerate().filter(|(at, _)| { return at % 2 == 0; }).map(|(_, part)| { return part; }).collect::<Vec<&str>>().join(" ").to_lowercase();
        for term in DENYLIST {
            assert!(!plain.split(|character: char| { return !character.is_alphanumeric(); }).any(|word| { return word == term; }), "{term} leaked: {shown}");
        }
    }
}

/// Both findings end the command with status 1,
/// land nothing,
/// and are one core finding of the concurrent-commit transform.
#[test]
fn both_endings_are_one_core_finding_with_status_one() {
    let conflict: ReplayConflict = ReplayConflict { paths: vec![b"a.txt".to_vec()], landed: None, git_dir: PathBuf::from("/r/.git") };
    for result in [conflict_result(&conflict, None), already_committed_result("main")] {
        assert_eq!(result.exit_code, 1);
        assert!(result.landed_oid.is_none());
        assert!(matches!(result.events.as_slice(), [PolicyEvent::CoreFinding { core_id: "concurrent-commit", .. }]));
    }
}

//region word-level second pass and attribute source
// What: the replay merge falls back to the word-level pass for a path Git's line merge and the subsumption rules leave in
//  conflict, and honours the committed `.gitattributes` of the landed tip when the worktree has none.
// Why: the owner decided on 2026-11-03 that the wrapper's unattended merges use the same second pass, and that the union line
//  of `.gitattributes` applies in the merges the wrapper computes from objects.

/// A stored corpus file of the word-level merge.
fn corpus(name: &str, extension: &str) -> Vec<u8> {
    return std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/merge/word_merge_corpus").join(format!("{name}.{extension}"))).expect("a corpus file");
}

/// A repository whose first commit holds `path` as the corpus input's base,
/// with the branch left on it.
fn corpus_world(name: &str, input: &str, path: &str) -> World {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::create_dir_all(repo.join(path).parent().expect("a parent")).expect("directories");
    std::fs::write(repo.join(path), corpus(input, "base")).expect("file");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::create_dir(root.join("scratch")).expect("scratch");
    return World { root, repo, context: git_context(), base };
}

/// The captured tree of a side branch that holds the corpus input's `theirs` text,
/// and the landed commit with its `ours` text.
fn corpus_sides(world: &World, input: &str, path: &str) -> Captured {
    git(world.repo.as_path(), &["checkout", "--quiet", "-b", "mine", world.base.as_str()]);
    std::fs::write(world.repo.join(path), corpus(input, "theirs")).expect("captured text");
    git(world.repo.as_path(), &["commit", "--quiet", "--all", "--message=mine"]);
    let tree: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(world.repo.join(path), corpus(input, "ours")).expect("landed text");
    git(world.repo.as_path(), &["commit", "--quiet", "--all", "--message=landed"]);
    return Captured { base: world.base.clone(), unborn: false, tree, captured: 0, landed: Vec::new(), conclusion: None };
}

/// An edit and an insertion right after it,
/// which Git's line merge reports as a conflict,
/// merge by word into one tree.
#[test]
fn an_adjacent_edit_and_insertion_merge_by_word() {
    let world: World = corpus_world("replay-word", "md1b", "notes.md");
    let mine: Captured = corpus_sides(&world, "md1b", "notes.md");
    let Merged::Combined { tree, files } = merge(&world, &mine) else { panic!("expected the word pass to merge the adjacent edits") };
    assert_eq!(files.len(), 1, "the one merged file is reported");
    assert_eq!(files[0].path, b"notes.md".to_vec());
    let blob: String = git_text(world.repo.as_path(), &["show", format!("{tree}:notes.md").as_str()]) + "\n";
    assert_eq!(blob.as_bytes(), corpus("md1b", "word.out").as_slice());
    remove(world.root.as_path());
}

/// Two sections appended at one point stay a conflict:
/// the word pass never invents an order.
#[test]
fn two_sections_appended_at_one_point_stay_a_conflict() {
    let world: World = corpus_world("replay-word-append", "md2", "notes.md");
    let mine: Captured = corpus_sides(&world, "md2", "notes.md");
    assert!(matches!(merge(&world, &mine), Merged::Conflict(paths) if paths == vec![b"notes.md".to_vec()]));
    remove(world.root.as_path());
}

/// The union line of the landed tip's `.gitattributes` applies although the worktree holds no such file.
#[test]
fn the_union_attribute_of_the_landed_tip_applies_without_a_worktree_file() {
    let world: World = corpus_world("replay-union", "md2", "doc/handover/log.md");
    std::fs::write(world.repo.join(".gitattributes"), "doc/handover/*.md merge=union\n").expect("attributes");
    git(world.repo.as_path(), &["add", "--all"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=attributes"]);
    let attributed: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let attributed_world: World = World { base: attributed, ..world };
    let mine: Captured = corpus_sides(&attributed_world, "md2", "doc/handover/log.md");
    // The checkout the merge runs from does not hold the attributes file (another branch, or a deletion not yet committed).
    std::fs::remove_file(attributed_world.repo.join(".gitattributes")).expect("remove the attributes file");
    git(attributed_world.repo.as_path(), &["rm", "--quiet", "--cached", ".gitattributes"]);
    let Merged::Tree(tree) = merge(&attributed_world, &mine) else { panic!("expected the union driver of the landed tip to merge the appends") };
    let blob: String = git_text(attributed_world.repo.as_path(), &["show", format!("{tree}:doc/handover/log.md").as_str()]);
    assert!(blob.contains("Ours risk.") && blob.contains("Theirs question."), "{blob}");
    remove(attributed_world.root.as_path());
}

/// The notice for a replayed commit lists every merged part,
/// says how to read the blobs it names,
/// and is absent when nothing was merged by word.
#[test]
fn the_notice_lists_each_merged_part_and_is_absent_without_one() {
    use crate::merge_second_pass::{Conflict, Merged as MergedFile, Stage};
    use crate::word_merge::{LineSpan, MergedPart};
    assert_eq!(word_merge_notice(&[]), None);
    let span = LineSpan { first: 2, count: 1 };
    let stage = |oid: &str| { return Some(Stage { mode: 0o100_644, oid: String::from(oid) }); };
    let file = MergedFile {
        path: b"notes.md".to_vec(),
        mode: 0o100_644,
        bytes: Vec::new(),
        parts: vec![MergedPart { base: span, ours: span, theirs: span, merged: b"joined\n".to_vec() }],
        stages: Conflict { path: b"notes.md".to_vec(), base: stage("aa12"), ours: stage("bb34"), theirs: stage("cc56") },
    };
    let Some(PolicyEvent::WordMerged { lines }) = word_merge_notice(&[file]) else { panic!("expected the event") };
    assert_eq!(lines.len(), 2, "the part and the sentence that says how to read its blobs: {lines:?}");
    assert!(lines[0].contains("word-level merge") && lines[0].contains("notes.md") && lines[0].contains("joined"), "{lines:?}");
    assert!(lines[0].ends_with("from base blob aa12, ours blob bb34, theirs blob cc56"), "{lines:?}");
    assert!(lines[1].contains("git cat-file blob"), "{lines:?}");
}

/// A merge-driver attribute of a path below a directory is honoured when the commit was run from a subdirectory of the
/// worktree:
/// the second pass reads attribute paths from the top,
/// so the driver's refusal stands and nothing is merged by word.
#[test]
fn a_driver_attribute_below_a_directory_is_honoured_from_a_subdirectory() {
    let world: World = corpus_world("replay-subdirectory-driver", "md1b", "docs/notes.md");
    std::fs::write(world.repo.join(".gitattributes"), "docs/*.md merge=binary\n").expect("attributes");
    git(world.repo.as_path(), &["add", "--all"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=attributes"]);
    let attributed: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let driven: World = World { base: attributed, ..world };
    let mine: Captured = corpus_sides(&driven, "md1b", "docs/notes.md");
    let tip: String = git_text(driven.repo.as_path(), &["rev-parse", "HEAD"]);
    let common: PathBuf = driven.repo.join(".git");
    let scratch: PathBuf = driven.root.join("scratch");
    let subdirectory: PathBuf = driven.repo.join("docs");
    let site: MergeSite<'_> =
        MergeSite { context: &driven.context, cwd: subdirectory.as_path(), common_dir: common.as_path(), object_directory: &common.join("objects"), scratch: scratch.as_path() };
    let merged: Merged = merge_captured(&site, &mine, tip.as_str(), 1).expect("the merge runs");
    assert!(matches!(merged, Merged::Conflict(_)), "{merged:?}");
    remove(driven.root.as_path());
}

/// A person's `attr.tree` is respected:
/// a configured tree without the union line is the one Git reads,
/// so the appends conflict
///  although the landed tip has the line.
#[test]
fn a_configured_attribute_tree_is_not_overridden() {
    let world: World = corpus_world("replay-attr-tree", "md2", "doc/handover/log.md");
    let without_attributes: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(world.repo.join(".gitattributes"), "doc/handover/*.md merge=union\n").expect("attributes");
    git(world.repo.as_path(), &["add", "--all"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=attributes"]);
    let attributed: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let attributed_world: World = World { base: attributed, ..world };
    let mine: Captured = corpus_sides(&attributed_world, "md2", "doc/handover/log.md");
    std::fs::remove_file(attributed_world.repo.join(".gitattributes")).expect("remove the attributes file");
    git(attributed_world.repo.as_path(), &["rm", "--quiet", "--cached", ".gitattributes"]);
    git(attributed_world.repo.as_path(), &["config", "attr.tree", without_attributes.as_str()]);
    assert!(matches!(merge(&attributed_world, &mine), Merged::Conflict(_)), "the person's attribute tree has no union line");
    git(attributed_world.repo.as_path(), &["config", "--unset", "attr.tree"]);
    assert!(matches!(merge(&attributed_world, &mine), Merged::Tree(_)), "with none configured the tip's line applies");
    remove(attributed_world.root.as_path());
}
//endregion

/// What:
///  The top of the worktree keeps every byte of a directory's name,
/// including a space at its end.
/// Why:
///   A repository may sit in a directory whose name ends in a space,
/// and trimming the line Git printed takes that space with it,
/// so every command of the
///  second pass then runs in a directory that does not exist and the commit is refused as an uncombinable change
/// (cross-vendor review of this branch,
/// finding 7).
#[test]
fn the_toplevel_keeps_a_trailing_space_in_a_directory_name() {
    assert_eq!(toplevel_of(b"/tmp/a repository \n"), PathBuf::from("/tmp/a repository "), "the space a directory's name ends in is part of the path");
    assert_eq!(toplevel_of(b"/tmp/repo\n"), PathBuf::from("/tmp/repo"), "the line break alone is removed");
    assert_eq!(toplevel_of(b"/tmp/repo"), PathBuf::from("/tmp/repo"), "a line without a break is the path");
}

/// What:
///  A `merge` rule the captured side introduces governs the replay's word pass,
/// so the file it excludes keeps its conflict.
/// Why:
///   The rule arrives with the captured tree,
/// so only the tree the replay's merge produced holds it,
/// and reading the landed tip alone
///  let the file be word-merged into a commit
/// (cross-vendor review of this branch,
/// finding 2;
/// the audit recorded this control as missing for the replay).
#[test]
fn a_rule_the_captured_side_introduces_keeps_the_replay_from_merging_the_file() {
    let world: World = corpus_world("replay-introduced-rule", "md1b", "data.txt");
    let mine: Captured = corpus_sides(&world, "md1b", "data.txt");
    git(world.repo.as_path(), &["checkout", "--quiet", "mine"]);
    std::fs::write(world.repo.join(".gitattributes"), "data.txt merge=binary\n").expect("the rule");
    git(world.repo.as_path(), &["add", "--all"]);
    git(world.repo.as_path(), &["commit", "--quiet", "--message=the rule"]);
    let with_rule: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
    git(world.repo.as_path(), &["checkout", "--quiet", "main"]);
    let captured: Captured = Captured { tree: with_rule, ..mine };
    let Merged::Conflict(paths) = merge(&world, &captured) else { panic!("the rule the merge introduces excludes the file from the word pass") };
    assert_eq!(paths, vec![b"data.txt".to_vec()], "the excluded path is the one reported");
    remove(world.root.as_path());
}

/// What:
///  A directory whose name ends in a line break keeps it.
/// Why:
///   Git's toplevel output ends with the directory's own line break and then the line's terminator,
/// so removing every line break removes one byte of the path
/// (cross-vendor review round 2,
/// finding 8).
#[test]
fn the_toplevel_keeps_a_line_break_that_belongs_to_a_directory_name() {
    assert_eq!(toplevel_of(b"/tmp/a repository \n\n"), PathBuf::from("/tmp/a repository \n"), "only the line's terminator is removed");
    assert_eq!(toplevel_of(b"/tmp/repo\r\n"), PathBuf::from("/tmp/repo"), "a carriage return and its line feed go together");
    assert_eq!(toplevel_of(b"/tmp/repo\n"), PathBuf::from("/tmp/repo"), "one line break is the terminator");
}

/// What:
///  A capture whose private index is gone is named,
///  and no tree is read from it.
/// Why:
///  `git write-tree` answers with the empty tree for an index that is gone,
///  so a read that found nothing rebuilt the commit as a deletion of every file
///  and reported a conflict naming paths the commit never changed
///  (reproduction `r02-index-commit-fails-when-foreign-commit-moves-branch.ts`,
///  `commit -a`,
///  2026-11-09).
#[test]
fn a_private_index_that_is_gone_is_named_instead_of_read_as_the_empty_tree() {
    let world: World = world("replay-captured-tree-gone");
    let common: PathBuf = world.repo.join(".git");
    let gone: PathBuf = world.root.join("scratch/commit.index");
    let stop: TransactionStop = captured_tree(&world.context, world.repo.as_path(), gone.as_path(), common.join("objects").as_path())
        .expect_err("a private index that is not there is not read");
    let text: String = match stop {
        TransactionStop::Message(text) => text,
        TransactionStop::Failed { message, .. } => message,
        TransactionStop::NativeExit(status) => panic!("a message, not Git's own exit {status}"),
    };
    assert!(text.contains(gone.display().to_string().as_str()), "the index is named: {text}");
    assert!(text.contains(crate::service_messages::NOTHING_CHANGED), "nothing changed: {text}");
    remove(world.root.as_path());
}

/// What:
///  A private index that is there is read,
///  so the guard does not stand in the way of the retry it protects.
/// Why:
///  The capture is one Git run over the index the attempt wrote,
///  and a guard that refused a real index would end every retry.
#[test]
fn a_private_index_that_is_there_is_read_into_the_tree_of_the_captured_change() {
    let world: World = world("replay-captured-tree-there");
    let common: PathBuf = world.repo.join(".git");
    let index: PathBuf = world.root.join("scratch/commit.index");
    std::fs::copy(common.join("index"), index.as_path()).expect("the private index of an attempt");
    let tree: String = captured_tree(&world.context, world.repo.as_path(), index.as_path(), common.join("objects").as_path()).expect("the tree of the captured change");
    assert_eq!(tree, git_text(world.repo.as_path(), &["rev-parse", "HEAD^{tree}"]), "the index holds the base, so its tree is the base's");
    remove(world.root.as_path());
}
