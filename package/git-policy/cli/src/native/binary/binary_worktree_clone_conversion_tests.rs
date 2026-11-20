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
use super::support::{Fixture, Observed, fixture, git, on_zfs_or_acknowledged, remove, repository, run_direct, run_wrapped_verbose, stderr_of};
use super::worktree_snapshot::{commit_all, tracked, tracked_content};
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
    let created: Observed = run_wrapped_verbose(fixture, repo, &["worktree", "add", "-b", "topic", "../topic"]);
    let control: Observed = run_direct(fixture, repo, &["worktree", "add", "--quiet", "-b", "control", "../control"]);
    assert_eq!(created.code, Some(0), "{}", stderr_of(&created));
    assert_eq!(control.code, Some(0), "{}", stderr_of(&control));
    let clones: bool = stderr_of(&created).contains("clones served");
    assert_eq!(clones, served, "the clone rung served the add: {}\n{}", served, stderr_of(&created));
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
        git(&fixture, repo, &["add", "--no-enforce-bulk-add", "--", "notes.txt"]);
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
        git(&fixture, repo, &["add", "--no-enforce-bulk-add", "--", "notes.txt"]);
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
        git(&fixture, repo, &["add", "--no-enforce-bulk-add", "--", "notes.txt"]);
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
        git(&fixture, repo, &["add", "--no-enforce-bulk-add", "--", "notes.txt"]);
        git(&fixture, repo, &["commit", "--quiet", "--message=added"]);
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
        git(&fixture, repo, &["add", "--no-enforce-bulk-add", "--", "notes.txt"]);
        git(&fixture, repo, &["commit", "--quiet", "--message=added"]);
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
    git(&fixture, repo.as_path(), &["add", "--no-enforce-bulk-add", "--force", "--", "edge/file.txt"]);
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
