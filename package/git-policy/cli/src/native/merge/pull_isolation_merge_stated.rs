//! What:
//!   Whether a pull is stated to merge,
//!   read from its command line and from the rebase settings Git's own pull reads.
//! Why:
//!   Git's pull merges when `--no-rebase` is typed,
//! or when `pull.rebase` / `branch.<name>.rebase` say no,
//! or,
//! with neither set,
//! when a way of reconciling is typed;
//! with nothing stated it refuses divergent branches with its advice,
//! and a setting that adds steps this does not mirror keeps the whole pull native.
//! The value is asked of Git (`git config --bool --get`) rather than read from the raw text,
//! so the last of several settings and every spelling of no are Git's own answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!mergeIsStated(plan, settings, rebaseStatement(context, branch))) return nativePull();
//! ```

/// The plan a pull's command line was read into.
use crate::pull_isolation_options::PullPlan;
/// Real Git for the queries.
use crate::transaction_git::{
    GitContext,
    GitOutput,
    GitRequest,
    run_git,
};
/// `Path` names the directory a query runs in.
use std::path::Path;

/// The settings that make the wrapper leave the pull to native Git:
/// they add steps this does not mirror (a stash around the
/// pull,
/// the update of submodules after it).
const NATIVE_ONLY_SETTINGS: &[&str] = &[
    "pull.autostash",
    "submodule.recurse",
    "fetch.recursesubmodules",
];

/// What:
///  What the rebase settings say about one pull.
/// Why:
///   `pull.rebase` and `branch.<name>.rebase` are booleans or one of `merges` and `interactive`,
/// and only a value Git reads as no is a merge.
/// A value that is a rebase,
/// or one Git refuses,
/// keeps the whole pull native,
/// which words that refusal itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RebaseStatement = 'unstated' | 'merge' | 'notMerge';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RebaseStatement {
    /// Neither setting is present,
    /// so a typed `--ff` or `--no-ff` is the only thing that states a merge.
    Unstated,
    /// The setting Git reads says no:
    /// this pull merges.
    Merge,
    /// The setting says a rebase,
    /// a mode this does not mirror,
    /// or a value Git refuses.
    NotMerge,
}

/// What:
///  Whether the command line and the settings state that this pull merges and mirror nothing else the pull would do.
/// Why:
///   Git's pull merges when `--no-rebase` is typed or `pull.rebase` / `branch.<name>.rebase` say no,
/// or,
/// with neither set,
/// when
///  a way of reconciling is typed (`--ff`,
/// `--no-ff`);
/// with nothing stated it refuses divergent branches with its advice,
/// and
///  that refusal stays native Git's.
/// `pull.ff` is left to Git,
/// because with `--no-rebase` it is rewritten by rules of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergeIsStated(plan: PullPlan, settings: string[], rebase: RebaseStatement): boolean;
/// ```
pub fn merge_is_stated(
    plan: &PullPlan,
    settings: &[String],
    rebase: RebaseStatement,
) -> bool {
    // The settings that add steps this does not mirror.
    for key in settings {
        if NATIVE_ONLY_SETTINGS.contains(&key.as_str()) {
            return false;
        }
    }
    // `pull.ff` is Git's to combine with the rest.
    if !plan.fast_forward_stated {
        for key in settings {
            if key == "pull.ff" {
                return false;
            }
        }
    }
    if plan.merge_stated {
        return true;
    }
    return match rebase {
        RebaseStatement::Merge => true,
        RebaseStatement::NotMerge => false,
        RebaseStatement::Unstated => plan.fast_forward_stated,
    };
}

/// What:
///  Run a read-only query against the invocation's repository.
/// Why:
///   Each query is one Git start;
/// the answer is the output whatever its status,
/// so a caller reads the code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ask(context: GitContext, args: string[]): Promise<GitOutput | undefined>;
/// ```
pub(crate) fn ask(
    context: &GitContext,
    arguments: &[&str],
) -> Option<GitOutput> {
    // `.ok()` turns a failed start into nothing, which the callers treat as "leave the pull to native Git".
    return run_git(context, &GitRequest::new(Path::new("."), arguments)).ok();
}

/// What:
///  What one query for a rebase setting answered.
/// Why:
///   The branch's setting is looked at before the general one,
/// and only a setting that is not there lets the next one decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettingAnswer = { kind: 'read'; statement: RebaseStatement } | { kind: 'absent' } | { kind: 'unreadable' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SettingAnswer {
    /// Git read the setting.
    Read(RebaseStatement),
    /// The setting is not present.
    Absent,
    /// Git refused the value,
    /// or the query could not start.
    Unreadable,
}

/// What:
///  What Git's own reading of one rebase setting says about the pull.
/// Why:
///   `git config --bool --get` answers with the value Git's pull reads:
/// the last of several settings,
/// in every boolean spelling Git accepts (`0`,
/// `00`,
/// `0x0`,
/// `0k`,
/// the empty value,
/// `false`,
/// `no`,
/// `off`).
/// A rebase mode (`merges`,
/// `interactive`) is no boolean,
/// so Git refuses it here and words that refusal itself when the pull runs natively.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rebaseAnswer(context, key): Promise<SettingAnswer>;
/// ```
fn rebase_answer(
    context: &GitContext,
    key: &str,
) -> SettingAnswer {
    // A query Git could not start states nothing this may act on.
    let Some(found) = ask(context, &["config", "--bool", "--get", key]) else {
        return SettingAnswer::Unreadable;
    };
    return match found.code {
        // Git prints `true` or `false` for a value it reads as a boolean and nothing else.
        Some(0) => {
            let text: String = String::from_utf8_lossy(&found.stdout).trim().to_owned();
            let statement: RebaseStatement = if text == "false" {
                RebaseStatement::Merge
            } else {
                RebaseStatement::NotMerge
            };
            SettingAnswer::Read(statement)
        }
        // Status 1 is Git's answer for a setting that is not there.
        Some(1) => SettingAnswer::Absent,
        // A value Git refuses, or a query that failed: native Git runs the pull and words the refusal.
        _ => SettingAnswer::Unreadable,
    };
}

/// What:
///  What the rebase settings of one branch say about a pull.
/// Why:
///   `config_get_rebase` reads `branch.<name>.rebase` before `pull.rebase`,
/// so the branch's own setting decides whenever it is present;
/// asking Git for the value keeps the last of several settings and every spelling of no,
/// which a reader of the raw text got wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rebaseStatement(context, branch): Promise<RebaseStatement>;
/// ```
pub(crate) fn rebase_statement(
    context: &GitContext,
    branch: &str,
) -> RebaseStatement {
    let branch_key: String = format!("branch.{branch}.rebase");
    for key in [branch_key.as_str(), "pull.rebase"] {
        match rebase_answer(context, key) {
            SettingAnswer::Read(statement) => return statement,
            SettingAnswer::Absent => {}
            SettingAnswer::Unreadable => return RebaseStatement::NotMerge,
        }
    }
    return RebaseStatement::Unstated;
}

/// Whether one byte is the NUL byte that ends each entry of a `-z` listing.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  The settings that add steps the split pull does not mirror,
/// as the keys present.
/// Why:
///   One `git config` call answers for all of them,
/// and only their presence decides anything,
/// so no value is read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pullSettings(context): Promise<string[] | undefined>;
/// ```
pub(crate) fn pull_settings(context: &GitContext) -> Option<Vec<String>> {
    let output: GitOutput = ask(
        context,
        &[
            "config",
            "--null",
            "--get-regexp",
            "^(pull\\.(ff|autostash)|submodule\\.recurse|fetch\\.recursesubmodules)$",
        ],
    )?;
    // Status 1 means no setting matched.
    if output.code != Some(0) && output.code != Some(1) {
        return None;
    }
    // The keys present, built entry by entry.
    let mut settings: Vec<String> = Vec::new();
    for entry in output.stdout.split(is_nul_byte) {
        if entry.is_empty() {
            continue;
        }
        let text: String = String::from_utf8_lossy(entry).into_owned();
        let key: &str = match text.split_once('\n') {
            // A value-less setting is present too, and only the key is read.
            Some((key, _value)) => key,
            None => text.as_str(),
        };
        settings.push(key.to_owned());
    }
    return Some(settings);
}

/// What:
///  The first line of a query's output,
/// trimmed,
/// when the query succeeded.
/// Why:
///   Names and object names are one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (output) => output.code === 0 ? output.stdout.trim() : undefined;
/// ```
pub(crate) fn line_of(output: Option<GitOutput>) -> Option<String> {
    let found: GitOutput = output?;
    if found.code != Some(0) {
        return None;
    }
    return Some(String::from_utf8_lossy(&found.stdout).trim().to_owned());
}
