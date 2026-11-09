//! What:
//!  What a copy from a main checkout leaves out of the ignored state it hands over:
//!  caches,
//! optionally the records of earlier runs,
//! the stage container of another command,
//! and every directory that is another checkout.
//! Why:
//!  A worktree added from the main checkout should work at once,
//! so builds and installed dependencies are copied,
//!  but a cache is rebuilt by its tool,
//! a log of an earlier run is no input,
//! another command's stage container holds
//!  another worktree's ignored state (credentials included),
//! and another checkout's files would make the
//!  new worktree a second claimant of that checkout's administrative state.
//!  The rule is a handful of directory names rather than a list of paths,
//! so a reader can predict it
//!  (`doc/planning/cli-git-main-worktree-ignored-copy.md` measures the set it leaves out).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prune = { kind: 'main-checkout', runRecords: COPY_RUN_RECORDS };
//! if (skipsPath(prune, 'package/x/.cache/typescript')) continue;
//! const STAGE_PREFIX = '.cli-git-worktree-copy-';
//! ```

/// The prefix of every stage container any rung makes beside its destination.
use crate::worktree_copy_selection::STAGE_PREFIX;

/// What:
///  Whether the records of earlier runs (logs,
/// mutation and test reports,
/// verification output) are copied.
/// Why:
///   The owner answered on 2026-11-02 "Skip run records (Recommended)",
/// so they are left out;
///  setting it to `true` is the one-line change that copies them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const COPY_RUN_RECORDS = false;
/// ```
pub const COPY_RUN_RECORDS: bool = false;

/// What:
///  Directory names that are caches wherever they sit below the source root.
/// Why:
///   A package-manager store or a build tool's project cache is rebuilt by the tool that made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CACHE_NAMES = ['.pnpm-store', '.gradle', '.kotlin', '.zig-cache'] as const;
/// ```
const CACHE_NAMES: [&str; 4] = [".pnpm-store", ".gradle", ".kotlin", ".zig-cache"];

/// What:
///  Directory names that are caches only inside a Cargo `target` directory.
/// Why:
///   `incremental` holds rustc's incremental-compilation state and `zig-global-cache` the Zig compiler's cache;
///  the compiled dependencies beside them are the build the new worktree needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TARGET_CACHE_NAMES = ['incremental', 'zig-global-cache'] as const;
/// ```
const TARGET_CACHE_NAMES: [&str; 2] = ["incremental", "zig-global-cache"];

/// What:
///  Directory names that hold the output of earlier test,
/// mutation and lint runs wherever they sit.
/// Why:
///   They are reports,
/// not inputs;
/// they are copied only while `COPY_RUN_RECORDS` is set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const RUN_RECORD_NAMES = ['mutants.out', 'mutants.out.old', 'playwright-report', 'test-results'] as const;
/// ```
const RUN_RECORD_NAMES: [&str; 4] = ["mutants.out", "mutants.out.old", "playwright-report", "test-results"];

/// What:
///  The name Git gives the entry that makes a directory a checkout of its own.
/// Why:
///   A directory that holds it,
/// file or directory,
/// is another checkout,
/// never ignored state of this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CHECKOUT_MARKER = '.git';
/// ```
const CHECKOUT_MARKER: &str = ".git";

/// What:
///  The leave-out rule one copy applies while it walks a source.
/// Why:
///   A linked or bare source hands over everything it ignores,
/// so only the main checkout has a rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prune = { kind: 'never' } | { kind: 'main-checkout'; runRecords: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Prune {
    /// Nothing is left out beyond the registered worktrees and the stage.
    Never,
    /// The main checkout's rule;
    /// `run_records` says whether the records of earlier runs are copied.
    MainCheckout {
        /// Whether run records (reports and logs of earlier runs) are copied.
        run_records: bool,
    },
}

/// What:
///  Whether any child name is the marker of a checkout.
/// Why:
///   A directory holding the marker is another checkout and is left out whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// childNames.includes(CHECKOUT_MARKER)
/// ```
fn names_checkout_marker(child_names: &[String]) -> bool {
    for name in child_names {
        if name == CHECKOUT_MARKER {
            return true;
        }
    }
    return false;
}

impl Prune {
    /// What:
    ///  The rule the main checkout uses,
    /// with the run-record setting the crate ships.
    /// Why:
    ///   One place reads the shipped constant,
    /// so changing the owner's answer is a one-line change.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// Prune.mainCheckout() // { kind: 'main-checkout', runRecords: COPY_RUN_RECORDS }
    /// ```
    pub fn main_checkout() -> Prune {
        return Prune::MainCheckout { run_records: COPY_RUN_RECORDS };
    }

    /// What:
    ///  Whether the path,
    /// relative to the source root,
    /// and everything below it is left out.
    /// Why:
    ///   The decision uses directory names only,
    /// so it is the same for a root Git listed and a child the walk found.
    /// A stage container is left out by every rule,
    /// because it belongs to the one add or copy that is
    ///  filling it and holds another worktree's ignored state.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// skipsPath(prune, 'package/x/target/debug/incremental') // true
    /// skipsPath(prune, '.claude/worktrees/.cli-git-worktree-copy-8f14e45fceea167a5a36dedd4bea2543') // true
    /// ```
    pub fn skips_path(self, relative_path: &str) -> bool {
        let components: Vec<&str> = relative_path.split('/').collect();
        // Another command's stage container is no state of the source, whatever the rule: its payload is a full copy of
        //  another worktree's ignored state, made while its own copy runs, and nothing names it in the new worktree.
        for component in &components {
            if component.starts_with(STAGE_PREFIX) {
                return true;
            }
        }
        let Prune::MainCheckout { run_records } = self else {
            return false;
        };
        for (index, component) in components.iter().enumerate() {
            let parent: Option<&str> = match index.checked_sub(1) {
                Some(before) => Some(components[before]),
                None => None,
            };
            let below_target: bool = components[..index].contains(&"target");
            // The repository root's own `.cache` is where the repository keeps tools and generated inputs that tasks read.
            let nested_cache: bool = *component == ".cache" && index > 0;
            let cache: bool = CACHE_NAMES.contains(component)
                || nested_cache
                || (below_target && TARGET_CACHE_NAMES.contains(component));
            let record: bool = RUN_RECORD_NAMES.contains(component)
                || (*component == ".monochromatic" && components[..index].contains(&"node_modules"))
                || (*component == "verification" && parent == Some("target"))
                || (*component == "cache" && parent == Some(".serena"));
            if cache || (record && !run_records) {
                return true;
            }
        }
        return false;
    }

    /// What:
    ///  Whether a directory with these child names is another checkout and is left out whole.
    /// Why:
    ///   The registered worktrees are excluded by path;
    /// this also catches a clone or a stale worktree directory
    ///  that Git no longer registers.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// skipsCheckout(prune, ['.git', 'src']) // true
    /// ```
    pub fn skips_checkout(self, child_names: &[String]) -> bool {
        return match self {
            Prune::Never => false,
            Prune::MainCheckout { .. } => names_checkout_marker(child_names),
        };
    }
}

/// Prune rule controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_prune_tests.rs"]
mod tests;
