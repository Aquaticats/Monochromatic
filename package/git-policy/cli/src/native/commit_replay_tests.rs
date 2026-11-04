//! What:
//!  Differential controls of `commit_replay`: pure tree comparison and captured-delta replay compared with real Git on
//!  disposable repositories, with explicit base, result and current inputs, changing no live ref, index or worktree.
//! Why:
//!  The oracle is `git diff-tree -r --name-only --no-renames` for the comparison and `git merge-tree --write-tree
//!  --name-only --merge-base=B C T` for the replay. Each scenario builds commits `B`, `C` (on `B`) and `T` (on `B`)
//!  in a throwaway repository, flattens their trees with `git ls-tree`, and requires the pure answer to match.
//!  The scenarios where the pure answer differs from Git's by design (renames, line merges) are named as such.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // test('disjoint edits replay cleanly', () => { expect(replayDelta(b, t, c)).toEqual(gitMergeTree(b, c, t)); });
//! ```

use crate::model_fixture::{git, git_output, git_text, scratch};
use git_policy_cli::commit_replay::{ConflictKind, Replayed, compare_trees, replay_delta};
use git_policy_cli::repo_read_tree::FlatEntry;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One edit: write the bytes (with an executable bit when asked) or delete the path.
enum Edit {
    Write(&'static str, &'static str),
    Executable(&'static str, &'static str),
    Delete(&'static str),
    Symlink(&'static str, &'static str),
}

/// A fresh repository on `main` with one empty root commit.
fn repository(label: &str) -> PathBuf {
    let directory: PathBuf = scratch(label);
    git(&directory, &["init", "--quiet", "--initial-branch=main"]);
    git(&directory, &["commit", "--quiet", "--allow-empty", "--message=root"]);
    return directory;
}

/// Checks out `parent`, applies `edits`, commits, and returns the new commit's name.
fn commit_on(directory: &Path, parent: &str, edits: &[Edit]) -> String {
    git(directory, &["checkout", "--quiet", "--detach", parent]);
    for edit in edits {
        match edit {
            Edit::Write(path, text) | Edit::Executable(path, text) => {
                let target: PathBuf = directory.join(path);
                std::fs::create_dir_all(target.parent().expect("parent")).expect("directories");
                // A symlink at the path must be replaced, not written through.
                let _ = std::fs::remove_file(&target);
                std::fs::write(&target, text).expect("write");
                let mode: u32 = if matches!(edit, Edit::Executable(_, _)) { 0o755 } else { 0o644 };
                std::fs::set_permissions(&target, std::os::unix::fs::PermissionsExt::from_mode(mode)).expect("mode");
            }
            Edit::Delete(path) => {
                std::fs::remove_file(directory.join(path)).expect("delete");
            }
            Edit::Symlink(path, target) => {
                let link: PathBuf = directory.join(path);
                let _ = std::fs::remove_file(&link);
                std::os::unix::fs::symlink(target, &link).expect("symlink");
            }
        }
    }
    git(directory, &["add", "--all"]);
    git(directory, &["commit", "--quiet", "--allow-empty", "--message=edit"]);
    return git_text(directory, &["rev-parse", "HEAD"]);
}

/// The flattened tree of a commit or tree name, from `git ls-tree -r -z`.
fn flatten(directory: &Path, name: &str) -> Vec<FlatEntry> {
    let bytes: Vec<u8> = git(directory, &["ls-tree", "-r", "-z", "--full-tree", name]);
    let mut entries: Vec<FlatEntry> = Vec::new();
    for record in bytes.split(|b| { return *b == 0; }) {
        if record.is_empty() {
            continue;
        }
        let tab: usize = record.iter().position(|b| { return *b == b'\t'; }).expect("tab");
        let head: String = String::from_utf8(record[..tab].to_vec()).expect("head");
        let parts: Vec<&str> = head.split(' ').collect();
        entries.push(FlatEntry {
            path: record[tab + 1..].to_vec(),
            mode: u32::from_str_radix(parts[0], 8).expect("mode"),
            id: parts[2].as_bytes().to_vec(),
        });
    }
    return entries;
}

/// Git's answer to the replay: whether it was clean, the result tree, and the conflicted paths.
struct Oracle {
    clean: bool,
    tree: String,
    conflicted: BTreeSet<Vec<u8>>,
}

/// Runs `git merge-tree --write-tree --name-only -z --merge-base=base current result` and parses its NUL-separated output:
/// the tree name, then the conflicted names up to an empty field. Names are raw bytes, never a displayed spelling.
fn oracle(directory: &Path, base: &str, current: &str, result: &str) -> Oracle {
    let base_argument: String = format!("--merge-base={}", base);
    let output = git_output(directory, &["merge-tree", "--write-tree", "--name-only", "-z", &base_argument, current, result]);
    let mut fields = output.stdout.split(|byte| { return *byte == 0; });
    let tree: String = String::from_utf8(fields.next().expect("tree field").to_vec()).expect("utf-8");
    let conflicted: BTreeSet<Vec<u8>> = fields.take_while(|field| { return !field.is_empty(); }).map(|field| { return field.to_vec(); }).collect();
    return Oracle { clean: output.status.success(), tree, conflicted };
}

/// The paths of a pure replay's conflicts.
fn conflict_paths(replayed: &Replayed) -> BTreeSet<Vec<u8>> {
    return replayed.conflicts.iter().map(|conflict| { return conflict.path.clone(); }).collect();
}

/// What:
///  Builds `B` from `base`, `C` from `current` and `T` from `result` (both on `B`), runs the pure replay on their
///  flattened trees and Git's merge on the commits, and requires them to agree on clean or conflicted, on the
///  resulting entries when clean, and on the conflicted paths otherwise. Returns the pure answer.
/// Why:
///  Every scenario below is this one comparison with different edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function differential(base: Edit[], current: Edit[], result: Edit[]): Replayed;
/// ```
fn differential(label: &str, base: &[Edit], current: &[Edit], result: &[Edit]) -> Replayed {
    let directory: PathBuf = repository(label);
    let root: String = git_text(&directory, &["rev-parse", "HEAD"]);
    let b: String = commit_on(&directory, &root, base);
    let c: String = commit_on(&directory, &b, current);
    let t: String = commit_on(&directory, &b, result);
    let pure: Replayed = replay_delta(&flatten(&directory, &b), &flatten(&directory, &t), &flatten(&directory, &c));
    let git_answer: Oracle = oracle(&directory, &b, &c, &t);
    assert_eq!(pure.conflicts.is_empty(), git_answer.clean, "{}: clean or conflicted", label);
    if git_answer.clean {
        assert_eq!(pure.entries, flatten(&directory, &git_answer.tree), "{}: resulting entries", label);
    } else {
        // Git lists the relocated side of a type conflict as `path~<label>`, the label being the name of the side it was given
        // (here the commit's object name); it is accepted only for a path the pure replay also reports, with exactly that label.
        let mut ids_at: BTreeSet<Vec<u8>> = BTreeSet::new();
        for conflict in &pure.conflicts {
            for side in [&c, &t] {
                let mut moved: Vec<u8> = conflict.path.clone();
                moved.push(b'~');
                moved.extend_from_slice(side.as_bytes());
                ids_at.insert(moved);
            }
        }
        let own: BTreeSet<Vec<u8>> = conflict_paths(&pure);
        let explained: BTreeSet<Vec<u8>> = git_answer.conflicted.iter().filter(|name| { return !ids_at.contains(*name); }).cloned().collect();
        assert_eq!(own, explained, "{}: conflicted paths", label);
    }
    // The comparison agrees with `git diff-tree` on the changed paths from B to T.
    let changed: BTreeSet<Vec<u8>> = compare_trees(&flatten(&directory, &b), &flatten(&directory, &t)).into_iter().map(|d| { return d.path; }).collect();
    let listed: Vec<u8> = git(&directory, &["diff-tree", "-r", "--name-only", "--no-renames", "-z", &b, &t]);
    let git_changed: BTreeSet<Vec<u8>> =
        listed.split(|x| { return *x == 0; }).filter(|part| { return !part.is_empty(); }).map(|part| { return part.to_vec(); }).collect();
    assert_eq!(changed, git_changed, "{}: changed paths", label);
    return pure;
}

/// Edits to different paths on the two sides replay cleanly, with an add and a delete.
#[test]
fn disjoint_edits_replay_cleanly() {
    let replayed: Replayed = differential(
        "replay-disjoint",
        &[Edit::Write("a.txt", "a\n"), Edit::Write("b.txt", "b\n"), Edit::Write("c.txt", "c\n")],
        &[Edit::Write("a.txt", "a changed\n"), Edit::Write("new-current.txt", "n\n")],
        &[Edit::Delete("c.txt"), Edit::Write("b.txt", "b changed\n")],
    );
    assert!(replayed.conflicts.is_empty());
}

/// Both sides making the same change to a path is that change.
#[test]
fn the_same_change_on_both_sides_is_clean() {
    differential(
        "replay-same",
        &[Edit::Write("x.txt", "old\n")],
        &[Edit::Write("x.txt", "same\n")],
        &[Edit::Write("x.txt", "same\n")],
    );
}

/// Both sides changing one path differently conflicts (single-line files, so Git has no line merge to make either).
#[test]
fn different_edits_of_one_path_conflict() {
    let replayed: Replayed = differential(
        "replay-conflict",
        &[Edit::Write("x.txt", "old\n")],
        &[Edit::Write("x.txt", "current\n")],
        &[Edit::Write("x.txt", "result\n")],
    );
    assert_eq!(replayed.conflicts[0].kind, ConflictKind::BothChanged);
}

/// Editing a path the other side deleted conflicts, both ways round.
#[test]
fn edit_against_delete_conflicts() {
    differential("replay-edit-delete", &[Edit::Write("x.txt", "old\n")], &[Edit::Write("x.txt", "edited\n")], &[Edit::Delete("x.txt")]);
    differential("replay-delete-edit", &[Edit::Write("x.txt", "old\n")], &[Edit::Delete("x.txt")], &[Edit::Write("x.txt", "edited\n")]);
}

/// Both sides adding one path with different content conflicts.
#[test]
fn add_against_add_conflicts() {
    differential("replay-add-add", &[Edit::Write("keep.txt", "k\n")], &[Edit::Write("new.txt", "one\n")], &[Edit::Write("new.txt", "two\n")]);
}

/// A mode change on one side and a content change on the other settle separately, as Git does.
#[test]
fn a_mode_change_and_a_content_change_settle_separately() {
    let replayed: Replayed = differential(
        "replay-mode",
        &[Edit::Write("run.sh", "echo\n")],
        &[Edit::Write("run.sh", "echo changed\n")],
        &[Edit::Executable("run.sh", "echo\n")],
    );
    assert_eq!(replayed.entries[0].mode, 0o100_755);
}

/// A file on one side and a directory of the same name on the other is a directory/file conflict.
#[test]
fn a_file_against_a_directory_conflicts() {
    let directory: PathBuf = repository("replay-directory-file");
    let root: String = git_text(&directory, &["rev-parse", "HEAD"]);
    let b: String = commit_on(&directory, &root, &[Edit::Write("d", "file\n"), Edit::Write("keep.txt", "k\n")]);
    let c: String = commit_on(&directory, &b, &[Edit::Write("d", "file edited\n")]);
    let t: String = commit_on(&directory, &b, &[Edit::Delete("d"), Edit::Write("d/inside.txt", "inside\n")]);
    let pure: Replayed = replay_delta(&flatten(&directory, &b), &flatten(&directory, &t), &flatten(&directory, &c));
    assert!(!pure.conflicts.is_empty());
    assert!(!oracle(&directory, &b, &c, &t).clean);
}

/// Paths with spaces, quotes and non-ASCII letters survive the comparison and the replay.
#[test]
fn awkward_path_names_replay_like_git() {
    differential(
        "replay-names",
        &[Edit::Write("dir with space/\u{00FC}n\u{00EE}.txt", "u\n"), Edit::Write("quote\"name.txt", "q\n")],
        &[Edit::Write("dir with space/\u{00FC}n\u{00EE}.txt", "u current\n")],
        &[Edit::Write("quote\"name.txt", "q result\n"), Edit::Write("dir with space/new file.txt", "n\n")],
    );
}

/// Where the pure replay differs from Git by design: Git follows a rename and applies the other side's edit to the new
/// name; the path-level replay sees a delete and an add and reports the edit against the delete.
#[test]
fn a_rename_is_where_the_pure_replay_differs_from_git() {
    let directory: PathBuf = repository("replay-rename");
    let root: String = git_text(&directory, &["rev-parse", "HEAD"]);
    let body: &'static str = "line one\nline two\nline three\nline four\nline five\nline six\n";
    let b: String = commit_on(&directory, &root, &[Edit::Write("old-name.txt", body)]);
    let c: String = commit_on(&directory, &b, &[Edit::Write("old-name.txt", "line one\nline two\nline three\nline four\nline five\nline six edited\n")]);
    let t: String = commit_on(&directory, &b, &[Edit::Delete("old-name.txt"), Edit::Write("new-name.txt", body)]);
    let git_answer: Oracle = oracle(&directory, &b, &c, &t);
    assert!(git_answer.clean, "Git's merge follows the rename");
    let pure: Replayed = replay_delta(&flatten(&directory, &b), &flatten(&directory, &t), &flatten(&directory, &c));
    assert!(!pure.conflicts.is_empty(), "the path-level replay does not follow it");
}

/// Where the pure replay differs from Git by design: two edits to different lines of one file merge in Git and are a
/// conflict at path granularity.
#[test]
fn a_line_merge_is_where_the_pure_replay_differs_from_git() {
    let directory: PathBuf = repository("replay-lines");
    let root: String = git_text(&directory, &["rev-parse", "HEAD"]);
    let body: &'static str = "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\n";
    let b: String = commit_on(&directory, &root, &[Edit::Write("f.txt", body)]);
    let c: String = commit_on(&directory, &b, &[Edit::Write("f.txt", "ONE\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nten\n")]);
    let t: String = commit_on(&directory, &b, &[Edit::Write("f.txt", "one\ntwo\nthree\nfour\nfive\nsix\nseven\neight\nnine\nTEN\n")]);
    assert!(oracle(&directory, &b, &c, &t).clean);
    let pure: Replayed = replay_delta(&flatten(&directory, &b), &flatten(&directory, &t), &flatten(&directory, &c));
    assert_eq!(conflict_paths(&pure), BTreeSet::from([b"f.txt".to_vec()]));
}

/// The mixed plain and named-path pending sequence: `R1` (plain, x=1), `R2` (named, x=2 captured on the logical tail
/// after `R1`) and `R3` (plain, y=1) replay in adoption order onto a tip a foreign commit moved; at every step the
/// pure result equals Git's merge, and no request is reinterpreted against another map.
#[test]
fn the_mixed_pending_sequence_replays_in_order_onto_a_moved_tip() {
    let directory: PathBuf = repository("replay-sequence");
    let root: String = git_text(&directory, &["rev-parse", "HEAD"]);
    let b0: String = commit_on(&directory, &root, &[Edit::Write("x", "0\n"), Edit::Write("y", "0\n")]);
    let t1: String = commit_on(&directory, &b0, &[Edit::Write("x", "1\n")]);
    let t2: String = commit_on(&directory, &t1, &[Edit::Write("x", "2\n")]);
    let t3: String = commit_on(&directory, &t2, &[Edit::Write("y", "1\n")]);
    let foreign: String = commit_on(&directory, &b0, &[Edit::Write("z", "1\n")]);
    let steps: [(&String, &String, &String); 3] = [(&b0, &t1, &foreign), (&t1, &t2, &foreign), (&t2, &t3, &foreign)];
    let mut tip_commit: String = foreign.clone();
    let mut tip_entries: Vec<FlatEntry> = flatten(&directory, &foreign);
    for (base, result, _) in steps {
        let pure: Replayed = replay_delta(&flatten(&directory, base), &flatten(&directory, result), &tip_entries);
        let git_answer: Oracle = oracle(&directory, base, &tip_commit, result);
        assert!(pure.conflicts.is_empty() && git_answer.clean);
        assert_eq!(pure.entries, flatten(&directory, &git_answer.tree));
        tip_commit = git_text(&directory, &["commit-tree", &git_answer.tree, "-p", &tip_commit, "-m", "replayed"]);
        tip_entries = pure.entries;
    }
    let names: Vec<(Vec<u8>, Vec<u8>)> = tip_entries.iter().map(|entry| { return (entry.path.clone(), entry.id.clone()); }).collect();
    let blob_of = |path: &str| -> Vec<u8> {
        return names.iter().find(|(name, _)| { return name == path.as_bytes(); }).expect("path").1.clone();
    };
    assert_eq!(String::from_utf8(blob_of("x")).expect("id"), git_text(&directory, &["rev-parse", &format!("{}:x", t2)]));
    assert_eq!(String::from_utf8(blob_of("y")).expect("id"), git_text(&directory, &["rev-parse", &format!("{}:y", t3)]));
    assert_eq!(String::from_utf8(blob_of("z")).expect("id"), git_text(&directory, &["rev-parse", &format!("{}:z", foreign)]));
}

/// An amendment whose target is no longer the tip: the amended tree relative to the target `D1`, replayed on the tip `C`
/// that a successor moved, keeps the successor's change and never rewrites it.
#[test]
fn an_amendment_whose_target_is_no_longer_the_tip_stacks_on_the_tip() {
    let directory: PathBuf = repository("replay-amend");
    let root: String = git_text(&directory, &["rev-parse", "HEAD"]);
    let d1: String = commit_on(&directory, &root, &[Edit::Write("x", "1\n"), Edit::Write("lexer", "none\n")]);
    let tip: String = commit_on(&directory, &d1, &[Edit::Write("lexer", "added by the successor\n")]);
    let amended: String = commit_on(&directory, &d1, &[Edit::Write("x", "1 amended\n")]);
    let pure: Replayed = replay_delta(&flatten(&directory, &d1), &flatten(&directory, &amended), &flatten(&directory, &tip));
    let git_answer: Oracle = oracle(&directory, &d1, &tip, &amended);
    assert!(pure.conflicts.is_empty() && git_answer.clean);
    assert_eq!(pure.entries, flatten(&directory, &git_answer.tree));
    assert_eq!(
        flatten(&directory, &git_answer.tree).iter().map(|e| { return e.path.clone(); }).collect::<Vec<_>>(),
        vec![b"lexer".to_vec(), b"x".to_vec()]
    );
}

/// Replaying onto the base gives the result; replaying a no-op delta gives the current tree; the delta of two trees
/// applied to the first gives the second. None of these needs Git.
#[test]
fn replay_identities_hold_without_git() {
    let entry = |path: &str, id: &str| -> FlatEntry { return FlatEntry { path: path.as_bytes().to_vec(), mode: 0o100_644, id: id.as_bytes().to_vec() }; };
    let base: Vec<FlatEntry> = vec![entry("a", "1"), entry("b", "2")];
    let result: Vec<FlatEntry> = vec![entry("a", "9"), entry("c", "3")];
    let current: Vec<FlatEntry> = vec![entry("b", "7"), entry("d", "4")];
    assert_eq!(replay_delta(&base, &result, &base).entries, result);
    assert_eq!(replay_delta(&base, &base, &current).entries, current);
    let deltas = compare_trees(&base, &result);
    assert_eq!(deltas.len(), 3);
    assert!(compare_trees(&result, &result).is_empty());
    let mut applied: Vec<FlatEntry> = base.clone();
    for delta in deltas {
        applied.retain(|existing| { return existing.path != delta.path; });
        if let Some((mode, id)) = delta.new {
            applied.push(FlatEntry { path: delta.path, mode, id });
        }
    }
    applied.sort_by(|x, y| { return x.path.cmp(&y.path); });
    assert_eq!(applied, result);
}

/// Review finding P1: a regular file edited on one side and replaced by a symlink on the other is a type conflict. Independent
/// merging of mode and object name would synthesize a symlink to the edited blob, which neither side asked for.
#[test]
fn a_regular_file_against_a_symlink_conflicts_like_git() {
    let replayed: Replayed = differential(
        "replay-symlink",
        &[Edit::Write("link-or-file", "old")],
        &[Edit::Write("link-or-file", "new")],
        &[Edit::Symlink("link-or-file", "old")],
    );
    assert_eq!(replayed.conflicts[0].kind, ConflictKind::BothChanged);
}

/// Re-review C: both descendants converting a symlink to a regular file is a conversion both asked for; the result's edit of the
/// content merges, as Git does.
#[test]
fn an_agreed_conversion_from_a_symlink_to_a_regular_file_merges_like_git() {
    let replayed: Replayed = differential(
        "replay-agreed-conversion",
        &[Edit::Symlink("thing", "old")],
        &[Edit::Write("thing", "old")],
        &[Edit::Write("thing", "new")],
    );
    assert!(replayed.conflicts.is_empty());
}

/// Re-review: three different gitlinks cannot be settled without the submodule's history, so the replay says so explicitly and
/// the caller falls back; it is not reported as a content conflict.
#[test]
fn three_different_gitlinks_ask_for_an_ancestry_check() {
    let link = |id: &str| -> Vec<FlatEntry> { return vec![FlatEntry { path: b"sub".to_vec(), mode: 0o160_000, id: id.as_bytes().to_vec() }]; };
    let replayed: Replayed = replay_delta(&link("s0"), &link("s2"), &link("s1"));
    assert_eq!(replayed.conflicts[0].kind, ConflictKind::GitlinkNeedsAncestry);
    assert!(replay_delta(&link("s0"), &link("s2"), &link("s0")).conflicts.is_empty());
}

/// Commits `parent` plus a gitlink `path` at `oid` and the `.gitmodules` file, without touching the nested repository: the index
/// is set by hand, because `git add --all` would record whatever the nested repository has checked out.
fn commit_gitlink(directory: &Path, parent: &str, oid: &str) -> String {
    git(directory, &["checkout", "--quiet", "--detach", parent]);
    std::fs::write(directory.join(".gitmodules"), "[submodule \"sub\"]\n\tpath = sub\n\turl = ./sub\n").expect("gitmodules");
    git(directory, &["add", ".gitmodules"]);
    git(directory, &["update-index", "--add", "--cacheinfo", &format!("160000,{},sub", oid)]);
    git(directory, &["commit", "--quiet", "--allow-empty", "--message=gitlink"]);
    return git_text(directory, &["rev-parse", "HEAD"]);
}

/// A populated submodule: base, current and result record `s0`, `s1`, `s2` of a linear history. Git (with the submodule's objects
/// at hand) can fast-forward it; the pure replay cannot know and says so with the explicit fallback kind, and the fallback
/// (an ancestry check in the submodule) reaches Git's answer.
#[test]
fn a_populated_submodule_fast_forward_is_an_explicit_ancestry_fallback() {
    let directory: PathBuf = repository("replay-submodule");
    let sub: PathBuf = directory.join("sub");
    std::fs::create_dir_all(&sub).expect("sub");
    git(&sub, &["init", "--quiet", "--initial-branch=main"]);
    let mut oids: Vec<String> = Vec::new();
    for number in 0..3 {
        std::fs::write(sub.join("f.txt"), format!("{}\n", number)).expect("write");
        git(&sub, &["add", "f.txt"]);
        git(&sub, &["commit", "--quiet", "--message=step"]);
        oids.push(git_text(&sub, &["rev-parse", "HEAD"]));
    }
    let root: String = git_text(&directory, &["rev-parse", "HEAD"]);
    let b: String = commit_gitlink(&directory, &root, &oids[0]);
    let c: String = commit_gitlink(&directory, &b, &oids[1]);
    let t: String = commit_gitlink(&directory, &b, &oids[2]);
    let pure: Replayed = replay_delta(&flatten(&directory, &b), &flatten(&directory, &t), &flatten(&directory, &c));
    assert_eq!(pure.conflicts.len(), 1);
    assert_eq!(pure.conflicts[0].kind, ConflictKind::GitlinkNeedsAncestry);
    // The fallback: the current commit is an ancestor of the result, so the result is the answer.
    let ancestor = git_output(&sub, &["merge-base", "--is-ancestor", &oids[1], &oids[2]]);
    assert!(ancestor.status.success());
    let git_answer: Oracle = oracle(&directory, &b, &c, &t);
    assert!(git_answer.clean, "Git fast-forwards a populated submodule");
    let merged: Vec<FlatEntry> = flatten(&directory, &git_answer.tree);
    let link: &FlatEntry = merged.iter().find(|entry| { return entry.path == b"sub".to_vec(); }).expect("the gitlink");
    assert_eq!(link.id, oids[2].as_bytes().to_vec());
}

/// Review 4 section 8, item 6: a request is built from a real repository, its spool carries both sides, every object of the
/// repository is then removed, and the replay runs from the reconstructed closure alone: clean against a tip that did not touch the
/// path, a conflict against a tip that changed it differently.
#[test]
fn a_replay_from_a_closure_reconstructed_after_the_objects_were_pruned_is_clean_and_conflicting() {
    use git_policy_cli::commit_integrity::sha256_hex;
    use git_policy_cli::commit_request::{Change, Operation, Request};
    use git_policy_cli::commit_spool::{read_closure, write_spool};
    let repository: PathBuf = repository("replay-closure");
    let root: String = git_text(&repository, &["rev-parse", "HEAD"]);
    let base: String = commit_on(&repository, &root, &[Edit::Write("kept.txt", "base bytes\n"), Edit::Write("other.txt", "o\n")]);
    let base_bytes: Vec<u8> = git(&repository, &["show", &format!("{}:kept.txt", base)]);
    let request = Request {
        id: "6".repeat(32),
        branch: b"main".to_vec(),
        base: base.clone(),
        message: b"m".to_vec(),
        operation: Operation::Commit,
        nonce: vec![1; 16],
        changes: vec![Change {
            path: b"kept.txt".to_vec(),
            mode: "100644".to_string(),
            sha256: sha256_hex(b"result bytes\n"),
            old_mode: "100644".to_string(),
            old_sha256: sha256_hex(&base_bytes),
        }],
    };
    write_spool(&repository, &request.id, &[b"result bytes\n".to_vec()], std::slice::from_ref(&base_bytes)).expect("spool");
    for entry in std::fs::read_dir(repository.join(".git/objects")).expect("objects") {
        std::fs::remove_dir_all(entry.expect("entry").path()).expect("prune");
    }
    let closure = read_closure(&repository, &request).expect("closure without any object");
    let side = |bytes: &[u8]| -> FlatEntry { return FlatEntry { path: b"kept.txt".to_vec(), mode: 0o100_644, id: sha256_hex(bytes).into_bytes() }; };
    let old: FlatEntry = side(&closure[0].old.as_ref().expect("old").1);
    let new: FlatEntry = side(&closure[0].new.as_ref().expect("new").1);
    let untouched: Vec<FlatEntry> = vec![old.clone()];
    let clean: Replayed = replay_delta(std::slice::from_ref(&old), std::slice::from_ref(&new), &untouched);
    assert!(clean.conflicts.is_empty());
    assert_eq!(clean.entries, vec![new.clone()]);
    let competing: Vec<FlatEntry> = vec![side(b"someone else's bytes\n")];
    let conflict: Replayed = replay_delta(std::slice::from_ref(&old), std::slice::from_ref(&new), &competing);
    assert_eq!(conflict.conflicts[0].kind, ConflictKind::BothChanged);
}

/// Review 5 item 3.7: a replay in which every path has competing edits settles every path at its current cell and reports one conflict
/// per path; the membership test for "already reported" must not scan the conflict list per leaf. A sorted workload of this size is
/// quick in a linear replay and takes tens of seconds when each of the leaves scans every conflict so far; the bound below is the
/// measured margin (the linear replay takes a small fraction of it in the same build).
#[test]
fn a_conflict_heavy_replay_is_not_quadratic() {
    let paths: usize = 120_000;
    let cell = |index: usize, id: &str| -> FlatEntry { return FlatEntry { path: format!("dir/file-{:06}", index).into_bytes(), mode: 0o100_644, id: id.as_bytes().to_vec() }; };
    let base: Vec<FlatEntry> = (0..paths).map(|index| { return cell(index, "o"); }).collect();
    let current: Vec<FlatEntry> = (0..paths).map(|index| { return cell(index, "a"); }).collect();
    let result: Vec<FlatEntry> = (0..paths).map(|index| { return cell(index, "b"); }).collect();
    let started = std::time::Instant::now();
    let replayed: Replayed = replay_delta(&base, &result, &current);
    let elapsed = started.elapsed();
    assert_eq!(replayed.conflicts.len(), paths);
    assert_eq!(replayed.entries.len(), paths);
    assert!(elapsed < std::time::Duration::from_secs(20), "{} conflicting paths took {:?}", paths, elapsed);
}
