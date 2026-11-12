//! Uncommitted differential audit probes. Histories are made by the sibling TypeScript generator,
//! while these tests call the unmodified replay entry used by production.

/// What: imports the production retry entry and its request/reply shapes.
/// Why: compare Git's own merge with the exact tree the wrapper would accept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { mergeCaptured } from 'production';
/// ```
use crate::service_replay::{Captured, MergeSite, Merged, merge_captured};
/// The ordinary context and the repository-isolating test environment.
use crate::transaction_git::GitContext;
/// Owned filesystem paths rather than borrowed `Path` values survive the manifest read.
use std::path::PathBuf;
/// A process builder and its captured result let each Git call have a deadline.
use std::process::{Command, Output};

/// Configuration probes share this probe's bounded Git runner.
#[path = "shadow_config_audit_probe.rs"]
mod config_probes;

/// What: read one required string from a JSON manifest as borrowed `&str`, not copied `String`.
/// Why: the manifest outlives the calls, so borrowing needs no allocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function field(manifest, key): string { return manifest[key]; }
/// ```
fn field<'a>(manifest: &'a serde_json::Value, key: &str) -> &'a str {
    // Index the manifest and require the generator's string field.
    return manifest[key].as_str().expect("manifest field");
}

/// What: start plain `git` through a deadline, returning its output even for a conflict.
/// Why: the same executable under native PATH generates and checks the fixture histories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawnSync('git', args, { cwd, timeout: 15000, env: isolated });
/// ```
fn git_at(repo: &std::path::Path, args: &[&str]) -> Output {
    // Construct a deadline command, with Git found through native PATH, never the installed wrapper.
    let mut command = Command::new("timeout");
    // Pass borrowed arguments without changing any real repository state.
    command.args(["--kill-after=2s", "15s", "git"]).args(args).current_dir(repo);
    // Existing fixture isolation supplies identity and disables real global/system configuration.
    command.envs(crate::test_support::isolated_overlay());
    // Native PATH is intentionally limited to the system tools for disposable fixtures.
    command.env("PATH", "/usr/bin:/bin");
    // A start failure is a probe failure, not a merge conflict.
    return command.output().expect("bounded Git starts");
}

/// What: run the exact captured-tree replay and compare one affected file with the expected Git result.
/// Why: expose a successful wrong tree, not merely an intermediate helper's suspicious answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function probe(name) { const manifest = readManifest(name); compare(nativeMerge(manifest), mergeCaptured(manifest)); }
/// ```
fn probe(name: &str) {
    // The explicit fixture root keeps every disposable write inside this worktree.
    let root = PathBuf::from(std::env::var_os("SHADOW_AUDIT_ROOT").expect("fixture root"));
    // Read the JSON generator's manifest, failing rather than using guessed defaults.
    let manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join(format!("{name}.json"))).expect("manifest")).expect("JSON");
    // Borrow the generated repository path through an owned path suitable for the merge context.
    let repo = PathBuf::from(field(&manifest, "repo"));
    // Own the current working directory, which can deliberately be a subdirectory.
    let cwd = PathBuf::from(field(&manifest, "cwd"));
    // Name the real object store, exactly as production replay does.
    let common = repo.join(".git");
    // The synthetic-base index is private to this probe.
    let scratch = repo.join("audit-scratch");
    // A repeated narrow check rewrites the scratch index from the same recorded base.
    std::fs::create_dir_all(&scratch).expect("scratch");
    // Own the context so its native executable can be selected through PATH.
    let mut context: GitContext = crate::test_support::git_context();
    // The child overlay resolves the plain executable to real Git in these disposable fixtures.
    context.real_git = PathBuf::from("git");
    // Add native PATH to the borrowed context overlay.
    context.overlay.push(("PATH".into(), "/usr/bin:/bin".into()));
    // Scratch input files belong to this repository, not to the build tree or the user's home.
    context.common_dir = Some(common.clone());
    // The production site borrows the context and paths for this one bounded test.
    let site = MergeSite { context: &context, cwd: &cwd, common_dir: &common, object_directory: &common.join("objects"), scratch: &scratch };
    // No known capture order: this is exactly the subsumption route for independent staged bytes.
    let captured = Captured { base: field(&manifest, "base").into(), tree: field(&manifest, "prepared").into(), captured: 0, landed: Vec::new() };
    // Real Git is the positive control: it reports conflicts and preserves the unrelated file.
    let native = git_at(&repo, &["merge-tree", "--write-tree", "-z", &format!("--merge-base={}", captured.base), field(&manifest, "landed"), &captured.tree]);
    // Extract the native tree ID before the first NUL, including for a conflicted result.
    let native_tree = String::from_utf8_lossy(&native.stdout).split('\0').next().expect("native tree").to_string();
    // These histories must take the conflict path, otherwise a pass would never exercise subsumption.
    assert_eq!(native.status.code(), Some(1), "expected a native conflict");
    // The comparison names the native result before the production result is examined.
    println!("{name}: native merge exit={:?}, tree={native_tree}", native.status.code());
    // The production entry either accepts a tree, combines it by word, or refuses it.
    let outcome = merge_captured(&site, &captured, field(&manifest, "landed"), 1).expect("production replay");
    // Report the enum so a clean exit cannot hide that a case never exercised the merge.
    println!("{name}: production result={outcome:?}");
    // A conflict is the expected result for incompatible binary/symlink/content changes.
    if field(&manifest, "expected") == "CONFLICT" {
        // Fail on a success-looking tree for an actual conflict.
        assert!(matches!(outcome, Merged::Conflict(_)), "unexpected accepted tree");
        return;
    }
    // Both accepted tree variants have the same consumer-visible tree to inspect.
    let tree = match outcome {
        Merged::Tree(tree) => tree,
        Merged::Combined { tree, .. } => tree,
        Merged::Conflict(paths) => panic!("unexpected conflict: {paths:?}"),
    };
    // Read exact bytes rather than normalizing newlines or rendering names through lossy text.
    let observed = git_at(&repo, &["show", &format!("{tree}:{}", field(&manifest, "expectedPath"))]);
    // Native Git's conflicted tree still carries its unrelated-path result intact.
    let native_blob = git_at(&repo, &["show", &format!("{native_tree}:{}", field(&manifest, "expectedPath"))]);
    // Human-readable evidence quotes escaped CRLF, NUL, and newline bytes.
    println!("{name}: native bytes={:?}, production bytes={:?}, expected={:?}", String::from_utf8_lossy(&native_blob.stdout), String::from_utf8_lossy(&observed.stdout), field(&manifest, "expected"));
    // Print the full tree: an unexpected added or missing filename must not hide behind a single-blob check.
    println!("{name}: full tree={}", String::from_utf8_lossy(&git_at(&repo, &["ls-tree", "-r", &tree]).stdout));
    // Guard each probe against a failed `git show` being mistaken for empty file contents.
    assert!(observed.status.success() && native_blob.status.success());
    // The executable-mode probe checks the mode as well as the bytes it retained.
    if name == "mode-control" {
        assert!(git_at(&repo, &["ls-tree", &tree]).stdout.starts_with(b"100755 "));
    }
    // The assertion deliberately fails on the unmodified implementation when data is lost.
    assert_eq!(observed.stdout, field(&manifest, "expected").as_bytes());
}

/// The context-fusion hypothesis is checked against the production diff-tree flags.
#[test]
fn audit_interhunk_configuration_does_not_drop_the_landed_edit() {
    probe("interhunk-config");
}

/// The non-ASCII path's Latin-1 carrier aliases another real filename when encoded as UTF-8.
#[test]
fn audit_path_collision_preserves_the_unrelated_landed_edit() {
    probe("path-collision");
}

/// ASCII names provide a positive control with the same history and file bytes.
#[test]
fn audit_ascii_names_preserve_the_unrelated_landed_edit() {
    probe("ascii-control");
}

/// A person's diff.relative configuration must not turn a same-line conflict into success.
#[test]
fn audit_relative_diff_configuration_preserves_a_conflict() {
    probe("relative-config");
}

/// CRLF content must survive the text containment route byte-for-byte.
#[test]
fn audit_crlf_bytes_survive_subsumption() {
    probe("crlf-control");
}

/// Different binary bytes must stay a conflict rather than becoming a text subsumption.
#[test]
fn audit_binary_bytes_are_not_subsumed() {
    probe("binary-control");
}

/// A content extension must still preserve the landed executable mode.
#[test]
fn audit_executable_mode_survives_subsumption() {
    probe("mode-control");
}

/// Symlink targets must stay outside the text containment rules.
#[test]
fn audit_symlink_targets_are_not_subsumed() {
    probe("link-control");
}
