//! What:
//!   The options of `git pull` cut into the words its fetch takes and the words its merge takes,
//!   and the plan that holds one command line's own reading.
//! Why:
//!   Native Git's `git pull` runs its merge as a child of Git,
//! so the wrapper never saw it;
//! a pull is completed here only when every option it typed goes to exactly one of the two halves,
//! and anything this does not read exactly is left to native Git's whole pull.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = planPull(region); // undefined for an option or operand not read exactly
//! ```

/// Owned operating-system text.
use std::ffi::OsString;

/// The options of `git pull` that go to its merge as typed (`builtin/pull.c`,
/// `run_merge`).
const MERGE_OPTIONS: &[&str] = &[
    "--no-edit",
    "--edit",
    "--ff",
    "--no-ff",
    "--stat",
    "--no-stat",
    "-n",
    "--log",
    "--no-log",
    "--commit",
    "--no-commit",
    "--squash",
    "--no-squash",
    "--signoff",
    "--no-signoff",
    "--verify",
    "--no-verify",
];

/// The options of `git pull` that go to its fetch as typed (`builtin/pull.c`,
/// `run_fetch`).
const FETCH_OPTIONS: &[&str] = &[
    "--all",
    "--prune",
    "-p",
    "--no-prune",
    "--tags",
    "-t",
    "--no-tags",
    "--force",
    "-f",
    "--ipv4",
    "-4",
    "--ipv6",
    "-6",
];

/// The options that go to both halves (`argv_push_verbosity`).
const SHARED_OPTIONS: &[&str] = &["-q", "--quiet", "-v", "--verbose"];

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
    /// `--ff` or `--no-ff` was typed,
    /// which is a stated way of reconciling divergent branches.
    pub fast_forward_stated: bool,
}

/// What:
///  Read the arguments after `pull`,
/// by exact spelling,
/// or nothing when any of them is not read exactly.
/// Why:
///   Git accepts unambiguous abbreviations and options this does not mirror (`--rebase`,
/// `--ff-only`,
/// `-s`,
/// `-X`,
///  `--autostash`,
/// `--dry-run`,
/// `--recurse-submodules`);
/// a pull with one of them is native Git's whole,
/// and so is a refspec
///  that could move the checked-out branch (it holds a colon).
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
    };
    // `for ... in` walks the tokens in order, like `for (const token of region)`.
    for token in region {
        // `?` returns nothing at once when the token is not text, like an early `return undefined`.
        let text: &str = token.to_str()?;
        if text == "--no-rebase" || text == "--rebase=false" {
            plan.merge_stated = true;
        } else if MERGE_OPTIONS.contains(&text) {
            plan.fast_forward_stated |= text == "--ff" || text == "--no-ff";
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
