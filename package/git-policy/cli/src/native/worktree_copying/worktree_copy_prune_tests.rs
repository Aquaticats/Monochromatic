//! The main checkout's leave-out rule:
//! each marker family,
//! the root `.cache` exception,
//! and the run-record toggle.

use super::*;

/// The rule with run records skipped,
/// which is what the crate ships.
const SKIPPING: Prune = Prune::MainCheckout { run_records: false };

/// The rule with run records copied,
/// which the toggle allows.
const COPYING: Prune = Prune::MainCheckout { run_records: true };

/// The shipped toggle skips run records,
/// and the shipped rule is built from it.
#[test]
fn shipped_rule_follows_the_toggle() {
    assert_eq!(Prune::main_checkout(), Prune::MainCheckout { run_records: COPY_RUN_RECORDS });
    // The owner answered 2026-11-02 "Skip run records (Recommended)", so the shipped rule leaves a run record out.
    assert!(Prune::main_checkout().skips_path(b"node_modules/.monochromatic/2026-11-01/log.jsonl"));
}

/// Dependencies and builds are never left out,
/// whichever setting applies.
#[test]
fn dependencies_and_builds_are_kept() {
    for prune in [SKIPPING, COPYING] {
        for kept in [
            "node_modules",
            "node_modules/.pnpm/pkg@1.0.0/node_modules/pkg/index.js",
            "package/git-policy/cli/node_modules/@scope/pkg",
            "package/git-policy/cli/target",
            "package/git-policy/cli/target/debug/deps/libx.rlib",
            "package/git-policy/cli/target/debug/build/x/out",
            "package/git-policy/cli/target/debug/.fingerprint/x",
            "package/x/dist/index.js",
            "package/x/build/out.bin",
            ".cache",
            ".cache/toolchain-bin/node",
            ".cache/forbidden-strings.rules.txt",
            "mise.local.toml",
            ".env.local.json",
        ] {
            assert!(!prune.skips_path(kept.as_bytes()), "{kept} is needed by a new worktree");
        }
    }
}

/// Caches are left out under both settings,
/// and the repository root's own `.cache` is the one exception.
#[test]
fn caches_are_left_out_except_the_root_cache() {
    for prune in [SKIPPING, COPYING] {
        for cache in [
            ".pnpm-store",
            ".pnpm-store/v11/files/ab",
            "node_modules/.pnpm-store",
            "node_modules/.cache",
            "node_modules/.cache/jiti/x.js",
            "package/x/.cache",
            "package/x/.cache/typescript/tsconfig.tsbuildinfo",
            "package/x/.gradle",
            "package/x/.kotlin/sessions",
            "package/x/.zig-cache",
            "package/x/.zig-cache/o/abc/main.o",
            "package/x/target/debug/incremental",
            "package/x/target/release/incremental/y",
            "package/x/target/x86_64-unknown-linux-gnu/debug/incremental",
            "package/x/target/zig-global-cache/z",
        ] {
            assert!(prune.skips_path(cache.as_bytes()), "{cache} is a cache");
        }
    }
}

/// Zig's build outputs are not caches:
/// only its project cache is,
/// and the installed artifacts are the build a worktree needs.
#[test]
fn zig_outputs_are_kept() {
    assert!(!SKIPPING.skips_path(b"package/x/zig-out"));
    assert!(!SKIPPING.skips_path(b"package/x/zig-out/bin/tool"));
}

/// `incremental` and `zig-global-cache` are caches only inside a build output directory.
#[test]
fn target_cache_names_apply_only_below_a_target_directory() {
    assert!(!SKIPPING.skips_path(b"package/x/incremental"));
    assert!(!SKIPPING.skips_path(b"package/x/zig-global-cache"));
    assert!(!SKIPPING.skips_path(b"package/x/src/incremental/data"));
}

/// Run records are left out only while the toggle says so.
#[test]
fn run_records_follow_the_toggle() {
    for record in [
        "node_modules/.monochromatic",
        "node_modules/.monochromatic/2026-11-01/log.jsonl",
        "package/x/node_modules/.monochromatic/log.jsonl",
        "node_modules/.pnpm/pkg/node_modules/.monochromatic",
        "package/x/target/verification",
        "package/x/target/verification/run/out.txt",
        "package/x/mutants.out",
        "package/x/mutants.out.old/outcomes.json",
        "playwright-report",
        "test-results/a.json",
        ".serena/cache/symbols",
    ] {
        assert!(SKIPPING.skips_path(record.as_bytes()), "{record} is a run record that is skipped");
        assert!(!COPYING.skips_path(record.as_bytes()), "{record} is a run record that the toggle copies");
    }
}

/// A `.monochromatic` outside a dependency tree,
/// a `verification` not directly under `target`,
/// and a `cache` not under
/// `.serena` are ordinary names.
#[test]
fn record_names_need_their_parents() {
    for kept in [".monochromatic", "package/x/.monochromatic/a", "package/x/verification/a", "package/x/cache/a", "target/debug/verification"] {
        assert!(!SKIPPING.skips_path(kept.as_bytes()), "{kept} is not a run record");
    }
}

/// Without a rule nothing is left out by name,
/// except another command's stage container.
#[test]
fn no_rule_leaves_out_nothing() {
    assert!(!Prune::Never.skips_path(b"node_modules/.pnpm-store"));
    assert!(!Prune::Never.skips_path(b"package/x/target/debug/incremental"));
    assert!(!Prune::Never.skips_checkout(&[b".git".to_vec()]));
}

/// What:
///  A stage container belongs to the one add or copy that is filling it,
/// so no rule carries it into another
///  worktree.
/// Why:
///   A container is an ordinary directory holding the other worktree's whole payload (its ignored state,
/// credentials
///  included) and its destination lies beside the one the snapshot is made for,
/// so a snapshot of the main
///  checkout carries it whole and nothing names it afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const prune of [SKIPPING, COPYING, Prune.Never]) assert(prune.skipsPath('.claude/worktrees/.cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543/payload/node_modules/pkg/index.js'))
/// ```
#[test]
fn a_stage_container_of_another_command_is_never_carried() {
    let container: &str = ".cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543";
    for prune in [SKIPPING, COPYING, Prune::Never] {
        for path in [
            container.to_string(),
            format!(".claude/worktrees/{container}"),
            format!(".claude/worktrees/{container}/payload/node_modules/pkg/index.js"),
        ] {
            assert!(prune.skips_path(path.as_bytes()), "{path} is another command's stage container");
        }
    }
    assert!(!SKIPPING.skips_path(b"package/x/node_modules"), "a dependency tree of the source is not a stage");
    assert!(!SKIPPING.skips_path(b"package/x/cli-git-worktree-copy-notes.md"), "a name that merely holds the prefix is not a stage");
}

/// A directory holding `.git`,
/// as a file or as a directory,
/// is another checkout;
/// a lookalike name is not.
#[test]
fn a_directory_holding_dot_git_is_another_checkout() {
    assert!(SKIPPING.skips_checkout(&[b"src".to_vec(), b".git".to_vec()]));
    assert!(COPYING.skips_checkout(&[b".git".to_vec()]));
    assert!(!SKIPPING.skips_checkout(&[b".github".to_vec(), b"gitignore".to_vec()]));
    assert!(!SKIPPING.skips_checkout(&[]));
}

/// What:
///  Only a directory named the way the wrapper names a trash is left out below the trash parent;
/// a directory of
///  the person's own there is state of the source and is copied with the rest.
/// Why:
///  `info/exclude` keeps the whole trash parent out of every commit,
/// so a person may keep their own files
///  there;
/// leaving out every path below the parent drops those files from the new worktree,
/// and on the
///  snapshot path it moves them into this add's trash,
/// where the service deletes them as if the wrapper had made
///  them.
/// The wrapper names a trash with forty lowercase hexadecimal digits (`new_trash_name`),
/// so a name of any
///  other shape is not a trash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const prune of [Prune.Never, Prune.main_checkout()]) {
///   assert(prune.skipsPath(`.claude/cli-git-trash/${"a".repeat(40)}/d0/payload`));
///   assert(!prune.skipsPath('.claude/cli-git-trash/personal/important-notes.txt'));
/// }
/// ```
#[test]
fn a_directory_below_the_trash_parent_that_is_no_trash_name_is_state_of_the_source() {
    let trash: String = format!(".claude/cli-git-trash/{}", "a".repeat(40));
    for prune in [SKIPPING, COPYING, Prune::Never] {
        assert!(prune.skips_path(trash.as_bytes()), "{trash} is a trash the wrapper made");
        assert!(prune.skips_path(format!("{trash}/d0/payload").as_bytes()), "and everything below it is its own");
        assert!(!prune.skips_path(b".claude/cli-git-trash"), "the parent is walked, so the person's own directories below it are seen");
        assert!(!prune.skips_path(b".claude/cli-git-trash/personal/important-notes.txt"), "a directory of the person's own below the parent is copied");
        assert!(!prune.skips_path(format!(".claude/cli-git-trash/{}", "b".repeat(39)).as_bytes()), "a name one digit short is not one the wrapper makes");
        assert!(!prune.skips_path(format!(".claude/cli-git-trash/{}", "A".repeat(40)).as_bytes()), "and neither is one in upper case");
    }
}
