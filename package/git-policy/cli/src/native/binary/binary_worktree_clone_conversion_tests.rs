//! What:
//!  Rung 2 of the cheap-worktree ladder under Git's checkout conversions,
//! through the built executable:
//! one control per conversion class,
//! each comparing a worktree the clone rung served with the one real Git made from the same commit.
//! Why:
//!  A clone copies the source's working files,
//! so it holds the source's bytes where Git's checkout would write the conversion of the blob,
//! and neither the stat data nor `git status` can tell
//!  (issue 6323).
//! Every control asserts that the clone rung served its add,
//! so a control that passes because the add fell to Git's checkout is not taken for a fence;
//! the last two name the classes where the swap of the source's bytes for Git's own is invisible
//! (`working-tree-encoding`,
//! whose clean and checkout are inverses here,
//! and a filter the design already declines).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const served = runWrapped(['worktree', 'add', '-b', 'topic', '../topic']);
//! // expect(trackedContent(topic)).toEqual(trackedContent(control));
//! ```

/// Import the shared fixtures and bounded process helpers.
use super::support::{Fixture, Observed, fixture, git, on_zfs_or_acknowledged, porcelain, remove, repository, run_direct, run_wrapped_verbose, stderr_of};
use super::worktree_snapshot::{commit_all, tracked, tracked_above_the_mean_edge, tracked_content};
use git_policy_cli::worktree::worktree_clone_conversion_fence::{DECLINE_PATHS_FLOOR, DECLINE_SHARE_DIVISOR};
use std::ffi::OsStr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// What:
///  A repository of the class's shape:
/// an LF text file in a directory,
/// `notes.txt`,
/// and an executable,
/// committed,
/// then the class's own state.
/// Why:
///   One fixture body serves every class,
/// so a difference between two controls is the class and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conversionFixture(fixture, name, attributes, notes, after): string;
/// ```
fn conversion_fixture(fixture: &Fixture, name: &str, attributes: &[u8], notes: &[u8], after: &dyn Fn(&Path)) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    // The anchor sits above the clone rung's mean-file edge on purpose: without it this clean source averages a few
    // dozen bytes a file, the size limit declines it, and the conversion class these controls are about is never reached.
    tracked_above_the_mean_edge(repo.as_path(), "bulk/anchor.bin", 4);
    tracked(repo.as_path(), "dir/file.txt", b"line one\nline two\n");
    tracked(repo.as_path(), "notes.txt", notes);
    tracked(repo.as_path(), "tool.sh", b"#!/bin/sh\necho hi\n");
    std::fs::set_permissions(repo.join("tool.sh"), std::fs::Permissions::from_mode(0o755)).expect("executable");
    if !attributes.is_empty() {
        tracked(repo.as_path(), ".gitattributes", attributes);
    }
    commit_all(fixture, repo.as_path(), "fixture");
    after(repo.as_path());
    return repo;
}

/// What:
///  Add the same source twice:
/// once through the built executable and once by real Git's own checkout,
/// then compare every tracked path's bytes and mode,
/// the new worktree's status,
/// and say whether the clone rung served the add.
/// Why:
///   The comparison is what the issue is about,
/// and the route is what says the comparison tested the rung.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assert_rung_equals_git(fixture, repo, served);
/// ```
fn assert_rung_equals_git(fixture: &Fixture, repo: &Path, served: bool) {
    assert_rung_equals_git_expecting(fixture, repo, served, "");
}

/// What:
///  As above,
/// and the words the fence must have printed for the route it took.
/// Why:
///   A declining control must show the decline,
/// not only that Git's checkout was the one that wrote the worktree.
fn assert_rung_equals_git_expecting(fixture: &Fixture, repo: &Path, served: bool, expected: &str) {
    let created: Observed = run_wrapped_verbose(fixture, repo, &["worktree", "add", "-b", "topic", "../topic"]);
    let control: Observed = run_direct(fixture, repo, &["worktree", "add", "--quiet", "-b", "control", "../control"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    let clones: bool = stderr_of(&created).contains("clones served");
    assert_eq!(clones, served, "the clone rung served the add: {}\n{}", served, stderr_of(&created));
    assert!(stderr_of(&created).contains(expected), "the fence said {expected:?}: {}", stderr_of(&created));
    let topic: PathBuf = fixture.root.join("topic");
    assert_eq!(tracked_content(fixture, topic.as_path()), tracked_content(fixture, fixture.root.join("control").as_path()));
    assert_eq!(git(fixture, topic.as_path(), &["status", "--porcelain=v1"]).stdout, b"", "the new worktree of the wrapper is clean");
    assert_eq!(git(fixture, fixture.root.join("control").as_path(), &["status", "--porcelain=v1"]).stdout, b"", "the new worktree of real Git is clean");
    std::fs::remove_dir_all(topic.as_path()).expect("remove the worktree");
    std::fs::remove_dir_all(fixture.root.join("control")).expect("remove the control");
    remove(fixture);
}

#[test]
fn a_text_attribute_asking_for_crlf_over_lf_files_is_written_by_git() {
    let fixture: Fixture = fixture("zfs-clone-conversion-eol-crlf");
    if !on_zfs_or_acknowledged(&fixture, "a_text_attribute_asking_for_crlf_over_lf_files_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"*.txt text eol=crlf\n", b"line one\nline two\n", &|_repo| {});
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn core_autocrlf_true_set_after_the_files_were_written_is_written_by_git() {
    let fixture: Fixture = fixture("zfs-clone-conversion-autocrlf");
    if !on_zfs_or_acknowledged(&fixture, "core_autocrlf_true_set_after_the_files_were_written_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"", b"line one\nline two\n", &|repo| {
        git(&fixture, repo, &["config", "core.autocrlf", "true"]);
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn an_ident_attribute_over_an_unexpanded_keyword_is_written_by_git() {
    let fixture: Fixture = fixture("zfs-clone-conversion-ident");
    if !on_zfs_or_acknowledged(&fixture, "an_ident_attribute_over_an_unexpanded_keyword_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"notes.txt ident\n", b"line one $Id$\nline two\n", &|_repo| {});
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_crlf_file_added_under_text_auto_eol_lf_is_written_by_git() {
    let fixture: Fixture = fixture("zfs-clone-conversion-normalized");
    if !on_zfs_or_acknowledged(&fixture, "a_crlf_file_added_under_text_auto_eol_lf_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"* text=auto eol=lf\n", b"line one\nline two\n", &|repo| {
        // The blob stays the LF one and the index records the CRLF file's stat, so no stat check sees the difference.
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        std::fs::write(repo.join("notes.txt"), b"line one\r\nline two\r\n").expect("CRLF file");
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        git(&fixture, repo, &["add", "--", "notes.txt"]);
        git(&fixture, repo, &["update-index", "--refresh"]);
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn an_eol_crlf_attribute_without_text_is_written_by_git() {
    // `convert_attrs` lets `eol` refine an action nothing has named yet, so `*.txt eol=crlf` alone writes carriage
    // returns; the prototype carried these LF files before the fix (the gate review's `m01`).
    let fixture: Fixture = fixture("zfs-clone-conversion-bare-eol-crlf");
    if !on_zfs_or_acknowledged(&fixture, "an_eol_crlf_attribute_without_text_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"*.txt eol=crlf\n", b"line one\nline two\n", &|_repo| {});
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn an_eol_lf_attribute_without_text_is_written_by_git() {
    // The same refinement in the other direction: a CRLF file added under `*.txt eol=lf` has an LF blob and the fence's
    // size proof sends it to Git; the prototype carried it before the fix (the gate review's `m02`).
    let fixture: Fixture = fixture("zfs-clone-conversion-bare-eol-lf");
    if !on_zfs_or_acknowledged(&fixture, "an_eol_lf_attribute_without_text_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"*.txt eol=lf\n", b"line one\nline two\n", &|repo| {
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        std::fs::write(repo.join("notes.txt"), b"line one\r\nline two\r\n").expect("CRLF file");
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        git(&fixture, repo, &["add", "--", "notes.txt"]);
        git(&fixture, repo, &["update-index", "--refresh"]);
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_legacy_crlf_attribute_beside_core_eol_crlf_is_written_by_git() {
    // The legacy `crlf` attribute is `text`'s fallback, and `core.eol=crlf` gives it the carriage return ending, so Git
    // writes these LF files; the prototype read no `crlf` attribute at all (the gate review's `m03`).
    let fixture: Fixture = fixture("zfs-clone-conversion-legacy-crlf");
    if !on_zfs_or_acknowledged(&fixture, "a_legacy_crlf_attribute_beside_core_eol_crlf_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"*.txt crlf\n", b"line one\nline two\n", &|repo| {
        git(&fixture, repo, &["config", "core.eol", "crlf"]);
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_legacy_crlf_attribute_over_an_added_crlf_file_is_written_by_git() {
    // `*.txt crlf` over a file added with carriage returns: the blob is the LF form and the file the CRLF one, so the
    // size proof sends it to Git (the gate review's `m03b`).
    let fixture: Fixture = fixture("zfs-clone-conversion-legacy-crlf-added");
    if !on_zfs_or_acknowledged(&fixture, "a_legacy_crlf_attribute_over_an_added_crlf_file_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"*.txt crlf\n", b"line one\nline two\n", &|repo| {
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        std::fs::write(repo.join("notes.txt"), b"line one\r\nline two\r\n").expect("CRLF file");
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        git(&fixture, repo, &["add", "--", "notes.txt"]);
        git(&fixture, repo, &["update-index", "--refresh"]);
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_config_rule_gone_after_the_add_is_written_by_git() {
    // `core.autocrlf=input` when the CRLF file was added and unset afterwards: no rule is left, the file is stat-exact,
    // and only the sizes differ (the gate review's `m04`).
    let fixture: Fixture = fixture("zfs-clone-conversion-config-gone");
    if !on_zfs_or_acknowledged(&fixture, "a_config_rule_gone_after_the_add_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"", b"line one\nline two\n", &|repo| {
        git(&fixture, repo, &["config", "core.autocrlf", "input"]);
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        std::fs::write(repo.join("notes.txt"), b"line one\r\nline two\r\n").expect("CRLF file");
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        git(&fixture, repo, &["add", "--", "notes.txt"]);
        // The add converted the CRLF file to the blob the base commit holds, so the tree is unchanged and the commit records
        // the index the add left; without `--allow-empty` native Git refuses it with nothing to commit.
        git(&fixture, repo, &["commit", "--quiet", "--allow-empty", "--message=added"]);
        git(&fixture, repo, &["config", "--unset", "core.autocrlf"]);
        git(&fixture, repo, &["update-index", "--refresh"]);
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_tracked_attributes_file_a_later_commit_deleted_is_written_by_git() {
    // `* text=auto` when the CRLF file was added, the tracked `.gitattributes` deleted by a later commit: the path has no
    // rule today, the file is stat-exact, and only the sizes differ (the gate review's `m06`).
    let fixture: Fixture = fixture("zfs-clone-conversion-attributes-deleted");
    if !on_zfs_or_acknowledged(&fixture, "a_tracked_attributes_file_a_later_commit_deleted_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"* text=auto\n", b"line one\nline two\n", &|repo| {
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        std::fs::write(repo.join("notes.txt"), b"line one\r\nline two\r\n").expect("CRLF file");
        super::worktree_snapshot::age(repo.join("notes.txt").as_path());
        git(&fixture, repo, &["add", "--", "notes.txt"]);
        // The add converted the CRLF file to the blob the base commit holds, so the tree is unchanged and the commit records
        // the index the add left; without `--allow-empty` native Git refuses it with nothing to commit.
        git(&fixture, repo, &["commit", "--quiet", "--allow-empty", "--message=added"]);
        git(&fixture, repo, &["rm", "--quiet", "--", ".gitattributes"]);
        git(&fixture, repo, &["commit", "--quiet", "--message=deleted"]);
        git(&fixture, repo, &["update-index", "--refresh"]);
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn an_ignored_untracked_attributes_file_sends_the_add_to_gits_checkout() {
    let fixture: Fixture = fixture("zfs-clone-conversion-ignored-attributes");
    if !on_zfs_or_acknowledged(&fixture, "an_ignored_untracked_attributes_file_sends_the_add_to_gits_checkout") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    tracked(repo.as_path(), ".gitignore", b"edge/\n");
    tracked(repo.as_path(), "dir/file.txt", b"line one\nline two\n");
    tracked(repo.as_path(), "notes.txt", b"line one\nline two\n");
    commit_all(&fixture, repo.as_path(), "fixture");
    // Git reads the ignored attributes file when the ignored path is added, and the new worktree will not have it:
    // the blob is the LF form while the index records the CRLF file's stat, so no stat check sees the difference.
    std::fs::create_dir_all(repo.join("edge")).expect("directory");
    // The pattern is relative to `edge/`, so it must not repeat the directory: `edge/file.txt` never matches there,
    // and the control stayed green on step 3 by its route assertion alone instead of by its bytes (the gate review's
    // `c06` and `c06b`).
    std::fs::write(repo.join("edge/.gitattributes"), b"file.txt text eol=lf\n").expect("ignored attributes");
    std::fs::write(repo.join("edge/file.txt"), b"edge line one\r\nedge line two\r\n").expect("CRLF file");
    super::worktree_snapshot::age(repo.join("edge/file.txt").as_path());
    git(&fixture, repo.as_path(), &["add", "--force", "--", "edge/file.txt"]);
    git(&fixture, repo.as_path(), &["commit", "--quiet", "--message=edge"]);
    git(&fixture, repo.as_path(), &["update-index", "--refresh"]);
    assert_rung_equals_git(&fixture, repo.as_path(), false);
}

#[test]
fn a_working_tree_encoding_carries_the_sources_bytes_as_gits_checkout_does() {
    let fixture: Fixture = fixture("zfs-clone-conversion-encoding");
    if !on_zfs_or_acknowledged(&fixture, "a_working_tree_encoding_carries_the_sources_bytes_as_gits_checkout_does") {
        remove(&fixture);
        return;
    }
    let utf16: Vec<u8> = b"line one\nline two\n".iter().flat_map(|byte| return [*byte, 0]).collect();
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"notes.txt working-tree-encoding=UTF-16LE\n", &utf16, &|_repo| {});
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_core_autocrlf_key_written_without_a_value_is_written_by_git() {
    // `git_parse_maybe_bool_text(NULL)` is 1 (`parse.c`) and `core.autocrlf` goes through `git_config_bool`
    // (`config.c`), so `[core]` with a line holding only `autocrlf` is true and Git writes carriage returns here. The
    // fence read the missing value as an absent key and carried these files before the gate re-check's `p01`.
    let fixture: Fixture = fixture("zfs-clone-conversion-valueless-autocrlf");
    if !on_zfs_or_acknowledged(&fixture, "a_core_autocrlf_key_written_without_a_value_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"", b"line one\nline two\n", &|repo| {
        let config_path: PathBuf = repo.join(".git/config");
        let mut config: Vec<u8> = std::fs::read(config_path.as_path()).expect("the source's configuration");
        // A section of its own, so the write cannot depend on what `git init` left in `[core]`.
        config.extend_from_slice(b"[core]\n\tautocrlf\n");
        std::fs::write(config_path.as_path(), config).expect("a valueless core.autocrlf");
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_literal_text_unset_value_under_core_autocrlf_true_is_written_by_git() {
    // `git check-attr` prints the literal value `text=unset` as `unset`, exactly as it prints `-text`, and Git reads
    // the literal value as `CRLF_UNDEFINED` (`git_path_check_crlf`), which `convert_attrs` gives to `core.autocrlf`;
    // under `core.autocrlf=true` it therefore writes carriage returns here (the gate re-check's `p02`).
    let fixture: Fixture = fixture("zfs-clone-conversion-text-unset-value");
    if !on_zfs_or_acknowledged(&fixture, "a_literal_text_unset_value_under_core_autocrlf_true_is_written_by_git") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = conversion_fixture(&fixture, "repo", b"*.txt text=unset\n", b"line one\nline two\n", &|repo| {
        git(&fixture, repo, &["config", "core.autocrlf", "true"]);
    });
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

/// What:
///  A repository whose converting share stands where the decline rule's boundaries are:
/// `converting` text paths under a carriage-return rule,
/// `other` paths no rule matches,
/// and the tracked `.gitattributes` the rule itself.
/// Why:
///   The rule is two boundaries,
/// a floor and a share,
/// and a source on each side of one must take the route the rule names.
fn decline_fixture(fixture: &Fixture, name: &str, converting: usize, other: usize) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    // Above the clone rung's mean-file edge on purpose: this clean source of tiny files would otherwise be declined by
    // its size before the decline share these controls are about is asked, and the served arms would fall to Git.
    tracked_above_the_mean_edge(repo.as_path(), "bulk/anchor.bin", (converting + other + 1) as u64);
    tracked(repo.as_path(), ".gitattributes", b"*.txt text eol=crlf\n");
    for index in 0..converting {
        tracked(repo.as_path(), format!("convert-{index}.txt").as_str(), b"line one\nline two\n");
    }
    for index in 0..other {
        tracked(repo.as_path(), format!("plain-{index}.dat").as_str(), b"line one\nline two\n");
    }
    commit_all(fixture, repo.as_path(), "fixture");
    return repo;
}

#[test]
fn a_converting_count_at_the_floor_keeps_the_clone_rung() {
    // A thousand converting paths of a tiny tree are a share far past a quarter, and the floor is what keeps the rung.
    let fixture: Fixture = fixture("zfs-clone-conversion-decline-floor-below");
    if !on_zfs_or_acknowledged(&fixture, "a_converting_count_at_the_floor_keeps_the_clone_rung") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = decline_fixture(&fixture, "repo", DECLINE_PATHS_FLOOR, DECLINE_SHARE_DIVISOR);
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_converting_count_one_past_the_floor_declines_the_rung() {
    let fixture: Fixture = fixture("zfs-clone-conversion-decline-floor-above");
    if !on_zfs_or_acknowledged(&fixture, "a_converting_count_one_past_the_floor_declines_the_rung") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = decline_fixture(&fixture, "repo", DECLINE_PATHS_FLOOR + 1, DECLINE_SHARE_DIVISOR);
    assert_rung_equals_git_expecting(&fixture, repo.as_path(), false, "so this rung declines and Git's checkout writes the worktree");
}

#[test]
fn a_converting_share_of_exactly_a_quarter_keeps_the_clone_rung() {
    let fixture: Fixture = fixture("zfs-clone-conversion-decline-quarter-exact");
    if !on_zfs_or_acknowledged(&fixture, "a_converting_share_of_exactly_a_quarter_keeps_the_clone_rung") {
        remove(&fixture);
        return;
    }
    let converting: usize = DECLINE_PATHS_FLOOR + 1;
    // The tracked `.gitattributes` takes one non-converting path, so the rest are the plain files no rule matches.
    let other: usize = converting * DECLINE_SHARE_DIVISOR - converting - 1;
    let repo: PathBuf = decline_fixture(&fixture, "repo", converting, other);
    assert_rung_equals_git(&fixture, repo.as_path(), true);
}

#[test]
fn a_converting_share_one_path_past_a_quarter_declines_the_rung() {
    let fixture: Fixture = fixture("zfs-clone-conversion-decline-quarter-above");
    if !on_zfs_or_acknowledged(&fixture, "a_converting_share_one_path_past_a_quarter_declines_the_rung") {
        remove(&fixture);
        return;
    }
    let converting: usize = DECLINE_PATHS_FLOOR + 1;
    let other: usize = converting * DECLINE_SHARE_DIVISOR - converting - 2;
    let repo: PathBuf = decline_fixture(&fixture, "repo", converting, other);
    assert_rung_equals_git_expecting(&fixture, repo.as_path(), false, "so this rung declines and Git's checkout writes the worktree");
}

/// How many plain files the older commit's tree lacks under `old/`,
/// so the fixture's shape is one constant rather than a literal at each use.
const OLD_PATHS_ADDED: usize = 40;

/// What:
///  Every path below `root`,
/// as `/`-joined names relative to it,
///  with the top-level names in `left_out` skipped,
/// sorted.
/// Why:
///   A control that requires the served worktree to equal Git's checkout compares every entry it holds,
/// including one the worktree's own `.gitignore` hides from `git status`;
/// the stack keeps a deep tree off the call stack,
///  and one sorted list compares with another as one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pathsBelow(root, new Set(['.git'])); // sorted relative names
/// ```
fn paths_below(root: &Path, left_out: &[&str]) -> Vec<String> {
    let mut pending: Vec<String> = vec![String::new()];
    let mut found: Vec<String> = Vec::new();
    while let Some(parent) = pending.pop() {
        let directory: PathBuf = if parent.is_empty() { root.to_path_buf() } else { root.join(parent.as_str()) };
        let Ok(listing) = std::fs::read_dir(directory.as_path()) else { continue };
        for entry in listing.flatten() {
            let name: String = entry.file_name().to_string_lossy().into_owned();
            let relative: String = if parent.is_empty() { name.clone() } else { format!("{parent}/{name}") };
            if parent.is_empty() && left_out.contains(&name.as_str()) {
                continue;
            }
            if entry.file_type().map(|kind: std::fs::FileType| { return kind.is_dir(); }).unwrap_or(false) {
                pending.push(relative.clone());
            }
            found.push(relative);
        }
    }
    found.sort();
    return found;
}

/// What:
///  A repository whose newer commit holds a converting path and a folder of paths the older commit lacks:
/// a `*.txt text eol=crlf` rule,
/// an LF `notes.txt`,
/// a `keep.dat` no rule matches,
/// and a commit `other` made before `extra.txt` and the `old/` folder were added.
/// Why:
///   The add targets `other`,
/// so the clone copies the source's whole tracked set into a worktree whose tree lacks 41 of those paths,
/// and the two-tree update refuses the zeroed entry of `extra.txt`:
///  the one shape a clone rung can end in after its copies (`m05` of the class controls).
fn older_target_fixture(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = conversion_fixture(fixture, "repo", b"*.txt text eol=crlf\n", b"line one\nline two\n", &|repo| {
        std::fs::write(repo.join("keep.dat"), b"line one\n").expect("a path no rule matches");
        super::worktree_snapshot::age(repo.join("keep.dat").as_path());
        git(fixture, repo, &["add", "--", "keep.dat"]);
        git(fixture, repo, &["update-index", "--refresh"]);
        git(fixture, repo, &["update-ref", "refs/heads/other", "HEAD"]);
    });
    tracked(repo.as_path(), "extra.txt", b"line one\nline two\n");
    for index in 0..OLD_PATHS_ADDED {
        tracked(repo.as_path(), format!("old/file-{index}.dat").as_str(), b"line one\n");
    }
    commit_all(fixture, repo.as_path(), "second");
    return repo;
}

/// What:
///  A repository whose newer commit holds a path the fence hands to Git by its size proof and a folder of paths the older
///  commit lacks:
///  an `*.txt text eol=lf` rule,
///  a `crlf.txt` whose worktree bytes are the CRLF form the checkout rewrites to the LF blob,
///  and a commit `other` made before `crlf.txt` and the `old/` folder were added.
/// Why:
///   The attributes alone do not convert this path (`eol=lf` writes no carriage return),
/// so the fence says so only with the blob's size against the size the index recorded,
///  and the skip decided before the copies reads no blob:
///  the shape is the ending the skip cannot catch,
///  which the ladder still owes a worktree equal to Git's checkout with nothing left behind.
fn size_fenced_older_target_fixture(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = conversion_fixture(fixture, "repo", b"*.txt text eol=lf\n", b"line one\nline two\n", &|_repo| {});
    tracked(repo.as_path(), "crlf.txt", b"line one\r\nline two\r\n");
    git(fixture, repo.as_path(), &["add", "--", "crlf.txt"]);
    git(fixture, repo.as_path(), &["update-index", "--refresh"]);
    git(fixture, repo.as_path(), &["update-ref", "refs/heads/other", "HEAD"]);
    for index in 0..OLD_PATHS_ADDED {
        tracked(repo.as_path(), format!("old/file-{index}.dat").as_str(), b"line one\n");
    }
    commit_all(fixture, repo.as_path(), "second");
    return repo;
}

/// What:
///  An add to a commit whose tree lacks a path the fence hands to Git by its size proof ends after its copies,
/// and the worktree it publishes equals Git's checkout of that commit:
/// every tracked path's bytes and mode,
/// a clean status,
/// and no path of any kind the clones copied.
/// Why:
///   The copied paths the target tree lacks are the leftover the class harness's `m05` reports on the branch and that `main`
/// does not leave on either filesystem;
/// a served worktree that ends after its copies owes the same tree Git's own checkout makes,
/// so a leftover is a regression against `main` and not a limitation.
/// A path the attributes alone convert is skipped before the copies
/// (`a_target_tree_that_lacks_a_fenced_path_is_skipped_before_any_copy`),
/// so this control keeps the ending the skip cannot catch.
#[test]
fn a_target_tree_that_lacks_a_fenced_path_leaves_no_path_of_the_clones_behind() {
    let fixture: Fixture = fixture("zfs-clone-conversion-older-target");
    if !on_zfs_or_acknowledged(&fixture, "a_target_tree_that_lacks_a_fenced_path_leaves_no_path_of_the_clones_behind") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = size_fenced_older_target_fixture(&fixture);
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "-b", "topic", "../topic", "other"]);
    let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "control", "../control", "other"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    let topic: PathBuf = fixture.root.join("topic");
    let reference: PathBuf = fixture.root.join("control");
    assert!(!stderr_of(&created).contains("clones served"), "the tree difference ends the rung: {}", stderr_of(&created));
    assert!(stderr_of(&created).contains("and Git's own checkout made the worktree"), "{}", stderr_of(&created));
    assert_eq!(tracked_content(&fixture, topic.as_path()), tracked_content(&fixture, reference.as_path()), "every tracked path is Git's checkout's");
    assert_eq!(porcelain(&fixture, topic.as_path()), "", "the served worktree is clean");
    assert_eq!(paths_below(topic.as_path(), &[".git"]), paths_below(reference.as_path(), &[".git"]), "a path the clones copied and the target tree lacks stays behind");
    assert!(!topic.join("crlf.txt").exists(), "the fenced path the older tree lacks is gone");
    assert!(!topic.join("old").exists(), "the folder the older tree lacks is gone");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    assert!(admin.join("index").is_file(), "the rung gives the index the worktree's name once Git's checkout is done");
    assert!(!admin.join("index.pending").exists() && !admin.join("index.pending.lock").exists(), "nothing of the rung's waiting name stays in the administrative directory: {:?}", paths_below(admin.as_path(), &[]));
    std::fs::remove_dir_all(topic.as_path()).expect("remove the worktree");
    std::fs::remove_dir_all(reference.as_path()).expect("remove the control");
    remove(&fixture);
}

/// What:
///  The add records of `repo`,
///  oldest first,
/// as the JSON objects the wrapper appended.
/// Why:
///   A control that reads the route an add took reads the record the wrapper wrote,
/// and the record is the only place the choice of rung outlives the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const records = readFileSync(recordPath).trimEnd().split('\n').map(JSON.parse);
/// ```
fn add_records(repo: &Path) -> Vec<serde_json::Value> {
    let path: PathBuf = repo.join(".git").join(git_policy_cli::worktree::add_timing::ADD_TIMING_FILE);
    let text: String = std::fs::read_to_string(path.as_path()).expect("the add's record");
    return text.lines().map(|line: &str| return serde_json::from_str::<serde_json::Value>(line).expect("one JSON object per line")).collect();
}

/// What:
///  An add to a commit whose tree lacks a path the fence hands to Git is skipped before the add's record is taken and before any copy,
///  so nothing of the source is copied and Git's own checkout writes the worktree.
/// Why:
///   The two-tree update refuses the zeroed entry of a fenced path the new tree changes,
/// so a rung that copies first pays the whole copy and then Git's checkout over it
///  (the closure check measured 508 ms against `main`'s 378 on a 10,002-file fixture,
/// `doc/planning/cli-git-clone-rung-conversion-fence.md`,
///  "Limitations").
/// The fence's own reads already say which paths it hands to Git,
///  so the rung can see the case before it claims anything or copies anything;
/// the record's route,
///  the absent clone steps and the empty claim are what say no copy ran,
///  and the sentence names the path and the commit.
#[test]
fn a_target_tree_that_lacks_a_fenced_path_is_skipped_before_any_copy() {
    let fixture: Fixture = fixture("zfs-clone-conversion-older-target-skip");
    if !on_zfs_or_acknowledged(&fixture, "a_target_tree_that_lacks_a_fenced_path_is_skipped_before_any_copy") {
        remove(&fixture);
        return;
    }
    let repo: PathBuf = older_target_fixture(&fixture);
    let created: Observed = run_wrapped_verbose(&fixture, repo.as_path(), &["worktree", "add", "-b", "topic", "../topic", "other"]);
    let control: Observed = run_direct(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "control", "../control", "other"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    let records: Vec<serde_json::Value> = add_records(repo.as_path());
    let record: &serde_json::Value = records.last().expect("the add's record");
    assert_eq!(record["notes"]["route"].as_str(), Some("Git's checkout"), "the route the add's record names, and not the clone rung: {record}");
    let steps: &serde_json::Map<String, serde_json::Value> = record["steps"].as_object().expect("the record's steps");
    for clone_step in ["cloneEntries", "conversionBesideTheClone", "claimStage", "installIndexFile"] {
        assert!(!steps.contains_key(clone_step), "no step of the clone rung ran, so nothing was claimed and no file was copied ({clone_step}): {record}");
    }
    assert!(stderr_of(&created).contains("differs from the source's index at extra.txt"), "the sentence names the path the target tree changes: {}", stderr_of(&created));
    assert!(stderr_of(&created).contains("so this rung declines and Git's checkout writes the worktree"), "the sentence says what the rung did: {}", stderr_of(&created));
    assert!(!stderr_of(&created).contains("clones served"), "the clone rung served nothing: {}", stderr_of(&created));
    let topic: PathBuf = fixture.root.join("topic");
    let reference: PathBuf = fixture.root.join("control");
    assert_eq!(tracked_content(&fixture, topic.as_path()), tracked_content(&fixture, reference.as_path()), "every tracked path is Git's checkout's");
    assert_eq!(porcelain(&fixture, topic.as_path()), "", "the worktree is clean");
    assert_eq!(paths_below(topic.as_path(), &[".git"]), paths_below(reference.as_path(), &[".git"]), "nothing of the source's tracked set stays behind");
    assert!(!topic.join("extra.txt").exists(), "the fenced path the older tree lacks is not there");
    let admin: PathBuf = repo.join(".git/worktrees/topic");
    assert!(admin.join("index").is_file(), "Git's own checkout wrote the index");
    assert!(!admin.join("index.pending").exists() && !admin.join("index.pending.lock").exists(), "the rung's waiting name never existed: {:?}", paths_below(admin.as_path(), &[]));
    std::fs::remove_dir_all(topic.as_path()).expect("remove the worktree");
    std::fs::remove_dir_all(reference.as_path()).expect("remove the control");
    remove(&fixture);
}
