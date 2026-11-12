//! What:
//!  What a pull reads from its command line:
//! the arguments of its fetch,
//! of its merge,
//! and the choice of merging
//!  it states.
//! Why:
//!   The two halves of a pull take different options (`builtin/pull.c`,
//! `run_fetch` and `run_merge`),
//!  and a pull merges only when the command
//! line or the settings say so;
//!  reading the command line exactly is what decides
//!  whether the wrapper may run the halves itself,
//!  so the options,
//! the plan and its reader live together,
//!  in a file
//!  the pull that runs them imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planPull(region); // undefined for anything not read exactly
//! ```

/// Owned operating-system text.
use std::ffi::OsString;

/// The options of `git pull` that go to its merge as typed (`builtin/pull.c`,
/// `run_merge`).
const MERGE_OPTIONS: [&str; 18] = [
    "--no-edit", "--edit", "--ff", "--no-ff", "--ff-only", "--stat", "--no-stat", "-n", "--log", "--no-log", "--commit",
    "--no-commit", "--squash", "--no-squash", "--signoff", "--no-signoff", "--verify", "--no-verify",
];

/// The options of `git pull` that go to its fetch as typed (`builtin/pull.c`,
/// `run_fetch`).
const FETCH_OPTIONS: [&str; 13] = [
    "--all", "--prune", "-p", "--no-prune", "--tags", "-t", "--no-tags", "--force", "-f", "--ipv4", "-4", "--ipv6", "-6",
];

/// The options that go to both halves (`argv_push_verbosity`).
const SHARED_OPTIONS: [&str; 4] = ["-q", "--quiet", "-v", "--verbose"];

/// The settings that make the wrapper leave the pull to native Git:
/// they add steps this does not mirror (a stash around the
/// pull,
/// the update of submodules after it).
pub(crate) const NATIVE_ONLY_SETTINGS: [&str; 3] = ["pull.autostash", "submodule.recurse", "fetch.recursesubmodules"];

/// What:
///  What a pull reads from its command line:
/// the arguments of its fetch,
/// of its merge,
/// and where the choice of merging
///  was stated.
/// Why:
///   The two halves of a pull take different options,
/// and a pull merges only when the command line or the settings say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PullPlan = { fetch: string[]; merge: string[]; places: string[]; mergeStated: boolean; fastForwardStated: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PullPlan {
    /// Options for `git fetch`,
    /// as typed.
    pub fetch: Vec<OsString>,
    /// Options for `git merge`,
    /// as typed.
    pub merge: Vec<OsString>,
    /// The repository and refspecs,
    /// after the options.
    pub places: Vec<OsString>,
    /// `--no-rebase` or `--rebase=false` was typed,
    /// which overrides every setting.
    pub merge_stated: bool,
    /// `--ff`,
    /// `--ff-only` or `--no-ff` was typed,
    /// which is a stated way of reconciling divergent branches.
    pub fast_forward_stated: bool,
    /// `--ff-only` was typed,
    /// which lets the branch move only by a fast-forward.
    pub fast_forward_only: bool,
}

/// What:
///  Read the arguments after `pull`,
/// by exact spelling,
/// or nothing when any of them is not read exactly.
/// Why:
///   Git accepts unambiguous abbreviations and options this does not mirror (`--rebase`,
/// `-s`,
/// `-X`,
///  `--autostash`,
/// `--dry-run`,
/// `--recurse-submodules`);
/// a pull with one of them is native Git's whole,
/// and so is a refspec
///  that could move the checked-out branch (it holds a colon).
///  `--ff-only` is read here and passed to the merge half,
///  and a pull whose fetched head cannot fast-forward is left to
///  native Git whole,
/// so its own refusal is the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planPull(region: string[]): PullPlan | undefined;
/// ```
pub fn plan_pull(region: &[OsString]) -> Option<PullPlan> {
    // The plan being built; `Vec::new()` is an empty list like `[]`.
    let mut plan: PullPlan = PullPlan {
        fetch: Vec::new(),
        merge: Vec::new(),
        places: Vec::new(),
        merge_stated: false,
        fast_forward_stated: false,
        fast_forward_only: false,
    };
    // `for ... in` walks the tokens in order, like `for (const token of region)`.
    for token in region {
        // `?` returns nothing at once when the token is not text, like an early `return undefined`.
        let text: &str = token.to_str()?;
        if text == "--no-rebase" || text == "--rebase=false" {
            plan.merge_stated = true;
        } else if MERGE_OPTIONS.contains(&text) {
            plan.fast_forward_stated |= text == "--ff" || text == "--no-ff" || text == "--ff-only";
            plan.fast_forward_only |= text == "--ff-only";
            plan.merge.push(token.clone());
        } else if FETCH_OPTIONS.contains(&text) {
            plan.fetch.push(token.clone());
        } else if SHARED_OPTIONS.contains(&text) {
            plan.fetch.push(token.clone());
            plan.merge.push(token.clone());
        } else if text.starts_with('-') {
            // An option this does not read exactly, `--` included.
            return None;
        } else if !plan.places.is_empty() && text.contains(':') {
            // A refspec that names a destination could update the branch that is checked out.
            return None;
        } else {
            plan.places.push(token.clone());
        }
    }
    return Some(plan);
}
